//! What:
//! The controls of this repository's `rust` row against a machine that holds exactly what the row
//! declares and nothing more:
//! a fixture toolchain built from the programs of the row's own components
//! and of rustup's `minimal` profile,
//! with an install of the repository's row run against it.
//! Why:
//! Issue 6630 is that the row declares `cargo-fmt` and `rustfmt`,
//! which no component the row declares installs,
//! so a machine that follows the plan has neither program
//! and its install names both as unresolved.
//! Every program of a component is read from a copy of that toolchain's own `manifest-*` files,
//! kept here as data with where and when it was taken,
//! so the fixture holds what rustup installs and not what this machine happens to have.
//! The install runs against a disposable home and the fake runner answers every `rustup` call,
//! so no real toolchain and no manager on this machine is read or started.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // the-rust-rows-own-toolchain-holds-every-bin-it-declares.test.ts
//! ```

// What:
// `use std::fs;` imports the standard library's file system module,
// which reads the repository's lock and the fixture toolchain's `bin` directory.
// Why:
// Both are files on disk,
// and the control reads what is written there rather than a value it remembers writing.
//
// In TS you'd write (pseudocode):
// ```ts
// import { readFileSync, readdirSync } from 'node:fs';
// ```
use std::fs;
// What:
// `use std::path::{Path, PathBuf}` imports two path types:
// `Path` is a borrowed path (the sibling of `&PathBuf`),
// and `PathBuf` is the owned, growable one (the sibling of `String` against `&str`).
// Why:
// The repository root is borrowed from the manifest path,
// while the fixture toolchain's directory is built and kept in a function of its own.
//
// In TS you'd write (pseudocode):
// ```ts
// // No 1:1 equivalent: Node's paths are plain strings.
// ```
use std::path::{
    Path,
    PathBuf,
};

// What:
// `use serde_json::{Map, Value}` imports the two JSON types the fixture's plan document is built
// from: a map of keys to values, and a value of any JSON shape.
// Why:
// A `list` control writes the plan file the command reads,
// and building it from the same types the program's own writer uses keeps the shapes equal.
// The siblings a reader might expect are a typed struct (`serde`'s derive), which cannot carry an
// arbitrary field list, and a `String` written by hand, which could disagree with the decoder.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Map, Value } from './json.ts';
// ```
use serde_json::{
    Map,
    Value,
};

// What:
// `use super::fakes::{Rig, write_executable}` imports the test rig,
// which holds a disposable scope and the two fakes,
// and the writer of the stand-in program a fixture toolchain holds.
// Why:
// The install needs a scope and a recording runner,
// and the fixture toolchain is made of stand-in programs written that way.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Rig, writeExecutable } from './fakes.ts';
// ```
use super::fakes::{
    Rig,
    write_executable,
};
// What:
// `use super::lock::parse_lock` imports the reader of `toolchain.lock.json`.
// Why:
// The row under test is the repository's own declaration,
// and this is the decoder the program itself uses on that file.
//
// In TS you'd write (pseudocode):
// ```ts
// import { parseLock } from './lock.ts';
// ```
use super::lock::parse_lock;
// What:
// `use super::lock::{Lock, render_lock}` imports the lock document and the writer of its text.
// Why:
// A `list` control reads the plan and the lock from the fixture's disk,
// so the fixture has to write a lock holding exactly the row under test.
//
// In TS you'd write (pseudocode):
// ```ts
// import { type Lock, renderLock } from './lock.ts';
// ```
use super::lock::{
    Lock,
    render_lock,
};
// What:
// `use super::model_encode::{lock_entry_value, render_pretty_json}` imports the encoder of one lock
// row and the JSON printer the plan's text is built with.
// Why:
// The plan entry of the fixture's one row is the lock row without `version` and `artifacts`,
// which is the shape the program's own encoder writes.
//
// In TS you'd write (pseudocode):
// ```ts
// import { lockEntryValue, renderPrettyJson } from './model-encode.ts';
// ```
use super::model_encode::{
    lock_entry_value,
    render_pretty_json,
};
// What:
// `use super::model::LockedTool` imports the type of one lock row:
// a declaration, a resolved version and the artifacts of that version.
// Why:
// The row is handed to the install and its channel is read from it.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { LockedTool } from './model.ts';
// ```
use super::model::LockedTool;
// What:
// `use super::runner::RunRequest` imports what the runner hands an effect:
// the program, its arguments, its environment and its working directory.
// Why:
// The stand-in for `rustup component add` is an effect the fake runner calls with that request.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { RunRequest } from './runner.ts';
// ```
use super::runner::RunRequest;
// What:
// `use super::scope::LOCK_FILE_NAME` imports the name of the lock file,
// so this file names the file the program names.
// Why:
// A second spelling of `toolchain.lock.json` here could drift from the program's.
//
// In TS you'd write (pseudocode):
// ```ts
// import { LOCK_FILE_NAME } from './scope.ts';
// ```
use super::scope::LOCK_FILE_NAME;
// What:
// `use super::status_cmd::run_status` imports the `list` command's own report.
// Why:
// The controls below read the lines a person reads,
// not an intermediate record.
//
// In TS you'd write (pseudocode):
// ```ts
// import { runStatus } from './status-cmd.ts';
// ```
use super::status_cmd::run_status;
// What:
// `use super::sync::sync_tools` imports the install of every tool a lock holds.
// Why:
// The controls' subject is what an install does with the row's components and bins.
//
// In TS you'd write (pseudocode):
// ```ts
// import { syncTools } from './sync.ts';
// ```
use super::sync::sync_tools;
// What:
// `use super::sync_report_text::format_sync_report` imports the builder of the text `install` prints.
// Why:
// The issue is about the last line of that text,
// and a person acts on those lines.
//
// In TS you'd write (pseudocode):
// ```ts
// import { formatSyncReport } from './sync-report-text.ts';
// ```
use super::sync_report_text::format_sync_report;

// What:
// `const CRATE_DIRECTORIES_BELOW_THE_ROOT: usize = 3` is how many directory levels
// this crate's folder sits below the repository root
// (`package/mpm-shadow/cli`).
// The integer siblings a reader might expect are `u32`, `u64`, `i32` and `i64`.
// Why:
// `Path::ancestors` counts a number of levels,
// so `usize` is the width its `nth` takes and no cast is needed at the call
// (not `u32` or `u64`, which would need one).
//
// In TS you'd write (pseudocode):
// ```ts
// const crateDirectoriesBelowTheRoot = 3;
// ```
const CRATE_DIRECTORIES_BELOW_THE_ROOT: usize = 3;

// What:
// `const THE_FIXTURE_HOST_TRIPLE: &str = "x86_64-unknown-linux-gnu"` is the second half of the
// directory name rustup gives an installed channel
// (`nightly-2026-11-16-x86_64-unknown-linux-gnu`).
// Its string sibling `String` is an owned buffer, which a constant never needs.
// Why:
// The farm reaches a toolchain's programs through `~/.rustup/toolchains/<channel>-<triple>/bin`,
// and rustup's own listings spell a component by appending this same triple,
// so one borrowed constant states the host both places take.
//
// In TS you'd write (pseudocode):
// ```ts
// const theFixtureHostTriple = 'x86_64-unknown-linux-gnu';
// ```
const THE_FIXTURE_HOST_TRIPLE: &str = "x86_64-unknown-linux-gnu";

// What:
// `const THE_COMPONENT_THIS_ISSUE_DECLARES: &str = "rustfmt"` is the rustup component whose two
// programs, `cargo-fmt` and `rustfmt`, the repository's own tasks run
// (`package/linter/monochromatic-lint/mise.toml` runs a bare `rustfmt`,
// and `cargo fmt` runs `cargo-fmt`).
// Why:
// The controls below have to name the one component the row gains,
// so they build a toolchain with it and without it by the same name.
//
// In TS you'd write (pseudocode):
// ```ts
// const theComponentThisIssueDeclares = 'rustfmt';
// ```
const THE_COMPONENT_THIS_ISSUE_DECLARES: &str = "rustfmt";

// What:
// `const A_TOOLCHAIN_OLDER_THAN_THE_PIN: &str = "nightly-2026-11-01"` is the channel of a second
// toolchain installed beside the repository's pin,
// fifteen days older than the pin's `nightly-2026-11-16`,
// so rustup's own directory name for it sorts before the pin's.
// Its string sibling `String` is an owned buffer,
// which a constant never needs.
// Why:
// The controls below are about a machine holding two toolchains,
// and the fault issue 6634 records shows itself only when the second one sorts first:
// a search that takes the first directory by name finds the older toolchain before the pin.
// Gotcha:
// The name is a stand-in for rustup's own directory: no toolchain was installed for it.
// Each control asserts the sort order it relies on instead of trusting this date.
//
// In TS you'd write (pseudocode):
// ```ts
// const aToolchainOlderThanThePin = 'nightly-2026-11-01';
// ```
const A_TOOLCHAIN_OLDER_THAN_THE_PIN: &str = "nightly-2026-11-01";

// What:
// `const A_TOOLCHAIN_NEWER_THAN_THE_PIN: &str = "nightly-2026-12-01"` is the channel of a second
// toolchain installed beside the repository's pin,
// later than the pin's `nightly-2026-11-16`,
// so rustup's own directory name for it sorts after the pin's.
// Its string sibling `String` is an owned buffer,
// which a constant never needs.
// Why:
// A search that took the last directory by name would pass the older-toolchain control for the
// wrong reason,
// so this second control holds a toolchain that sorts on the other side of the pin.
// Gotcha:
// A stand-in for rustup's own directory, as the older one is;
// each control asserts the sort order it relies on.
//
// In TS you'd write (pseudocode):
// ```ts
// const aToolchainNewerThanThePin = 'nightly-2026-12-01';
// ```
const A_TOOLCHAIN_NEWER_THAN_THE_PIN: &str = "nightly-2026-12-01";

// What:
// `struct Component` is one rustup component as the fixtures record it:
// the name the plan writes and `rustup component add` takes,
// the `manifest-*` file the lines below were copied from,
// and the programs that component puts in the toolchain's `bin` directory.
// `programs` is `&'static [&'static str]`, a borrowed view of a fixed list written in this file;
// its siblings are `[&'static str; N]` (a fixed-size array,
// which cannot be written as this field without naming every length)
// and `Vec<&'static str>` (growable and allocated, which a constant cannot be).
// Why:
// One component's three facts belong together,
// so the fixture is built from them and the provenance of each program stays beside it.
//
// In TS you'd write (pseudocode):
// ```ts
// type Component = { name: string; manifest: string; programs: readonly string[] };
// ```
struct Component {
    // What:
    // `name: &'static str` is the component's name as the plan declares it and as `rustup`
    // takes it on a command line (`clippy`, `rustfmt`).
    // Its string sibling `String` is an owned buffer, which a constant never needs.
    // Why:
    // `rustup component add <name>` is what an install runs for it,
    // and a lock's `rustComponents` row holds this same text.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // name: string;
    // ```
    name: &'static str,
    // What:
    // `manifest: &'static str` is the name of the file the programs below were read from,
    // under the toolchain's `lib/rustlib` directory
    // (`manifest-clippy-preview-<triple>`, `manifest-rust-src`).
    // Why:
    // rustup's file name for a component is not always the name it installs by
    // (`clippy` lives in `manifest-clippy-preview-<triple>`),
    // so the source of each line is written beside it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // manifest: string;
    // ```
    manifest: &'static str,
    // What:
    // `programs: &'static [&'static str]` is the list of `bin/` entries that manifest names,
    // which is the programs the component puts in the toolchain's `bin` directory.
    // An empty list means the component installs no program there
    // (`rust-src` is source only, and `llvm-tools`' programs sit under
    // `lib/rustlib/<triple>/bin`).
    // Why:
    // These are the files the fixture toolchain holds and the programs the farm has to link.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // programs: readonly string[];
    // ```
    programs: &'static [&'static str],
}

// What:
// `impl Component` holds the two questions this file asks of the data:
// the name `rustup component list --installed` prints for the component,
// and whether it provides a program the row declares.
// Why:
// The listing's spelling is a fact about rustup, not about the row,
// so it is derived once from the manifest name rather than written twice.
//
// In TS you'd write (pseudocode):
// ```ts
// class Component { listedWithItsTriple(): boolean; ... }
// ```
impl Component {
    // What:
    // `fn listed_with_its_host_triple(&self) -> bool` answers whether rustup appends the host
    // triple when it prints this component in `rustup component list --installed`.
    // Why:
    // A component built for this host has a manifest named after that triple
    // (`manifest-clippy-preview-x86_64-unknown-linux-gnu` prints as
    // `clippy-x86_64-unknown-linux-gnu`),
    // and one with no triple in its manifest name prints bare (`rust-src`),
    // so the manifest name is what decides it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // listedWithItsHostTriple(): boolean { return this.manifest.endsWith(theFixtureHostTriple); }
    // ```
    fn listed_with_its_host_triple(&self) -> bool {
        // What:
        // `.ends_with(...)` is a borrowed-text test that says whether the tail of the manifest
        // name is the host triple.
        // Why:
        // That is the whole question, and no allocation is needed to ask it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return this.manifest.endsWith(theFixtureHostTriple);
        // ```
        return self.manifest.ends_with(THE_FIXTURE_HOST_TRIPLE);
    }

