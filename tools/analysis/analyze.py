#!/usr/bin/env python3
"""Stem separation and song structure analysis for Veltrunox (ADR-015).

    analyze.py INPUT OUTPUT.zip TITLE
    analyze.py INPUT OUTPUT.zip TITLE --from-results DIR

Writes a stem package: manifest.json plus one Ogg Vorbis file per stem, at the sample rate
of INPUT. Prints "progress N" lines. With --from-results nothing is analysed; DIR holds
analysis.json and stems/*.wav of an earlier run.
"""
import argparse
import hashlib
import json
import os
import re
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


def analyse(source, work):
    """Runs the models; returns (analysis, directory with the stem WAV files)."""
    import torch
    import allin1_infer
    from allin1_infer.stems import DemucsProvider

    wav = work / "input.wav"
    run_ffmpeg("-i", source, "-ar", "44100", "-ac", "2", "-c:a", "pcm_s16le", wav)
    progress(5)

    provider = DemucsProvider(device="cpu")
    if torch.backends.mps.is_available():
        # all-in-one-infer rejects "mps" when resolving devices; Demucs itself runs on it.
        provider.device = "mps"
        patch_istft_for_mps()
    separate = provider.get_stems

    def reporting(*arguments, **keywords):
        result = separate(*arguments, **keywords)
        progress(55)
        return result

    provider.get_stems = reporting
    stems = work / "stems"
    result = allin1_infer.analyze(
        wav,
        out_dir=work / "json",
        model="harmonix-all",
        device="cpu",
        demix_dir=stems,
        spec_dir=work / "spec",
        keep_byproducts=True,
        overwrite=True,
        multiprocess=False,
        stem_provider=provider,
    )
    progress(85)
    analysis = {
        "bpm": result.bpm,
        "beats": result.beats,
        "downbeats": result.downbeats,
        "segments": [
            {"start": s.start, "end": s.end, "label": s.label} for s in result.segments
        ],
    }
    return analysis, stems / "htdemucs" / wav.stem


def patch_istft_for_mps():
    """demucs-infer 4.4.0 computes the inverse STFT of MPS tensors on the CPU and does not
    move the result back."""
    import demucs_infer.htdemucs as htdemucs

    original = htdemucs.ispectro

    def moved_back(z, *arguments, **keywords):
        return original(z, *arguments, **keywords).to(z.device)

    htdemucs.ispectro = moved_back


def manifest_of(source, title, analysis):
    def millis(seconds):
        return round(seconds * 1000)

    bpm = analysis.get("bpm")
    return {
        "format_version": 1,
        "recording": {
            "content_hash": hashlib.sha256(source.read_bytes()).hexdigest(),
            "title": title,
        },
        "stems": [{"name": name, "file": f"stems/{name}.ogg"} for name in STEMS],
        "analysis": {
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
        },
    }


def write_package(source, target, title, analysis, stems, work):
    rate = sample_rate(source)
    with zipfile.ZipFile(target, "w", zipfile.ZIP_STORED) as package:
        for name in STEMS:
            encoded = work / f"{name}.ogg"
            run_ffmpeg(
                "-i", stems / f"{name}.wav", "-ar", rate, "-c:a", "libvorbis",
                "-q:a", QUALITY, encoded,
            )
            package.write(encoded, f"stems/{name}.ogg")
        package.writestr("manifest.json", json.dumps(manifest_of(source, title, analysis)))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("title")
    parser.add_argument("--from-results", type=Path)
    options = parser.parse_args()
    os.environ.setdefault("PYTORCH_ENABLE_MPS_FALLBACK", "1")
    with tempfile.TemporaryDirectory() as temporary:
        work = Path(temporary)
        if options.from_results:
            analysis = json.loads((options.from_results / "analysis.json").read_text())
            stems = options.from_results / "stems"
        else:
            analysis, stems = analyse(options.input, work)
        write_package(
            options.input, options.output, options.title, analysis, stems, work
        )
    progress(95)


if __name__ == "__main__":
    main()
