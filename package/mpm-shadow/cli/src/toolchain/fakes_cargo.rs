//! Test doubles for `cargo binstall --root`:
//! the effect a successful install has on disk,
//! and a helper that finishes an install by hand.

/// Imports `Path` from `std::path`.
use std::path::{Path, PathBuf};

/// Imports `read_recorded_installs` from `super::cargo_record`.
use super::cargo_record::read_recorded_installs;
/// Imports `write_executable` from `super::fakes`.
use super::fakes::write_executable;
/// Imports `RunRequest` from `super::runner`.
use super::runner::RunRequest;

/// Writes cargo's record for a finished install under `root` and an executable stand-in for each of `bins`.
pub(crate) fn finish_install(root: &Path, crate_name: &str, version: &str, bins: &[&str]) {
    for bin in bins {
        write_executable(&root.join("bin").join(bin));
    }
    std::fs::create_dir_all(root).expect("make the root");
    std::fs::write(
        root.join(".crates.toml"),
        format!("[v1]\n\"{crate_name} {version} (registry+https://example.invalid/index)\" = [\"{}\"]\n", bins[0]),
    )
    .expect("write cargo's record");
}

/// What:
/// The effect of a `cargo binstall --root <root> --version <version> <crate>` that succeeded,
/// for `FakeRunner::on`.
/// Why:
/// Install tests prove where an install landed and what `list` reads back,
/// without running cargo.
pub(crate) fn fake_binstall(bins: &'static [&'static str]) -> Box<dyn Fn(&RunRequest)> {
    return Box::new(move |request: &RunRequest| {
        let argument_after = |flag: &str| -> String {
            let position: usize = request.args.iter().position(|argument| return argument == flag).expect("the flag is passed");
            return request.args[position + 1].clone();
        };
        let root: PathBuf = PathBuf::from(argument_after("--root"));
        let version: String = argument_after("--version");
        let crate_name: String = request.args.last().expect("a crate name").clone();
        finish_install(&root, &crate_name, &version, bins);
    });
}

/// What:
/// The effect of a `cargo uninstall --root <root> --bin <bin> <crate>` that succeeded,
/// for `FakeRunner::on`:
/// the bin is deleted from `<root>/bin` and dropped from the record,
/// and an install left with no bin leaves the record.
/// Why:
/// Cleanup tests prove which copy went and what cargo's record says afterwards,
/// without running cargo.
pub(crate) fn fake_uninstall() -> Box<dyn Fn(&RunRequest)> {
    return Box::new(move |request: &RunRequest| {
        let argument_after = |flag: &str| -> String {
            let position: usize = request.args.iter().position(|argument| return argument == flag).expect("the flag is passed");
            return request.args[position + 1].clone();
        };
        let root: PathBuf = PathBuf::from(argument_after("--root"));
        let bin: String = argument_after("--bin");
        std::fs::remove_file(root.join("bin").join(&bin)).expect("cargo removes the bin it records");
        let mut record: String = String::from("[v1]\n");
        for install in read_recorded_installs(&root) {
            let kept: Vec<String> = install.bins.iter().filter(|name| return **name != bin).cloned().collect();
            if kept.is_empty() {
                continue;
            }
            let quoted: Vec<String> = kept.iter().map(|name| return format!("\"{name}\"")).collect();
            record.push_str(&format!(
                "\"{} {} (registry+https://example.invalid/index)\" = [{}]\n",
                install.crate_name,
                install.version,
                quoted.join(", ")
            ));
        }
        std::fs::write(root.join(".crates.toml"), record).expect("write cargo's record");
    });
}
