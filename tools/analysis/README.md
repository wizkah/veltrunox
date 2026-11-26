# Analysis program

Separates a recording into drums, bass, vocals and the rest (Demucs) and, from those stems,
finds the song structure (all-in-one-infer), for the stems and mixer of Veltrunox (ADR-015). It is optional
and runs outside the server.

```sh
tools/analysis/install.sh "$HOME/Library/Application Support/Veltrunox"   # macOS app
tools/analysis/install.sh data                                            # headless server
```

The server uses `<data_dir>/analysis/run`, or the program named by `analysis_program` in its
configuration. It splits with `PROGRAM INPUT OUTPUT.zip TITLE` and expects a stem package at
`OUTPUT.zip`; it creates regions with `PROGRAM --structure OUTPUT.json --stem NAME=FILE ...`
and expects the `analysis` object of the package manifest at `OUTPUT.json`. Both report
`progress N` lines on standard output. After an update run `install.sh` again.

The models are downloaded on first use and are **not** part of Veltrunox. Check their terms
before using them: the structure model weights are CC-BY-NC-SA-4.0 (non-commercial), the
Demucs weights are described by their authors as meant for research. A run of a 4.5-minute
song takes about a minute and up to about 3 GB of memory on an Apple M3; it does not fit a
Raspberry Pi. Packages made elsewhere can be imported in the mixer.