    // What:
    // `fn listed_as(&self) -> String` is the line `rustup component list --installed` prints for
    // this component: its name, then the host triple when the component is built for the host.
    // Why:
    // The stand-in listing in the controls below has to be the text a real rustup prints,
    // or the reconciler is tested against a shape that never occurs.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // listedAs(): string { return this.listedWithItsHostTriple() ? `${this.name}-${theFixtureHostTriple}` : this.name; }
    // ```
    fn listed_as(&self) -> String {
        if self.listed_with_its_host_triple() {
            // What:
            // `format!("{}-{}", self.name, THE_FIXTURE_HOST_TRIPLE)` builds one owned string by
            // pasting the two borrowed pieces around a hyphen.
            // Why:
            // The listing line is one `String`, and this is the host-built spelling of it.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return `${this.name}-${theFixtureHostTriple}`;
            // ```
            return format!("{}-{}", self.name, THE_FIXTURE_HOST_TRIPLE);
        }
        // What:
        // `self.name.to_string()` copies the borrowed name into an owned `String`,
        // because the caller's answer is one type either way.
        // Why:
        // A component with no triple in its manifest name prints bare, and the caller joins these
        // lines into one listing text.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return this.name;
        // ```
        return self.name.to_string();
    }
}

// What:
// `const THE_COMPONENTS: [Component; 8]` is every component this file's controls need,
// with the programs its manifest lists and the manifest's own name.
// Why:
// The programs a component installs are what decides whether the row's `bins` can resolve,
// and they are copied from the installed manifests' `file:bin/...` lines rather than recalled.
// Gotcha:
// The lines were taken on 2026-11-27T17:34:07-05:00 from
// `/home/user/.rustup/toolchains/nightly-2026-11-16-x86_64-unknown-linux-gnu/lib/rustlib/manifest-*`
// (the toolchain this lock pins), by
// `grep '^file:bin/' manifest-<name> | sed 's/^file:bin\///' | sort`;
// they are the files that toolchain holds, so a later rustup that moves a program needs this
// table read again.
// The table holds only components that come with the host;
// a standard library built for another target is a component of its own
// (`rust-std-<triple>`), which `rustup target add` installs and which puts no program in `bin`,
// so the row's `rustTargets` are handled where the listing is built and not here.
//
// In TS you'd write (pseudocode):
// ```ts
// const theComponents: readonly Component[] = [ ... ];
// ```
const THE_COMPONENTS: [Component; 8] = [
    // `cargo` is the profile's own compiler driver, from `manifest-cargo-<triple>`.
    Component {
        name: "cargo",
        manifest: "manifest-cargo-x86_64-unknown-linux-gnu",
        programs: &["cargo"],
    },
    // `rust-std` is the profile's own standard library, from `manifest-rust-std-<triple>`:
    // it installs libraries and no program.
    Component {
        name: "rust-std",
        manifest: "manifest-rust-std-x86_64-unknown-linux-gnu",
        programs: &[],
    },
    // `rustc` is the profile's own compiler and documentation tools,
    // from `manifest-rustc-<triple>`.
    Component {
        name: "rustc",
        manifest: "manifest-rustc-x86_64-unknown-linux-gnu",
        programs: &["rustc", "rustdoc", "rust-gdb", "rust-gdbgui", "rust-lldb"],
    },
    // `clippy` is a component this repository's row declares,
    // and rustup's file for it is `manifest-clippy-preview-<triple>`.
    Component {
        name: "clippy",
        manifest: "manifest-clippy-preview-x86_64-unknown-linux-gnu",
        programs: &["cargo-clippy", "clippy-driver"],
    },
    // `rust-src` is a component this repository's row declares,
    // and its manifest name carries no triple: it installs the compiler's sources.
    Component {
        name: "rust-src",
        manifest: "manifest-rust-src",
        programs: &[],
    },
    // `llvm-tools` is a component this repository's row declares,
    // and its manifest is `manifest-llvm-tools-preview-<triple>`.
    // Its programs (`llvm-cov`, `llvm-nm` and the rest) are under
    // `lib/rustlib/<triple>/bin`, never the toolchain's own `bin`, so no line here names one.
    Component {
        name: "llvm-tools",
        manifest: "manifest-llvm-tools-preview-x86_64-unknown-linux-gnu",
        programs: &[],
    },
    // `rustc-dev` is a component this repository's row declares,
    // and its manifest is `manifest-rustc-dev-<triple>`.
    // It installs compiler libraries, so it puts no program in the toolchain's `bin`.
    Component {
        name: "rustc-dev",
        manifest: "manifest-rustc-dev-x86_64-unknown-linux-gnu",
        programs: &[],
    },
    // `rustfmt` is the component this issue declares, and rustup's file for it is
    // `manifest-rustfmt-preview-<triple>`: it is the only component that installs
    // `cargo-fmt` and `rustfmt`.
    Component {
        name: "rustfmt",
        manifest: "manifest-rustfmt-preview-x86_64-unknown-linux-gnu",
        programs: &["cargo-fmt", "rustfmt"],
    },
];

// What:
// `const THE_MINIMAL_PROFILES_COMPONENTS: [&str; 3]` is the components rustup's `minimal` profile
// installs on a Linux host:
// `rustc`, `cargo` and `rust-std`.
// It is a fixed-size array of borrowed strings;
// its siblings are `Vec<&str>` (growable, allocated) and `&[&str]` (a borrowed view).
// Why:
// A toolchain installed from this lock's channel gets that profile before any declared component,
// so the fixture has to hold the profile's programs too.
// Gotcha:
// The profile line was read on 2026-11-27T17:34:07-05:00 from
// `/home/user/.rustup/toolchains/nightly-2026-11-16-x86_64-unknown-linux-gnu/lib/rustlib/multirust-channel-manifest.toml`,
// section `[profiles]`:
// `minimal = ["rustc", "cargo", "rust-std", "rust-mingw"]`.
// `rust-mingw` is left out because it is a Windows-only component with no manifest here,
// and it installs nothing on this host.
//
// In TS you'd write (pseudocode):
// ```ts
// const theMinimalProfilesComponents = ['rustc', 'cargo', 'rust-std'];
// ```
const THE_MINIMAL_PROFILES_COMPONENTS: [&str; 3] = ["rustc", "cargo", "rust-std"];

// What:
// `const THE_COMPONENT_LISTING_THIS_MACHINES_RUSTUP_PRINTS: [&str; 10]` is what
// `rustup component list --installed --toolchain nightly-2026-11-16` printed on this machine.
// Why:
// The stand-in listing the controls answer with is derived, so this capture is what checks the
// derivation: a spelling this list does not hold is one no real rustup would print.
// Gotcha:
// The lines are a real capture, taken on 2026-11-27T17:35:46-05:00 into
// `/home/user/temp/agent/issue-6630-the-rust-rows-bins-are-the-programs-its-declared-components-install-scratch/keep/rustup-queries-2026-11-27.txt`
// (rustup 1.29.3).
// That toolchain holds a profile's three components, the row's four declared ones,
// `rustfmt` (which that machine had added by hand), and the two Android standard libraries
// the row's `rustTargets` install, which is the ten lines here.
//
// In TS you'd write (pseudocode):
// ```ts
// const theComponentListingThisMachinesRustupPrints = [...];
// ```
const THE_COMPONENT_LISTING_THIS_MACHINES_RUSTUP_PRINTS: [&str; 10] = [
    "cargo-x86_64-unknown-linux-gnu",
    "clippy-x86_64-unknown-linux-gnu",
    "llvm-tools-x86_64-unknown-linux-gnu",
    "rust-src",
    "rust-std-aarch64-linux-android",
    "rust-std-x86_64-linux-android",
    "rust-std-x86_64-unknown-linux-gnu",
    "rustc-x86_64-unknown-linux-gnu",
    "rustc-dev-x86_64-unknown-linux-gnu",
    "rustfmt-x86_64-unknown-linux-gnu",
];

// What:
// `fn component_named(name: &str) -> &'static Component` finds the fixture record of one
// component, and fails the test when there is none.
// Why:
// A component the row declares but this table does not hold would leave the fixture silently
// short of programs,
// so the failure has to name the component and say what to do.
//
// In TS you'd write (pseudocode):
// ```ts
// function componentNamed(name: string): Component  // throws when the table has no record
// ```
fn component_named(name: &str) -> &'static Component {
    // What:
    // the loop walks the table in written order and `return` hands back the one record whose name
    // is this text.
    // Why:
    // The table is small and written here, so a walk needs no map and no allocation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const component of theComponents) { if (component.name === name) return component; }
    // ```
    for component in &THE_COMPONENTS {
        if component.name == name {
            return component;
        }
    }
    // What:
    // `panic!(...)` ends the test with this sentence,
    // and `{name}` pastes the missing component's name into it.
    // Why:
    // A person reading the failure has to learn which component has no manifest lines and where
    // they come from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // throw new Error(`no manifest lines are recorded for the component ${name}; copy them from the toolchain's lib/rustlib/manifest-* files`);
    // ```
    panic!(
        "no manifest lines are recorded for the component {name}; copy its `file:bin/...` lines from the toolchain's lib/rustlib/manifest-* files"
    );
}

// What:
// `fn the_programs_of(names: &[String]) -> Vec<String>` is every program those components install
// into the toolchain's `bin` directory, sorted and without duplicates.
// `Vec<String>` is a growable list of owned strings
// (the siblings are `&[String]`, a borrowed slice, and `[String; N]`, a fixed-size array).
// Why:
// The fixture toolchain is built from this list, and the farm links exactly these names.
//
// In TS you'd write (pseudocode):
// ```ts
// function theProgramsOf(names: string[]): string[]
// ```
fn the_programs_of(names: &[String]) -> Vec<String> {
    // What:
    // `let mut programs: Vec<String> = Vec::new()` starts an empty growable list of owned strings.
    // Why:
    // The programs are collected from several components, so the number is only known at run time.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const programs: string[] = [];
    // ```
    let mut programs: Vec<String> = Vec::new();
    for name in names {
        for program in component_named(name).programs {
            // What:
            // `.push(program.to_string())` adds an owned copy of the borrowed name to the list.
            // Why:
            // The list is returned past this loop's borrow of the constant table.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // programs.push(program);
            // ```
            programs.push(program.to_string());
        }
    }
    programs.sort();
    programs.dedup();
    // What:
    // `programs` is the tail expression, so Rust returns this list to the caller.
    // Why:
    // The caller needs the whole list to write the fixture and to compare it with the directory.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return programs;
    // ```
    return programs;
}

// What:
// `fn the_components_the_row_declares(row: &LockedTool) -> Vec<String>` is the row's
// `rustComponents` list, or an empty list when the row declares none.
// Why:
// These are the components an install adds, and the fixture has to hold their programs.
//
// In TS you'd write (pseudocode):
// ```ts
// function theComponentsTheRowDeclares(row: LockedTool): string[]
// ```
fn the_components_the_row_declares(row: &LockedTool) -> Vec<String> {
    // What:
    // `.clone()` copies the declaration's list out of the lock entry,
    // and `.unwrap_or_default()` substitutes an empty list when the row holds none
    // (its sibling `.unwrap_or(value)` takes a value).
    // Why:
    // The caller reads the list after the borrow of the row ends, and a row without the option
    // has to answer "no components" rather than fail.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return [...(row.declaration.options.rustComponents ?? [])];
    // ```
    return row
        .declaration
        .options
        .rust_components
        .clone()
        .unwrap_or_default();
}

// What:
// `fn the_components_a_toolchain_of_this_row_holds(row: &LockedTool) -> Vec<String>` is the
// profile's components followed by the row's declared ones.
// Why:
// An install of this row first gets the channel's `minimal` profile and then adds what the row
// declares, so a machine that follows the plan holds exactly these.
//
// In TS you'd write (pseudocode):
// ```ts
// function theComponentsAToolchainOfThisRowHolds(row: LockedTool): string[]
// ```
fn the_components_a_toolchain_of_this_row_holds(row: &LockedTool) -> Vec<String> {
    // What:
    // `let mut names: Vec<String> = Vec::new()` is the list being built:
    // the profile's components, then the row's.
    // Why:
    // Both groups mean the same thing to the fixture, so they are one list before it is used.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const names: string[] = [];
    // ```
    let mut names: Vec<String> = Vec::new();
    for name in THE_MINIMAL_PROFILES_COMPONENTS {
        // What:
        // `.push(name.to_string())` adds an owned copy of the profile component's borrowed name.
        // Why:
        // The list holds `String`s because the row's own names are owned too.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // names.push(name);
        // ```
        names.push(name.to_string());
    }
    names.extend(the_components_the_row_declares(row));
    return names;
}

