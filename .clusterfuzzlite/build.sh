#!/bin/bash -eu
cargo fuzz build -O
cp fuzz/target/x86_64-unknown-linux-gnu/release/fuzz_prompt_sanitizer $OUT/
cp fuzz/target/x86_64-unknown-linux-gnu/release/fuzz_chunked_decoder $OUT/
