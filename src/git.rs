use crate::questions::SurveyResults;
use anyhow::{anyhow, Result};
use git2::{Commit, Error, ObjectType, Oid, Repository, RepositoryOpenFlags, Status};
use once_cell::sync::{Lazy, OnceCell};
use std::{collections::HashMap, ffi::OsStr, path::Path, sync::Mutex};

/// All default conventional commit types alongside their description.
pub static DEFAULT_TYPES: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    let mut m = HashMap::new();

    m.insert("fix", "修复bug: 对应CMM问题单或issue");
    m.insert("feat", "新增特性：新增软件功能或适配新硬件");
    m.insert("build", "构建系统修改: Makefile, CMake, pipeline, hook等");
    m.insert("refactor", "重构代码: 既不修复bug, 也不新增特性");
    m.insert("perf", "提升性能: 建议先做profiling再进行代码优化性能");
    m.insert("chore", "杂项修改：代码注释，空格等不影响代码的修改");
    m.insert("test", "测试功能修改: 单元测试, 集成测试等测试代码修改");

    m
});

// Singleton pattern
pub fn get_repository(repo: &Path) -> Result<&Mutex<Repository>, Error> {
    static REPO: OnceCell<Mutex<Repository>> = OnceCell::new();
    REPO.get_or_try_init(|| {
        let repo = Repository::open_ext(
            repo.as_os_str(),
            RepositoryOpenFlags::empty(),
            vec![OsStr::new("")],
        );
        match repo {
            Ok(r) => Ok(Mutex::new(r)),
            Err(e) => Err(e),
        }
    })
}

pub fn check_staged_files_exist(repo: &Path) -> bool {
    let res;
    let repo = get_repository(repo).unwrap().lock().unwrap();
    match repo.statuses(Option::None) {
        Ok(s) => {
            res = s.iter().fold(false, |acc, se| {
                acc | se.status().intersects(
                    Status::INDEX_NEW
                        | Status::INDEX_MODIFIED
                        | Status::INDEX_DELETED
                        | Status::INDEX_RENAMED
                        | Status::INDEX_TYPECHANGE,
                )
            });
        }
        Err(e) => panic!("Error: {}", e),
    };
    res
}

fn format_footer(commit_type: &str, issues_list: &[String]) -> String {
    let footer_key = match commit_type {
        "fix" => "Fixes",
        "feat" => "Closes",
        _ => "Referenced-issues",
    };
    let mut footer_value = String::new();
    issues_list.iter().for_each(|s| {
        footer_value.push_str(s);
        footer_value.push(' ')
    });

    let footer_separator = match footer_value.chars().next() {
        Some('#') => " ",
        _ => ": ",
    };

    format!("{}{}{}", footer_key, footer_separator, footer_value)
}

/// Generates the commit message by selectively appending all parts that the
/// user entered.
pub fn generate_commit_msg(survey: SurveyResults) -> String {
    let commit_type_and_scope = match survey.scope {
        Some(scope) => format!("{}({})", survey.commit_type, scope),
        None => survey.commit_type.to_owned(),
    };
    let pre_colon = match survey.breaking_changes_desc {
        Some(_) => format!("{}!", commit_type_and_scope),
        None => commit_type_and_scope,
    };
    let with_short_msg = format!("{}: {}", pre_colon, survey.short_msg);
    let with_long_msg = match survey.long_msg {
        Some(long_msg) => format!("{}\n\n{}", with_short_msg, long_msg),
        None => with_short_msg,
    };
    let with_breaking_change = match survey.breaking_changes_desc {
        Some(desc) => format!("{}\n\nBREAKING CHANGE: {}", with_long_msg, desc),
        None => with_long_msg,
    };
    match survey.affected_open_issues {
        Some(issues_list) => format!(
            "{}\n\n{}",
            with_breaking_change,
            format_footer(&survey.commit_type, &issues_list)
        ),
        None => with_breaking_change,
    }
}

///Finds the last commit inside a git repository.
fn find_last_commit(repo: &Repository) -> Result<Commit> {
    let obj = repo.head()?.resolve()?.peel(ObjectType::Commit)?;
    obj.into_commit()
        .map_err(|_| anyhow!("could not find commit"))
}

/// Commits the added changes to the repository.
///
/// # Arguments
///
/// - `msg`: The commit message to use.
///
/// - `repository`: Path to the git repository
///
/// # Returns
///
/// The hash of the added commit.
///
/// # Note
///
/// The method uses the default username and email address found for the
/// repository. Defaults to the globally configured when needed.
pub fn commit_to_repo(msg: &str, repo: &Path) -> Result<Oid> {
    let repo = get_repository(repo).unwrap().lock().unwrap();
    let mut index = repo.index()?;
    let oid = index.write_tree()?;
    let signature = repo.signature()?;

    let tree = repo.find_tree(oid)?;
    let parent_commit = find_last_commit(&repo);

    let oid = match parent_commit {
        Ok(parent_commit) => repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            msg,
            &tree,
            &[&parent_commit],
        )?,
        Err(_) => repo.commit(Some("HEAD"), &signature, &signature, msg, &tree, &[])?,
    };

    Ok(oid)
}
