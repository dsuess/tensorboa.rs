# pylint: disable=redefined-outer-name

from pathlib import Path
from typing import List, NamedTuple

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
    targets: List[np.ndarray]


# TODO tensorboardX seems to convert grayscale to RGB, so we need to come up
#      with a different testing-strategy for other image types
@pt.fixture(scope="function", params=[3, 4])
def tb_images(request, tmp_path: Path) -> ImageTestCase:
    outdir = tmp_path / "simple_scalar"
    n_channels = request.param
    # We use CHW channel order for convenience only. Internally, images are
    # stored in HWC and only converted back during conversion to array. All
    # images loaded by SummaryReader are contiguous in HWC channel order only!
    shape = (n_channels, 10, 10)

    targets = []
    with tbx.SummaryWriter(str(outdir)) as writer:
        for i in range(2):
            img = np.random.randint(0, 255, size=shape, dtype=np.uint8)
            targets.append(img)
            writer.add_image("img", img, global_step=i, dataformats="CHW")

    return ImageTestCase(str(detect_tbfile(outdir)), targets)


def test_summary_reader_image(
    tb_images: ImageTestCase,
):
    reader = SummaryReader(tb_images.path)
    imgs = list(reader)

    assert len(imgs) == len(tb_images.targets)
    for img, target in zip(imgs, tb_images.targets):
        np.testing.assert_array_equal(img, target)
        assert img.transpose((1, 2, 0)).flags["C_CONTIGUOUS"]
