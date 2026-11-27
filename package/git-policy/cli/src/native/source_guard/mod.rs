//! What:
//!  Controls that read the crate's own source text.
//! Why:
//!  They guard the layout of every other folder.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // source_guard/index.ts: one `export * from './file'` per module below.
//! ```

/// The modules a library root reaches,
/// with the visibility of every `mod` declaration on the way,
/// which is what decides whether another crate can name an item.
#[cfg(test)]
#[path = "modules_a_library_root_reaches_tests.rs"]
pub(crate) mod modules_a_library_root_reaches_tests;

/// The names another crate can reach through the library root,
/// read once from the module tree and the `pub use` lines.
#[cfg(test)]
#[path = "nameable_items_of_a_source_tree_tests.rs"]
pub(crate) mod nameable_items_of_a_source_tree_tests;

/// The names the library target publishes,
/// read from the `pub mod` chains and the public re-exports the source text writes,
/// and the ban on publishing one without a line in the list beside the check.
#[cfg(test)]
#[path = "nothing_becomes_nameable_from_outside_tests.rs"]
mod nothing_becomes_nameable_from_outside_tests;

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

/// The passage each piece built ahead of its command has in both release
/// documents,
/// because a reader of either one must learn that the piece exists and that no
/// command reaches it.
#[cfg(test)]
#[path = "spec_and_readme_reach_tests.rs"]
mod spec_and_readme_reach_tests;

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

/// The commit path's notes:
/// no file that says one writes standard error on the command's own thread.
#[cfg(test)]
#[path = "the_commit_path_writes_its_notes_off_its_thread_tests.rs"]
mod the_commit_path_writes_its_notes_off_its_thread_tests;

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

/// The ban on a control reading a starter's marker to decide whether its body runs here:
/// a control hands the starter its body,
/// which runs in a child of the test binary.
#[cfg(test)]
#[path = "only_the_process_wide_starter_reads_its_marker_tests.rs"]
mod only_the_process_wide_starter_reads_its_marker_tests;

/// The ban on a list of text written as an array with a hand-counted length:
/// a list of names,
/// sites or words is a slice,
/// so adding an entry never edits a count beside it.
#[cfg(test)]
#[path = "lists_are_slices_tests.rs"]
mod lists_are_slices_tests;

/// The ban on a control taking the runner the whole service process shares:
/// a control that reviews a commit declares one of its own and reaches the review through `review_item_in`.
#[cfg(test)]
#[path = "no_control_takes_the_production_runner_tests.rs"]
mod no_control_takes_the_production_runner_tests;

/// The ban on a fake service manager answering from a value the whole process shares:
/// a control writes the word it wants into the folder its own watch names.
#[cfg(test)]
#[path = "heartbeat_controls_own_their_manager_switch_tests.rs"]
mod heartbeat_controls_own_their_manager_switch_tests;

/// The ban on a control writing the service client's manager stand-in or its stop flag in the process the run holds:
/// such a control starts a child of its own first,
/// whose stand-in and flag hold nothing but what that control wrote (issue 6522).
#[cfg(test)]
#[path = "service_client_controls_own_their_process_wide_state_tests.rs"]
mod service_client_controls_own_their_process_wide_state_tests;

/// The ban on a control asking a question about the whole kill registry in the process the run holds:
/// such a control starts a child of its own first,
/// whose registry no other control planted in.
#[cfg(test)]
#[path = "supervisor_registry_questions_in_a_child_process_tests.rs"]
mod supervisor_registry_questions_in_a_child_process_tests;

/// The ban on a control reading the preparation's named steps in the process the run holds:
/// such a control starts a child of its own first,
/// whose ledger holds nothing but what that control noted.
#[cfg(test)]
#[path = "no_control_reads_the_preparation_steps_in_the_run_tests.rs"]
mod no_control_reads_the_preparation_steps_in_the_run_tests;

/// The ban on a control counting one of a commit's four ledgers in the process the run holds:
/// such a control starts a child of its own first,
/// whose ledgers hold nothing but what that control noted.
#[cfg(test)]
#[path = "no_control_reads_the_commit_ledgers_in_the_run_tests.rs"]
mod no_control_reads_the_commit_ledgers_in_the_run_tests;

