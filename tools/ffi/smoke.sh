#!/bin/sh
# Builds the C ABI static library and links a C program against include/zebra_mermaid.h.
set -eu
root=$(cd "$(dirname "$0")/../.." && pwd)
cd "$root"
cargo build -p zebra-mermaid-ffi --release
mkdir -p target/ffi-smoke
cc -Wall -Werror -I crates/zebra-mermaid-ffi/include tools/ffi/smoke.c target/release/libzebra_mermaid_ffi.a -o target/ffi-smoke/smoke
./target/ffi-smoke/smoke
