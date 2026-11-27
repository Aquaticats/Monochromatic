//! What:
//!  The pin of every attribute in release code that can set a lint level,
//!  compared as anchors rather than lines.
//! Why:
//!  One `#[allow(...)]` line can excuse the typed bans of `clippy.toml` in a
//!  file no text pin covers,
//!  so the release tree's lint-level attributes are held to a reviewed set
//!  (`doc/planning/cli-git-zfs-root-modes-in-the-one-wrapper-executable-design.md`,
//!  "The lint-attribute pin").
//!  An anchor is the squeezed text from `#[` through the attribute's closing
//!  `]`,
//!  with each `reason = …` meta item left out:
//!  so an allow split over two lines is one anchor,
//!  an allow whose lint path a macro builds is an anchor no entry names,
//!  and a reason's wording never re-pins.
//!  Nothing here is masked,
//!  because a masking mistake is what hid three release files from the readers
//!  this pin replaces.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(anchorsOverTheReleaseTree()).toEqual(LINT_LEVELS);
//! ```

use super::{
    release_sources_in,
    squeezed,
};
use crate::source_scan_tests::{
    real_roots,
    source_root,
};
use std::path::PathBuf;

/// The attributes that name a lint level, and so can silence one.
const LINT_LEVEL_ATTRIBUTES: &[&str] = &[
    "allow(",
    "expect(",
    "warn(",
    "deny(",
    "forbid(",
    "cfg_attr(",
];

