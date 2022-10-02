import numpy as np

from tensorboars import SummaryReader


def test_summary_reader_simple_scalar(
    tbfile_simple_scalars: str,
):
    reader = SummaryReader(tbfile_simple_scalars)
    np.testing.assert_array_equal(list(reader), np.linspace(0, 9, 10))
