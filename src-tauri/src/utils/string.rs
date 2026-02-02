/// Safely truncate a string to approximately the given byte length,
/// ensuring the result doesn't split a multi-byte UTF-8 character.
/// Returns the truncated string without an ellipsis suffix.
pub fn truncate_utf8(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }

    // Find the last character boundary that fits entirely within max_bytes
    let boundary = s
        .char_indices()
        .take_while(|(i, c)| i + c.len_utf8() <= max_bytes)
        .last()
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(0);

    &s[..boundary]
}

/// Safely truncate a string and append "..." if truncation occurred.
pub fn truncate_with_ellipsis(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        s.to_string()
    } else {
        format!("{}...", truncate_utf8(s, max_bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_ascii() {
        let s = "Hello, world!";
        assert_eq!(truncate_utf8(s, 5), "Hello");
        assert_eq!(truncate_utf8(s, 100), s);
    }

    #[test]
    fn test_truncate_utf8_boundary() {
        // "→" is 3 bytes (0xE2 0x86 0x92)
        let s = "abc→def";
        // 'a'=0, 'b'=1, 'c'=2, '→'=3..6, 'd'=6, 'e'=7, 'f'=8
        assert_eq!(truncate_utf8(s, 3), "abc"); // right before the arrow
        assert_eq!(truncate_utf8(s, 4), "abc"); // inside the arrow, should stop at 'c'
        assert_eq!(truncate_utf8(s, 5), "abc"); // still inside the arrow
        assert_eq!(truncate_utf8(s, 6), "abc→"); // includes the full arrow
        assert_eq!(truncate_utf8(s, 7), "abc→d");
    }

    #[test]
    fn test_truncate_with_ellipsis() {
        let s = "Hello, world!";
        assert_eq!(truncate_with_ellipsis(s, 5), "Hello...");
        assert_eq!(truncate_with_ellipsis(s, 100), "Hello, world!");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(truncate_utf8("", 5), "");
        assert_eq!(truncate_with_ellipsis("", 5), "");
    }
}
