//! `file:` URI conversions for the LSP server.
//!
//! `lsp-types` 0.95 re-exported `url::Url`, whose `to_file_path` and
//! `from_file_path` did this. 0.97 models URIs with `fluent-uri` instead,
//! which deliberately has no filesystem knowledge, so the conversion lives
//! here.
//!
//! Only `file:` URIs map to a path. Editors also send schemes that have no
//! filesystem location at all (`untitled:` for unsaved buffers,
//! `vscode-notebook-cell:` for notebook cells); those yield `None` so callers
//! fall back to treating the document as pathless rather than inventing a
//! path.

use std::path::PathBuf;

use lsp_types::Uri;
use percent_encoding::percent_decode_str;

/// Characters escaped in a URI path segment. `/` is excluded because it is
/// the separator we are building with; everything with syntactic meaning in a
/// URI reference, plus `%` itself, is escaped so the result round-trips.
///
/// The server only ever decodes URIs the client sent, so the encoder below
/// exists to round-trip [`to_path`] in tests rather than to build URIs at
/// runtime.
#[cfg(test)]
const PATH_ESCAPE: &percent_encoding::AsciiSet = &percent_encoding::CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'[')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}');

/// Convert a `file:` URI to a local path.
///
/// Returns `None` for any other scheme, for a non-local authority
/// (`file://host/share` on a Unix host), and for percent-escapes that are not
/// valid UTF-8.
pub fn to_path(uri: &Uri) -> Option<PathBuf> {
    // A relative reference has no scheme at all, so this is an `Option`.
    if !uri
        .scheme()
        .is_some_and(|scheme| scheme.as_str().eq_ignore_ascii_case("file"))
    {
        return None;
    }

    let authority = uri
        .authority()
        .map(|authority| authority.as_str())
        .unwrap_or_default();
    let is_local = authority.is_empty() || authority.eq_ignore_ascii_case("localhost");

    let decoded = decode(uri.path().as_str())?;

    if !is_local {
        // A UNC share is only meaningful on Windows; elsewhere there is no
        // path that names it.
        if cfg!(windows) {
            let host = decode(authority)?;
            return Some(PathBuf::from(format!(
                r"\\{host}{}",
                decoded.replace('/', r"\")
            )));
        }
        return None;
    }

    Some(PathBuf::from(strip_drive_prefix(&decoded)))
}

/// Build a `file:` URI for `path`, which must be absolute.
#[cfg(test)]
fn from_path(path: &std::path::Path) -> Option<Uri> {
    if !path.is_absolute() {
        return None;
    }
    let text = path.to_str()?;

    // Windows paths are backslash-separated and start with a drive letter or
    // a UNC prefix; URI paths always use `/` and always start with one.
    let text = text.replace('\\', "/");
    let (authority, text) = match text.strip_prefix("//") {
        Some(unc) => match unc.split_once('/') {
            Some((host, rest)) => (host.to_string(), format!("/{rest}")),
            None => (unc.to_string(), "/".to_string()),
        },
        None => (String::new(), text),
    };
    let text = if text.starts_with('/') {
        text
    } else {
        format!("/{text}")
    };

    let encoded: String = text
        .split('/')
        .map(|segment| percent_encoding::utf8_percent_encode(segment, PATH_ESCAPE).to_string())
        .collect::<Vec<_>>()
        .join("/");
    format!("file://{authority}{encoded}").parse().ok()
}

/// Build a `file:` URI for a directory, with the trailing slash LSP clients
/// use for workspace folders.
#[cfg(test)]
fn from_directory_path(path: &std::path::Path) -> Option<Uri> {
    let uri = from_path(path)?;
    let text = uri.as_str();
    if text.ends_with('/') {
        return Some(uri);
    }
    format!("{text}/").parse().ok()
}

fn decode(text: &str) -> Option<String> {
    Some(percent_decode_str(text).decode_utf8().ok()?.into_owned())
}

/// `file:///C:/dir` has the URI's leading `/` in front of the drive letter.
fn strip_drive_prefix(path: &str) -> &str {
    let mut chars = path.chars();
    if chars.next() != Some('/') {
        return path;
    }
    let Some(drive) = chars.next() else {
        return path;
    };
    if !drive.is_ascii_alphabetic() {
        return path;
    }
    // Accept both the `C:` form and the legacy `C|` encoding.
    match chars.next() {
        Some(':') | Some('|') => &path[1..],
        _ => path,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn uri(text: &str) -> Uri {
        text.parse().expect("valid URI")
    }

    #[test]
    fn unix_paths_round_trip() {
        let path = Path::new("/home/user/project/input.md");
        let built = from_path(path).expect("absolute path");
        assert_eq!(built.as_str(), "file:///home/user/project/input.md");
        assert_eq!(to_path(&built).as_deref(), Some(path));
    }

    #[test]
    fn spaces_and_non_ascii_are_escaped_and_decoded() {
        let path = Path::new("/home/user/my docs/日本語 #1.md");
        let built = from_path(path).expect("absolute path");
        assert_eq!(
            built.as_str(),
            "file:///home/user/my%20docs/%E6%97%A5%E6%9C%AC%E8%AA%9E%20%231.md"
        );
        assert_eq!(to_path(&built).as_deref(), Some(path));
    }

    #[test]
    fn percent_signs_in_names_round_trip() {
        let path = Path::new("/tmp/100%25/a%2Fb.md");
        let built = from_path(path).expect("absolute path");
        assert_eq!(to_path(&built).as_deref(), Some(path));
    }

    #[test]
    fn windows_shaped_uris_drop_the_leading_slash() {
        assert_eq!(
            to_path(&uri("file:///C:/Users/me/input.md")),
            Some(PathBuf::from("C:/Users/me/input.md"))
        );
        // `|` is not legal in a URI, so the legacy drive form always arrives
        // percent-encoded.
        assert_eq!(
            to_path(&uri("file:///c%7C/Users/me/input.md")),
            Some(PathBuf::from("c|/Users/me/input.md"))
        );
    }

    #[test]
    fn a_leading_slash_before_a_non_drive_is_kept() {
        assert_eq!(
            to_path(&uri("file:///etc/hosts")),
            Some(PathBuf::from("/etc/hosts"))
        );
    }

    #[test]
    fn localhost_authority_is_local() {
        assert_eq!(
            to_path(&uri("file://localhost/etc/hosts")),
            Some(PathBuf::from("/etc/hosts"))
        );
    }

    #[test]
    fn pathless_schemes_have_no_path() {
        assert_eq!(to_path(&uri("untitled:Untitled-1")), None);
        assert_eq!(
            to_path(&uri("vscode-notebook-cell:/a/b.ipynb#W0sZmlsZQ")),
            None
        );
        assert_eq!(to_path(&uri("https://example.com/a.md")), None);
    }

    #[test]
    fn the_scheme_comparison_ignores_case() {
        assert_eq!(
            to_path(&uri("FILE:///etc/hosts")),
            Some(PathBuf::from("/etc/hosts"))
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn a_remote_share_has_no_local_path() {
        assert_eq!(to_path(&uri("file://server/share/input.md")), None);
    }

    #[test]
    fn relative_paths_are_rejected() {
        assert_eq!(from_path(Path::new("relative/input.md")), None);
    }

    #[test]
    fn directory_uris_end_with_a_slash() {
        let built = from_directory_path(Path::new("/home/user/project")).expect("absolute path");
        assert_eq!(built.as_str(), "file:///home/user/project/");
    }
}