/// The ban on a control folding one earlier log twice in the process the run holds:
/// the fold's journal lock is dropped as the fold answers,
/// so such a control starts a child of its own first,
/// which forks nothing while the lock is held.
#[cfg(test)]
#[path = "no_control_folds_a_log_twice_in_the_run_tests.rs"]
mod no_control_folds_a_log_twice_in_the_run_tests;

/// The ban on a control recording,
/// preparing or reading the service's stop in the process the run holds:
/// such a control starts a child of its own first,
/// whose statics hold nothing but what that control recorded.
#[cfg(test)]
#[path = "no_control_records_or_reads_the_service_stop_in_the_run_tests.rs"]
mod no_control_records_or_reads_the_service_stop_in_the_run_tests;

/// The ban on a control leaving `SIGPIPE` at its default in the process the run holds:
/// such a control starts a child of its own first,
/// so its action reaches no other thread's next write to a pipe nobody reads.
#[cfg(test)]
#[path = "no_control_leaves_sigpipe_at_its_default_tests.rs"]
mod no_control_leaves_sigpipe_at_its_default_tests;

/// The ban on a control taking the process's one queue of log lines in the process the run holds:
/// such a control starts a child of its own first,
/// whose queue holds nothing but what that control handed it.
#[cfg(test)]
#[path = "no_control_takes_the_log_line_queue_in_the_run_tests.rs"]
mod no_control_takes_the_log_line_queue_in_the_run_tests;

/// The ban on a control stepping a notice without the control window that decides its ceiling and its looks:
/// such a control takes the window first or starts a child of its own.
#[cfg(test)]
#[path = "notice_controls_take_their_window_tests.rs"]
mod notice_controls_take_their_window_tests;

/// The ban on a control publishing,
/// ending or reading in the process the run holds the step a waiting add's notice names:
/// such a control starts a child of its own first,
/// whose record holds nothing but what that control published.
#[cfg(test)]
#[path = "standing_notice_controls_own_the_published_step_tests.rs"]
mod standing_notice_controls_own_the_published_step_tests;

/// The ban on a control starting the interactive selection in the process the run holds:
/// such a control starts a child of its own first,
/// whose standard input is the null device and not the terminal the run was started from.
#[cfg(test)]
#[path = "no_control_starts_an_interactive_git_on_the_run_s_input_tests.rs"]
mod no_control_starts_an_interactive_git_on_the_run_s_input_tests;

/// The ban on a control of the Git version cache writing its fixture Git in the process the run holds:
/// such a control starts a child of its own first,
/// so no other thread of the run forks while one of its write descriptors is open.
#[cfg(test)]
#[path = "no_control_writes_a_fixture_git_in_the_run_tests.rs"]
mod no_control_writes_a_fixture_git_in_the_run_tests;

/// The ban on a release file reading a test-only switch itself:
/// every such value is taken through `foundation/test_only_switches.rs`,
/// whose read a release build does not hold.
#[cfg(test)]
#[path = "no_release_read_of_a_test_only_switch_tests.rs"]
mod no_release_read_of_a_test_only_switch_tests;

/// The ban on a release file reading this session's runtime directory or its session-bus address outside the
/// one place that reads them:
/// a build made for tests answers those two only from what a control handed in.
#[cfg(test)]
#[path = "no_release_read_of_the_session_places_tests.rs"]
mod no_release_read_of_the_session_places_tests;

/// The ban on a sentence that shows a name another program chose without the one escape:
/// the list of the sentences this program has,
/// and the accessors no `format!` may interpolate raw.
#[cfg(test)]
#[path = "every_sentence_that_names_a_process_shows_it_tests.rs"]
mod every_sentence_that_names_a_process_shows_it_tests;

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

/// Which bytes of a Rust source are code and which sit in a comment or a text literal,
/// read once so every scan of this module reads the same answer;
/// never in the release build.
#[cfg(test)]
#[path = "source_code_places_tests.rs"]
pub(crate) mod source_code_places_tests;

/// The scans that find a test file's unowned child start and its signal to its own process;
/// never in the release build.
#[cfg(test)]
#[path = "test_children_owned_scan_tests.rs"]
pub(crate) mod test_children_owned_scan_tests;

/// The one scan behind the bans on a control that reads a fact belonging to the whole process:
/// a ban states the names that make a control a reader and the calls that excuse it,
/// and the split of a source into its controls happens here;
/// never in the release build.
#[cfg(test)]
#[path = "process_wide_fact_readers_tests.rs"]
pub(crate) mod process_wide_fact_readers_tests;

