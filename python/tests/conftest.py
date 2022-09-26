# pylint: disable=redefined-outer-name
from pathlib import Path

import pytest


@pytest.fixture(scope="session")
def test_data_dir() -> Path:
    return Path(__file__).parents[2] / "data"


@pytest.fixture(scope="session")
def tbfile_simple_scalars(test_data_dir: Path) -> str:
    return str(test_data_dir / "events.out.tfevents.1661684667.applepie4")
