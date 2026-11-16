//! Small dependency-free JSON output for the page bridges.
use std::fmt::Write;

/// `value` as a JSON string literal; every control character is escaped.
pub(super) fn quote(value: &str) -> String {
    let mut text = String::from("\"");
    for c in value.chars() {
        match c {
            '"' => text.push_str("\\\""),
            '\\' => text.push_str("\\\\"),
            '\n' => text.push_str("\\n"),
            '\r' => text.push_str("\\r"),
            '\t' => text.push_str("\\t"),
            c if c < ' ' => {
                write!(text, "\\u{:04x}", c as u32).unwrap();
            }
            c => text.push(c),
        }
    }
    text.push('"');
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn escapes_all_json_control_characters_and_source_labels() {
        assert_eq!(
            quote("a\"b\\\n\t\r\u{1}帽"),
            "\"a\\\"b\\\\\\n\\t\\r\\u0001帽\""
        );
    }
}