// What:
// `fn repository_root() -> PathBuf` is the repository's own folder,
// found from the folder this crate was compiled from.
// Why:
// The row under test is this repository's declaration, so the control reads the committed lock
// instead of a copy that could drift from it.
//
// In TS you'd write (pseudocode):
// ```ts
// function repositoryRoot(): string
// ```
fn repository_root() -> PathBuf {
    // What:
    // `env!("CARGO_MANIFEST_DIR")` is the crate's folder, pasted in at compile time.
    // `Path::new` borrows it as a path, and `ancestors()` yields that folder and then each parent,
    // so `.nth(3)` is the repository root.
    // Why:
    // The lock is three levels above the crate,
    // and a derived path does not depend on the directory the test runs in.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const manifest = import.meta.dirname;
    // const root = dirname(dirname(dirname(manifest)));
    // ```
    let ancestor: Option<&Path> = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(CRATE_DIRECTORIES_BELOW_THE_ROOT);
    // What:
    // `.expect(...)` unwraps the present variant of `Option`
    // and ends the test with this sentence when it is `None`.
    // Why:
    // A layout change that moves the crate should say so here,
    // not fail later with a missing file.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (ancestor === undefined) throw new Error('the crate sits three levels below the root');
    // ```
    let root: &Path = ancestor.expect("the crate sits three levels below the repository root");
    // What:
    // `root.to_path_buf()` copies the borrowed path into an owned `PathBuf`,
    // so the caller can keep it past this frame.
    // Why:
    // The lock's path is joined onto it in another function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return root;
    // ```
    return root.to_path_buf();
}

// What:
// `fn the_repository_rust_row() -> LockedTool` is the lock's row for this repository's Rust
// toolchain, decoded from `toolchain.lock.json` at the repository root.
// Why:
// The control's subject is the committed row,
// so it reads that file rather than a row written into this test.
//
// In TS you'd write (pseudocode):
// ```ts
// function theRepositoryRustRow(): LockedTool
// ```
fn the_repository_rust_row() -> LockedTool {
    // What:
    // `.join(...)` builds the lock's path from the repository root,
    // and the name is the constant the scope uses.
    // Why:
    // The file read is the one `install` reads, not a copy.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const lockFile = pathJoin(repositoryRoot(), LOCK_FILE_NAME);
    // ```
    let lock_file: PathBuf = repository_root().join(LOCK_FILE_NAME);
    // What:
    // `fs::read_to_string` returns `Result<String, io::Error>`,
    // and `.expect(...)` unwraps its success value.
    // Why:
    // A missing lock is a broken checkout, and the test says which file it wanted.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const text = readFileSync(lockFile, 'utf8');
    // ```
    let text: String =
        fs::read_to_string(&lock_file).expect("the repository's toolchain.lock.json is readable");
    // What:
    // `.display()` renders the path for a message,
    // and `.to_string()` copies that text into an owned `String` the decoder can keep.
    // Why:
    // A decode error names its file, and the label must outlive the borrow of the path expression.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const label = String(lockFile);
    // ```
    let label: String = lock_file.display().to_string();
    // What:
    // `parse_lock` decodes the lock's text, and `.expect(...)` unwraps the `Result`,
    // so a lock this crate cannot read fails here with the decoder's own sentence.
    // Why:
    // The row is only trustworthy if the committed file decodes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const parsed = parseLock(text, label);
    // ```
    let parsed = parse_lock(&text, &label).expect("the repository's lock decodes");
    for tool in parsed.lock.tools {
        if tool.declaration.id == "rust" {
            return tool;
        }
    }
    // What:
    // `panic!(...)` ends the test with this sentence.
    // Why:
    // The repository's plan and lock both hold a `rust` row;
    // a lock without one means the control no longer has a subject.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // throw new Error(`${label} has no rust row`);
    // ```
    panic!("{label} has no rust row");
}

// What:
// `fn the_toolchain_bin(rig: &Rig, row: &LockedTool) -> PathBuf` is
// `<fixture home>/.rustup/toolchains/<channel>-<triple>/bin`, the directory the farm reads a
// `rustup-toolchain` row's programs from.
// Why:
// The fixture has to sit exactly where `bin_search_dirs` looks,
// or the control would pass or fail for the wrong reason.
//
// In TS you'd write (pseudocode):
// ```ts
// function theToolchainBin(rig: Rig, row: LockedTool): string
// ```
fn the_toolchain_bin(
    rig: &Rig,
    row: &LockedTool,
) -> PathBuf {
    // What:
    // the chain joins the home, `.rustup`, `toolchains`, the channel's own directory name and `bin`;
    // `format!` builds that name from the row's channel and the host triple.
    // Why:
    // `rustup` names an installed channel `<channel>-<triple>`, and the search descends into it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return pathJoin(rig.scope.home, '.rustup', 'toolchains', `${row.rustupChannel()}-${theFixtureHostTriple}`, 'bin');
    // ```
    return rig
        .scope
        .home
        .join(".rustup")
        .join("toolchains")
        .join(format!(
            "{}-{}",
            row.rustup_channel(),
            THE_FIXTURE_HOST_TRIPLE
        ))
        .join("bin");
}

// What:
// `fn the_program_names_in(directory: &Path) -> Vec<String>` is the names of the files directly
// inside one directory, sorted.
// Why:
// The control has to show what the fixture toolchain holds,
// not only what the test meant to write into it.
//
// In TS you'd write (pseudocode):
// ```ts
// function theProgramNamesIn(directory: string): string[]
// ```
// What:
// `fn a_second_toolchain(rig: &Rig, channel: &str, programs: &[String]) -> PathBuf` writes one more
// installed toolchain into the fixture's own rustup home and hands back its directory.
// Why:
// The farm reads a `rustup-toolchain` row's programs under `<home>/.rustup/toolchains`,
// so a second toolchain the controls can see has to sit there,
// named `<channel>-<triple>` the way rustup names one.
//
// In TS you'd write (pseudocode):
// ```ts
// function aSecondToolchain(rig: Rig, channel: string, programs: string[]): string
// ```
fn a_second_toolchain(
    rig: &Rig,
    channel: &str,
    programs: &[String],
) -> PathBuf {
    // What:
    // the chain joins the home, `.rustup`, `toolchains` and the directory name `format!` builds from
    // the channel and the host triple.
    // Why:
    // That is the one place the search looks,
    // and a name built any other way would be a directory no rustup ever makes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const directory = pathJoin(rig.scope.home, '.rustup', 'toolchains', `${channel}-${theFixtureHostTriple}`);
    // ```
    let directory: PathBuf = rig
        .scope
        .home
        .join(".rustup")
        .join("toolchains")
        .join(format!("{channel}-{THE_FIXTURE_HOST_TRIPLE}"));
    // What:
    // one stand-in program per name, each written into the toolchain's own `bin` directory.
    // Why:
    // A toolchain holds its programs there,
    // and a program written anywhere else would not be the toolchain's.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const program of programs) { writeExecutable(pathJoin(directory, 'bin', program)); }
    // ```
    for program in programs {
        write_executable(&directory.join("bin").join(program));
    }
    return directory;
}

// What:
// `fn a_toolchain_named(rig: &Rig, directory_name: &str, programs: &[String]) -> PathBuf` writes one
// more installed toolchain into the fixture's own rustup home under exactly the directory name it is
// given, and hands back that directory.
// Why:
// A custom linked toolchain's directory is the link's own name and nothing else,
// and a toolchain for another host triple or one whose name extends a channel is a directory of its own,
// so a control about which directory answers for a channel has to write those names.
// A name built as `<channel>-<host triple>` would be the one directory no control here is about.
//
// In TS you'd write (pseudocode):
// ```ts
// function aToolchainNamed(rig: Rig, directoryName: string, programs: string[]): string
// ```
fn a_toolchain_named(
    rig: &Rig,
    directory_name: &str,
    programs: &[String],
) -> PathBuf {
    // What:
    // the chain joins the fixture home, `.rustup`, `toolchains` and the directory name as given.
    // Why:
    // That is where rustup keeps every installed toolchain, and the name is the control's subject.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const directory = pathJoin(rig.scope.home, '.rustup', 'toolchains', directoryName);
    // ```
    let directory: PathBuf = rig
        .scope
        .home
        .join(".rustup")
        .join("toolchains")
        .join(directory_name);
    // What:
    // one stand-in program per name, each written into that toolchain's own `bin` directory.
    // Why:
    // The farm links a program it finds there,
    // so the directory a program is written into is what decides which directory answered.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const program of programs) { writeExecutable(pathJoin(directory, 'bin', program)); }
    // ```
    for program in programs {
        write_executable(&directory.join("bin").join(program));
    }
    // What:
    // `directory` is the tail expression, so the caller gets the toolchain's own path.
    // Why:
    // The controls compare the farm's links against it,
    // and the two other directories a state holds have to be told apart.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return directory;
    // ```
    return directory;
}

// What:
// `fn answer_which(rig: &Rig, channel: &str, rustc: &Path)` makes the fake runner answer
// `rustup which --toolchain <channel> rustc` with that program's path and exit code 0.
// Why:
// Which installed toolchain a channel is belongs to `rustup`,
// so a control's state is the answer the manager gives, not a name this file would guess.
//
// In TS you'd write (pseudocode):
// ```ts
// function answerWhich(rig: Rig, channel: string, rustc: string): void
// ```
fn answer_which(
    rig: &Rig,
    channel: &str,
    rustc: &Path,
) {
    // What:
    // `rig.runner.answer(prefix, stdout, code)` makes every request whose line starts with the prefix
    // answer that text and exit code; `format!` builds each of the two texts.
    // Why:
    // The observation and the farm both ask this one question,
    // and their answers have to be the same toolchain's path.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // rig.runner.answer(`rustup which --toolchain ${channel} rustc`, `${rustc}\n`, 0);
    // ```
    rig.runner.answer(
        &format!("rustup which --toolchain {channel} rustc"),
        &format!("{}\n", rustc.display()),
        0,
    );
}

// What:
// `fn the_farm_names_pointing_into(rig: &Rig, directory: &Path) -> Vec<String>` is every name in the
// exposure farm whose link resolves into that directory, sorted.
// Why:
// A control's subject is which toolchain's directory answered for the row's programs,
// so the names have to be read back from the folder a person meets.
//
// In TS you'd write (pseudocode):
// ```ts
// function theFarmNamesPointingInto(rig: Rig, directory: string): string[]
// ```
fn the_farm_names_pointing_into(
    rig: &Rig,
    directory: &Path,
) -> Vec<String> {
    // What:
    // the names being collected, and the farm's own directory's entries.
    // Why:
    // The folder's content is only known at run time,
    // and a name is what a reader compares.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const names: string[] = [];
    // const entries = readdirSync(rig.scope.exposureDir, { withFileTypes: true });
    // ```
    let mut names: Vec<String> = Vec::new();
    let entries = match fs::read_dir(&rig.scope.exposure_dir) {
        Ok(read) => read,
        // No farm yet: no name points anywhere.
        Err(_) => return names,
    };
    for entry in entries.flatten() {
        let path: PathBuf = entry.path();
        // What:
        // `.read_link` hands back the link's target and fails for an entry that is not a link.
        // Why:
        // Only a link names a program, and the target is what says which directory answered.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const target = statSync(path).isSymbolicLink() ? readlinkSync(path) : undefined;
        // ```
        let Ok(target) = fs::read_link(&path) else {
            continue;
        };
        // What:
        // `.starts_with(directory)` asks whether the target sits inside the given directory, and
        // `.file_name()` with `.to_string_lossy()` renders the entry's own name.
        // Why:
        // The answer is a name and its directory in one, and a target elsewhere is not this directory's.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (target.startsWith(directory)) names.push(entry.name);
        // ```
        if target.starts_with(directory) {
            names.push(entry.file_name().to_string_lossy().to_string());
        }
    }
    names.sort();
    return names;
}

// What:
// `fn the_directory_name_of(toolchain: &Path) -> String` is the name rustup gave one installed
// toolchain (`nightly-2026-11-16-x86_64-unknown-linux-gnu`).
// Why:
// The controls assert the sort order of two such names,
// which is the state the fault needs,
// and a comparison of whole paths would also compare the folders above them.
//
// In TS you'd write (pseudocode):
// ```ts
// function theDirectoryNameOf(toolchain: string): string
// ```
fn the_directory_name_of(toolchain: &Path) -> String {
    // What:
    // `.file_name()` is the path's last segment as an `Option`,
    // `.expect(...)` unwraps it,
    // and `.to_string_lossy()` renders it as text for a comparison.
    // Why:
    // Every directory these controls build has a name,
    // and a name that is not valid UTF-8 still has to appear in a failure rather than be dropped.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return decodeURIComponent(entry.name);
    // ```
    return toolchain
        .file_name()
        .expect("a toolchain directory has a name")
        .to_string_lossy()
        .to_string();
}

// What:
// `fn the_program_names_in(directory: &Path) -> Vec<String>` is the names of the files directly
// inside one directory, sorted.
// Why:
// The control has to show what the fixture toolchain holds,
// not only what the test meant to write into it.
//
// In TS you'd write (pseudocode):
// ```ts
// function theProgramNamesIn(directory: string): string[]
// ```
fn the_program_names_in(directory: &Path) -> Vec<String> {
    // What:
    // `let mut names: Vec<String> = Vec::new()` is the list of names being collected.
    // Why:
    // The directory's entry count is only known at run time.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const names: string[] = [];
    // ```
    let mut names: Vec<String> = Vec::new();
    // What:
    // `fs::read_dir` returns a `Result`, and `.expect(...)` unwraps the directory's entries,
    // which the loop then visits one at a time.
    // Why:
    // A directory that cannot be read is a failed fixture, not an empty toolchain.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const entry of readdirSync(directory)) { ... }
    // ```
    for entry in fs::read_dir(directory).expect("the toolchain's bin directory is readable") {
        // What:
        // `entry` is a `Result<DirEntry, io::Error>`, and `.expect(...)` unwraps one entry;
        // `.file_name()` is the entry's own name, and `.to_string_lossy()` renders it as text
        // for the comparison.
        // Why:
        // The names are what the row declares, and a name that is not valid UTF-8 still has to
        // appear in the failure rather than be dropped.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // names.push(decodeURIComponent(entry.name));
        // ```
        let name: String = entry
            .expect("the entry is readable")
            .file_name()
            .to_string_lossy()
            .to_string();
        names.push(name);
    }
    names.sort();
    return names;
}

