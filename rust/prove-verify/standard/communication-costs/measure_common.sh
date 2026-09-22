#!/usr/bin/env bash
# Shared helpers for Longfellow proof-size measure scripts.
# Sourced by build_measure_longfellow_*.sh (expects SCRIPT_DIR already set).

STANDARD_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Longfellow is a vendored third-party tree and is not duplicated into the Rust
# stack, so the original Node checkout is the last candidate.
#   LONGFELLOW_ROOT   explicit override
#   <stack>/longfellow-zk        alongside the crate, as in the Docker image
#   <stack>                      flat layout (/bench in Docker)
#   <repo>/prove-verify/standard/longfellow-zk   the original checkout
if [[ -n "${LONGFELLOW_ROOT:-}" && -f "${LONGFELLOW_ROOT}/lib/CMakeLists.txt" ]]; then
  LF_ROOT="${LONGFELLOW_ROOT}"
elif [[ -f "${STANDARD_ROOT}/longfellow-zk/lib/CMakeLists.txt" ]]; then
  LF_ROOT="${STANDARD_ROOT}/longfellow-zk"
elif [[ -f "${STANDARD_ROOT}/lib/CMakeLists.txt" ]]; then
  LF_ROOT="${STANDARD_ROOT}"
elif [[ -f "${STANDARD_ROOT}/../../../prove-verify/standard/longfellow-zk/lib/CMakeLists.txt" ]]; then
  LF_ROOT="$(cd "${STANDARD_ROOT}/../../../prove-verify/standard/longfellow-zk" && pwd)"
else
  LF_ROOT="${STANDARD_ROOT}/longfellow-zk"
fi
LIB="${LF_ROOT}/lib"

if [[ -n "${LONGFELLOW_BUILD_DIR:-}" && -d "${LONGFELLOW_BUILD_DIR}" ]]; then
  BLD="${LONGFELLOW_BUILD_DIR}"
else
  BLD="${LF_ROOT}/clang-build-release"
fi

CXX_BIN="${CXX:-c++}"

# OpenSSL / zstd: Homebrew on macOS; system paths on Debian/Ubuntu (Docker).
OPENSSL_INC="${OPENSSL_INC:-}"
LINK_LIB_DIRS=()
if [[ -z "${OPENSSL_INC}" ]]; then
  if [[ -d /opt/homebrew/include/openssl ]]; then
    OPENSSL_INC="-I/opt/homebrew/include"
    LINK_LIB_DIRS+=("-L/opt/homebrew/lib")
  elif [[ -d /usr/local/opt/openssl@3/include ]]; then
    OPENSSL_INC="-I/usr/local/opt/openssl@3/include"
    LINK_LIB_DIRS+=("-L/usr/local/opt/openssl@3/lib")
  fi
fi
