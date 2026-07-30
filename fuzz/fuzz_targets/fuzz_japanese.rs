#![no_main]

use ayame_spell_core::japanese::{
    consistency_issues, JapaneseChecker, JapaneseOptions, KatakanaOcc, KatakanaStyle,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let text = String::from_utf8_lossy(data);
    let checker = JapaneseChecker::new(KatakanaStyle::Consistency, JapaneseOptions::default());
    let mut issues = Vec::new();
    let mut occurrences: Vec<KatakanaOcc> = Vec::new();
    let mut offset = 0;
    for (line_index, raw_line) in text.split_inclusive('\n').enumerate() {
        let line = raw_line.trim_end_matches('\n').trim_end_matches('\r');
        checker.check_line(
            line,
            line_index as u32 + 1,
            offset,
            false,
            &mut issues,
            Some(&mut occurrences),
        );
        offset += raw_line.len();
    }
    issues.extend(consistency_issues(&occurrences));
    issues.extend(checker.document_issues(&text));
    for issue in issues {
        let end = issue.offset + issue.len;
        assert!(end <= text.len());
        assert!(text.is_char_boundary(issue.offset));
        assert!(text.is_char_boundary(end));
    }
});
