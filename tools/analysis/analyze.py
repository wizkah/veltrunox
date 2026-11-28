#!/usr/bin/env python3
"""Stem separation and song structure analysis for Veltrunox (ADR-015).

    analyze.py INPUT OUTPUT.zip TITLE [--model NAME]
    analyze.py INPUT OUTPUT.zip TITLE --from-results DIR
    analyze.py --structure OUTPUT.json --stem NAME=FILE ...

The first form separates INPUT with the model NAME (see MODELS; demucs4 if not said) and
writes a stem package: manifest.json plus one Ogg Vorbis
file per stem, at the sample rate of INPUT. With --from-results nothing is separated; DIR
holds stems/*.wav and possibly analysis.json of an earlier run (--model NAME says which model
made them). The package names its model. With --structure the song
structure is found from the stems drums, bass and vocals, with all further stems together
as the rest, and written as JSON.
Prints "progress N" lines. The work runs in a second process that guard.py kills before it
can take the machine down.
"""
import argparse
import hashlib
import json
import os
import random
import re
import shutil
import subprocess
import sys
import tempfile
import zipfile
from pathlib import Path

import guard

# The order in which a package lists them.
STEMS = ("drums", "bass", "vocals", "other", "guitar", "piano")
# What the structure model takes.
STRUCTURE_STEMS = STEMS[:4]
HERE = Path(__file__).resolve().parent
MODELS_DIRECTORY = HERE / "models"
# Directory with a directory per RoFormer model, as download_roformer.sh makes it.
ROFORMER_DIRECTORY = HERE / "roformer"
# Demucs by the number of its stems, and the RoFormer models by their directory.
DEMUCS = {"demucs4": 4, "demucs6": 6}
MODELS = (*DEMUCS, "bs4", "sw6", "mel4", "vocals")
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


def decoded(source, work):
    """The recording as the model takes it: [channels, samples] at 44100 Hz."""
    import numpy
    import torch

    raw = work / "input.f32"
    run_ffmpeg("-i", source, "-ar", "44100", "-ac", "2", "-f", "f32le", raw)
    samples = numpy.fromfile(raw, dtype=numpy.float32)
    return torch.from_numpy(samples.reshape(-1, 2).T.copy())


def separation_progress(fraction):
    progress(5 + int(fraction * 80))


def with_demucs(mix, stem_count):
    """Returns {name: [channels, samples]}."""
    from demucs_infer.apply import apply_model
    from openvino_demucs import OpenVinoDemucs

    model = OpenVinoDemucs(MODELS_DIRECTORY, stem_count)
    level = mix.mean(0)
    mean, deviation = level.mean(), level.std() + 1e-8
    length = mix.shape[-1]
    done = [0]

    def piece(state):
        fraction = min(state.get("segment_offset", 0) / length, 1)
        if fraction > done[0]:
            done[0] = fraction
            separation_progress(fraction)

    # apply_model shifts the input by a random amount; the same input shall give the same.
    random.seed(0)
    separated = apply_model(model, ((mix - mean) / deviation)[None], callback=piece)[0]
    return dict(zip(model.sources, separated * deviation + mean))


def separate(source, work, model):
    """Splits the recording; returns the directory with the stem WAV files."""
    from demucs_infer.audio import save_audio

    mix = decoded(source, work)
    progress(5)
    if model in DEMUCS:
        waves = with_demucs(mix, DEMUCS[model])
    else:
        import roformer

        waves = roformer.separate(ROFORMER_DIRECTORY / model, mix, separation_progress)
    progress(85)
    stems = work / "stems"
    stems.mkdir()
    for name, wave in waves.items():
        save_audio(wave, str(stems / f"{name}.wav"), 44100)
    return stems


