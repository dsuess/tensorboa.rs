from pathlib import Path

import pytest as pt
import tensorboardX as tbx
from tensorboard.backend.event_processing.event_accumulator import EventAccumulator
from tests.utils import detect_tbfile

from tensorboars import SummaryReader


@pt.mark.benchmark
@pt.mark.parametrize("implementation", ["this", "tensorboard"])
@pt.mark.parametrize("n_samples", [10000, 100000])
def test_benchmark_scalars(
    implementation: str, tmp_path: Path, n_samples: int, benchmark
):
    outdir = tmp_path / "simple_scalar"
    with tbx.SummaryWriter(str(outdir)) as writer:
        for i in range(n_samples):
            writer.add_scalar("tag", i, global_step=i)

    path = str(detect_tbfile(outdir))

    if implementation == "this":

        def run():
            reader = SummaryReader(path)
            list(reader)

    elif implementation == "tensorboard":

        def run():
            reader = EventAccumulator(path)
            reader.Reload()
            list(reader.Scalars("tag"))

    else:
        raise ValueError(f"Invalid {implementation=}")

    benchmark(run)
