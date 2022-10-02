# pylint: disable=redefined-outer-name

import itertools as it
from pathlib import Path
from typing import NamedTuple, Tuple

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
    # TODO Test tags & step
    np.testing.assert_array_equal(list(reader), np.linspace(0, 9, 10))


class ImageTestCase(NamedTuple):
    path: str
    dtype: np.dtype
    n_channels: int
    shape: Tuple[int, int]


# TODO Add dataformats
@pt.fixture(scope="function", params=it.product([np.uint8, np.float32], [1, 3, 4]))
def tb_image(request, tmp_path: Path) -> ImageTestCase:
    outdir = tmp_path / "simple_scalar"
    dtype, n_channels = request.param
    with tbx.SummaryWriter(str(outdir)) as writer:
        for i in range(2):
            img = i * np.ones((n_channels, 10, 10), dtype=dtype)
            writer.add_image("img", img, global_step=i)

    return ImageTestCase(str(detect_tbfile(outdir)), dtype, n_channels, (10, 10))


@pt.mark.dev
def test_summary_reader_image(
    tb_image: ImageTestCase,
):
    reader = SummaryReader(tb_image.path)
    imgs = list(reader)

    shape = (*tb_image.shape, tb_image.n_channels)
    assert len(imgs) == 2
    np.testing.assert_array_equal(imgs[0], np.zeros(shape, dtype=tb_image.dtype))
    np.testing.assert_array_equal(imgs[1], np.ones(shape, dtype=tb_image.dtype))
