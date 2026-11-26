#!/usr/bin/env python3
"""Stem separation and song structure analysis for Veltrunox (ADR-015).

    analyze.py INPUT OUTPUT.zip TITLE
    analyze.py INPUT OUTPUT.zip TITLE --from-results DIR
    analyze.py --structure OUTPUT.json --stem NAME=FILE ...

The first form separates INPUT and writes a stem package: manifest.json plus one Ogg Vorbis
file per stem, at the sample rate of INPUT. With --from-results nothing is separated; DIR
holds stems/*.wav and possibly analysis.json of an earlier run. With --structure the song
structure is found from the stems drums, bass, vocals and other, and written as JSON.
Prints "progress N" lines.
"""
import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import tempfile
import zipfile
from pathlib import Path

STEMS = ("drums", "bass", "vocals", "other")
# Ogg Vorbis quality; 6 is about 190 kbit/s for stereo.
QUALITY = "6"


def progress(percent):
    print(f"progress {percent}", flush=True)


def ffmpeg():
    import imageio_ffmpeg

    return imageio_ffmpeg.get_ffmpeg_exe()


def sample_rate(path):
    # ffmpeg describes its input on stderr and fails without an output; that is fine.
    described = subprocess.run(
        [ffmpeg(), "-hide_banner", "-i", str(path)], capture_output=True, text=True
    ).stderr
    found = re.search(r"Audio:.*?(\d+) Hz", described)
    if not found:
        raise SystemExit("cannot determine the sample rate of the recording")
    return int(found.group(1))


def run_ffmpeg(*arguments):
    subprocess.run([ffmpeg(), "-v", "error", "-y", *map(str, arguments)], check=True)


def separate(source, work):
    """Runs Demucs; returns the directory with the stem WAV files."""
    import torch
    from allin1_infer import stems as providers

    wav = work / "input.wav"
    run_ffmpeg("-i", source, "-ar", "44100", "-ac", "2", "-c:a", "pcm_s16le", wav)
    progress(5)

    provider = providers.DemucsProvider(device="cpu")
    if torch.backends.mps.is_available():
        # all-in-one-infer rejects "mps" when resolving devices; Demucs itself runs on it.
        provider.device = "mps"
        patch_istft_for_mps()
    report_separation(providers, 5, 85)
    return Path(provider.get_stems(wav, work / "stems"))


def report_separation(providers, start, end):
    """Demucs works through the recording piece by piece; report each finished piece."""
    apply_model = providers.apply_model

    def reporting(model, mix, **keywords):
        length = mix.shape[-1]
        done = [start]

        def piece(state):
            fraction = min(state.get("segment_offset", 0) / length, 1)
            percent = start + int(fraction * (end - start))
            if percent > done[0]:
                done[0] = percent
                progress(percent)

        result = apply_model(model, mix, callback=piece, **keywords)
        progress(end)
        return result

    providers.apply_model = reporting


def find_structure(stem_files, work):
    """Finds the song structure from stems that are already separated."""
    import allin1_infer
    import torch
    from allin1_infer.models.allinone import AllInOne

    demix = work / "demix" / "htdemucs" / "input"
    demix.mkdir(parents=True)
    for count, name in enumerate(STEMS, 1):
        run_ffmpeg(
            "-i", stem_files[name], "-ar", "44100", "-ac", "2", "-c:a", "pcm_s16le",
            demix / f"{name}.wav",
        )
        progress(count * 3)
    # The models take the stems only; the path just names the run.
    source = work / "input.wav"
    shutil.copy(demix / "other.wav", source)

    # The ensemble runs its folds one after the other.
    folds = [0]

    def fold_done(module, *_):
        if isinstance(module, AllInOne):
            folds[0] += 1
            progress(min(30 + folds[0] * 8, 94))

    hook = torch.nn.modules.module.register_module_forward_hook(fold_done)
    try:
        result = allin1_infer.analyze(
            source,
            out_dir=work / "json",
            model="harmonix-all",
            device="cpu",
            demix_dir=work / "demix",
            spec_dir=work / "spec",
            keep_byproducts=True,
            overwrite=True,
            multiprocess=False,
            skip_separation=True,
        )
    finally:
        hook.remove()
    return {
        "bpm": result.bpm,
        "beats": result.beats,
        "downbeats": result.downbeats,
        "segments": [
            {"start": s.start, "end": s.end, "label": s.label} for s in result.segments
        ],
    }


def patch_istft_for_mps():
    """demucs-infer 4.4.0 computes the inverse STFT of MPS tensors on the CPU and does not
    move the result back."""
    import demucs_infer.htdemucs as htdemucs

    original = htdemucs.ispectro

    def moved_back(z, *arguments, **keywords):
        return original(z, *arguments, **keywords).to(z.device)

    htdemucs.ispectro = moved_back


def structure_of(analysis):
    def millis(seconds):
        return round(seconds * 1000)

    bpm = analysis.get("bpm")
    return {
        "bpm": None if bpm is None else round(bpm),
        "beats_ms": [millis(t) for t in analysis.get("beats", [])],
        "downbeats_ms": [millis(t) for t in analysis.get("downbeats", [])],
        "sections": [
            {
                "start_ms": millis(s["start"]),
                "end_ms": millis(s["end"]),
                "label": s["label"],
            }
            for s in analysis.get("segments", [])
        ],
    }


def manifest_of(source, title, analysis):
    manifest = {
        "format_version": 1,
        "recording": {
            "content_hash": hashlib.sha256(source.read_bytes()).hexdigest(),
            "title": title,
        },
        "stems": [{"name": name, "file": f"stems/{name}.ogg"} for name in STEMS],
    }
    if analysis:
        manifest["analysis"] = structure_of(analysis)
    return manifest


def write_package(source, target, title, analysis, stems, work):
    rate = sample_rate(source)
    with zipfile.ZipFile(target, "w", zipfile.ZIP_STORED) as package:
        for count, name in enumerate(STEMS, 1):
            encoded = work / f"{name}.ogg"
            run_ffmpeg(
                "-i", stems / f"{name}.wav", "-ar", rate, "-c:a", "libvorbis",
                "-q:a", QUALITY, encoded,
            )
            package.write(encoded, f"stems/{name}.ogg")
            progress(85 + count * 3)
        package.writestr("manifest.json", json.dumps(manifest_of(source, title, analysis)))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path, nargs="?")
    parser.add_argument("output", type=Path, nargs="?")
    parser.add_argument("title", nargs="?")
    parser.add_argument("--from-results", type=Path)
    parser.add_argument("--structure", type=Path, metavar="OUTPUT.json")
    parser.add_argument("--stem", action="append", default=[], metavar="NAME=FILE")
    options = parser.parse_args()
    os.environ.setdefault("PYTORCH_ENABLE_MPS_FALLBACK", "1")
    with tempfile.TemporaryDirectory() as temporary:
        work = Path(temporary)
        if options.structure:
            files = dict(entry.split("=", 1) for entry in options.stem)
            missing = [name for name in STEMS if name not in files]
            if missing:
                parser.error(f"missing stems: {', '.join(missing)}")
            found = structure_of(find_structure(files, work))
            options.structure.write_text(json.dumps(found))
        elif options.title is None:
            parser.error("INPUT OUTPUT.zip TITLE are required")
        else:
            if options.from_results:
                found = options.from_results / "analysis.json"
                analysis = json.loads(found.read_text()) if found.exists() else None
                stems = options.from_results / "stems"
            else:
                analysis, stems = None, separate(options.input, work)
            write_package(
                options.input, options.output, options.title, analysis, stems, work
            )
    progress(99)


if __name__ == "__main__":
    main()
