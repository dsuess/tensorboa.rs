import shutil
from pathlib import Path

import tensorboardX as tbx


def shared_data_dir() -> Path:
    git_root = Path(__file__).parents[2]
    ddir: Path = git_root / "data"
    try:
        shutil.rmtree(ddir)
    except FileNotFoundError:
        pass
    ddir.mkdir(exist_ok=True)
    return ddir


def scalar_only_tbfile(datadir: Path):
    with tbx.SummaryWriter(str(datadir)) as writer:
        for i in range(10):
            print(i)
            writer.add_scalar("tag", i, global_step=i)


def main():
    datadir = shared_data_dir()
    scalar_only_tbfile(datadir)


if __name__ == "__main__":
    main()
