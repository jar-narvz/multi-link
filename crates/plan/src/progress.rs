use crate::markdown::{command, sections, valid_id};
use multi_link_model::{Entry, Verify, VerifyResult, VerifySource};

/// Parse `(step_id, entry)` pairs in file order, including ids absent from PLAN.md.
pub fn parse_progress(input: &str) -> Vec<(String, Entry)> {
    sections(input)
        .into_iter()
        .filter_map(|(heading, verify)| {
            let (date, rest) = heading.strip_prefix("### ")?.split_once(" — Step ")?;
            if !valid_date(date) {
                return None;
            }
            let (id, title) = rest.split_once(": ")?;
            if !valid_id(id) {
                return None;
            }
            let agent = title
                .rsplit_once(" — ")
                .map(|(_, agent)| agent.trim().to_owned());
            Some((
                id.to_owned(),
                Entry {
                    date: date.to_owned(),
                    agent,
                    verify: parse_verify(verify),
                },
            ))
        })
        .collect()
}

fn valid_date(date: &str) -> bool {
    date.len() == 10
        && date.bytes().enumerate().all(|(index, byte)| {
            if index == 4 || index == 7 {
                byte == b'-'
            } else {
                byte.is_ascii_digit()
            }
        })
}

fn parse_verify(line: Option<&str>) -> Verify {
    let result_text = line
        .and_then(|line| line.split_once('→'))
        .map_or("", |(_, result)| result.trim());
    let lower = result_text.to_lowercase();
    let result = if lower.starts_with("pass") || lower.starts_with("ok") {
        VerifyResult::Pass
    } else if lower.starts_with("fail") {
        VerifyResult::Fail
    } else {
        VerifyResult::Unknown
    };
    Verify {
        command: line.and_then(command),
        result,
        result_text: result_text.to_owned(),
        source: VerifySource::Reported,
    }
}
