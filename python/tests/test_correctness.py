# pylint: disable=redefined-outer-name

from pathlib import Path

import numpy as np
import pytest as pt
import tensorboardX as tbx

from tensorboars import SummaryReader


def detect_tbfile(outdir: Path) -> Path:
    (match,) = outdir.glob("events.out.tfevents.*")
    return match


@pt.fixture(scope="function")
def tb_simple_value(tmp_path: Path) -> str:
    outdir = tmp_path / "simple_scalar"
    with tbx.SummaryWriter(str(outdir)) as writer:
        for i in range(10):
            writer.add_scalar("tag", i, global_step=i)

    return str(detect_tbfile(outdir))


def test_summary_reader_simple_scalar(
    tb_simple_value: str,
):
    reader = SummaryReader(tb_simple_value)
    np.testing.assert_array_equal(list(reader), np.linspace(0, 9, 10))
