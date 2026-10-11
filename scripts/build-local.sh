#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
# Prefer the distro development package. If only its runtime library is present,
# supply a project-local linker alias; never modify /usr or disable seccomp.
if [[ -z "${LIBSECCOMP_LIB_PATH:-}" ]] && ! pkg-config --exists libseccomp; then
  seccomp_library=$(ldconfig -p | awk '/libseccomp.so.2.*x86-64/ && !found {found=$NF} END {print found}')
  if [[ -z "$seccomp_library" ]]; then
    echo 'Install libseccomp development libraries or use nix develop.' >&2
    exit 1
  fi
  mkdir -p work/native-lib
  ln -sfn "$seccomp_library" work/native-lib/libseccomp.so
  export LIBSECCOMP_LIB_PATH="$PWD/work/native-lib"
fi
cargo build --locked "$@"