// What:
// `fn rig_holding_the_programs_of(component_names: &[String]) -> Rig` is a disposable scope whose
// fixture toolchain holds one stand-in program for every program those components install,
// and nothing else.
// Why:
// An install reads the toolchain's `bin` directory to link the row's bins,
// so the fixture decides which of them resolve.
//
// In TS you'd write (pseudocode):
// ```ts
// function rigHoldingTheProgramsOf(componentNames: string[]): Rig
// ```
fn rig_holding_the_programs_of(component_names: &[String]) -> Rig {
    // What:
    // `Rig::new()` builds the fakes and a scope under the crate's own scratch directory.
    // Why:
    // The install needs a scope, a recording runner and a fake network,
    // and nothing in it may reach the real machine.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const rig = Rig.new();
    // ```
    let mut rig: Rig = Rig::new();
    // What:
    // the row is the repository's own declaration with its locked version,
    // which names the toolchain the fixture imitates.
    // Why:
    // The fixture's directory name has to be the channel the install asks rustup about.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const row = theRepositoryRustRow();
    // ```
    let row: LockedTool = the_repository_rust_row();
    // What:
    // the directory the fixture's programs are written into.
    // Why:
    // Every program is written there, and the install finds them through it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const bin = theToolchainBin(rig, row);
    // ```
    let bin: PathBuf = the_toolchain_bin(&rig, &row);
    for program in the_programs_of(component_names) {
        // What:
        // `write_executable(&bin.join(&program))` writes one stand-in program,
        // a shell script with no body, through a child process.
        // Why:
        // The farm links a program it can find and can run,
        // and a child writes it because the test binary must not hold it open
        // (`fixture_programs` says why).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // writeExecutable(pathJoin(bin, program));
        // ```
        write_executable(&bin.join(&program));
    }
    // What:
    // the rig with only this row in its lock.
    // Why:
    // The sample lock's other tools would install archives and crates the fixture does not hold,
    // and the controls are about this row alone.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // rig.lock.tools = [row];
    // ```
    rig.lock.tools = vec![row];
    // What:
    // `rig` is the tail expression, so the caller keeps the scope and both fakes alive.
    // Why:
    // The install borrows all three from it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return rig;
    // ```
    return rig;
}

// What:
// `fn the_targets_the_row_declares(row: &LockedTool) -> Vec<String>` is the row's `rustTargets`
// list, or an empty list when the row declares none.
// Why:
// `rustup target add <triple>` installs the component `rust-std-<triple>`,
// so a machine that follows the row holds one standard library per declared target.
//
// In TS you'd write (pseudocode):
// ```ts
// function theTargetsTheRowDeclares(row: LockedTool): string[]
// ```
fn the_targets_the_row_declares(row: &LockedTool) -> Vec<String> {
    // What:
    // `.clone()` copies the declaration's target list out of the lock entry,
    // and `.unwrap_or_default()` substitutes an empty list when the row holds none.
    // Why:
    // A row with no `rustTargets` has to answer "no targets" rather than fail.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return [...(row.declaration.options.rustTargets ?? [])];
    // ```
    return row
        .declaration
        .options
        .rust_targets
        .clone()
        .unwrap_or_default();
}

// What:
// `fn the_component_listing_of(held: &[String], row: &LockedTool) -> Vec<String>` is the text
// `rustup component list --installed --toolchain <channel>` prints for a toolchain of those
// components and of the row's declared targets, one line per component, sorted.
// Why:
// This is the answer the controls below give the reconciler,
// and one builder means the capture check reads the same lines the install is given.
//
// In TS you'd write (pseudocode):
// ```ts
// function theComponentListingOf(held: string[], row: LockedTool): string[]
// ```
fn the_component_listing_of(
    held: &[String],
    row: &LockedTool,
) -> Vec<String> {
    // What:
    // `let mut lines: Vec<String> = Vec::new()` is the listing being collected.
    // Why:
    // The number of components a machine holds is only known at run time.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const lines: string[] = [];
    // ```
    let mut lines: Vec<String> = Vec::new();
    for name in held {
        lines.push(component_named(name).listed_as());
    }
    for target in the_targets_the_row_declares(row) {
        // What:
        // `format!("rust-std-{target}")` builds the component name a declared target installs.
        // Why:
        // The standard library of a non-host target is its own component,
        // and rustup lists it under exactly this name.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // lines.push(`rust-std-${target}`);
        // ```
        lines.push(format!("rust-std-{target}"));
    }
    lines.sort();
    return lines;
}

// What:
// `fn answer_the_listings_of_the_installed_toolchain(rig: &Rig, held: &[String])` makes the fake
// runner answer what a real rustup answers about a toolchain holding those components and the
// row's targets:
// the channel's compiler, the targets, and the components as installed.
// Why:
// With those answers the install finds the toolchain already at its version
// and goes on to ask whether the row's components are all there,
// which is the path an install on a machine that follows the plan takes.
//
// In TS you'd write (pseudocode):
// ```ts
// function answerTheListingsOfTheInstalledToolchain(rig: Rig, held: string[]): void
// ```
fn answer_the_listings_of_the_installed_toolchain(
    rig: &Rig,
    held: &[String],
) {
    let row: LockedTool = the_repository_rust_row();
    // What:
    // `let mut targets: Vec<String> = vec![...]` is the target listing a toolchain of this row
    // holds: the host triple first, then each target the row declares.
    // Why:
    // `x86_64-unknown-linux-gnu`, `aarch64-linux-android` and `x86_64-linux-android` are the three
    // lines `rustup target list --installed` prints on the toolchain this machine holds
    // (captured on 2026-11-27T17:35:46-05:00, `keep/rustup-queries-2026-11-27.txt`),
    // and a machine that follows the plan holds exactly them.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const targets = [theFixtureHostTriple, ...theTargetsTheRowDeclares(row)];
    // ```
    let mut targets: Vec<String> = vec![THE_FIXTURE_HOST_TRIPLE.to_string()];
    targets.extend(the_targets_the_row_declares(&row));
    answer_the_listings_of_a_toolchain_holding(rig, held, &targets);
}

// What:
// `fn answer_the_listings_of_a_toolchain_holding(rig: &Rig, held: &[String], targets: &[String])`
// makes the fake runner answer what a real rustup answers about one toolchain:
// the channel's compiler,
// those targets as installed,
// and those components as installed.
// Why:
// The two listings are what `install` reconciles and what `list` now reads,
// so a control about a part the toolchain lacks needs to choose each listing on its own.
//
// In TS you'd write (pseudocode):
// ```ts
// function answerTheListingsOfAToolchainHolding(rig: Rig, held: string[], targets: string[]): void
// ```
fn answer_the_listings_of_a_toolchain_holding(
    rig: &Rig,
    held: &[String],
    targets: &[String],
) {
    let row: LockedTool = the_repository_rust_row();
    // What:
    // the text `rustup component list --installed` prints for those components:
    // one line each from `the_component_listing_of`, joined with newlines and ended by one.
    // Why:
    // The reconciler reads that text line by line, and a real listing ends each line.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const listing = theComponentListingOf(held, row).join('\n') + '\n';
    // ```
    let mut listing: String = String::new();
    for line in the_component_listing_of(held, &row) {
        listing.push_str(&line);
        listing.push('\n');
    }
    // What:
    // `rig.runner.answer(prefix, stdout, code)` makes every request whose line starts with the
    // prefix answer that text and exit code.
    // Why:
    // The listing's own lines decide which components the install asks for,
    // so this answer is the whole difference between the two controls below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // rig.runner.answer('rustup component list --installed', listing, 0);
    // ```
    rig.runner
        .answer("rustup component list --installed", &listing, 0);
    // What:
    // `format!("{}/rustc", bin.display())` is the path `rustup which --toolchain <channel> rustc`
    // prints, and it is what makes the install find the channel installed.
    // Why:
    // Without it the observation answers "not installed", the install takes the path that
    // unconditionally adds every declared part, and no listing is ever read.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // rig.runner.answer(`rustup which --toolchain ${row.rustupChannel()} rustc`, `${bin}/rustc\n`, 0);
    // ```
    let compiler: PathBuf = the_toolchain_bin(rig, &row).join("rustc");
    rig.runner.answer(
        &format!("rustup which --toolchain {} rustc", row.rustup_channel()),
        &format!("{}\n", compiler.display()),
        0,
    );
    // What:
    // the text `rustup target list --installed` prints: one target per line, ended by a newline.
    // Why:
    // Both `install`'s reconciler and `list` read it line by line,
    // the host target's spelling is what lets a component listed as `clippy-<triple>` count as the
    // declared `clippy`,
    // and a real listing ends each line.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // rig.runner.answer('rustup target list --installed', targets.map((target) => `${target}\n`).join(''), 0);
    // ```
    let mut listed_targets: String = String::new();
    for target in targets {
        listed_targets.push_str(target);
        listed_targets.push('\n');
    }
    rig.runner
        .answer("rustup target list --installed", &listed_targets, 0);
}

// What:
// `#[test] fn the_component_listing_the_stand_in_answers_is_the_one_this_machines_rustup_printed()`
// compares the listing this file builds for a toolchain of the repository's row with the real
// capture, in both directions.
// Why:
// The two controls below answer `rustup component list --installed` with that listing.
// A spelling no rustup prints would test the reconciler against a shape that never occurs,
// and a component the capture holds that this file cannot build would leave the stand-in short of
// a part the machine has, which is the same kind of gap as the missing `rustfmt` component.
//
// In TS you'd write (pseudocode):
// ```ts
// test('the component listing the stand-in answers is the one this machines rustup printed', () => { ... });
// ```
#[test]
fn the_component_listing_the_stand_in_answers_is_the_one_this_machines_rustup_printed() {
    let row: LockedTool = the_repository_rust_row();
    let held: Vec<String> = the_components_a_toolchain_of_this_row_holds(&row);
    // What:
    // the lines this file's stand-in would print, sorted.
    // Why:
    // They are the subject of the comparison.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const derived = theComponentListingOf(held, row);
    // ```
    let derived: Vec<String> = the_component_listing_of(&held, &row);
    // What:
    // the captured lines as owned strings, sorted the same way.
    // Why:
    // The capture is an array of `&str`, and both sides are compared in one order.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const captured = [...theComponentListingThisMachinesRustupPrints].sort();
    // ```
    let mut captured: Vec<String> = Vec::new();
    for line in THE_COMPONENT_LISTING_THIS_MACHINES_RUSTUP_PRINTS {
        captured.push(line.to_string());
    }
    captured.sort();
    assert_eq!(
        derived, captured,
        "the stand-in's listing is what this machine's `rustup component list --installed` printed for a toolchain of this row; a difference means the row gained a part (read the listing again and record it) or lost one"
    );
}

// What:
// `#[test] fn an_install_of_a_toolchain_holding_the_rows_own_components_names_no_unresolved_bin()`
// runs an install of the repository's `rust` row against a fixture toolchain holding exactly the
// programs that row's components install, and reads the report.
// Why:
// This is issue 6630's whole request:
// a machine that follows the plan ends with no unresolved bin.
// It fails while the row declares a program no component it declares installs,
// and the fixture is built from the row's own declaration,
// so the row's components and its bins are what meet in the test.
//
// In TS you'd write (pseudocode):
// ```ts
// test('an install of a toolchain holding the rows own components names no unresolved bin', () => { ... });
// ```
#[test]
fn an_install_of_a_toolchain_holding_the_rows_own_components_names_no_unresolved_bin() {
    let row: LockedTool = the_repository_rust_row();
    let held: Vec<String> = the_components_a_toolchain_of_this_row_holds(&row);
    // What:
    // the rig whose home holds a toolchain of exactly those components' programs.
    // Why:
    // The install writes nothing to this machine.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const rig = rigHoldingTheProgramsOf(held);
    // ```
    let rig: Rig = rig_holding_the_programs_of(&held);
    let bin: PathBuf = the_toolchain_bin(&rig, &row);
    // What:
    // the programs the fixture toolchain holds, read back from its directory.
    // Why:
    // "exactly what the plan declares, and no more" is a claim about the directory,
    // and a program left there by an earlier round would hide a component the row fails to declare.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const inTheFixture = theProgramNamesIn(bin);
    // ```
    let in_the_fixture: Vec<String> = the_program_names_in(&bin);
    assert_eq!(
        in_the_fixture,
        the_programs_of(&held),
        "the fixture toolchain holds exactly the programs of the components the row declares"
    );
    // What:
    // `sync_tools` installs every tool the lock holds,
    // and `None` says "not a subset";
    // its report lists what was installed, what was left alone and the bins no farm link resolves.
    // Why:
    // This is the run whose printed line the issue reports.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const report = syncTools(rig.context(), undefined);
    // ```
    let report = sync_tools(&rig.context(), None);
    // What:
    // `format_sync_report` builds the text `install` prints from the report.
    // Why:
    // The issue is about the last line of that text, not about the record.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const text = formatSyncReport(report);
    // ```
    let text: String = format_sync_report(&report);
    // What:
    // the assertion compares the report's unresolved bins with an empty list,
    // and `Vec::<String>::new()` is that empty list.
    // Why:
    // An empty list is the whole of the issue's "names no unresolved bin".
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(report.missingBins).toEqual([]);
    // ```
    assert_eq!(report.missing_bins, Vec::<String>::new(), "{text}");
    // What:
    // the assertion says no line of the report holds the unresolved-bins sentence.
    // Why:
    // The last line is the one a person acts on,
    // and the assertion's message is the whole report so a failure shows what was printed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(text).not.toContain('unresolved bins');
    // ```
    assert!(!text.contains("unresolved bins"), "{text}");
}

