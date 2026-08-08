#!/usr/bin/env bash
#
# Build DevProjectsCleaner and copy the binaries into ./dist.
#
#   ./build.sh            build for the current host only
#   ./build.sh --all      cross-compile for every supported platform (uses zig)
#
# Cross-compiling (--all) requires:
#   - zig        (brew install zig  |  https://ziglang.org/download/)
#   - cargo-zigbuild
#     (cargo install cargo-zigbuild --locked)
#
# Windows ARM64 (aarch64-pc-windows-msvc) is NOT produced here: it needs an
# ARM64 Windows host with the MSVC linker. It is built by the GitHub Actions
# release workflow on a native windows-11-arm runner.
#
# The GitHub Actions release workflow (.github/workflows/release.yml) is the
# recommended way to produce all binaries on native runners per release.
set -euo pipefail

HOST=$(rustc -vV | awk '/^host:/ {print $2}')

if [[ "${1:-}" == "--all" ]]; then
    TARGETS=(
        "aarch64-apple-darwin"
        "x86_64-apple-darwin"
        "x86_64-unknown-linux-gnu"
        "aarch64-unknown-linux-gnu"
        "x86_64-pc-windows-gnu"
    )
    for t in "${TARGETS[@]}"; do
        rustup target add "$t" >/dev/null 2>&1 || true
    done

    # Zig cross-compiles everything from any host.
    BUILD=(cargo zigbuild --release)
else
    TARGETS=("$HOST")
    BUILD=(cargo build --release)
fi

mkdir -p bin

for t in "${TARGETS[@]}"; do
    echo "==> Building for $t"
    "${BUILD[@]}" --target "$t"

    BIN="DevProjectsCleaner"
    case "$t" in
        *-windows-*) BIN="DevProjectsCleaner.exe" ;;
    esac

    # Friendly artifact names, matching .github/workflows/release.yml
    case "$t" in
        aarch64-apple-darwin)   OUT="DevProjectsCleaner-macos-arm64" ;;
        x86_64-apple-darwin)    OUT="DevProjectsCleaner-macos-x86_64" ;;
        x86_64-unknown-linux-gnu) OUT="DevProjectsCleaner-linux-x86_64" ;;
        aarch64-unknown-linux-gnu) OUT="DevProjectsCleaner-linux-arm64" ;;
        x86_64-pc-windows-gnu)  OUT="DevProjectsCleaner-windows-x86_64.exe" ;;
        *)                      OUT="DevProjectsCleaner-$t" ;;
    esac
    OUT="bin/$OUT"

    cp "target/$t/release/$BIN" "$OUT"

    # macOS kills ad-hoc (linker) signed binaries if they are copied/modified
    # after linking, so re-sign every darwin artifact explicitly.
    if [[ "$t" == *apple-darwin ]]; then
        codesign --force --sign - "$OUT"
    fi

    echo "    -> $OUT"
done

echo "Done. Binaries are in ./bin"
