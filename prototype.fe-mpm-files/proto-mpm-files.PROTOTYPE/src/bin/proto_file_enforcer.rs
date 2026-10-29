// PROTOTYPE (throwaway) stand-in for the Rust `file-enforcer` toolchain rules. Not production code.
//
// usage: proto-file-enforcer <write|check|user write|user check> --candidate a|b|c|d [--dir DIR]
// Candidate A and D: file-enforcer has no toolchain role (prints a notice, exits 0).
// Candidate B: `check` verifies lock vs plan and both mpm files through the shared library; `write` never touches them.
// Candidate C: `write` renders both mpm files from the lock; `check` verifies lock vs plan and the mpm files.

use proto_mpm_files_prototype::*;
use std::path::PathBuf;

fn main() {
    std::process::exit(real_main());
}

fn real_main() -> i32 {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let user = args.first().map(String::as_str) == Some("user");
    if user {
        args.remove(0);
    }
    let Some(cmd) = args.first().cloned() else {
        eprintln!("missing subcommand");
        return 2;
    };
    let mut cand = Candidate::A;
    let mut dir = PathBuf::from(".");
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--candidate" => {
                i += 1;
                match Candidate::parse(args.get(i).map(String::as_str).unwrap_or("")) {
                    Ok(c) => cand = c,
                    Err(e) => {
                        eprintln!("{e}");
                        return 2;
                    }
                }
            }
            "--dir" => {
                i += 1;
                dir = PathBuf::from(args.get(i).cloned().unwrap_or_default());
            }
            other => {
                eprintln!("unknown argument {other}");
                return 2;
            }
        }
        i += 1;
    }
    let scope = if user {
        match Scope::global() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{e}");
                return 2;
            }
        }
    } else {
        Scope::repo(&dir)
    };
    let role = matches!(cand, Candidate::B | Candidate::C);
    if !role {
        println!("file-enforcer: no toolchain rules under candidate {cand:?}");
        return 0;
    }
    match cmd.as_str() {
        "write" => {
            if cand == Candidate::B {
                println!("file-enforcer: toolchain files are written by mpm-shadow (candidate B)");
                return 0;
            }
            let lock = match std::fs::read_to_string(scope.lock()).map_err(|e| e.to_string()).and_then(|t| serde_json::from_str(&t).map_err(|e| e.to_string())) {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("file-enforcer: cannot read lock: {e}");
                    return 2;
                }
            };
            match write_mpm_files(&scope, &lock, cand.header()) {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("{e}");
                    2
                }
            }
        }
        "check" => {
            let mut drift = check_lock_vs_plan(&scope);
            drift.extend(check_mpm_files(&scope, cand.header()));
            for d in &drift {
                println!("{d}");
            }
            if drift.is_empty() {
                println!("OK");
                0
            } else {
                1
            }
        }
        other => {
            eprintln!("unknown subcommand {other}");
            2
        }
    }
}
