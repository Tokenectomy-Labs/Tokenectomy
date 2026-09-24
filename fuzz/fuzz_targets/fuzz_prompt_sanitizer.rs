#![no_main]
use libfuzzer_sys::fuzz_target;
use tokenectomy::proxy::sanitize_prompt_payload;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(s) {
            let _ = sanitize_prompt_payload(&val);
        }
    }
});
