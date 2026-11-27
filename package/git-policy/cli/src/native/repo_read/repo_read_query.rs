//! What:
//!  Answers,
//! without a Git process,
//! to the exact read-only Git command lines the wrapper itself issues:
//!  `ls-files --stage`,
//! `ls-files --unmerged`,
//! `diff-index --cached`,
//! `diff --cached --name-only`,
//! `diff-tree --root -r -m`
//!  and `show --no-patch --format=%s`.
//! Why:
//!  Each is a start of Git for a fact two in-process readers (the index,
//! the object database) can produce byte for byte.
//!  The reader takes the same argument list the caller would give Git and returns Git's standard output and exit code,
//!  or `None` for any command line,
//! repository or configuration it does not fully understand,
//! so a caller tries it first
//!  and starts Git on `None`.
//!  This file is the one entry point and the dispatch,
//!  and each family of answers is a sibling:
//!  the checks a content answer's configuration must pass,
//!  the answers about the index and the working tree,
//!  and the answers that read an object by name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = answerInProcess({ args, directory, variables }); // undefined: start Git
//! ```

#![cfg(unix)]

use crate::repo_read_config_scan::ScopeEnvironment;
use crate::repo_read_layout::{
    RepoLayout,
    env_config_avoids,
    read_layout_with,
};
use crate::repo_read_object::name_length;
use crate::repo_read_query_refs::{
    config_list,
    for_each_ref,
    remote_url,
    rev_parse,
    symbolic_ref,
};
pub use crate::repo_read_query_request_answer_and_context::{
    Answer,
    Query,
};
use crate::repo_read_query_request_answer_and_context::{
    Context,
    Parsed,
    parse,
};
use crate::the_answers_about_commits_trees_and_objects::{
    batch_objects,
    diff_tree,
    diff_tree_names,
    show_subject,
};
use crate::the_answers_about_the_index_and_the_working_tree::{
    diff_cached,
    list_files,
};
use crate::the_environment_and_config_a_content_answer_needs::{
    content_config_is_plain,
    content_key_of_any_case,
    environment,
};
use std::ffi::OsString;
use std::path::PathBuf;

/// What:
///  The answer to `query`,
/// or `None` when Git must be started.
/// Why:
///   This is the one entry point;
/// every reason to decline (an unknown command line,
/// a repository or configuration
///  the readers do not fully understand,
/// a missing object) is the same `None`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = answer(query); // undefined: start Git
/// ```
pub fn answer(query: &Query<'_>) -> Option<Answer> {
    let mut base: Vec<(OsString, OsString)> = std::env::vars_os().collect();
    base.extend(query.variables.iter().cloned());
    let scope: ScopeEnvironment = ScopeEnvironment::from_variables(base.as_slice());
    return answer_with(query, std::env::vars_os().collect(), &scope);
}

/// The same with the process environment and the configuration scope facts given,
/// so a control does not depend on its host.
pub fn answer_with(
    query: &Query<'_>,
    base: Vec<(OsString, OsString)>,
    scope: &ScopeEnvironment,
) -> Option<Answer> {
    let mut start: PathBuf = match query.directory {
        Some(directory) => directory.to_path_buf(),
        None => std::env::current_dir().ok()?,
    };
    let parsed: Parsed<'_> = parse(query.arguments, &mut start)?;
    if !matches!(
        parsed.command,
        "ls-files"
            | "diff-index"
            | "diff"
            | "diff-tree"
            | "show"
            | "config"
            | "remote"
            | "rev-parse"
            | "for-each-ref"
            | "symbolic-ref"
            | "cat-file"
    ) {
        return None;
    }
    let (variables, index_variable, objects_variable) = environment(base, query.variables);
    let layout: RepoLayout = read_layout_with(start.as_path(), variables.as_slice(), scope)?;
    let index_path: PathBuf = match index_variable {
        Some(file) => start.join(file),
        None => layout.git_dir.join("index"),
    };
    let mut objects: Option<PathBuf> = None;
    if let Some(directory) = objects_variable {
        objects = Some(start.join(directory));
    }
    let content_command: bool = matches!(
        parsed.command,
        "ls-files" | "diff-index" | "diff" | "diff-tree" | "show"
    );
    if content_command && !content_config_is_plain(&layout, scope)? {
        return None;
    }
    // Entries injected through the environment are configuration too.
    if content_command && !env_config_avoids(variables.as_slice(), &content_key_of_any_case) {
        return None;
    }
    let context: Context = Context {
        layout,
        index_path,
        objects,
        scope: scope.clone(),
        variables,
    };
    if name_length(context.layout.object_format) == 0 {
        return None;
    }
    return match parsed.command {
        "ls-files" => list_files(&context, &parsed),
        "diff-index" => diff_cached(&context, &parsed, false),
        "diff" => diff_cached(&context, &parsed, true),
        "diff-tree" if parsed.flags.contains(&"--root") => diff_tree(&context, &parsed),
        "diff-tree" => diff_tree_names(&context, &parsed),
        "show" => show_subject(&context, &parsed),
        "config" => config_list(&context, &parsed),
        "remote" => remote_url(&context, &parsed),
        "for-each-ref" => for_each_ref(&context, &parsed),
        "rev-parse" => rev_parse(&context, &parsed),
        "symbolic-ref" => symbolic_ref(&context, &parsed),
        "cat-file" => batch_objects(&context, &parsed, query.input),
        _ => None,
    };
}

#[cfg(test)]
#[path = "repo_read_query_tests.rs"]
pub(crate) mod tests;

/// Controls for the tree-to-tree listings stay out of the release executable.
#[cfg(test)]
#[path = "repo_read_pair_tests.rs"]
mod pair_tests;