// What:
// `#[test] fn an_install_asks_the_manager_for_a_declared_component_the_toolchain_lacks()` runs the
// same install against a toolchain lacking the component this issue declares,
// and reads what the stand-in runner recorded.
// Why:
// The row's fix is only real if an install puts the component on a machine that has not got it,
// and the record is where that request is visible.
// It is also the positive control of the test below:
// without it, "asked for nothing" could mean "the record was never read".
//
// In TS you'd write (pseudocode):
// ```ts
// test('an install asks the manager for a declared component the toolchain lacks', () => { ... });
// ```
#[test]
fn an_install_asks_the_manager_for_a_declared_component_the_toolchain_lacks() {
    let row: LockedTool = the_repository_rust_row();
    // What:
    // `let mut held: Vec<String> = ...` is the components the fixture toolchain will hold:
    // every component the row's toolchain has, less the one this issue declares.
    // Why:
    // The difference is the one fact the two controls below turn on.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const held = theComponentsAToolchainOfThisRowHolds(row).filter((name) => name !== theComponentThisIssueDeclares);
    // ```
    let mut held: Vec<String> = the_components_a_toolchain_of_this_row_holds(&row);
    held.retain(|name| return name != THE_COMPONENT_THIS_ISSUE_DECLARES);
    let rig: Rig = rig_holding_the_programs_of(&held);
    let bin: PathBuf = the_toolchain_bin(&rig, &row);
    answer_the_listings_of_the_installed_toolchain(&rig, &held);
    // What:
    // the programs the missing component installs, and the directory they belong in.
    // Why:
    // The stand-in for `rustup component add` writes them there,
    // because a real rustup unpacks them there, and the farm reads that directory afterwards.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const programs = componentNamed(theComponentThisIssueDeclares).programs;
    // ```
    let programs: &'static [&'static str] =
        component_named(THE_COMPONENT_THIS_ISSUE_DECLARES).programs;
    // What:
    // `rig.runner.on(prefix, Box::new(move |_request: &RunRequest| { ... }))` registers an effect
    // that runs when a request whose line starts with that prefix arrives:
    // the closure captures the programs and the directory, and writes each program.
    // Why:
    // This is the stand-in for the manager writing files,
    // and it is a stand-in: no real rustup runs here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // rig.runner.on(`rustup component add ${theComponentThisIssueDeclares} `, () => { for (const program of programs) writeExecutable(pathJoin(bin, program)); });
    // ```
    rig.runner.on(
        &format!("rustup component add {THE_COMPONENT_THIS_ISSUE_DECLARES} "),
        Box::new(move |_request: &RunRequest| {
            for program in programs {
                write_executable(&bin.join(program));
            }
        }),
    );
    let report = sync_tools(&rig.context(), None);
    let text: String = format_sync_report(&report);
    // What:
    // the requests the fake runner recorded whose line starts with `rustup component add`.
    // Why:
    // The manager is asked for a component through exactly that command,
    // so the list is what "asks the manager" means.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const adds = rig.runner.lines().filter((line) => line.startsWith('rustup component add'));
    // ```
    let mut adds: Vec<String> = Vec::new();
    for line in rig.runner.lines() {
        if line.starts_with("rustup component add") {
            adds.push(line);
        }
    }
    // What:
    // the one request this run must have made, in the words the runner records:
    // the command, the component and the channel.
    // Why:
    // A request that added some other component, or named another channel, is not the fix.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const expected = `rustup component add ${theComponentThisIssueDeclares} --toolchain ${row.rustupChannel()}`;
    // ```
    let expected: String = format!(
        "rustup component add {} --toolchain {}",
        THE_COMPONENT_THIS_ISSUE_DECLARES,
        row.rustup_channel()
    );
    // What:
    // the assertion compares the recorded adds with a list holding that one request.
    // Why:
    // The listing answered every other declared component as installed,
    // so exactly one add is right: the missing component, once.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(adds).toEqual([expected]);
    // ```
    assert_eq!(adds, vec![expected], "{text}");
    // What:
    // the report of that install, whose unresolved-bins list must be empty because the stand-in
    // wrote the missing component's programs.
    // Why:
    // A component that is asked for but never reaches the farm would still leave the row's two
    // programs unresolved, which is the state this control has to exclude.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(report.missingBins).toEqual([]);
    // ```
    assert_eq!(report.missing_bins, Vec::<String>::new(), "{text}");
}

// What:
// `#[test] fn an_install_whose_toolchain_already_holds_the_declared_components_asks_for_nothing()`
// runs the same install against a toolchain holding every declared component,
// and reads what the stand-in runner recorded.
// Why:
// An install that adds a component the toolchain already has would download it on every run,
// and the reconciler exists so a complete machine's install asks for nothing.
//
// In TS you'd write (pseudocode):
// ```ts
// test('an install whose toolchain already holds the declared components asks for nothing', () => { ... });
// ```
#[test]
fn an_install_whose_toolchain_already_holds_the_declared_components_asks_for_nothing() {
    let row: LockedTool = the_repository_rust_row();
    let held: Vec<String> = the_components_a_toolchain_of_this_row_holds(&row);
    let rig: Rig = rig_holding_the_programs_of(&held);
    answer_the_listings_of_the_installed_toolchain(&rig, &held);
    let report = sync_tools(&rig.context(), None);
    let text: String = format_sync_report(&report);
    // What:
    // the requests the fake runner recorded whose line starts with `rustup component add`,
    // which must be none.
    // Why:
    // The listing shows every declared component installed,
    // so the install has nothing to ask the manager for.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const adds = rig.runner.lines().filter((line) => line.startsWith('rustup component add'));
    // ```
    let mut adds: Vec<String> = Vec::new();
    for line in rig.runner.lines() {
        if line.starts_with("rustup component add") {
            adds.push(line);
        }
    }
    assert_eq!(adds, Vec::<String>::new(), "{text}");
    // What:
    // the assertion that the report's reconciled sentences are empty:
    // nothing was added to a toolchain that was already complete.
    // Why:
    // The record of what a run did is read by a person,
    // and a sentence there would say a part was added when the listing said otherwise.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(report.reconciled).toEqual([]);
    // ```
    assert_eq!(report.reconciled, Vec::<String>::new(), "{text}");
    // What:
    // the assertion that the run still names no unresolved bin,
    // since a toolchain holding every declared component resolves every declared program.
    // Why:
    // "asks for nothing" must not be reached by finding nothing installed either.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(report.missingBins).toEqual([]);
    // ```
    assert_eq!(report.missing_bins, Vec::<String>::new(), "{text}");
}

// What:
// `#[test] fn an_install_links_the_rows_programs_from_the_toolchain_the_row_names_though_an_older_toolchain_sorts_first()`
// runs an install against a machine holding the pin and a second toolchain older than it,
// each with every program the row declares,
// and reads where the farm's links point.
// Why:
// Issue 6634:
// the farm linked the first toolchain in name order,
// so on the day the pin moved to a later date the programs of the older toolchain took the farm's
// `rustc`, `cargo` and `rustfmt` while `list` said the pin was in force.
//
// In TS you'd write (pseudocode):
// ```ts
// test('an install links the rows programs from the toolchain the row names though an older toolchain sorts first', () => { ... });
// ```
#[test]
fn an_install_links_the_rows_programs_from_the_toolchain_the_row_names_though_an_older_toolchain_sorts_first()
 {
    let row: LockedTool = the_repository_rust_row();
    let held: Vec<String> = the_components_a_toolchain_of_this_row_holds(&row);
    let rig: Rig = rig_holding_the_programs_of(&held);
    // What:
    // the pin's own `bin` directory: where the row's programs belong.
    // Why:
    // The assertion compares every farm link with it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const pinBin = theToolchainBin(rig, row);
    // ```
    let pin_bin: PathBuf = the_toolchain_bin(&rig, &row);
    answer_the_listings_of_the_installed_toolchain(&rig, &held);
    // What:
    // the second toolchain, older than the pin, holding every program the row declares.
    // Why:
    // A toolchain missing a program would leave the link to the pin for that name and hide the fault
    // this control is about.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const older = aSecondToolchain(rig, aToolchainOlderThanThePin, theProgramsOf(held));
    // ```
    let older: PathBuf = a_second_toolchain(
        &rig,
        A_TOOLCHAIN_OLDER_THAN_THE_PIN,
        &the_programs_of(&held),
    );
    // What:
    // the name rustup gives the pin's directory,
    // and the assertion that the older toolchain's name sorts before it.
    // Why:
    // The fault shows itself only in that order,
    // so a control whose names sorted the other way would pass for the wrong reason.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const pinName = `${row.rustupChannel()}-${theFixtureHostTriple}`;
    // expect(theDirectoryNameOf(older) < pinName).toBe(true);
    // ```
    let pin_name: String = format!("{}-{}", row.rustup_channel(), THE_FIXTURE_HOST_TRIPLE);
    assert!(
        the_directory_name_of(&older) < pin_name,
        "the second toolchain's directory sorts before the pin's, which is the state issue 6634 needs"
    );
    let report = sync_tools(&rig.context(), None);
    let text: String = format_sync_report(&report);
    // What:
    // one assertion per declared bin:
    // the farm's link points into the pin's `bin` directory.
    // Why:
    // That is the whole of the issue:
    // the name the row declares has to be the pin's program, whatever other toolchains hold.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const name of row.declaration.bins) { expect(readLink(pathJoin(rig.scope.exposureDir, name))).toBe(pathJoin(pinBin, name)); }
    // ```
    for name in &row.declaration.bins {
        assert_eq!(
            fs::read_link(rig.scope.exposure_dir.join(name)).expect("the farm holds a link"),
            pin_bin.join(name),
            "the farm's link for {name} names the pin's own program; {text}"
        );
    }
    assert_eq!(report.missing_bins, Vec::<String>::new(), "{text}");
}

// What:
// `#[test] fn an_install_links_the_rows_programs_from_the_toolchain_the_row_names_though_a_newer_toolchain_sorts_last()`
// is the same control with the second toolchain sorting after the pin.
// Why:
// A search that took the last directory by name would pass the older-toolchain control,
// so this one holds the state that a "take the last" rule gets wrong.
//
// In TS you'd write (pseudocode):
// ```ts
// test('an install links the rows programs from the toolchain the row names though a newer toolchain sorts last', () => { ... });
// ```
#[test]
fn an_install_links_the_rows_programs_from_the_toolchain_the_row_names_though_a_newer_toolchain_sorts_last()
 {
    let row: LockedTool = the_repository_rust_row();
    let held: Vec<String> = the_components_a_toolchain_of_this_row_holds(&row);
    let rig: Rig = rig_holding_the_programs_of(&held);
    let pin_bin: PathBuf = the_toolchain_bin(&rig, &row);
    answer_the_listings_of_the_installed_toolchain(&rig, &held);
    let newer: PathBuf = a_second_toolchain(
        &rig,
        A_TOOLCHAIN_NEWER_THAN_THE_PIN,
        &the_programs_of(&held),
    );
    let pin_name: String = format!("{}-{}", row.rustup_channel(), THE_FIXTURE_HOST_TRIPLE);
    assert!(
        the_directory_name_of(&newer) > pin_name,
        "the second toolchain's directory sorts after the pin's, which is the state this control needs"
    );
    let report = sync_tools(&rig.context(), None);
    let text: String = format_sync_report(&report);
    for name in &row.declaration.bins {
        assert_eq!(
            fs::read_link(rig.scope.exposure_dir.join(name)).expect("the farm holds a link"),
            pin_bin.join(name),
            "the farm's link for {name} names the pin's own program; {text}"
        );
    }
    assert_eq!(report.missing_bins, Vec::<String>::new(), "{text}");
}

