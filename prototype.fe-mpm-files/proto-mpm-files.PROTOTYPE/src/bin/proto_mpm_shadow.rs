// PROTOTYPE (throwaway) stand-in for `mpm-shadow` toolchain subcommands. Not production code.
//
// usage: proto-mpm-shadow <lock|update|sync|status|render> --candidate a|b|c|d [--dir DIR | --global]
//        [--check] [--guard none|cas|mutex] [--resolve id=version]... [--no-fallback]
//        render --lock FILE --out DIR [--legacy-header]
// Exit codes: 0 ok, 1 drift or mismatch, 2 usage or input error, 3 reserved, 4 lock written but mpm files not rendered.

use proto_mpm_files_prototype::*;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    std::process::exit(real_main());
}

fn real_main() -> i32 {
    let mut args = std::env::args().skip(1);
    let Some(cmd) = args.next() else {
        eprintln!("missing subcommand");
        return 2;
    };
    let mut cand = Candidate::A;
    let mut dir = PathBuf::from(".");
    let mut global = false;
    let mut check = false;
    let mut guard = Guard::None;
    let mut fallback = true;
    let mut resolutions = BTreeMap::new();
    let mut lock_file = None;
    let mut out = None;
    let mut legacy = false;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--candidate" => match Candidate::parse(&args.next().unwrap_or_default()) {
                Ok(c) => cand = c,
                Err(e) => {
                    eprintln!("{e}");
                    return 2;
                }
            },
            "--dir" => dir = PathBuf::from(args.next().unwrap_or_default()),
            "--global" => global = true,
            "--check" => check = true,
            "--guard" => {
                guard = match args.next().unwrap_or_default().as_str() {
                    "none" => Guard::None,
                    "cas" => Guard::Cas,
                    "mutex" => Guard::Mutex,
                    other => {
                        eprintln!("unknown guard {other}");
                        return 2;
                    }
                }
            }
            "--no-fallback" => fallback = false,
            "--resolve" => {
                let kv = args.next().unwrap_or_default();
                match kv.rsplit_once('=') {
                    Some((k, v)) => {
                        resolutions.insert(k.to_string(), v.to_string());
                    }
                    None => {
                        eprintln!("--resolve wants id=version");
                        return 2;
                    }
                }
            }
            "--lock" => lock_file = args.next(),
            "--out" => out = args.next(),
            "--legacy-header" => legacy = true,
            other => {
                eprintln!("unknown argument {other}");
                return 2;
            }
        }
    }
    if cmd == "render" {
        let (Some(lock_file), Some(out)) = (lock_file, out) else {
            eprintln!("render wants --lock and --out");
            return 2;
        };
        let lock = match read_json(std::path::Path::new(&lock_file)) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("{e}");
                return 2;
            }
        };
        let header = if legacy { render::LEGACY_HEADER } else { cand.header() };
        let out = std::path::Path::new(&out);
        let _ = std::fs::write(out.join("mpm.packages.toml"), render::render_packages(&lock, header));
        let _ = std::fs::write(out.join("mpm.managers.toml"), render::render_managers(&lock, header));
        return 0;
    }
    let scope = if global {
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
    match cmd.as_str() {
        "lock" | "update" if check => {
            let mut drift = check_lock_vs_plan(&scope);
            if cand.shadow_writes_mpm_files() {
                drift.extend(check_mpm_files(&scope, cand.header()));
            }
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
        "lock" | "update" => {
            if let Err(e) = write_cycle(&scope, cand, guard, &resolutions) {
                eprintln!("{e}");
                return 2;
            }
            if cand == Candidate::C {
                // C: the lock changed, so ask file-enforcer to render the mpm files from it.
                let bin = std::env::var("PROTO_FE_BIN").unwrap_or_else(|_| "proto-file-enforcer".to_string());
                let mut c = Command::new(&bin);
                if global {
                    c.args(["user", "write"]);
                } else {
                    c.args(["write", "--dir", &dir.to_string_lossy()]);
                }
                c.args(["--candidate", "c"]);
                match c.status() {
                    Ok(s) if s.success() => 0,
                    Ok(s) => {
                        eprintln!("file-enforcer write failed: {s}");
                        4
                    }
                    Err(e) => {
                        eprintln!("lock written, but cannot run {bin} to render the mpm files: {e}");
                        4
                    }
                }
            } else {
                0
            }
        }
        "sync" => sync(&scope, cand, fallback),
        "status" => {
            if status(&scope) == 0 {
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
