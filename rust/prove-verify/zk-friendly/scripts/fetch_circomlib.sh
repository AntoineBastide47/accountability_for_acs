#!/usr/bin/env bash
#
# circomlib is a circuit library, not a JavaScript dependency: `circom` needs
# its `.circom` sources on the include path. The Node stack got them from
# node_modules; here they are cloned into circom-libs/.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
LIB_DIR="${CIRCOM_LIB_PATH:-${ROOT_DIR}/circom-libs}"
TARGET="${LIB_DIR}/circomlib"
VERSION="${CIRCOMLIB_VERSION:-v2.0.5}"

if [[ -d "${TARGET}/circuits" ]]; then
  echo "circomlib already present at ${TARGET}"
  exit 0
fi

mkdir -p "${LIB_DIR}"
echo "Cloning circomlib ${VERSION} into ${TARGET} ..."
git clone --depth 1 --branch "${VERSION}" https://github.com/iden3/circomlib.git "${TARGET}"
rm -rf "${TARGET}/.git"
echo "Saved ${TARGET}"