// What:
// `#[test] fn a_program_the_rows_own_toolchain_lacks_is_unresolved_though_another_toolchain_holds_it()`
// runs an install whose pin is short of one program the row declares while a second toolchain holds
// that program,
// and reads the unresolved-bins line.
// Why:
// The row's programs come from the toolchain the row names and from no other:
// a program the pin lacks is unresolved,
// and another toolchain's copy must not stand in for it under the pin's name.
//
// In TS you'd write (pseudocode):
// ```ts
// test('a program the rows own toolchain lacks is unresolved though another toolchain holds it', () => { ... });
// ```
#[test]
fn a_program_the_rows_own_toolchain_lacks_is_unresolved_though_another_toolchain_holds_it() {
    let row: LockedTool = the_repository_rust_row();
    let held: Vec<String> = the_components_a_toolchain_of_this_row_holds(&row);
    let rig: Rig = rig_holding_the_programs_of(&held);
    let pin_bin: PathBuf = the_toolchain_bin(&rig, &row);
    // What:
    // the programs the declared formatter component installs,
    // the first of which the fixture's own toolchain is cut short of.
    // Why:
    // The component still shows in `rustup component list --installed`,
    // so the install asks for no add and the missing program is the whole difference:
    // a stand-in for a toolchain whose `bin` lost one file.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const programs = componentNamed(theComponentThisIssueDeclares).programs;
    // ```
    let programs_of_the_formatter: &'static [&'static str] =
        component_named(THE_COMPONENT_THIS_ISSUE_DECLARES).programs;
    assert!(
        !programs_of_the_formatter.is_empty(),
        "the formatter component installs at least one program, or this control has no subject"
    );
    let cut_short: &'static str = programs_of_the_formatter[0];
    // What:
    // removes that one program from the pin's `bin` directory.
    // Why:
    // The listing keeps naming its component as installed,
    // which is the state a cut-short program leaves.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // unlinkSync(pathJoin(pinBin, cutShort));
    // ```
    fs::remove_file(pin_bin.join(cut_short)).expect("remove one of the pin's own programs");
    answer_the_listings_of_the_installed_toolchain(&rig, &held);
    a_second_toolchain(
        &rig,
        A_TOOLCHAIN_OLDER_THAN_THE_PIN,
        &the_programs_of(&held),
    );
    let report = sync_tools(&rig.context(), None);
    let text: String = format_sync_report(&report);
    // What:
    // the report's unresolved bins, which must be the one declared program the pin lacks.
    // Why:
    // On a machine whose older toolchain sorts first,
    // the fault this control holds answered that name with the older toolchain's copy.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(report.missingBins).toEqual([cutShort]);
    // ```
    assert_eq!(
        report.missing_bins,
        vec![cut_short.to_string()],
        "the pin lacks {cut_short}, so the farm names it unresolved; {text}"
    );
    // What:
    // the assertion that the farm holds no link under that name.
    // Why:
    // A name with no program in the row's own toolchain is reported,
    // never linked to another toolchain's copy.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(existsSync(pathJoin(rig.scope.exposureDir, cutShort))).toBe(false);
    // ```
    assert!(
        !rig.scope.exposure_dir.join(cut_short).exists(),
        "the farm links no {cut_short}, because the pin holds none; {text}"
    );
}

// What:
// `const A_LINK_AN_EARLIER_ROW_LEFT_IN_THE_FARM: &str = "rustup"` is the manager's own name,
// which the repository's row declared before issue 6626 and declares no longer.
// Its string sibling `String` is an owned buffer,
// which a constant never needs.
// Why:
// A machine that installed with `0.2.7-alpha.16`'s row keeps a `rustup` link in the farm,
// and issue 6634's Low point 3 is that installing the newer row leaves it there.
//
// In TS you'd write (pseudocode):
// ```ts
// const aLinkAnEarlierRowLeftInTheFarm = 'rustup';
// ```
const A_LINK_AN_EARLIER_ROW_LEFT_IN_THE_FARM: &str = "rustup";

// What:
// `const A_FILE_A_PERSON_PUT_IN_THE_FARM: &str = "a-note-from-me"` is a regular file in the exposure
// directory whose name no row declares.
// Its string sibling `String` is an owned buffer,
// which a constant never needs.
// Why:
// The farm is a directory on a person's machine,
// so an install's report has to name the links it removed and leave a file alone.
//
// In TS you'd write (pseudocode):
// ```ts
// const aFileAPersonPutInTheFarm = 'a-note-from-me';
// ```
const A_FILE_A_PERSON_PUT_IN_THE_FARM: &str = "a-note-from-me";

// What:
// `#[test] fn an_install_removes_the_farm_link_no_row_declares_and_names_it()`
// runs an install on a machine whose farm already holds a link for `rustup` and a person's file,
// and reads the farm and the printed report.
// Why:
// Issue 6634's Low point 3:
// the manager's link stays after the row stops declaring it,
// dangles when rustup's proxies go,
// and nothing says so.
// The farm holds what the lock declares,
// and the report names what it removed.
//
// In TS you'd write (pseudocode):
// ```ts
// test('an install removes the farm link no row declares and names it', () => { ... });
// ```
#[test]
fn an_install_removes_the_farm_link_no_row_declares_and_names_it() {
    let row: LockedTool = the_repository_rust_row();
    let held: Vec<String> = the_components_a_toolchain_of_this_row_holds(&row);
    let rig: Rig = rig_holding_the_programs_of(&held);
    let pin_bin: PathBuf = the_toolchain_bin(&rig, &row);
    answer_the_listings_of_the_installed_toolchain(&rig, &held);
    // What:
    // the assertion that the row declares no `rustup`, which is what makes the link an orphan.
    // Why:
    // A row that declared the name would make the link the plan's, and the control would pass for
    // another reason.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(row.declaration.bins).not.toContain(aLinkAnEarlierRowLeftInTheFarm);
    // ```
    assert!(
        !row.declaration
            .bins
            .iter()
            .any(|bin| return bin == A_LINK_AN_EARLIER_ROW_LEFT_IN_THE_FARM),
        "the row declares no {A_LINK_AN_EARLIER_ROW_LEFT_IN_THE_FARM}"
    );
    fs::create_dir_all(&rig.scope.exposure_dir).expect("make the farm");
    // What:
    // the orphan link, standing where an install of the older row left it: a link to a program of the
    // fixture, under the manager's name.
    // Why:
    // The link is what an earlier row made and this row does not declare.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // symlinkSync(pathJoin(pinBin, 'rustc'), pathJoin(rig.scope.exposureDir, aLinkAnEarlierRowLeftInTheFarm));
    // ```
    let orphan: PathBuf = rig
        .scope
        .exposure_dir
        .join(A_LINK_AN_EARLIER_ROW_LEFT_IN_THE_FARM);
    std::os::unix::fs::symlink(pin_bin.join("rustc"), &orphan)
        .expect("leave the link an earlier install made");
    // What:
    // a person's regular file beside it.
    // Why:
    // The removal is about links the tool made, and this file proves the rebuild tells them apart.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // writeFileSync(pathJoin(rig.scope.exposureDir, aFileAPersonPutInTheFarm), "mine\n");
    // ```
    let the_file: PathBuf = rig.scope.exposure_dir.join(A_FILE_A_PERSON_PUT_IN_THE_FARM);
    fs::write(&the_file, "mine\n").expect("write a person's file into the farm");
    let report = sync_tools(&rig.context(), None);
    let text: String = format_sync_report(&report);
    // What:
    // the orphan link's own entry, which must be gone: `symlink_metadata` reads the entry itself.
    // Why:
    // The name no row declares is not part of the farm an install leaves behind.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(lstatSync(orphan, { throwIfNoEntry: false })).toBeUndefined();
    // ```
    assert!(
        fs::symlink_metadata(&orphan).is_err(),
        "the farm holds no link for a name no row declares; {text}"
    );
    // What:
    // the report's own lines, one of which has to name the removed link.
    // Why:
    // An install that removes a name from a folder a person sees has to say which.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(text.split('\n').some((line) => line.startsWith('toolchain install: removed ') && line.includes(aLinkAnEarlierRowLeftInTheFarm))).toBe(true);
    // ```
    assert!(
        text.lines().any(|line| {
            return line.starts_with("toolchain install: removed ")
                && line.contains(A_LINK_AN_EARLIER_ROW_LEFT_IN_THE_FARM);
        }),
        "the report names the link it removed; {text}"
    );
    // What:
    // the person's file, still holding its text, and no report line naming it as removed.
    // Why:
    // A regular file of a name no row declares is a person's, so it stays and the report says nothing
    // about removing it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(readFileSync(theFile, 'utf8')).toBe("mine\n");
    // expect(text.split('\n').some((line) => line.startsWith('toolchain install: removed ') && line.includes(aFileAPersonPutInTheFarm))).toBe(false);
    // ```
    assert_eq!(
        fs::read_to_string(&the_file).expect("a person's file stays where it is"),
        "mine\n",
        "the install leaves a regular file of a name no row declares"
    );
    assert!(
        !text.lines().any(|line| {
            return line.starts_with("toolchain install: removed ")
                && line.contains(A_FILE_A_PERSON_PUT_IN_THE_FARM);
        }),
        "the report never says it removed a person's file; {text}"
    );
}

// What:
// `const A_COMPONENT_LISTING_OF_THREE_QUESTIONS: usize = 3` is how many `rustup` questions a `list`
// on a machine that is in sync asks about this row.
// The integer siblings a reader might expect are `u32` and `i32`;
// `usize` is the width `Vec::len` answers, so the comparison needs no cast.
// Why:
// Issue 6634's Low point 5 prices the extra question:
// `list` asked one (`rustup which`), and asking the two listings makes three.
// The number is written here so a run that asks more or fewer fails loudly.
//
// In TS you'd write (pseudocode):
// ```ts
// const aComponentListingOfThreeQuestions = 3;
// ```
const THE_QUESTIONS_A_LIST_ASKS: usize = 3;

// What:
// `fn write_a_plan_and_lock_of_the_rows_rust_row(rig: &Rig, row: &LockedTool)` writes the fixture's
// `toolchain.plan.jsonc` and `toolchain.lock.json` so both hold the repository's rust row alone.
// Why:
// `list` reads the two files from the scope,
// while `install` takes the rig's in-memory lock,
// so a control about `list` has to write what a person's tree holds.
//
// In TS you'd write (pseudocode):
// ```ts
// function writeAPlanAndLockOfTheRowsRustRow(rig: Rig, row: LockedTool): void
// ```
fn write_a_plan_and_lock_of_the_rows_rust_row(
    rig: &Rig,
    row: &LockedTool,
) {
    // What:
    // the row as the plan writes it: the lock's own entry without `version` and `artifacts`,
    // which are not plan fields.
    // Why:
    // One encoder writes both files, so the plan the fixture writes is the shape the program's own
    // writer produces.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const entry = lockEntryValue(row);
    // delete entry.version; delete entry.artifacts;
    // ```
    let mut entry: Value = lock_entry_value(row);
    if let Value::Object(members) = &mut entry {
        members.remove("version");
        members.remove("artifacts");
    }
    // What:
    // the document's `tools` record, holding this one row under its id.
    // Why:
    // A plan is a record of rows by id, and the fixture holds one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const tools = { [row.declaration.id]: entry };
    // ```
    let mut tools: Map<String, Value> = Map::new();
    tools.insert(row.declaration.id.clone(), entry);
    // What:
    // the whole plan document, whose `schemaVersion` is the one field every plan carries.
    // Why:
    // The decoder refuses a plan without it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const document = { schemaVersion: 1, tools };
    // ```
    let mut document: Map<String, Value> = Map::new();
    document.insert("schemaVersion".to_string(), Value::from(1));
    document.insert("tools".to_string(), Value::Object(tools));
    // What:
    // the files themselves: the plan's text, then the lock's text for a lock holding this row.
    // Why:
    // `list` loads both from the scope and refuses when they disagree.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // writeFileSync(pathJoin(rig.scope.root, PLAN_FILE_NAME), renderPrettyJson(document) + '\n');
    // writeFileSync(pathJoin(rig.scope.root, LOCK_FILE_NAME), renderLock({ tools: [row] }));
    // ```
    fs::write(
        &rig.scope.plan_file,
        format!("{}\n", render_pretty_json(&Value::Object(document))),
    )
    .expect("write the fixture's plan");
    let lock: Lock = Lock {
        tools: vec![row.clone()],
    };
    fs::write(&rig.scope.lock_file, render_lock(&lock)).expect("write the fixture's lock");
}

// What:
// `fn the_questions_a_list_asks(rig: &Rig) -> Vec<String>` is every `rustup` request the run
// recorded, in order.
// Why:
// Issue 6634's Low point 5 asks what the extra question costs,
// and the record is where a run's own questions are counted.
//
// In TS you'd write (pseudocode):
// ```ts
// function theQuestionsAListAsks(rig: Rig): string[]
// ```
fn the_questions_a_list_asks(rig: &Rig) -> Vec<String> {
    // What:
    // the recorded requests filtered to those whose program is `rustup`.
    // Why:
    // A `list` on this row starts nothing else,
    // and naming the program keeps another tool's question from being counted here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return rig.runner.lines().filter((line) => line.startsWith('rustup '));
    // ```
    let mut asked: Vec<String> = Vec::new();
    for line in rig.runner.lines() {
        if line.starts_with("rustup ") {
            asked.push(line);
        }
    }
    return asked;
}

