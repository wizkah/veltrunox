# Analysis program

Separates a recording into stems with one of the models below and, from drums, bass, vocals and
the rest, finds the song structure (all-in-one-infer), for the stems and mixer of Veltrunox (ADR-015). It is optional
and runs outside the server.

```sh
tools/analysis/install.sh "$HOME/Library/Application Support/Veltrunox"   # macOS app
tools/analysis/install.sh data                                            # headless server
```

The server uses `<data_dir>/analysis/run`, or the program named by `analysis_program` in its
configuration. It splits with `PROGRAM INPUT OUTPUT.zip TITLE` (with `--model NAME` after it for another model than
`demucs4`) and expects a stem package at
`OUTPUT.zip`; it creates regions with `PROGRAM --structure OUTPUT.json --stem NAME=FILE ...`
and expects the `analysis` object of the package manifest at `OUTPUT.json`. Both report
`progress N` lines on standard output. After an update run `install.sh` again.

## Models

| Name | Model | Stems | Weights, terms as stated | Size |
|---|---|---|---|---|
| `demucs4` | Demucs v4, run by OpenVINO from [Intel's conversion](https://huggingface.co/Intel/demucs-openvino) | drums, bass, vocals, other | Intel: MIT; the Demucs author: only for scientific purposes | 106 MB |
| `demucs6` | the same | also guitar, piano (piano is weak) | as above | 76 MB |
| `bs4` | BS-RoFormer, [ZFTurbo v1.0.12](https://github.com/ZFTurbo/Music-Source-Separation-Training/releases/tag/v1.0.12) | drums, bass, vocals, other | none stated | 527 MB |
| `sw6` | BS-RoFormer SW, [mirror](https://huggingface.co/koidn/BS-RoFormer-SW-6stem) | also guitar, piano | unknown | 699 MB |
| `mel4` | Mel-Band RoFormer, [SYH99999](https://huggingface.co/SYH99999/MelBandRoformer4StemFTLarge) | drums, bass, vocals, other | Apache-2.0 | 3.8 GB |
| `vocals` | Mel-Band RoFormer, [Kimberley Jensen](https://huggingface.co/KimberleyJSN/melbandroformer) | vocals, other | MIT | 913 MB |

The models are **not** part of Veltrunox; check their terms before using them. The structure
model, downloaded on first use, is CC-BY-NC-SA-4.0 (non-commercial). `install.sh` downloads the
Demucs models. For RoFormer, download them first and name their directory:

```sh
tools/analysis/download_roformer.sh /path/to/models bs4 sw6 vocals
tools/analysis/install.sh DATA_DIR /path/to/models
```

Song structure needs drums, bass and vocals, so there are no regions for `vocals` stems.

## Resources

On an Apple M3 with 8 GB, Demucs splits a 3-minute song in about a minute. The RoFormer models
run on the CPU in chunks of at most 8 s and take several times as long. Splitting needs up to
about 5 GB of memory; it does not fit a Raspberry Pi. `guard.py` watches every run and kills it
when it needs more than three quarters of the physical memory (or
`VELTRUNOX_ANALYSIS_MEMORY_GB`), when the system reports critical memory pressure, or when the
system disk is nearly full. Packages made elsewhere can be imported in the mixer.
