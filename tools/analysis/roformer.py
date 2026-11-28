"""Separation with models of the RoFormer family (BS-RoFormer and Mel-Band RoFormer).

Each model is a directory with `model.ckpt` and `config.yaml`, in the form of
Music-Source-Separation-Training. The network code is openmirlab's."""
import contextlib
import inspect

import torch
import yaml

SAMPLE_RATE = 44100
# The attention's memory grows with the square of the chunk; longer chunks than this do not
# fit an 8 GB machine.
LONGEST_CHUNK = 8 * SAMPLE_RATE
# A model file this large leaves less room for the chunk.
LARGE_MODEL_BYTES = 2 * 1024**3
LONGEST_CHUNK_OF_A_LARGE_MODEL = 4 * SAMPLE_RATE


class Loader(yaml.SafeLoader):
    pass


Loader.add_constructor(
    "tag:yaml.org,2002:python/tuple",
    lambda loader, node: tuple(loader.construct_sequence(node)),
)


@contextlib.contextmanager
def without_initialisation():
    """Linear layers keep their memory untouched: every weight is replaced by the model
    file, and filling them first would take as much memory again."""
    initialise = torch.nn.Linear.reset_parameters
    torch.nn.Linear.reset_parameters = lambda _layer: None
    try:
        yield
    finally:
        torch.nn.Linear.reset_parameters = initialise


def add_skip_connections(network):
    """Every block also gets what all blocks before it put out, as in the training code's
    `skip_connection`, which openmirlab's networks leave out. For one chunk at a time."""
    earlier = []
    for number, (over_time, over_frequency) in enumerate(network.layers):

        def before(_module, arguments, number=number):
            if number == 0:
                earlier.clear()
                return None
            # Kept as [time, bands, features]; here it is [bands, time, features].
            return (arguments[0] + sum(earlier).transpose(0, 1),)

        over_time.register_forward_pre_hook(before)
        over_frequency.register_forward_hook(
            lambda _module, _arguments, output: earlier.append(output)
        )


def network_of(config):
    if "num_bands" in config["model"]:
        from mel_band_roformer.mel_band_roformer import MelBandRoformer as network
    else:
        from bs_roformer.bs_roformer import BSRoformer as network
    known = inspect.signature(network.__init__).parameters
    settings = {k: v for k, v in config["model"].items() if k in known}
    # A setting the network does not know would change the result unnoticed. Left aside are
    # those for training, those switched off and the one added below.
    unknown = [
        name
        for name, value in config["model"].items()
        if name not in known and value and name not in ("use_torch_checkpoint", "skip_connection")
    ]
    if unknown:
        raise SystemExit(f"the model needs settings that are not supported: {unknown}")
    with without_initialisation():
        made = network(**settings)
    if config["model"].get("skip_connection") and "skip_connection" not in known:
        add_skip_connections(made)
    return made


def load(directory):
    """The network, the names of what it puts out, and its configuration."""
    checkpoint, settings = directory / "model.ckpt", directory / "config.yaml"
    if not (checkpoint.exists() and settings.exists()):
        raise SystemExit(f"model missing in {directory} (see download_roformer.sh)")
    config = yaml.load(settings.read_text(), Loader=Loader)
    network = network_of(config)
    # weights_only: a model file must not be able to run code.
    weights = torch.load(checkpoint, map_location="cpu", weights_only=True, mmap=True)
    network.load_state_dict(weights.get("state_dict", weights))
    target = config["training"].get("target_instrument")
    names = [target] if target else list(config["training"]["instruments"])
    return network.eval(), names, config


def window_of(chunk, overlap):
    """Fades where neighbouring chunks overlap."""
    window = torch.ones(chunk)
    fade = torch.linspace(0, 1, overlap + 2)[1:-1]
    window[:overlap] = fade
    window[-overlap:] = fade.flip(0)
    return window


def in_chunks(network, mix, chunk, report):
    """[channels, samples] to [stems, channels, samples], chunk by chunk."""
    overlap = chunk // 4
    step = chunk - overlap
    length = mix.shape[-1]
    # Silence around the recording, so that its ends are not at a chunk's edge.
    padded = torch.nn.functional.pad(mix, (overlap, chunk))
    total, weight = None, torch.zeros(padded.shape[-1])
    window = window_of(chunk, overlap)
    for start in range(0, length + overlap, step):
        piece = padded[:, start : start + chunk]
        with torch.inference_mode():
            result = network(piece[None])[0]
        if result.dim() == 2:
            result = result[None]
        if total is None:
            total = torch.zeros(result.shape[0], mix.shape[0], padded.shape[-1])
        total[..., start : start + chunk] += result * window
        weight[start : start + chunk] += window
        report(min((start + step) / (length + overlap), 1))
    return (total / weight.clamp(min=1e-8))[..., overlap : overlap + length]


def separate(directory, mix, report):
    """Returns {name: [channels, samples]}; a model of one stem also gives the rest."""
    network, names, config = load(directory)
    # The network gives back whole hops only.
    hop = config["model"].get("stft_hop_length", 512)
    large = (directory / "model.ckpt").stat().st_size > LARGE_MODEL_BYTES
    longest = LONGEST_CHUNK_OF_A_LARGE_MODEL if large else LONGEST_CHUNK
    chunk = min(config["audio"]["chunk_size"], longest) // hop * hop
    stems = dict(zip(names, in_chunks(network, mix, chunk, report)))
    if len(stems) == 1:
        stems["other"] = mix - next(iter(stems.values()))
    return stems
