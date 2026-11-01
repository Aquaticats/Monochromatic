//! The sandbox request's contents.

use super::{Invocation, LOCATION_VARIABLES, launch_request};
use crate::language::{confine::recipe::recipe, toolchain::ToolchainRequest};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// The program the lookup would run:
///  the worktree's own `mpm-shadow` link, absolute.
const PROGRAM: &str = "/home/someone/Monochromatic/.cache/toolchain-bin/mpm-shadow";

/// The arguments of the lookup after the program name.
const BIN: [&str; 2] = ["bin", "--json"];

fn project(root: &str) -> ToolchainRequest {
    return ToolchainRequest {
        project_root: PathBuf::from(root),
        project_spellings: Vec::new(),
        state_root: Some(PathBuf::from("/state")),
    };
}

fn lookup(root: &str) -> Invocation {
    return Invocation {
        program: PathBuf::from(PROGRAM),
        arguments: BIN.map(str::to_string).to_vec(),
        directory: PathBuf::from(root),
    };
}

/// The exact arguments bubblewrap gets for a lookup run.
fn arguments(root: &str, locations: &[(String, String)]) -> Vec<String> {
    let request = launch_request(&lookup(root), &project(root), locations);
    let inherited = vec![("HOME".to_string(), "/home/someone".to_string())];
    return recipe(
        &request,
        Path::new("/state/p-0/toolchain"),
        "/usr/bin/bwrap",
        &inherited,
    )
    .expect("recipe")
    .args;
}

/// The index of the first `--` argument,
///  which ends the sandbox options.
fn separator(args: &[String]) -> usize {
    for (index, argument) in args.iter().enumerate() {
        if argument == "--" {
            return index;
        }
    }
    panic!("separator");
}

/// The index where the arguments `wanted` first appear in a row.
fn position(args: &[String], wanted: &[&str]) -> Option<usize> {
    for (index, window) in args.windows(wanted.len()).enumerate() {
        let mut same = true;
        for (argument, expected) in window.iter().zip(wanted.iter()) {
            if argument != expected {
                same = false;
                break;
            }
        }
        if same {
            return Some(index);
        }
    }
    return None;
}

fn setenv(arguments: &[String], name: &str) -> Option<String> {
    for window in arguments.windows(3) {
        if window[0] == "--setenv" && window[1] == name {
            return Some(window[2].clone());
        }
    }
    return None;
}

#[test]
fn the_lookup_runs_in_the_servers_sandbox_without_network_and_with_a_cleared_environment() {
    let args = arguments("/work/project", &[]);
    for flag in [
        "--unshare-net",
        "--unshare-pid",
        "--unshare-user",
        "--die-with-parent",
        "--clearenv",
    ] {
        assert!(
            args.contains(&flag.to_string()),
            "{flag} is missing from {args:?}"
        );
    }
    assert!(
        position(&args, &["--ro-bind", "/", "/"]).is_some(),
        "{args:?}"
    );
    assert!(
        position(&args, &["--ro-bind", "/work/project", "/work/project"]).is_some(),
        "the project is not bound read-only in {args:?}"
    );
    // Nothing but the private state and the private /tmp is writable.
    let mut writable: Vec<&[String]> = Vec::new();
    for window in args.windows(3) {
        if window[0] == "--bind" {
            writable.push(window);
        }
    }
    assert_eq!(writable.len(), 2, "{writable:?}");
    let mut inside_state = true;
    for window in writable.iter() {
        if !window[1].starts_with("/state/") {
            inside_state = false;
        }
    }
    assert!(inside_state, "{writable:?}");
    let after = separator(&args);
    assert_eq!(
        args[after + 1..],
        [PROGRAM, BIN[0], BIN[1]].map(str::to_string),
        "the sandbox runs the worktree's mpm-shadow by its explicit path, then the two lookup arguments"
    );
}

#[test]
fn the_project_is_never_an_option_so_a_dashed_name_cannot_become_one() {
    for root in [
        "/work/-rf",
        "/work/--cd=/etc",
        "/work/a b/'c'/\"d\"",
        "/work/new\nline",
    ] {
        let args = arguments(root, &[]);
        let after = separator(&args);
        let tail = &args[after + 1..];
        assert_eq!(
            tail,
            [PROGRAM, BIN[0], BIN[1]].map(str::to_string),
            "for {root}"
        );
        assert!(
            tail[0].starts_with('/'),
            "the program would be read as an option: {}",
            tail[0]
        );
        // The root appears only as a bind source and destination, each its own argument.
        let mut mentions = 0;
        for argument in args.iter() {
            if argument.as_str() == root {
                mentions += 1;
            }
        }
        assert_eq!(mentions, 2, "{args:?}");
    }
}

#[test]
fn location_variables_reach_the_lookup_and_nothing_else_of_the_application_does() {
    let locations = vec![
        ("HOME".to_string(), "/home/someone".to_string()),
        ("XDG_DATA_HOME".to_string(), "/custom/data".to_string()),
    ];
    let args = arguments("/work/project", &locations);
    assert_eq!(setenv(&args, "HOME").as_deref(), Some("/home/someone"));
    assert_eq!(
        setenv(&args, "XDG_DATA_HOME").as_deref(),
        Some("/custom/data")
    );
    // A credential-like variable is not on the list of locations.
    let mut credential_free = true;
    for name in LOCATION_VARIABLES.iter() {
        if name.contains("TOKEN") || name.contains("KEY") {
            credential_free = false;
        }
    }
    assert!(credential_free);
    assert!(setenv(&args, "GITHUB_TOKEN").is_none());
    assert!(setenv(&args, "NODE_OPTIONS").is_none());
}

#[test]
fn every_location_variable_names_a_place() {
    for name in LOCATION_VARIABLES.iter() {
        assert!(
            *name == "HOME" || name.starts_with("XDG_"),
            "{name} is not a place"
        );
    }
}

#[test]
fn a_location_variable_below_a_replaced_directory_is_shown_again_read_only() {
    let root = crate::test_scratch::scratch().expect("directory");
    let data = root.path().join("data");
    fs::create_dir_all(&data).expect("directory");
    let request = launch_request(
        &lookup("/work/project"),
        &project("/work/project"),
        &[(
            "XDG_DATA_HOME".to_string(),
            data.to_str().expect("Unicode").to_string(),
        )],
    );
    assert_eq!(request.read_only_binds, [data]);
    let none = launch_request(
        &lookup("/work/project"),
        &project("/work/project"),
        &[],
    );
    assert!(none.read_only_binds.is_empty());
}
