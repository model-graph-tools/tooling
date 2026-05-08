#!/usr/bin/env bash
set -euo pipefail

# Publishes platform-specific npm packages for mgt.
# Called by CI after binaries are uploaded to a GitHub Release.
#
# Usage: ./scripts/publish-npm.sh <version-tag>
# Example: ./scripts/publish-npm.sh v0.3.3

if [[ $# -ne 1 ]]; then
  echo "Usage: $0 <version-tag>" >&2
  exit 1
fi

VERSION_TAG="$1"
VERSION="${VERSION_TAG#v}"
REPO="model-graph-tools/tooling"
BASE_URL="https://github.com/${REPO}/releases/download/${VERSION_TAG}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
NPM_DIR="${SCRIPT_DIR}/../npm"
WORK_DIR="$(mktemp -d)"

trap 'rm -rf "${WORK_DIR}"' EXIT

# Map: npm-package-name -> rust-target -> archive-extension
declare -A TARGET_MAP=(
  ["mgt-linux-x64"]="x86_64-unknown-linux-gnu"
  ["mgt-linux-arm64"]="aarch64-unknown-linux-gnu"
  ["mgt-darwin-x64"]="x86_64-apple-darwin"
  ["mgt-darwin-arm64"]="aarch64-apple-darwin"
  ["mgt-win32-x64"]="x86_64-pc-windows-msvc"
)

for pkg in "${!TARGET_MAP[@]}"; do
  target="${TARGET_MAP[$pkg]}"
  pkg_dir="${NPM_DIR}/${pkg}"

  if [[ ! -d "${pkg_dir}" ]]; then
    echo "Skipping ${pkg}: directory not found" >&2
    continue
  fi

  # Determine archive format and binary name
  if [[ "${pkg}" == *win32* ]]; then
    archive="mgt-${target}.zip"
    binary="mgt.exe"
  else
    archive="mgt-${target}.tar.gz"
    binary="mgt"
  fi

  echo "--- Publishing ${pkg} (${target}) ---"

  # Download archive
  curl -fsSL -o "${WORK_DIR}/${archive}" "${BASE_URL}/${archive}"

  # Extract binary
  if [[ "${archive}" == *.zip ]]; then
    unzip -qo "${WORK_DIR}/${archive}" -d "${WORK_DIR}/${pkg}"
  else
    mkdir -p "${WORK_DIR}/${pkg}"
    tar -xzf "${WORK_DIR}/${archive}" -C "${WORK_DIR}/${pkg}"
  fi

  # Place binary
  cp "${WORK_DIR}/${pkg}/${binary}" "${pkg_dir}/bin/${binary}"
  chmod +x "${pkg_dir}/bin/${binary}"

  # Update version in package.json
  sed -i.bak "s/\"version\": \".*\"/\"version\": \"${VERSION}\"/" "${pkg_dir}/package.json"
  rm -f "${pkg_dir}/package.json.bak"

  # Publish
  (cd "${pkg_dir}" && npm publish --access public)

  # Clean up binary (don't commit it)
  rm -f "${pkg_dir}/bin/${binary}"

  echo "Published @model-graph-tools/${pkg}@${VERSION}"
done

echo "All platform packages published."
