//! What:
//!  Controls that read the crate's own source text.
//! Why:
//!  They guard the layout of every other folder.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // source_guard/index.ts: one `export * from './file'` per module below.
//! ```

/// The ban on Git starting the wrapper for a hook,
/// and the pin of the one entry that decides what a start is.
#[cfg(test)]
#[path = "hook_self_invocation_tests.rs"]
mod hook_self_invocation_tests;

/// The pin of the one module that may read a start name,
/// and the wiring assertions no text pin can see.
#[cfg(test)]
#[path = "root_mode_name_tests.rs"]
mod root_mode_name_tests;

/// The pin of every lint-level attribute in release code,
/// because one such attribute can excuse a typed ban in a file no text pin
/// covers.
#[cfg(test)]
#[path = "lint_attribute_tests.rs"]
mod lint_attribute_tests;

/// The pin of the typed ban's own configuration,
/// because a level set to `allow` silences every call the ban names without
/// changing a line of source.
#[cfg(test)]
#[path = "typed_ban_configuration_tests.rs"]
mod typed_ban_configuration_tests;

/// The lock inventory of the service and its support modules:
/// a lock stands only on the inventory,
/// with its reason beside it.
#[cfg(test)]
#[path = "service_lock_inventory_tests.rs"]
mod service_lock_inventory_tests;

/// The crate declares only the one executable the release archive holds;
/// a `[[bin]]` the archive lacks breaks every cargo-binstall install.
#[cfg(test)]
#[path = "release_bins_tests.rs"]
mod release_bins_tests;

/// The door every diagnostic line goes through:
/// it hands each line to the thread that never holds its caller,
/// that thread writes the descriptor rather than the shared handle,
/// and no release file says a line with `eprintln!`.
#[cfg(test)]
#[path = "diagnostic_door_tests.rs"]
mod diagnostic_door_tests;

/// The mechanical ban:
/// no release code starts the wrapper,
/// passes it management words or writes shell text.
#[cfg(test)]
#[path = "no_self_invocation_tests.rs"]
mod no_self_invocation_tests;

/// The mechanical ban on changing the directory of the process:
/// a control that wants the code under test to start somewhere passes that directory as an input.
#[cfg(test)]
#[path = "no_test_changes_the_process_directory_tests.rs"]
mod no_test_changes_the_process_directory_tests;

/// Every record folder this program fills is read through `plain_file`:
/// a named pipe at a record's name answers at once instead of holding every wrapped command.
#[cfg(test)]
#[path = "records_read_without_blocking_tests.rs"]
mod records_read_without_blocking_tests;

/// No release file outside the plain-file reader reads a file with a call that can wait,
/// and the sites that stay are on a list that only shrinks,
/// each entry naming the class whose reason keeps it.
#[cfg(test)]
#[path = "raw_file_reads_tests.rs"]
mod raw_file_reads_tests;

/// The owner's move to tokio needs its workers to await:
/// no thread,
/// no sleeping clock,
/// no unretried child start
/// and no blocking work inside a task.
#[cfg(test)]
#[path = "blocking_code_off_the_runtime_tests.rs"]
mod blocking_code_off_the_runtime_tests;

/// The slot glue a forwarded command reaches reads the command's words before it reads the working
/// directory,
/// a record or the registry,
/// and returns between them;
/// the command's printed answer cannot show that order.
#[cfg(test)]
#[path = "slot_command_reads_its_words_first_tests.rs"]
mod slot_command_reads_its_words_first_tests;

/// The structural check that the review's network client is reachable only from the service run;
/// never in the release build.
#[cfg(test)]
mod standards_review_boundary_tests;

/// The shared base of the checks that read the crate's own source text:
/// a strict walk and the module tree,
/// so a check
/// is right for nested module directories;
/// never in the release build.
#[cfg(test)]
pub(crate) mod source_scan_tests;

/// The owner's rule that nothing of the wrapper starts at system startup or at login:
/// no install line,
/// no enabling call,
/// only listed members.
#[cfg(test)]
#[path = "no_start_at_login_tests.rs"]
pub(crate) mod no_start_at_login_tests;

/// The published package holds no copy of the tree under a `uses/` link:
/// cargo follows each link,
/// so `Cargo.toml`'s `include` list must negate the links' targets.
#[cfg(test)]
#[path = "packaged_uses_links_tests.rs"]
mod packaged_uses_links_tests;

/// The removal's command path runs the removal's read-only reading and no entry that writes:
/// plan step 16's wiring is the change that names the writing entry,
/// and it removes this check with it.
#[cfg(test)]
#[path = "removal_command_path_writes_nothing_tests.rs"]
mod removal_command_path_writes_nothing_tests;

/// The add route names no entry that destroys a clone,
/// an origin snapshot or a snapshot set,
/// which is why its grant table refuses six and warns on the three a removal
/// uses:
/// the step that gives the add a destroying failure path moves those two
/// `destroy` grants into the add's own table,
/// and it removes this check with them.
#[cfg(test)]
#[path = "add_route_issues_no_destroy_tests.rs"]
mod add_route_issues_no_destroy_tests;

/// The text with every whitespace character removed,
/// so `args_os ()` reads as `args_os()` and `clippy ::` as `clippy::`.
///
/// Why:
///  A needle and a pinned anchor are about the code,
///  and Rust's meaning does not change with the blanks between its tokens.
pub(super) fn squeezed(text: &str) -> String {
    return text.chars().filter(|character| { return !character.is_whitespace(); }).collect();
}

/// What:
///  One pinned file's code text: each line as written,
///  a line that is only a comment dropped, leading and trailing blanks trimmed.
/// Why:
///  The two files this reads are held to comments on their own line,
///  which the assertion below checks,
///  so cutting at a leading `//` is exact and no lexer decides what a line is.
///  Every byte of every literal stays, because a module's constants and its format text are behaviour.
pub(super) fn pinned_code_lines(source: &str, what: &str) -> Vec<String> {
    assert_whole_line_comments_only(what, source);
    return source.lines().map(|line| {
        let trimmed: &str = line.trim();
        return if trimmed.starts_with("//") { String::new() } else { String::from(trimmed) };
    }).collect();
}

/// Every `//` in the source begins its own line,
/// so cutting a line at the first one hides nothing,
/// and no block comment spans lines a reader would have to reconstruct.
pub(super) fn assert_whole_line_comments_only(what: &str, source: &str) {
    assert!(!source.contains("/*"), "{what} holds a block comment, which no line-at-a-time reader can cut exactly");
    for line in source.lines() {
        if let Some(at) = line.find("//") {
            assert!(line[..at].trim().is_empty(), "a comment starts after code in {what}, and a line-at-a-time reader would cut what follows it: {line}");
        }
    }
}

/// What:
///  The release source files under `root`,
///  as `/`-separated paths relative to it:
///  every file the module tree reaches from a crate root through declarations
///  that no `cfg(test)` gates.
/// Why:
///  A pin over a whole tree cannot be maintained file by file,
///  so the tree's scans classify it once and every one of them reads the same
///  list:
///  test sidecars, binary-level controls and support that only tests compile
///  are told apart by the declarations that reach them, never by file name,
///  and a tree that cannot be fully classified is an error.
pub(super) fn release_sources_in(root: &std::path::Path, roots: &[source_scan_tests::CrateRoot]) -> Result<Vec<String>, String> {
    let mut found: Vec<String> = Vec::new();
    for file in source_scan_tests::classified_sources(root, roots)? {
        if !file.test_only {
            found.push(file.path);
        }
    }
    return Ok(found);
}
