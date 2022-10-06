setup:
    python -m pip install -U pip
    python -m pip install -r requirements.dev.txt
    python -m pip install -e .
    mkdir -p .git/hooks
    ln -f -s `pwd`/hooks/* .git/hooks

test:
    maturin develop
    pytest

dev:
    maturin develop
    pytest -m dev

lint:
    isort --verbose python/tensorboars python/tests
    black --check --include .py --exclude ".pyc|.pyi|.so" python/tensorboars python/tests
    black --check --pyi --include .pyi --exclude ".pyc|.py|.so" python/tensorboars python/tests
    cargo fmt --check
    ruff python/
    pyright python/tensorboars python/tests

fix:
   isort python/tensorboars python/tests
   black --include .py --exclude ".pyc|.pyi|.so" python/tensorboars python/tests
   black --pyi --include .pyi --exclude ".pyc|.py|.so" python/tensorboars python/tests
   cargo fmt
   ruff --fix python/

list-todo:
    pylint --disable=all --enable=fixme --score=no python/tensorboars python/tests