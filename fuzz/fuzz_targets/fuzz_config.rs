#![no_main]

use ayame_spell_core::config::{validate_config, RawConfig};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let text = String::from_utf8_lossy(data);
    let _ = RawConfig::parse(&text);
    let _ = validate_config(&text);
});
