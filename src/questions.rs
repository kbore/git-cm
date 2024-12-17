use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use std::collections::HashMap;

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
pub fn ask(types: HashMap<&str, &str>) -> SurveyResults {
    let mut results = SurveyResults::new();

    // Select the scope of the commit.
    let type_options = types
        .iter()
        .map(|(name, desc)| (name, desc))
        .collect::<Vec<_>>();
    let items = type_options
        .iter()
        .map(|(name, desc)| format!("{:<10} [{}]", format!("{name}:"), desc))
        .collect::<Vec<_>>();

    let selected_index = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select the type of change that you're committing:")
        .default(0)
        .items(&items)
        .interact()
        .unwrap();
    let selected_commit_type = &type_options[selected_index];
    results.commit_type = (*selected_commit_type.0).to_string();

    let scope = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Denote the scope of this change (compiler, runtime, stdlib, etc.):")
        .allow_empty(true)
        .interact_text()
        .ok()
        .filter(|v: &String| !v.is_empty());
    results.scope = scope;

    let short_msg: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Write a short, imperative tense description of the change:")
        .allow_empty(true)
        .interact_text()
        .unwrap();
    results.short_msg = short_msg;

    let long_msg: Option<String> = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Provide a longer description of the change:")
        .allow_empty(true)
        .interact_text()
        .ok()
        .filter(|v: &String| !v.is_empty());
    results.long_msg = long_msg;

    let is_breaking_change = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Are there any breaking changes?")
        .default(false)
        .interact()
        .unwrap();

    if is_breaking_change {
        let breaking_changes_desc = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Describe the breaking changes:")
            .interact_text()
            .ok();
        results.breaking_changes_desc = breaking_changes_desc;
    }

    let are_issues_affected = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Does this change affect any open issues?")
        .default(false)
        .interact()
        .unwrap();

    if are_issues_affected {
        let affected_open_issues: Option<String> = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Add issue references (space-separated, e.g. \"#123\" or \"12 13\"):")
            .interact_text()
            .ok();
        results.affected_open_issues =
            affected_open_issues.map(|s| s.split(' ').map(|e| e.to_string()).collect());
    }

    results
}
