use crate::markdown::{command, sections, valid_id};
use multi_link_model::Step;

/// Parse supported step headings in file order; no valid headings means unsupported.
pub fn parse_plan(input: &str) -> Option<Vec<Step>> {
    let steps: Vec<_> = sections(input)
        .into_iter()
        .filter_map(|(heading, verify)| {
            let (id, title) = heading.strip_prefix("### Step ")?.split_once(" — ")?;
            valid_id(id).then(|| Step {
                id: id.to_owned(),
                title: title.to_owned(),
                verify_command: verify.and_then(command),
                progress: Vec::new(),
            })
        })
        .collect();
    (!steps.is_empty()).then_some(steps)
}
