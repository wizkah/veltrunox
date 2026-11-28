"""Demucs v4 with the network run by OpenVINO, from Intel's conversion
(https://huggingface.co/Intel/demucs-openvino). That conversion leaves out the transforms
around the network; they are the ones of demucs-infer's HTDemucs.forward."""
from fractions import Fraction
from types import SimpleNamespace

import openvino
import openvino.opset15 as operations
import torch
from demucs_infer.htdemucs import HTDemucs

# The stems of each model, in the order of its outputs.
MODELS = {
    4: ("htdemucs_v4", ("drums", "bass", "other", "vocals")),
    6: ("htdemucs_v4_6s", ("drums", "bass", "other", "vocals", "guitar", "piano")),
}
# What the transforms of HTDemucs read from the model.
SPECTRUM = SimpleNamespace(nfft=4096, hop_length=1024, cac=True)


def normalised(values, dimensions):
    mean = values.mean(dim=dimensions, keepdim=True)
    deviation = values.std(dim=dimensions, keepdim=True)
    return (values - mean) / (1e-5 + deviation), mean, deviation


def avoid_transposed_convolution_1d(network):
    """OpenVINO's CPU plugin computes 1-D transposed convolutions wrongly on Apple Silicon
    (seen with 2026.2 and 2026.4). The same operation on a picture one pixel high is right,
    so each of them is rewritten as that."""
    height = operations.constant([2])
    for node in network.get_ordered_ops():
        if node.get_type_name() != "ConvolutionBackpropData":
            continue
        data, filters = node.input_value(0), node.input_value(1)
        if len(data.get_partial_shape()) != 3:
            continue
        attributes = node.get_attributes()
        flat = operations.convolution_backprop_data(
            operations.unsqueeze(data, height),
            operations.unsqueeze(filters, height),
            strides=[1, *attributes["strides"]],
            pads_begin=[0, *attributes["pads_begin"]],
            pads_end=[0, *attributes["pads_end"]],
            dilations=[1, *attributes["dilations"]],
            output_padding=[0, *attributes["output_padding"]],
        )
        node.output(0).replace(operations.squeeze(flat, height).output(0))
    network.validate_nodes_and_infer_types()
    return network


class OpenVinoDemucs(torch.nn.Module):
    """Has what demucs-infer's apply_model asks of a model."""

    samplerate = 44100
    audio_channels = 2
    segment = Fraction(39, 5)
    length = int(segment * samplerate)

    def __init__(self, models_directory, stem_count):
        super().__init__()
        folder, self.sources = MODELS[stem_count]
        path = models_directory / folder / "htdemucs_fwd.xml"
        if not path.exists():
            raise SystemExit(f"model missing: {path} (run install.sh)")
        core = openvino.Core()
        # Half precision is twice as fast, needs far less memory and gives the same stems
        # here; it is what Apple Silicon does unless told otherwise.
        self.network = core.compile_model(
            avoid_transposed_convolution_1d(core.read_model(str(path))), "CPU"
        )

    def valid_length(self, length):
        return self.length

    def forward(self, mix):
        """One segment: [batch, channels, length] to [batch, stems, channels, length]."""
        spectrum = HTDemucs._spec(SPECTRUM, mix)
        x, mean, deviation = normalised(HTDemucs._magnitude(SPECTRUM, spectrum), (1, 2, 3))
        xt, mean_t, deviation_t = normalised(mix, (1, 2))
        result = self.network({"x": x.numpy(), "xt": xt.numpy()})
        x = torch.from_numpy(result["x_out"].copy())
        xt = torch.from_numpy(result["xt_out"].copy())

        batch, _, frequencies, frames = x.shape
        stems = len(self.sources)
        x = x.view(batch, stems, -1, frequencies, frames)
        x = x * deviation[:, None] + mean[:, None]
        # Real and imaginary part lie next to each other in the channels.
        x = x.view(batch, stems, -1, 2, frequencies, frames).permute(0, 1, 2, 4, 5, 3)
        waves = HTDemucs._ispec(SPECTRUM, torch.view_as_complex(x.contiguous()), self.length)
        xt = xt.view(batch, stems, -1, self.length)
        return waves + xt * deviation_t[:, None] + mean_t[:, None]
