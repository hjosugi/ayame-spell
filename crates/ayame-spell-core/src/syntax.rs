//! Lightweight syntax masking that preserves byte offsets.
//!
//! This deliberately uses bounded heuristics instead of parser trees: the
//! checker stays dependency-light and handles incomplete files, while users
//! can select `all` when they want every token checked.

use std::path::Path;

use crate::config::SyntaxProfile;

const MARKUP_EXTENSIONS: [&str; 3] = ["md", "markdown", "mdx"];
const SOURCE_EXTENSIONS: [&str; 24] = [
    "c", "cc", "cpp", "cs", "css", "go", "h", "hpp", "java", "js", "jsx", "kt", "kts", "lua",
    "php", "py", "rb", "rs", "scala", "sh", "swift", "ts", "tsx", "vue",
];

pub fn mask(text: &str, path: Option<&Path>, profile: SyntaxProfile) -> String {
    let extension = path
        .and_then(Path::extension)
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase);
    let selected = match profile {
        SyntaxProfile::Auto => match extension.as_deref() {
            Some(extension) if MARKUP_EXTENSIONS.contains(&extension) => SyntaxProfile::Prose,
            Some(extension) if SOURCE_EXTENSIONS.contains(&extension) => SyntaxProfile::Source,
            _ => SyntaxProfile::All,
        },
        profile => profile,
    };
    match selected {
        SyntaxProfile::Prose
            if extension
                .as_deref()
                .is_some_and(|extension| MARKUP_EXTENSIONS.contains(&extension)) =>
        {
            mask_markdown(text)
        }
        SyntaxProfile::Source => mask_source(text, extension.as_deref()),
        SyntaxProfile::Auto | SyntaxProfile::Prose | SyntaxProfile::All => text.to_string(),
    }
}

fn mask_markdown(text: &str) -> String {
    let mut keep = vec![true; text.len()];
    let mut fenced: Option<(u8, usize)> = None;
    let mut front_matter = text.lines().next().is_some_and(|line| line.trim() == "---");
    let mut offset = 0usize;

    for raw_line in text.split_inclusive('\n') {
        let line = raw_line.trim_end_matches(['\n', '\r']);
        let line_end = offset + line.len();
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();

        if front_matter {
            if offset == 0 || trimmed == "---" || trimmed == "..." {
                set_range(&mut keep, offset, line_end, false);
                if offset > 0 {
                    front_matter = false;
                }
            } else if let Some(colon) = line.find(':') {
                set_range(&mut keep, offset, offset + colon + 1, false);
            }
            offset += raw_line.len();
            continue;
        }

        let fence = fence_marker(trimmed);
        if let Some((marker, width)) = fenced {
            set_range(&mut keep, offset, line_end, false);
            if fence.is_some_and(|(candidate, count)| candidate == marker && count >= width) {
                fenced = None;
            }
            offset += raw_line.len();
            continue;
        }
        if let Some(marker) = fence {
            set_range(&mut keep, offset + indent, line_end, false);
            fenced = Some(marker);
            offset += raw_line.len();
            continue;
        }

        mask_inline_code(line, offset, &mut keep);
        mask_link_targets(line, offset, &mut keep);
        offset += raw_line.len();
    }
    render_mask(text, &keep)
}

fn fence_marker(line: &str) -> Option<(u8, usize)> {
    let marker = *line.as_bytes().first()?;
    if !matches!(marker, b'`' | b'~') {
        return None;
    }
    let width = line.bytes().take_while(|byte| *byte == marker).count();
    (width >= 3).then_some((marker, width))
}

fn mask_inline_code(line: &str, line_offset: usize, keep: &mut [bool]) {
    let bytes = line.as_bytes();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        if bytes[cursor] != b'`' || (cursor > 0 && bytes[cursor - 1] == b'\\') {
            cursor += 1;
            continue;
        }
        let width = bytes[cursor..]
            .iter()
            .take_while(|byte| **byte == b'`')
            .count();
        let mut end = cursor + width;
        while end + width <= bytes.len() {
            if bytes[end..end + width].iter().all(|byte| *byte == b'`') {
                end += width;
                set_range(keep, line_offset + cursor, line_offset + end, false);
                cursor = end;
                break;
            }
            end += 1;
        }
        if cursor < end {
            cursor += 1;
        }
    }
}

fn mask_link_targets(line: &str, line_offset: usize, keep: &mut [bool]) {
    let bytes = line.as_bytes();
    let mut cursor = 0usize;
    while cursor + 1 < bytes.len() {
        if bytes[cursor] == b']' && bytes[cursor + 1] == b'(' {
            if let Some(relative_end) = bytes[cursor + 2..].iter().position(|byte| *byte == b')') {
                let end = cursor + 2 + relative_end + 1;
                set_range(keep, line_offset + cursor + 1, line_offset + end, false);
                cursor = end;
                continue;
            }
        }
        cursor += 1;
    }
}

