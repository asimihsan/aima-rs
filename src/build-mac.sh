#!/usr/bin/env bash

set -euo pipefail

pyenv local aima-rs

export LIBTORCH=$(python -c 'import torch; from pathlib import Path; print(Path(torch.__file__).parent)')
export DYLD_LIBRARY_PATH=${LIBTORCH}/lib

cargo build --all
