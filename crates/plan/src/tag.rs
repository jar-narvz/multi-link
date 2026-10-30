//! Explicit tags in raw launch text, using the PLAN.md identifier grammar.

use crate::markdown::valid_id;

/// Return the first valid step tag. Boundaries are ASCII, as in the schema.
pub fn step_tag(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let word = |byte: u8| byte.is_ascii_alphanumeric() || byte == b'_';
    for start in 0..bytes.len() {
        if (start > 0 && word(bytes[start - 1]))
            || !bytes[start..]
                .get(..4)
                .is_some_and(|tag| tag.eq_ignore_ascii_case(b"step"))
        {
            continue;
        }
        let mut id_start = start + 4;
        while bytes
            .get(id_start)
            .is_some_and(|byte| matches!(byte, b' ' | b'\t'))
        {
            id_start += 1;
        }
        if id_start == start + 4 {
            continue;
        }
        // Consume a whole candidate rather than accepting a valid prefix of an
        // invalid id. A dot belongs to the candidate only when followed by a digit.
        let mut end = id_start;
        while let Some(&byte) = bytes.get(end) {
            if word(byte) || (byte == b'.' && bytes.get(end + 1).is_some_and(u8::is_ascii_digit)) {
                end += 1;
            } else {
                break;
            }
        }
        let id = &text[id_start..end];
        if valid_id(id) {
            return Some(id.to_owned());
        }
    }
    None
}