fn mask_source(text: &str, extension: Option<&str>) -> String {
    let bytes = text.as_bytes();
    let mut keep = vec![false; bytes.len()];
    for (index, byte) in bytes.iter().enumerate() {
        if *byte == b'\n' || *byte == b'\r' {
            keep[index] = true;
        }
    }

    let hash_comments = matches!(
        extension,
        Some("py" | "rb" | "sh" | "php" | "yaml" | "yml" | "toml")
    );
    let mut cursor = 0usize;
    let mut block_comment = false;
    while cursor < bytes.len() {
        if block_comment {
            let end = find_bytes(bytes, cursor, b"*/").map_or(bytes.len(), |end| end + 2);
            set_range(&mut keep, cursor, end, true);
            block_comment = end == bytes.len();
            cursor = end;
            continue;
        }
        if bytes[cursor..].starts_with(b"/*") {
            let end = find_bytes(bytes, cursor + 2, b"*/").map_or(bytes.len(), |end| end + 2);
            set_range(&mut keep, cursor, end, true);
            block_comment = end == bytes.len();
            cursor = end;
            continue;
        }
        if bytes[cursor..].starts_with(b"//") || (hash_comments && bytes[cursor] == b'#') {
            let end = bytes[cursor..]
                .iter()
                .position(|byte| *byte == b'\n')
                .map_or(bytes.len(), |end| cursor + end);
            set_range(&mut keep, cursor, end, true);
            cursor = end;
            continue;
        }
        if let Some(delimiter) = string_delimiter(bytes, cursor) {
            let end = find_string_end(bytes, cursor, delimiter);
            set_range(&mut keep, cursor, end, true);
            cursor = end;
            continue;
        }
        cursor += 1;
    }
    render_mask(text, &keep)
}

fn string_delimiter(bytes: &[u8], cursor: usize) -> Option<&'static [u8]> {
    if bytes[cursor..].starts_with(b"\"\"\"") {
        Some(b"\"\"\"")
    } else if bytes[cursor..].starts_with(b"'''") {
        Some(b"'''")
    } else {
        match bytes[cursor] {
            b'"' => Some(b"\""),
            b'\'' => Some(b"'"),
            b'`' => Some(b"`"),
            _ => None,
        }
    }
}

fn find_string_end(bytes: &[u8], start: usize, delimiter: &[u8]) -> usize {
    let mut cursor = start + delimiter.len();
    while cursor < bytes.len() {
        if delimiter.len() == 1 && bytes[cursor] == b'\\' {
            cursor = (cursor + 2).min(bytes.len());
            continue;
        }
        if bytes[cursor..].starts_with(delimiter) {
            return cursor + delimiter.len();
        }
        cursor += 1;
    }
    bytes.len()
}

fn find_bytes(haystack: &[u8], start: usize, needle: &[u8]) -> Option<usize> {
    haystack[start..]
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|position| start + position)
}

fn set_range(keep: &mut [bool], start: usize, end: usize, value: bool) {
    if let Some(range) = keep.get_mut(start.min(keep.len())..end.min(keep.len())) {
        range.fill(value);
    }
}

fn render_mask(text: &str, keep: &[bool]) -> String {
    let mut output = String::with_capacity(text.len());
    for (offset, character) in text.char_indices() {
        if character == '\n' || character == '\r' || keep[offset] {
            output.push(character);
        } else {
            output.push(match character.len_utf8() {
                1 => ' ',
                2 => '\u{00a0}',
                3 => '\u{2003}',
                _ => '\u{10000}',
            });
        }
    }
    debug_assert_eq!(output.len(), text.len());
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_masks_code_targets_and_front_matter_keys() {
        let text = "---\ntitle: A recieve guide\nslug: recieve\n---\n\
                    Prose recieve and `inline recieve`.\n\
                    [recieve label](https://recieve.example)\n\
                    ```rust\nlet recieve = 1;\n```\n";
        let masked = mask(text, Some(Path::new("guide.md")), SyntaxProfile::Auto);
        assert_eq!(masked.len(), text.len());
        assert!(masked.contains("A recieve guide"));
        assert!(masked.contains("Prose recieve"));
        assert!(masked.contains("recieve label"));
        assert!(!masked.contains("inline recieve"));
        assert!(!masked.contains("https://recieve.example"));
        assert!(!masked.contains("let recieve"));
        assert!(!masked.contains("slug:"));
    }

    #[test]
    fn source_masks_identifiers_but_keeps_comments_and_strings() {
        let text = "let recieve = \"recieve string\"; // recieve comment\n\
                    /* recieve block */\n";
        let masked = mask(text, Some(Path::new("main.rs")), SyntaxProfile::Auto);
        assert_eq!(masked.len(), text.len());
        assert!(!masked.contains("let recieve"));
        assert!(masked.contains("recieve string"));
        assert!(masked.contains("recieve comment"));
        assert!(masked.contains("recieve block"));
    }
}