/// The ledger of the child starts the tree still holds,
/// and the comparison that fails on a new site and on a line whose site is gone;
/// never in the release build.
#[cfg(test)]
#[path = "test_children_owned_ledger_tests.rs"]
pub(crate) mod test_children_owned_ledger_tests;

/// The control that refuses an unowned child start and a signal to the run's own process,
/// with the ledger of the sites the tree still holds;
/// never in the release build.
#[cfg(test)]
#[path = "test_children_owned_tests.rs"]
mod test_children_owned_tests;

/// The control that refuses a control of the relay's end rule which never owns the group of the child it reads:
/// the relay ends with the child,
/// and what the child left is the test's to end (issue 6501).
#[cfg(test)]
#[path = "relay_controls_own_the_child_s_group_tests.rs"]
mod relay_controls_own_the_child_s_group_tests;

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

/// Every release call that replaces this process puts back the action this process had for
/// `SIGPIPE` when the replacement fails,
/// because every path that goes on after it writes the wrapper's own words.
#[cfg(test)]
#[path = "every_exec_puts_the_signal_action_back_tests.rs"]
mod every_exec_puts_the_signal_action_back_tests;

/// The text with every whitespace character removed,
/// so `args_os ()` reads as `args_os()` and `clippy ::` as `clippy::`.
///
/// Why:
///  A needle and a pinned anchor are about the code,
///  and Rust's meaning does not change with the blanks between its tokens.
pub(super) fn squeezed(text: &str) -> String {
    return text
        .chars()
        .filter(|character| {
            return !character.is_whitespace();
        })
        .collect();
}

/// What:
///  One pinned file's code text:
/// each line as written,
///  a line that is only a comment dropped,
/// leading and trailing blanks trimmed.
/// Why:
///  The two files this reads are held to comments on their own line,
///  which the assertion below checks,
///  so cutting at a leading `//` is exact and no lexer decides what a line is.
///  Every byte of every literal stays,
/// because a module's constants and its format text are behaviour.
pub(super) fn pinned_code_lines(
    source: &str,
    what: &str,
) -> Vec<String> {
    assert_whole_line_comments_only(what, source);
    return source
        .lines()
        .map(|line| {
            let trimmed: &str = line.trim();
            return if trimmed.starts_with("//") {
                String::new()
            } else {
                String::from(trimmed)
            };
        })
        .collect();
}

/// Every `//` in the source begins its own line,
/// so cutting a line at the first one hides nothing,
/// and no block comment spans lines a reader would have to reconstruct.
pub(super) fn assert_whole_line_comments_only(
    what: &str,
    source: &str,
) {
    assert!(
        !source.contains("/*"),
        "{what} holds a block comment, which no line-at-a-time reader can cut exactly"
    );
    for line in source.lines() {
        if let Some(at) = line.find("//") {
            assert!(
                line[..at].trim().is_empty(),
                "a comment starts after code in {what}, and a line-at-a-time reader would cut what follows it: {line}"
            );
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
///  test sidecars,
/// binary-level controls and support that only tests compile
///  are told apart by the declarations that reach them,
/// never by file name,
///  and a tree that cannot be fully classified is an error.
pub(super) fn release_sources_in(
    root: &std::path::Path,
    roots: &[source_scan_tests::CrateRoot],
) -> Result<Vec<String>, String> {
    let mut found: Vec<String> = Vec::new();
    for file in source_scan_tests::classified_sources(root, roots)? {
        if !file.test_only {
            found.push(file.path);
        }
    }
    return Ok(found);
}

/// The scan a ban over part of the release tree reads:
/// which sources it covers,
/// which of their code lines hold the text it forbids,
/// which entries excuse one,
/// and the anchor that proves the scope is still the code the ban is about;
/// never in the release build.
#[cfg(test)]
#[path = "source_scan_banned_lines_tests.rs"]
pub(crate) mod source_scan_banned_lines_tests;

/// That scan under the names each ban of this folder calls it by.
pub(crate) use source_scan_banned_lines_tests::{
    HoldingLine,
    MayHold,
    anchors_the_scope_no_longer_holds,
    entries_that_match_nothing,
    lines_holding_one_of,
    lines_no_entry_excuses,
    release_sources_holding,
    release_sources_under,
};