// What:
// `#[test] fn list_calls_a_toolchain_that_lacks_a_declared_component_drift_and_names_it()`
// runs `list` against a toolchain holding every declared component but the formatter one,
// and reads the lines a person reads.
// Why:
// Issue 6634's Low point 5:
// `list` printed `ok      rust  nightly-2026-11-16` and `1 tools in sync` while the declared
// `rustfmt` component was missing, and only the next `install` added it.
// `list` says what `install` would do, so the missing part is drift and is named.
//
// In TS you'd write (pseudocode):
// ```ts
// test('list calls a toolchain that lacks a declared component drift and names it', () => { ... });
// ```
#[test]
fn list_calls_a_toolchain_that_lacks_a_declared_component_drift_and_names_it() {
    let row: LockedTool = the_repository_rust_row();
    // What:
    // the components such a toolchain holds: every one the row's toolchain has, less the formatter.
    // Why:
    // The missing component is the whole difference from the state the install leaves behind.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const held = theComponentsAToolchainOfThisRowHolds(row).filter((name) => name !== theComponentThisIssueDeclares);
    // ```
    let mut held: Vec<String> = the_components_a_toolchain_of_this_row_holds(&row);
    held.retain(|name| return name != THE_COMPONENT_THIS_ISSUE_DECLARES);
    let rig: Rig = rig_holding_the_programs_of(&held);
    write_a_plan_and_lock_of_the_rows_rust_row(&rig, &row);
    let mut targets: Vec<String> = vec![THE_FIXTURE_HOST_TRIPLE.to_string()];
    targets.extend(the_targets_the_row_declares(&row));
    answer_the_listings_of_a_toolchain_holding(&rig, &held, &targets);
    let report = run_status(&rig.scope, &rig.env, &rig.runner, &rig.host).expect("a report");
    let text: String = report.text.clone();
    assert!(report.drifted, "{text}");
    // What:
    // the `drift` line for this row, naming the component the toolchain lacks.
    // Why:
    // A person acting on the line has to find which part the next `install` would add.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(text.split('\n').some((line) => line.startsWith('drift   rust ') && line.includes(theComponentThisIssueDeclares))).toBe(true);
    // ```
    assert!(
        text.lines().any(|line| {
            return line.starts_with("drift   rust ")
                && line.contains(THE_COMPONENT_THIS_ISSUE_DECLARES);
        }),
        "the drift line names the component the toolchain lacks; {text}"
    );
    assert!(
        !text.contains("ok      rust"),
        "a toolchain missing a declared component is not in sync; {text}"
    );
}

// What:
// `#[test] fn list_calls_a_toolchain_that_lacks_a_declared_target_drift_and_names_it()`
// runs `list` against a toolchain holding every declared component and only the host's own target,
// and reads the lines a person reads.
// Why:
// The row's `rustTargets` are parts an `install` adds the same way components are,
// so a toolchain without one is drift and the line names which target.
//
// In TS you'd write (pseudocode):
// ```ts
// test('list calls a toolchain that lacks a declared target drift and names it', () => { ... });
// ```
#[test]
fn list_calls_a_toolchain_that_lacks_a_declared_target_drift_and_names_it() {
    let row: LockedTool = the_repository_rust_row();
    let held: Vec<String> = the_components_a_toolchain_of_this_row_holds(&row);
    let rig: Rig = rig_holding_the_programs_of(&held);
    write_a_plan_and_lock_of_the_rows_rust_row(&rig, &row);
    let declared: Vec<String> = the_targets_the_row_declares(&row);
    assert!(
        !declared.is_empty(),
        "the repository's row declares a target, or this control has no subject"
    );
    // What:
    // the targets such a toolchain holds: the host's own, and none of the declared ones.
    // Why:
    // That is the state a machine reaches when it never ran `rustup target add` for the row.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const heldTargets = [theFixtureHostTriple];
    // ```
    let held_targets: Vec<String> = vec![THE_FIXTURE_HOST_TRIPLE.to_string()];
    answer_the_listings_of_a_toolchain_holding(&rig, &held, &held_targets);
    let report = run_status(&rig.scope, &rig.env, &rig.runner, &rig.host).expect("a report");
    let text: String = report.text.clone();
    assert!(report.drifted, "{text}");
    // What:
    // the `drift` line for this row, naming each declared target the toolchain lacks.
    // Why:
    // The line has to say which target the next `install` would add.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const target of declared) { expect(text).toContain(target); }
    // ```
    assert!(
        text.lines().any(|line| {
            return line.starts_with("drift   rust ")
                && declared.iter().all(|target| return line.contains(target));
        }),
        "the drift line names every declared target the toolchain lacks; {text}"
    );
}

// What:
// `#[test] fn list_on_a_toolchain_that_holds_every_declared_part_asks_the_manager_three_questions()`
// runs `list` against a toolchain holding everything the row declares and counts the `rustup`
// requests the run made.
// Why:
// Issue 6634's Low point 5 asks what the extra question costs a machine that is in sync:
// before the change `list` asked one question (`rustup which`), after it three,
// and this control is the measurement.
//
// In TS you'd write (pseudocode):
// ```ts
// test('list on a toolchain that holds every declared part asks the manager three questions', () => { ... });
// ```
#[test]
fn list_on_a_toolchain_that_holds_every_declared_part_asks_the_manager_three_questions() {
    let row: LockedTool = the_repository_rust_row();
    let held: Vec<String> = the_components_a_toolchain_of_this_row_holds(&row);
    let rig: Rig = rig_holding_the_programs_of(&held);
    write_a_plan_and_lock_of_the_rows_rust_row(&rig, &row);
    answer_the_listings_of_the_installed_toolchain(&rig, &held);
    let report = run_status(&rig.scope, &rig.env, &rig.runner, &rig.host).expect("a report");
    let text: String = report.text.clone();
    assert!(
        !report.drifted,
        "a machine holding every declared part is in sync; {text}"
    );
    // What:
    // every `rustup` request the run made, which is what the extra question costs.
    // Why:
    // The count is the whole of what Low point 5 asks this control to show,
    // and the lines themselves say which questions they are.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const asked = theQuestionsAListAsks(rig);
    // expect(asked.length).toBe(3);
    // ```
    let asked: Vec<String> = the_questions_a_list_asks(&rig);
    assert_eq!(
        asked.len(),
        THE_QUESTIONS_A_LIST_ASKS,
        "a `list` on a machine in sync asks three questions: which toolchain, its components and its targets; {asked:?}"
    );
    // What:
    // the three questions by their own words: the channel's compiler, the components, the targets.
    // Why:
    // The count alone would pass for three questions about nothing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(asked[0]).toBe(`rustup which --toolchain ${row.rustupChannel()} rustc`);
    // expect(asked[1]).toBe(`rustup component list --installed --toolchain ${row.rustupChannel()}`);
    // expect(asked[2]).toBe(`rustup target list --installed --toolchain ${row.rustupChannel()}`);
    // ```
    assert_eq!(
        asked,
        vec![
            format!("rustup which --toolchain {} rustc", row.rustup_channel()),
            format!(
                "rustup target list --installed --toolchain {}",
                row.rustup_channel()
            ),
            format!(
                "rustup component list --installed --toolchain {}",
                row.rustup_channel()
            ),
        ],
        "{text}"
    );
}

// What:
// `const A_CUSTOM_LINKED_CHANNEL: &str = "mine"` is a channel a person made with
// `rustup toolchain link mine <directory>`: a toolchain of their own under a name no release serves.
// Its string sibling `String` is an owned buffer, which a constant never needs.
// Why:
// Such a toolchain's directory is the link's own name,
// so a rule that accepts any name extending the channel lets a neighbour answer for it.
//
// In TS you'd write (pseudocode):
// ```ts
// const aCustomLinkedChannel = 'mine';
// ```
const A_CUSTOM_LINKED_CHANNEL: &str = "mine";

// What:
// `const THE_SECOND_HOST_TRIBE: &str = "aarch64-unknown-linux-gnu"` is another host triple the same
// channel can be installed for, sorted before this host's own.
// Its string sibling `String` is an owned buffer, which a constant never needs.
// Why:
// `rustup toolchain install <channel>-<triple>` can install a channel for another host,
// and the name extends the channel exactly as this host's does,
// so it is the directory a rule that only strips the channel cannot tell apart.
//
// In TS you'd write (pseudocode):
// ```ts
// const theSecondHostTriple = 'aarch64-unknown-linux-gnu';
// ```
const THE_SECOND_HOST_TRIPLE: &str = "aarch64-unknown-linux-gnu";

// What:
// `fn row_of_a_custom_linked_channel() -> LockedTool` is the repository's own Rust row with its channel,
// its version and its `packageId` renamed to the custom link's name.
// Why:
// The controls below are about which directory answers for a channel,
// and a custom link is a channel `rustup` names by the link alone.
//
// In TS you'd write (pseudocode):
// ```ts
// function rowOfACustomLinkedChannel(): LockedTool
// ```
fn row_of_a_custom_linked_channel() -> LockedTool {
    // What:
    // the repository's row, whose declaration is the one these controls keep.
    // Why:
    // The row's `bins`, components and targets stay this repository's,
    // so the only difference from the pinned state is the channel a person linked.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const row = theRepositoryRustRow();
    // ```
    let mut row: LockedTool = the_repository_rust_row();
    // What:
    // `Some(A_CUSTOM_LINKED_CHANNEL.to_string())` is the present variant of `Option` holding an owned
    // copy of the link's name, and `.to_string()` copies the borrowed constant into it.
    // Why:
    // `LockedTool::rustup_channel` reads `packageId` first,
    // so the question `rustup` is asked has to carry the link's name there.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // row.declaration.packageId = aCustomLinkedChannel;
    // ```
    row.declaration.package_id = Some(A_CUSTOM_LINKED_CHANNEL.to_string());
    // What:
    // the same name in the row's `version`, because a custom link is not a release and its channel text
    // is its whole identity.
    // Why:
    // An observation compares a release the compiler reports with a locked release only when the two
    // texts differ; equal texts keep the state to the one question this control is about.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // row.version = aCustomLinkedChannel;
    // ```
    row.version = A_CUSTOM_LINKED_CHANNEL.to_string();
    return row;
}

// What:
// `fn rig_holding_only(row: &LockedTool) -> Rig` is a disposable scope whose lock holds that one row,
// with no toolchain written yet.
// Why:
// These controls build their own machine states,
// so the rig must not already hold the pin's programs under the pin's name.
//
// In TS you'd write (pseudocode):
// ```ts
// function rigHoldingOnly(row: LockedTool): Rig
// ```
fn rig_holding_only(row: &LockedTool) -> Rig {
    // What:
    // `Rig::new()` builds the fakes and a scope under the crate's own scratch directory.
    // Why:
    // The install needs a scope, a recording runner and a fake network,
    // and nothing in it may reach the real machine.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const rig = Rig.new();
    // ```
    let mut rig: Rig = Rig::new();
    // What:
    // `vec![row.clone()]` is a one-element `Vec`, `.clone()` duplicating the borrowed row into an owned one.
    // Why:
    // `sync_tools` installs every tool of the rig's lock, and this control's lock holds the one row.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // rig.lock.tools = [row];
    // ```
    rig.lock.tools = vec![row.clone()];
    // What:
    // `rig` is the tail expression, so the caller keeps the scope and both fakes alive.
    // Why:
    // The install borrows all three from it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return rig;
    // ```
    return rig;
}

