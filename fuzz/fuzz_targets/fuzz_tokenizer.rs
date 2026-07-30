#![no_main]

use ayame_spell_core::tokenizer::{words_in_line, TokenizerOptions};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let text = String::from_utf8_lossy(data);
    let options = TokenizerOptions::default();
    for line in text.lines() {
        let words = words_in_line(line, &options);
        for word in words {
            assert!(line.is_char_boundary(word.start));
            assert!(line.is_char_boundary(word.start + word.text.len()));
            assert_eq!(&line[word.start..word.start + word.text.len()], word.text);
        }
    }
});
