/// Sections delimited by level 1–3 headings, with their first Verify line.
/// Fenced code is excluded from both heading and Verify recognition.
pub(crate) fn sections(input: &str) -> Vec<(&str, Option<&str>)> {
    let mut sections = Vec::new();
    let mut current = None;
    let mut verify = None;
    let mut fence = 0;
    for line in input.lines() {
        let trimmed = line.trim_start();
        let ticks = trimmed.bytes().take_while(|byte| *byte == b'`').count();
        if fence > 0 {
            if ticks >= fence && trimmed[ticks..].trim().is_empty() {
                fence = 0;
            }
            continue;
        }
        if ticks >= 3 {
            fence = ticks;
            continue;
        }
        if line.starts_with("# ") || line.starts_with("## ") || line.starts_with("### ") {
            if let Some(heading) = current {
                sections.push((heading, verify));
            }
            current = Some(line);
            verify = None;
        } else if current.is_some()
            && verify.is_none()
            && (trimmed.starts_with("- Verify:") || trimmed.starts_with("- **Verify:**"))
        {
            verify = Some(trimmed);
        }
    }
    if let Some(heading) = current {
        sections.push((heading, verify));
    }
    sections
}

pub(crate) fn valid_id(id: &str) -> bool {
    let bytes = id.as_bytes();
    let numeric = if bytes.last().is_some_and(u8::is_ascii_lowercase) {
        &id[..id.len() - 1]
    } else {
        id
    };
    let mut parts = numeric.split('.');
    let digits = |part: &str| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
    parts.next().is_some_and(digits) && parts.next().is_none_or(digits) && parts.next().is_none()
}

pub(crate) fn command(line: &str) -> Option<String> {
    let (_, after) = line.split_once('`')?;
    let (command, _) = after.split_once('`')?;
    Some(command.to_owned())
}
