#![no_main]
use libfuzzer_sys::fuzz_target;
use tokenectomy::proxy::decode_chunked_body;

fuzz_target!(|data: &[u8]| {
    let _ = decode_chunked_body(data);
});
