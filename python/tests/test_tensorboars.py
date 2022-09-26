import pytest

from tensorboars import SummaryReader


def test_summary_reader_implements_context_manager_protocol(
    tbfile_simple_scalars: str,
):
    # pylint: disable=unnecessary-dunder-call
    reader = SummaryReader(tbfile_simple_scalars)
    reader.__enter__()
    with pytest.raises(RuntimeError):
        reader.__enter__()

    reader = SummaryReader(tbfile_simple_scalars)
    with pytest.raises(RuntimeError):
        reader.__exit__(None, None, None)
