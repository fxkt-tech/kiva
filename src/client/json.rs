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

/// A number, or `null` when it is not finite: JSON has no NaN or infinity.
#[cfg(any(target_arch = "wasm32", test))]
pub(super) fn number(value: f32) -> String {
    if value.is_finite() {
        value.to_string()
    } else {
        "null".into()
    }
}

/// A JSON object written field by field, in order.
#[cfg(any(target_arch = "wasm32", test))]
pub(super) struct Object(String);

#[cfg(any(target_arch = "wasm32", test))]
impl Object {
    pub(super) fn new() -> Self {
        Self(String::from("{"))
    }
    fn key(mut self, name: &str) -> Self {
        if self.0.len() > 1 {
            self.0.push(',');
        }
        self.0.push_str(&quote(name));
        self.0.push(':');
        self
    }
    /// `value` must already be JSON text.
    pub(super) fn raw(self, name: &str, value: &str) -> Self {
        let mut object = self.key(name);
        object.0.push_str(value);
        object
    }
    pub(super) fn number(self, name: &str, value: f32) -> Self {
        self.raw(name, &number(value))
    }
    pub(super) fn integer(self, name: &str, value: u32) -> Self {
        self.raw(name, &value.to_string())
    }
    pub(super) fn boolean(self, name: &str, value: bool) -> Self {
        self.raw(name, if value { "true" } else { "false" })
    }
    pub(super) fn string(self, name: &str, value: &str) -> Self {
        self.raw(name, &quote(value))
    }
    /// The object's JSON, or `null` for none.
    pub(super) fn optional(self, name: &str, value: Option<String>) -> Self {
        self.raw(name, value.as_deref().unwrap_or("null"))
    }
    pub(super) fn finish(mut self) -> String {
        self.0.push('}');
        self.0
    }
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

    #[test]
    fn objects_keep_field_order_and_write_non_finite_numbers_as_null() {
        let json = Object::new()
            .number("a", 0.25)
            .number("b", f32::NAN)
            .integer("c", 3)
            .boolean("d", true)
            .string("e", "\"")
            .optional("f", None)
            .raw("g", "[1]")
            .finish();
        assert_eq!(
            json,
            r#"{"a":0.25,"b":null,"c":3,"d":true,"e":"\"","f":null,"g":[1]}"#
        );
        assert_eq!(Object::new().finish(), "{}");
    }
}
