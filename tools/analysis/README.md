# Analysis program

Separates a recording into four stems (drums, bass, vocals, the rest) or six (also guitar and
piano) with Demucs v4, run by OpenVINO from [Intel's conversion](https://huggingface.co/Intel/demucs-openvino),
and, from those stems, finds the song structure (all-in-one-infer), for the stems and mixer of Veltrunox (ADR-015). It is optional
and runs outside the server.

```sh
tools/analysis/install.sh "$HOME/Library/Application Support/Veltrunox"   # macOS app
tools/analysis/install.sh data                                            # headless server
```

The server uses `<data_dir>/analysis/run`, or the program named by `analysis_program` in its
configuration. It splits with `PROGRAM INPUT OUTPUT.zip TITLE` (with `--stems 6` after it for six stems) and expects a stem package at
`OUTPUT.zip`; it creates regions with `PROGRAM --structure OUTPUT.json --stem NAME=FILE ...`
and expects the `analysis` object of the package manifest at `OUTPUT.json`. Both report
`progress N` lines on standard output. After an update run `install.sh` again.

The models are **not** part of Veltrunox. `install.sh` downloads the separation models (about
180 MB), the structure model is downloaded on first use. Check their terms before using them:
the structure model weights are CC-BY-NC-SA-4.0 (non-commercial). Intel labels its conversion
MIT, while the author of Demucs describes the original weights as provided only for scientific
purposes. Splitting a 3-minute song takes about a minute and up to about 5 GB of memory on an
Apple M3 with 8 GB; it does not fit a Raspberry Pi. Of the six stems, expect piano to be the
weakest. Packages made elsewhere can be imported in the mixer.
