#!/usr/bin/env bash
# Print the diff against the base commit, for cargo-mutants' --in-diff.
# Argument: the base commit (origin/<base branch> for pull_request, the pre-push
# SHA for push). Without a usable base (e.g. the first push of a branch), compare
# with the previous commit.
#
# Must run inside src-tauri/: cargo-mutants matches diff paths against paths
# relative to the crate (src/lib.rs). A repository-relative diff
# (src-tauri/src/lib.rs) matches nothing, and cargo-mutants then silently finds
# 0 mutants and exits 0 (measured 2026-11-27). `git diff --relative` strips the
# src-tauri/ prefix and drops changes outside src-tauri/.
set -euo pipefail
if [[ ! -f Cargo.toml || "$(git rev-parse --show-prefix)" != "src-tauri/" ]]; then
  echo "run this from src-tauri/" >&2
  exit 1
fi
base="${1:-}"
if [[ -z "$base" || "$base" =~ ^0+$ ]] || ! git cat-file -e "${base}^{commit}" 2>/dev/null; then
  echo "base '${base}' not found; comparing with HEAD^" >&2
  base="HEAD^"
fi
echo "base: $(git rev-parse "$base")" >&2
git diff --relative "${base}...HEAD"