def mixed_down(files, target):
    """Writes the sum of the files as one WAV file."""
    inputs = [part for file in files for part in ("-i", file)]
    mix = [] if len(files) == 1 else [
        "-filter_complex", f"amix=inputs={len(files)}:normalize=0"
    ]
    run_ffmpeg(*inputs, *mix, "-ar", "44100", "-ac", "2", "-c:a", "pcm_s16le", target)


def find_structure(stem_files, work):
    """Finds the song structure from stems that are already separated."""
    import allin1_infer
    import torch
    from allin1_infer.models.allinone import AllInOne

    demix = work / "demix" / "htdemucs" / "input"
    demix.mkdir(parents=True)
    rest = [file for name, file in stem_files.items() if name not in STRUCTURE_STEMS[:3]]
    for count, name in enumerate(STRUCTURE_STEMS, 1):
        files = rest if name == "other" else [stem_files[name]]
        mixed_down(files, demix / f"{name}.wav")
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


def manifest_of(source, title, model, analysis, names):
    manifest = {
        "format_version": 1,
        "recording": {
            "content_hash": hashlib.sha256(source.read_bytes()).hexdigest(),
            "title": title,
        },
        "stems": [{"name": name, "file": f"stems/{name}.ogg"} for name in names],
    }
    if model:
        manifest["model"] = model
    if analysis:
        manifest["analysis"] = structure_of(analysis)
    return manifest


def write_package(source, target, title, model, analysis, stems, work):
    rate = sample_rate(source)
    names = [name for name in STEMS if (stems / f"{name}.wav").exists()]
    with zipfile.ZipFile(target, "w", zipfile.ZIP_STORED) as package:
        for count, name in enumerate(names, 1):
            encoded = work / f"{name}.ogg"
            run_ffmpeg(
                "-i", stems / f"{name}.wav", "-ar", rate, "-c:a", "libvorbis",
                "-q:a", QUALITY, encoded,
            )
            package.write(encoded, f"stems/{name}.ogg")
            progress(85 + count * 13 // len(names))
        manifest = manifest_of(source, title, model, analysis, names)
        package.writestr("manifest.json", json.dumps(manifest))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path, nargs="?")
    parser.add_argument("output", type=Path, nargs="?")
    parser.add_argument("title", nargs="?")
    parser.add_argument("--model", choices=MODELS)
    parser.add_argument("--from-results", type=Path)
    parser.add_argument("--structure", type=Path, metavar="OUTPUT.json")
    parser.add_argument("--stem", action="append", default=[], metavar="NAME=FILE")
    options = parser.parse_args()
    with tempfile.TemporaryDirectory() as temporary:
        work = Path(temporary)
        if options.structure:
            files = dict(entry.split("=", 1) for entry in options.stem)
            # The rest may be spread over several stems.
            missing = [name for name in STRUCTURE_STEMS if name not in files]
            if missing:
                parser.error(f"missing stems: {', '.join(missing)}")
            found = structure_of(find_structure(files, work))
            options.structure.write_text(json.dumps(found))
        elif options.title is None:
            parser.error("INPUT OUTPUT.zip TITLE are required")
        else:
            # Unknown for earlier results unless --model says it.
            model = options.model
            if options.from_results:
                found = options.from_results / "analysis.json"
                analysis = json.loads(found.read_text()) if found.exists() else None
                stems = options.from_results / "stems"
            else:
                model = model or "demucs4"
                analysis, stems = None, separate(options.input, work, model)
            write_package(
                options.input,
                options.output,
                options.title,
                model,
                analysis,
                stems,
                work,
            )
    progress(99)


def guarded():
    """Runs this program again as a process that is watched."""
    # The directory of temporary files goes away even if that process is killed.
    with tempfile.TemporaryDirectory() as temporary:
        environment = {**os.environ, "VELTRUNOX_GUARDED": "1", "TMPDIR": temporary}
        return guard.run([sys.executable, __file__, *sys.argv[1:]], environment)


if __name__ == "__main__":
    if os.environ.get("VELTRUNOX_GUARDED"):
        main()
    else:
        sys.exit(guarded())
