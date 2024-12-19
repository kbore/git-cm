use std::collections::HashMap;

use indexmap::IndexMap;
use inquire::{
    error::{CustomUserError, InquireResult},
    min_length, required, Confirm, CustomType, Select, Text,
};

/// The result of the questioning process.
#[derive(Debug, Default)]
pub struct SurveyResults {
    pub commit_type: String,
    pub scope: Option<String>,
    pub short_msg: String,
    pub long_msg: Option<String>,
    pub breaking_changes_desc: Option<String>,
    pub affected_open_issues: Option<Vec<String>>,
}

impl SurveyResults {
    /// Creates a default `SurveyResult`.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Asks the user all needed questions.
///
/// # Arguments
///
/// - `types`: A `HashMap` whose keys are the commit types and values are the
///   descriptions of the type.
///
/// # Returns
///
/// A `SurveyResult`.
pub fn ask(types: IndexMap<&str, &str>) -> SurveyResults {
    let mut results = SurveyResults::new();

    // <name####desc, desc>
    let commit_type_map: IndexMap<String, &str> = types
        .iter()
        .map(|(&name, &desc)| (format!("{:<10} [{}]", format!("{name}:"), desc), name))
        .collect();

    let items = commit_type_map.iter().map(|(key, _)| key).collect();

    let selected_item = Select::new("Select the type of change that you're committing:", items)
        .with_page_size(8)
        .with_starting_cursor(0)
        .with_formatter(&|show_item| commit_type_map[*show_item.value].to_string())
        .with_help_message("Use arrow keys to select")
        .prompt()
        .unwrap();

    results.commit_type = commit_type_map[selected_item].to_string();

    let scope = Text::new("What is the scope of this change (e.g. component or file name):")
        .with_validator(required!())
        .prompt()
        .ok();
    results.scope = scope;

    let short_msg = Text::new("Write a short, imperative tense description of the change:")
        .with_validator(min_length!(5))
        .prompt()
        .unwrap();
    results.short_msg = short_msg;

    let long_msg = Text::new("Provide a longer description of the change:")
        .with_help_message("Press enter to skip")
        .prompt()
        .ok();
    results.long_msg = long_msg;

    let is_breaking_change = Confirm::new("Are there any breaking changes?")
        .with_default(false)
        .prompt()
        .unwrap();

    if is_breaking_change {
        let breaking_changes_desc = Text::new("Describe the breaking changes:")
            .with_validator(min_length!(5))
            .prompt()
            .ok();
        results.breaking_changes_desc = breaking_changes_desc;
    }

    let are_issues_affected = Confirm::new("Does this change affect any open issues?")
        .with_default(false)
        .prompt()
        .unwrap();

    if are_issues_affected {
        let affected_open_issues =
            Text::new(r##"Add issue references (space-separated, e.g. "#123" or "12 13")"##)
                .with_validator(min_length!(2))
                .prompt()
                .ok();
        results.affected_open_issues =
            affected_open_issues.map(|s| s.split(' ').map(|e| e.to_string()).collect());
    }
    results
}