/// Every release attribute that can set a lint level, with the file it stands in: the whole set.
/// An anchor is the squeezed text from `#[` through the attribute's closing `]`,
/// so an attribute split over two lines is one anchor and a reason's prose is not part of it.
const LINT_LEVELS: &[(&str, &str)] = &[
    (
        "foundation/call_catching_panics.rs",
        "#[cfg_attr(dylint_lib=\"monochromatic_lint_closure\",expect(avoidable_closure))]",
    ),
    (
        "foundation/scoped_start.rs",
        "#[cfg_attr(dylint_lib=\"monochromatic_lint_closure\",expect(avoidable_closure))]",
    ),
    (
        "foundation/side_by_side_threads.rs",
        "#[cfg_attr(dylint_lib=\"monochromatic_lint_closure\",expect(avoidable_closure))]",
    ),
    (
        "invocation/real_git.rs",
        "#[allow(clippy::disallowed_methods)]",
    ),
    (
        "landing/landing_state_carried_by_its_task.rs",
        "#[cfg_attr(dylint_lib=\"monochromatic_lint_closure\",expect(avoidable_closure))]",
    ),
    (
        "landing/landing_state_carried_by_its_task.rs",
        "#[cfg_attr(dylint_lib=\"monochromatic_lint_closure\",expect(avoidable_closure))]",
    ),
    (
        "landing/service_land_finish_branch_questions.rs",
        "#[cfg_attr(not(test),expect(dead_code))]",
    ),
    (
        "landing/service_land_git_steps_and_import_stream.rs",
        "#[cfg_attr(not(test),expect(dead_code))]",
    ),
    (
        "landing/service_land_git_steps_and_import_stream.rs",
        "#[cfg_attr(not(test),expect(dead_code))]",
    ),
    (
        "landing/service_land_git_steps_and_import_stream.rs",
        "#[cfg_attr(not(test),expect(dead_code))]",
    ),
    (
        "landing/service_land_git_steps_and_import_stream.rs",
        "#[cfg_attr(not(test),expect(dead_code))]",
    ),
    (
        "landing/service_land_git_steps_and_import_stream.rs",
        "#[cfg_attr(not(test),expect(dead_code))]",
    ),
    (
        "landing/service_land_side_reference_write.rs",
        "#[cfg_attr(not(test),expect(dead_code))]",
    ),
    (
        "landing/service_land_side_reference_write.rs",
        "#[cfg_attr(not(test),expect(dead_code))]",
    ),
    (
        "landing/service_land_side_reference_write.rs",
        "#[cfg_attr(not(test),expect(dead_code))]",
    ),
    (
        "landing/service_land_side_reference_write.rs",
        "#[cfg_attr(not(test),expect(dead_code))]",
    ),
    (
        "landing/service_land_side_reference_write.rs",
        "#[cfg_attr(not(test),expect(dead_code))]",
    ),
    (
        "landing/service_land_side_reference_write.rs",
        "#[cfg_attr(not(test),expect(dead_code))]",
    ),
    (
        "landing/service_prepare_own_git_steps.rs",
        "#[cfg_attr(not(test),expect(dead_code))]",
    ),
    (
        "landing/service_prepare_own_git_steps.rs",
        "#[cfg_attr(not(test),expect(dead_code))]",
    ),
    (
        "landing/service_scratch_ref_of_a_dropped_batch_awaiting.rs",
        "#[cfg_attr(not(test),expect(dead_code))]",
    ),
    (
        "landing/service_scratch_ref_of_a_dropped_batch_awaiting.rs",
        "#[cfg_attr(not(test),expect(dead_code))]",
    ),
    (
        "landing/service_work_git_handlers.rs",
        "#[cfg_attr(not(test),expect(unused_mut))]",
    ),
    (
        "landing/service_work_git_handlers.rs",
        "#[cfg_attr(not(test),expect(unused_mut))]",
    ),
    ("main.rs", "#[allow(clippy::disallowed_methods)]"),
    (
        "process/process_identity.rs",
        "#[cfg_attr(all(target_os=\"linux\",not(test)),allow(dead_code))]",
    ),
    (
        "push/auto_push_reconcile.rs",
        "#[cfg_attr(dylint_lib=\"monochromatic_lint_closure\",expect(avoidable_closure))]",
    ),
    (
        "push/bounded_query_input.rs",
        "#[cfg_attr(not(test),expect(dead_code))]",
    ),
    (
        "worktree_clone_mount_handler/record_checkout_mode_record_file.rs",
        "#[allow(clippy::disallowed_methods)]",
    ),
    (
        "worktree_clone_unmount/staged_folder_record.rs",
        "#[allow(clippy::disallowed_methods)]",
    ),
    (
        "worktree_clone_unmount/staged_unmount.rs",
        "#[allow(clippy::large_enum_variant)]",
    ),
    (
        "worktree_clone_unmount/staging_claim.rs",
        "#[allow(clippy::disallowed_methods)]",
    ),
    (
        "worktree_clone_unmount/unmount_checks.rs",
        "#[allow(clippy::result_large_err)]",
    ),
    (
        "worktree_clone_unmount/unmount_record.rs",
        "#[allow(clippy::result_large_err)]",
    ),
    (
        "worktree_clone_unmount/unmount_record.rs",
        "#[allow(clippy::result_large_err)]",
    ),
    (
        "worktree_clone_unmount/user_cache_pin.rs",
        "#[allow(clippy::disallowed_methods)]",
    ),
];

/// Every lint-level anchor in one source, in the order the source writes them.
/// Nothing is cut from the source first:
///  the scan's needles are often literal text, and this pin's claim is about code,
///  so the two cannot share one reader that decides what a line is.
fn anchors(source: &str) -> Vec<String> {
    let flat: String = squeezed(source);
    let bytes: Vec<char> = flat.chars().collect();
    let mut found: Vec<String> = Vec::new();
    let mut at: usize = 0;
    while at < bytes.len() {
        let opened: Option<usize> = if bytes.get(at) == Some(&'#')
            && bytes.get(at + 1) == Some(&'!')
            && bytes.get(at + 2) == Some(&'[')
        {
            Some(3)
        } else if bytes.get(at) == Some(&'#') && bytes.get(at + 1) == Some(&'[') {
            Some(2)
        } else {
            None
        };
        let Some(start) = opened else {
            at += 1;
            continue;
        };
        let rest: String = bytes[at + start..].iter().collect();
        if !LINT_LEVEL_ATTRIBUTES.iter().any(|name| {
            return rest.starts_with(name);
        }) {
            at += 1;
            continue;
        }
        let mut anchor: Vec<char> = Vec::new();
        let mut depth: usize = 0;
        let mut index: usize = at;
        while index < bytes.len() {
            let here: String = bytes[index..].iter().take(7).collect();
            if here == "reason=" {
                if anchor.last() == Some(&',') {
                    anchor.pop();
                }
                index = after_one_meta_value(&bytes, index + here.len());
                continue;
            }
            let character: char = bytes[index];
            anchor.push(character);
            match character {
                '(' => depth += 1,
                ')' => depth = depth.saturating_sub(1),
                ']' if depth == 0 => {
                    index += 1;
                    break;
                }
                _ => {}
            }
            index += 1;
        }
        found.push(anchor.into_iter().collect());
        at = index;
    }
    return found;
}

