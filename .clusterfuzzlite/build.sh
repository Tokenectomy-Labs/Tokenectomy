#!/bin/bash -eu
cd $SRC/tokenectomy
cargo fuzz build -O

# Resolve fuzz binary output path
FUZZ_DIR="fuzz/target/x86_64-unknown-linux-gnu/release"
if [ ! -d "$FUZZ_DIR" ]; then
    FUZZ_DIR="target/x86_64-unknown-linux-gnu/release"
fi

cp "$FUZZ_DIR/fuzz_prompt_sanitizer" $OUT/
cp "$FUZZ_DIR/fuzz_chunked_decoder" $OUT/