// What:
// `#[test] fn an_install_links_a_custom_linked_channels_programs_from_that_link_and_not_from_a_neighbour()`
// runs an install of a row whose channel is a custom link beside a neighbour whose name extends it,
// and reads the farm and the unresolved-bins line.
// Why:
// `rustup toolchain link mine <directory>` makes a toolchain under the link's own name,
// and a second link called `mine-two` is a toolchain of its own.
// A rule that accepts any directory whose name starts with the channel let the neighbour answer for
// the programs the channel's own toolchain lacks,
// while `rustup which --toolchain mine` names the channel's own directory.
//
// In TS you'd write (pseudocode):
// ```ts
// test('an install links a custom linked channels programs from that link and not from a neighbour', () => { ... });
// ```
#[test]
fn an_install_links_a_custom_linked_channels_programs_from_that_link_and_not_from_a_neighbour() {
    let row: LockedTool = row_of_a_custom_linked_channel();
    let rig: Rig = rig_holding_only(&row);
    // What:
    // the components a toolchain of this row holds, less the formatter one.
    // Why:
    // The channel's own toolchain is short of that component's two programs,
    // which is the state the neighbour would fill.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const held = theComponentsAToolchainOfThisRowHolds(row).filter((name) => name !== theComponentThisIssueDeclares);
    // ```
    let mut held: Vec<String> = the_components_a_toolchain_of_this_row_holds(&row);
    held.retain(|name| return name != THE_COMPONENT_THIS_ISSUE_DECLARES);
    // What:
    // the channel's own directory, holding the programs of those components.
    // Why:
    // It is the directory `rustup which --toolchain mine` names,
    // and the one every link has to point into.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const mine = aToolchainNamed(rig, aCustomLinkedChannel, theProgramsOf(held));
    // ```
    let mine: PathBuf = a_toolchain_named(&rig, A_CUSTOM_LINKED_CHANNEL, &the_programs_of(&held));
    // What:
    // the neighbour's directory, whose name is the channel's plus a hyphen and a word,
    // holding every program the row declares.
    // Why:
    // It is the directory a rule that only strips the channel accepts,
    // so the control needs it to hold what the channel's toolchain lacks.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const neighbour = aToolchainNamed(rig, `${aCustomLinkedChannel}-two`, theProgramsOf(held every component));
    // ```
    let neighbour: PathBuf = a_toolchain_named(
        &rig,
        &format!("{A_CUSTOM_LINKED_CHANNEL}-two"),
        &the_programs_of(&the_components_a_toolchain_of_this_row_holds(&row)),
    );
    // What:
    // the two directories' names in sorted order, which is the order a search by name would take.
    // Why:
    // The channel's own name sorts first, so a neighbour answering teaches the fault only where the
    // channel's toolchain is short of a program.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(theDirectoryNameOf(mine) < theDirectoryNameOf(neighbour)).toBe(true);
    // ```
    assert!(
        the_directory_name_of(&mine) < the_directory_name_of(&neighbour),
        "the channel's own directory sorts before the neighbour's, which is the state this control needs"
    );
    // What:
    // what the manager answers for the channel, and what it lists the toolchain as holding.
    // Why:
    // The answer is the whole state: which directory a channel is belongs to `rustup`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // answerWhich(rig, aCustomLinkedChannel, pathJoin(mine, 'bin', 'rustc'));
    // answerTheListingsOfAToolchainHolding(rig, held, [hostTriple, ...targets]);
    // ```
    answer_which(&rig, A_CUSTOM_LINKED_CHANNEL, &mine.join("bin").join("rustc"));
    let mut targets: Vec<String> = vec![THE_FIXTURE_HOST_TRIPLE.to_string()];
    targets.extend(the_targets_the_row_declares(&row));
    answer_the_listings_of_a_toolchain_holding(&rig, &held, &targets);
    let report = sync_tools(&rig.context(), None);
    let text: String = format_sync_report(&report);
    // What:
    // the farm's names that point into the neighbour's directory, which must be none.
    // Why:
    // A program the channel's toolchain lacks is unresolved,
    // and another toolchain's copy must not stand in for it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(theFarmNamesPointingInto(rig, pathJoin(neighbour, 'bin'))).toEqual([]);
    // ```
    assert_eq!(
        the_farm_names_pointing_into(&rig, &neighbour.join("bin")),
        Vec::<String>::new(),
        "no farm name points into the neighbour the route must not read; {text}"
    );
    // What:
    // the neighbour's directory read directly, so the control shows it holds the two programs the farm
    // could have taken from it.
    // Why:
    // An assertion about a link that would have been right either way proves nothing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const program of componentNamed(theComponentThisIssueDeclares).programs) expect(existsSync(pathJoin(neighbour, 'bin', program))).toBe(true);
    // ```
    for program in component_named(THE_COMPONENT_THIS_ISSUE_DECLARES).programs {
        assert!(
            neighbour.join("bin").join(program).exists(),
            "the neighbour holds {program}, so the farm's link would have resolved there"
        );
    }
    // What:
    // one assertion per declared program the channel's own toolchain holds:
    // its farm link names that directory's program.
    // Why:
    // That is the whole of the control:
    // every program the row declares comes from the directory the manager named.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const name of row.declaration.bins) if (existsSync(pathJoin(mine, 'bin', name))) expect(readLink(pathJoin(rig.scope.exposureDir, name))).toBe(pathJoin(mine, 'bin', name));
    // ```
    for name in &row.declaration.bins {
        if !mine.join("bin").join(name).exists() {
            continue;
        }
        assert_eq!(
            fs::read_link(rig.scope.exposure_dir.join(name)).expect("the farm holds a link"),
            mine.join("bin").join(name),
            "the farm's link for {name} names the channel's own program; {text}"
        );
    }
    // What:
    // the report's unresolved bins, which must be the two programs of the component the channel's
    // toolchain lacks.
    // Why:
    // The neighbour holds them, and the farm must still name them unresolved rather than link them.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect([...report.missingBins].sort()).toEqual([...componentNamed(theComponentThisIssueDeclares).programs].sort());
    // ```
    let mut unresolved: Vec<String> = report.missing_bins.clone();
    unresolved.sort();
    let mut the_formatters: Vec<String> = Vec::new();
    for program in component_named(THE_COMPONENT_THIS_ISSUE_DECLARES).programs {
        the_formatters.push(program.to_string());
    }
    the_formatters.sort();
    assert_eq!(
        unresolved, the_formatters,
        "the programs the channel's toolchain lacks are named unresolved though the neighbour holds them; {text}"
    );
}

// What:
// `#[test] fn an_install_links_the_rows_programs_from_the_toolchain_rustup_names_and_not_a_linked_name_that_extends_the_pin()`
// runs an install of the repository's row on a machine holding the pin's own directory and a custom
// link whose name extends the pin,
// and reads the farm.
// Why:
// `rustup toolchain link nightly-2026-11-16-custom <directory>` is a toolchain under a name that
// starts with the pin,
// and `rustup 1.29.3` accepts it.
// A rule that strips the channel and a hyphen answers with that directory,
// and the farm's `rustc` then comes from a toolchain the row never named.
//
// In TS you'd write (pseudocode):
// ```ts
// test('an install links the rows programs from the toolchain rustup names and not a linked name that extends the pin', () => { ... });
// ```
#[test]
fn an_install_links_the_rows_programs_from_the_toolchain_rustup_names_and_not_a_linked_name_that_extends_the_pin() {
    let row: LockedTool = the_repository_rust_row();
    let held: Vec<String> = the_components_a_toolchain_of_this_row_holds(&row);
    let rig: Rig = rig_holding_only(&row);
    // What:
    // the pin's own directory, holding every program the row declares.
    // Why:
    // It is the directory `rustup which --toolchain <the pin>` names.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const pinBin = pathJoin(rig.scope.home, '.rustup', 'toolchains', `${row.rustupChannel()}-${hostTriple}`, 'bin');
    // ```
    let pin_bin: PathBuf = the_toolchain_bin(&rig, &row);
    // What:
    // that directory's programs, then a second directory named `<the pin>-custom` with its own.
    // Why:
    // The second name extends the pin exactly as the host triple's does,
    // and sorts before it, which is the order a search by name takes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const program of theProgramsOf(held)) writeExecutable(pathJoin(pinBin, program));
    // const extended = aToolchainNamed(rig, `${row.rustupChannel()}-custom`, theProgramsOf(held));
    // ```
    for program in the_programs_of(&held) {
        write_executable(&pin_bin.join(&program));
    }
    let extended: PathBuf = a_toolchain_named(
        &rig,
        &format!("{}-custom", row.rustup_channel()),
        &the_programs_of(&held),
    );
    // What:
    // the two names in sorted order, with the extending one first.
    // Why:
    // A rule that takes the first matching name answers with it,
    // so the control has to hold that order.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(theDirectoryNameOf(extended) < theDirectoryNameOf(pinBin's parent)).toBe(true);
    // ```
    assert!(
        the_directory_name_of(&extended)
            < format!("{}-{}", row.rustup_channel(), THE_FIXTURE_HOST_TRIPLE),
        "the extending name sorts before the pin's own directory, which is the state this control needs"
    );
    answer_the_listings_of_the_installed_toolchain(&rig, &held);
    let report = sync_tools(&rig.context(), None);
    let text: String = format_sync_report(&report);
    // What:
    // the farm's names pointing into the extending directory, which must be none.
    // Why:
    // That directory is another toolchain's, and no program of this row comes from it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(theFarmNamesPointingInto(rig, pathJoin(extended, 'bin'))).toEqual([]);
    // ```
    assert_eq!(
        the_farm_names_pointing_into(&rig, &extended.join("bin")),
        Vec::<String>::new(),
        "no farm name points into the linked name that extends the pin; {text}"
    );
    // What:
    // one assertion per declared program: its link names the pin's own program.
    // Why:
    // The pin is the toolchain the row names, so every program is its own.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const name of row.declaration.bins) expect(readLink(pathJoin(rig.scope.exposureDir, name))).toBe(pathJoin(pinBin, name));
    // ```
    for name in &row.declaration.bins {
        assert_eq!(
            fs::read_link(rig.scope.exposure_dir.join(name)).expect("the farm holds a link"),
            pin_bin.join(name),
            "the farm's link for {name} names the pin's own program; {text}"
        );
    }
    assert_eq!(report.missing_bins, Vec::<String>::new(), "{text}");
}

// What:
// `#[test] fn an_install_links_the_rows_programs_from_the_toolchain_rustup_names_and_not_a_second_host_triple()`
// runs an install of the repository's row on a machine holding the pin for this host and the pin for
// another host triple,
// and reads the farm.
// Why:
// `rustup toolchain install <channel>-aarch64-unknown-linux-gnu` installs a channel for another host,
// and its directory's name extends the channel exactly as this host's does.
// Only `rustup` knows which triple is this host's: it reads the kernel's own answer and its own build's
// C library (see the report's quotation of rustup's `TargetTuple::from_host`),
// so a rule that parses the suffix as a host triple cannot tell the two apart.
//
// In TS you'd write (pseudocode):
// ```ts
// test('an install links the rows programs from the toolchain rustup names and not a second host triple', () => { ... });
// ```
#[test]
fn an_install_links_the_rows_programs_from_the_toolchain_rustup_names_and_not_a_second_host_triple() {
    let row: LockedTool = the_repository_rust_row();
    let held: Vec<String> = the_components_a_toolchain_of_this_row_holds(&row);
    let rig: Rig = rig_holding_only(&row);
    let pin_bin: PathBuf = the_toolchain_bin(&rig, &row);
    for program in the_programs_of(&held) {
        write_executable(&pin_bin.join(&program));
    }
    // What:
    // the same channel installed for a second host triple, with its own programs.
    // Why:
    // Its name is the channel, a hyphen and a host triple,
    // so it is the directory a rule about `<channel>-<host triple>` has to reject.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const foreign = aToolchainNamed(rig, `${row.rustupChannel()}-${theSecondHostTriple}`, theProgramsOf(held));
    // ```
    let foreign: PathBuf = a_toolchain_named(
        &rig,
        &format!("{}-{THE_SECOND_HOST_TRIPLE}", row.rustup_channel()),
        &the_programs_of(&held),
    );
    assert!(
        the_directory_name_of(&foreign)
            < format!("{}-{}", row.rustup_channel(), THE_FIXTURE_HOST_TRIPLE),
        "the other host's directory sorts before this host's, which is the state this control needs"
    );
    answer_the_listings_of_the_installed_toolchain(&rig, &held);
    let report = sync_tools(&rig.context(), None);
    let text: String = format_sync_report(&report);
    assert_eq!(
        the_farm_names_pointing_into(&rig, &foreign.join("bin")),
        Vec::<String>::new(),
        "no farm name points into the channel's directory for another host; {text}"
    );
    for name in &row.declaration.bins {
        assert_eq!(
            fs::read_link(rig.scope.exposure_dir.join(name)).expect("the farm holds a link"),
            pin_bin.join(name),
            "the farm's link for {name} names this host's own program; {text}"
        );
    }
    assert_eq!(report.missing_bins, Vec::<String>::new(), "{text}");
}

// What:
// `#[test] fn list_says_the_parts_could_not_be_read_when_a_listing_fails()`
// runs `list` on a toolchain the row is in sync with while `rustup target list --installed` fails,
// and reads the lines a person reads.
// Why:
// `list` asks `rustup` what the toolchain holds.
// A listing that failed proves nothing about the parts,
// so the row is not in sync and the line carries the failure's own words.
// The other answer, "whatever the listing did not name is missing", would add parts to a toolchain
// that may hold them all, which is what the next `install` would do.
//
// In TS you'd write (pseudocode):
// ```ts
// test('list says the parts could not be read when a listing fails', () => { ... });
// ```
#[test]
fn list_says_the_parts_could_not_be_read_when_a_listing_fails() {
    let row: LockedTool = the_repository_rust_row();
    let held: Vec<String> = the_components_a_toolchain_of_this_row_holds(&row);
    let rig: Rig = rig_holding_the_programs_of(&held);
    write_a_plan_and_lock_of_the_rows_rust_row(&rig, &row);
    // What:
    // the stand-in's words for a listing that could not read the toolchain's manifest.
    // Why:
    // They are a stand-in, written by hand, and the control asserts that `list` quotes them;
    // a real capture of this failure would be read with the same words.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const theFailuresWords = 'error: the manifest is unreadable';
    // ```
    let the_failures_words: &str = "error: the manifest is unreadable";
    // What:
    // the failing answer, registered before the two listings that succeed.
    // Why:
    // The fake answers from the first rule whose prefix matches,
    // so this rule is what the target question meets.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // rig.runner.fail('rustup target list --installed', `${theFailuresWords}\n`);
    // ```
    rig.runner.fail(
        "rustup target list --installed",
        &format!("{the_failures_words}\n"),
    );
    answer_the_listings_of_the_installed_toolchain(&rig, &held);
    let report = run_status(&rig.scope, &rig.env, &rig.runner, &rig.host).expect("a report");
    let text: String = report.text.clone();
    // What:
    // the two assertions: the row is drift, and the line says the parts could not be read in the
    // listing's own words.
    // Why:
    // "In sync" would tell a person nothing is to do while the state could not be read,
    // and a line without the failure's words leaves them nothing to act on.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(report.drifted).toBe(true);
    // expect(text).toContain('its components and targets could not be read');
    // expect(text).toContain(theFailuresWords);
    // ```
    assert!(report.drifted, "{text}");
    assert!(
        text.contains("its components and targets could not be read"),
        "the drift line says the parts could not be read; {text}"
    );
    assert!(
        text.contains(the_failures_words),
        "the drift line carries the failure's own words; {text}"
    );
    // What:
    // the one assertion against a clean pass: no line may call this row `ok`.
    // Why:
    // A row whose state could not be read is not a row in sync.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(text.includes('ok      rust')).toBe(false);
    // ```
    assert!(
        !text.contains("ok      rust"),
        "a toolchain whose listing failed is not in sync; {text}"
    );
}
