use crate::{
    args::App,
    git::{
        check_staged_files_exist, commit_to_repo, generate_commit_msg, get_repository,
        hooks_post_commit, hooks_pre_commit, DEFAULT_TYPES,
    },
    questions::{ask, SurveyResults},
};
use clap::Parser;
use indexmap::IndexMap;
use std::path::Path;

mod args;
mod cargo;
mod git;
mod questions;
use std::process::Command;

fn run_dialog() -> Option<SurveyResults> {
    let mut types: IndexMap<&str, &str> = IndexMap::with_capacity(10);
    types.extend(&*DEFAULT_TYPES);

    Some(ask(types))
}

// fn create_commit(commit_msg: &str, repo: &Path) {
//     let hash = commit_to_repo(commit_msg, repo).expect("Failed to create commit");
//     println!("Wrote commit: {}", hash);
// }

fn create_commit(commit_msg: &str, repo: &Path) {
    Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(format!("{commit_msg}"))
        .status()
        .expect("git command failed");
    // let hash = commit_to_repo(commit_msg, repo).expect("Failed to create commit");
    // println!("Wrote commit: {}", hash);
}

fn run(app: App) {
    // No point to continue if repo doesn't exist or there are no staged files
    if check_staged_files_exist(app.repo_path.as_path()) {
        let survey = run_dialog();
        let commit_msg = survey.map(generate_commit_msg).and_then(|msg| {
            if app.edit {
                edit::edit(msg).ok()
            } else {
                Some(msg)
            }
        });

        match commit_msg {
            Some(msg) => create_commit(&msg, app.repo_path.as_path()),
            None => eprintln!("Empty commit message specified!"),
        }
    } else {
        eprintln!("Nothing to commit!");
    }
}

fn print_version() {
    let exe_path = std::env::current_exe().unwrap();
    let exe_name = exe_path.file_name().unwrap().to_str().unwrap();

    println!("{exe_name} version 1.0.0")
}

fn main() {
    let app: App = App::parse();

    if app.show_version {
        print_version();
        return;
    }

    // Early return if the path doesn't exist.
    if !app.repo_path.exists() || get_repository(app.repo_path.as_path()).is_err() {
        eprintln!("Invalid path to repository: {}", app.repo_path.display());
    } else {
        // When terminating the CLI during the dialoguer phase, the cursor will be
        // hidden. The callback here makes sure that the cursor is visible in these
        // cases.
        // let _ = ctrlc::set_handler(move || {
        //     let term = dialoguer::console::Term::stderr();
        //     let _ = term.show_cursor();
        //     std::process::exit(1);
        // });
        run(app);
    }
}
