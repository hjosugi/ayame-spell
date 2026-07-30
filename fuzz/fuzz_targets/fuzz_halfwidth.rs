#![no_main]

use ayame_spell_core::japanese::halfwidth_to_fullwidth;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let text = String::from_utf8_lossy(data);
    let converted = halfwidth_to_fullwidth(&text);
    assert!(std::str::from_utf8(converted.as_bytes()).is_ok());
});
