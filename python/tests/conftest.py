# pylint: disable=redefined-outer-name
from pathlib import Path

import pytest


@pytest.fixture(scope="session")
def test_data_dir() -> Path:
    return Path(__file__).parents[2] / "data"


# https://tensorboard.dev/static/log_set-2021-03-11.zip
