//! What:
//!  Controls proving the argument generators reach every classification outcome.
//! Why:
//!  An invariant that is never reached proves nothing;
//!  these controls count the
//!      outcomes the generators produce and run the invariants on fixed hard cases.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(outcomesReachedBy(generatedArguments)).toContain('MissingValue');
//! ```

/// Import the generators and invariants under control.
use super::{
    MAX_ARGUMENTS, arguments_from_bytes, check_config_loading, check_global_layout,
    generated_arguments,
};
use git_policy_cli::config_loading::{ConfigLoading, classify_config_loading};
use git_policy_cli::global_arguments::{GlobalOutcome, global_layout};
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;

/// NUL splitting keeps every other byte,
///  including non-UTF-8 and empty pieces,
///  and is bounded.
#[test]
fn raw_bytes_split_only_at_nul() {
    assert_eq!(arguments_from_bytes(b""), Vec::<OsString>::new());
    assert_eq!(
        arguments_from_bytes(b"-C\0dir-\xff\0\0status"),
        vec![
            OsString::from("-C"),
            OsString::from_vec(b"dir-\xff".to_vec()),
            OsString::new(),
            OsString::from("status"),
        ]
    );
    assert_eq!(
        arguments_from_bytes(b"\0"),
        vec![OsString::new(), OsString::new()]
    );
    assert_eq!(arguments_from_bytes(&[0; 200]).len(), MAX_ARGUMENTS);
    assert_eq!(generated_arguments(&[7; 200]).len(), MAX_ARGUMENTS);
    assert_eq!(generated_arguments(&[]), Vec::<OsString>::new());
}

/// Two-byte generated inputs alone reach every layout outcome and both loading decisions.
#[test]
fn generated_arguments_reach_every_outcome() {
    let mut commands: usize = 0;
    let mut queries: usize = 0;
    let mut missing: usize = 0;
    let mut invalid: usize = 0;
    let mut none: usize = 0;
    let mut skipped: usize = 0;
    let mut required: usize = 0;
    for first in 0..=u8::MAX {
        for second in [0_u8, 22, 25, 28, 29, 34, 55] {
            for data in [
                vec![first],
                vec![first, second],
                vec![second, first, second],
            ] {
                let arguments: Vec<OsString> = generated_arguments(data.as_slice());
                check_global_layout(arguments.as_slice());
                check_config_loading(arguments.as_slice());
                match global_layout(arguments.as_slice()).outcome {
                    GlobalOutcome::Command => commands += 1,
                    GlobalOutcome::Query => queries += 1,
                    GlobalOutcome::MissingValue => missing += 1,
                    GlobalOutcome::InvalidOption => invalid += 1,
                    GlobalOutcome::NoCommand => none += 1,
                }
                if classify_config_loading(arguments.as_slice()) == ConfigLoading::Skip {
                    skipped += 1;
                } else {
                    required += 1;
                }
            }
        }
    }
    for (name, count) in [
        ("command", commands),
        ("query", queries),
        ("missing value", missing),
        ("invalid option", invalid),
        ("no command", none),
        ("skip", skipped),
        ("required", required),
    ] {
        assert!(count > 100, "{name} reached only {count} times");
    }
}

/// Fixed hard cases run through the same invariants the fuzzer uses.
#[test]
fn invariants_hold_for_fixed_boundary_cases() {
    for raw in [
        b"".as_slice(),
        b"status",
        b"-C\0\0-C\0link/..\0-c\0x.y=-h\0status",
        b"--git-dir\0-h\0status\0--\0-D",
        b"--exec-path\0commit",
        b"--exec-path=/x\0branch\0--list\0--\0-D",
        b"-C",
        b"--\0status",
        b"\xff\xfe\0--help",
        b"branch\0--color\0new-branch",
        b"branch\0--format\0--delete",
        b"branch\0--contains",
        b"tag\0-n3\0v*",
        b"tag\0--\0--list",
        b"-c\0alias.x=commit\0x",
        b"cli-git\0check\0--all",
        b"branch\0-\xff",
        b"tag\0\0",
    ] {
        let arguments: Vec<OsString> = arguments_from_bytes(raw);
        check_global_layout(arguments.as_slice());
        check_config_loading(arguments.as_slice());
    }
}

/// The mutation invariant can fail:
///  a listing flag in first position does not require configuration.
#[test]
fn mutation_invariant_distinguishes_listing_from_mutating_flags() {
    let listing: Vec<OsString> = arguments_from_bytes(b"branch\0--list\0feature-*");
    assert_eq!(
        classify_config_loading(listing.as_slice()),
        ConfigLoading::Skip
    );
    let deleting: Vec<OsString> = arguments_from_bytes(b"branch\0--delete\0--list\0feature-*");
    assert_eq!(
        classify_config_loading(deleting.as_slice()),
        ConfigLoading::Required
    );
}
