#!/usr/bin/env bash
# Install cargo-mutants from its official release binary into ~/.cargo/bin after
# checking the sha256 (avoids a ~4 minute `cargo install` build on every run).
# The version and sha256 come from env vars set in .github/workflows/mutation.yml.
set -euo pipefail
: "${CARGO_MUTANTS_VERSION:?}" "${CARGO_MUTANTS_SHA256:?}"

work=$(mktemp -d)
curl -fsSL -o "${work}/a.tar.gz" \
  "https://github.com/sourcefrog/cargo-mutants/releases/download/v${CARGO_MUTANTS_VERSION}/cargo-mutants-x86_64-unknown-linux-gnu.tar.gz"
echo "${CARGO_MUTANTS_SHA256}  ${work}/a.tar.gz" | sha256sum -c -
tar -xzf "${work}/a.tar.gz" --no-same-owner -C "$work"
install -m 0755 "${work}/cargo-mutants" "${HOME}/.cargo/bin/cargo-mutants"

cargo mutants --version