/// Where the value of one skipped meta item ends: a parenthesised group, a string, or a bare word.
fn after_one_meta_value(
    bytes: &[char],
    from: usize,
) -> usize {
    let mut cursor: usize = from;
    match bytes.get(cursor) {
        Some('(') => {
            let mut depth: usize = 0;
            while cursor < bytes.len() {
                match bytes[cursor] {
                    '(' => depth += 1,
                    ')' => {
                        depth = depth.saturating_sub(1);
                        if depth == 0 {
                            return cursor + 1;
                        }
                    }
                    _ => {}
                }
                cursor += 1;
            }
            return cursor;
        }
        Some('"') => {
            cursor += 1;
            while cursor < bytes.len() {
                if bytes[cursor] == '\\' {
                    cursor += 2;
                    continue;
                }
                if bytes[cursor] == '"' {
                    return cursor + 1;
                }
                cursor += 1;
            }
            return cursor;
        }
        _ => {
            while cursor < bytes.len() && bytes[cursor] != ',' && bytes[cursor] != ')' {
                cursor += 1;
            }
            return cursor;
        }
    }
}

/// The release code sets a lint level only where this pin names it.
#[test]
fn the_release_code_sets_a_lint_level_only_where_this_pin_names_it() {
    let root: PathBuf = source_root();
    let mut found: Vec<(String, String)> = Vec::new();
    for file in release_sources_in(&root, &real_roots()).expect("the release sources") {
        let text: String = std::fs::read_to_string(root.join(file.as_str())).expect("source file");
        for anchor in anchors(text.as_str()) {
            found.push((file.clone(), anchor));
        }
    }
    let mut named: Vec<(String, String)> = LINT_LEVELS
        .iter()
        .map(|(file, anchor)| {
            return (String::from(*file), String::from(*anchor));
        })
        .collect();
    found.sort();
    named.sort();
    assert_eq!(
        found, named,
        "release code holds a lint-level attribute this pin does not name, and an allow lists the lints the typed bans stop applying"
    );
    assert!(
        !found.is_empty(),
        "the pin read no attribute at all, so it would pass a tree it never looked at"
    );
}

/// What each attribute kind looks like,
/// so a reader of a re-pin sees the anchor's shape rather than a squeezed line.
#[test]
fn the_anchors_leave_a_reason_out_and_join_a_split_attribute() {
    let split: &str = "#[allow(\n    clippy::disallowed_methods,\n    reason = \"the one collection main makes\"\n)]\nfn main() {}\n";
    assert_eq!(
        anchors(split),
        vec![String::from("#[allow(clippy::disallowed_methods)]")]
    );
    let ordinary: &str = "#[cfg_attr(not(test), expect(dead_code))]\nfn helper() {}\n";
    assert_eq!(
        anchors(ordinary),
        vec![String::from("#[cfg_attr(not(test),expect(dead_code))]")]
    );
    let not_a_lint: &str = "#[derive(Debug)]\nstruct x;\n";
    assert_eq!(anchors(not_a_lint), Vec::<String>::new());
}
