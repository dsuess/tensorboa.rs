from pathlib import Path


def detect_tbfile(outdir: Path) -> Path:
    (match,) = outdir.glob("events.out.tfevents.*")
    return match
