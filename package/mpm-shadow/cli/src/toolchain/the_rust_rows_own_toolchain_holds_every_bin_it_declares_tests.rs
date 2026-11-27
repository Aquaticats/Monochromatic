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
        programs: &[
            "rustc",
            "rustdoc",
            "rust-gdb",
            "rust-gdbgui",
            "rust-lldb",
        ],
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
// (rustup 1.29.3),
// and that toolchain holds the profile, the row's four components and `rustfmt`,
// which is why ten lines are here.
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
    return row.declaration.options.rust_components.clone().unwrap_or_default();
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
// `fn answer_the_listings_of_the_installed_toolchain(rig: &Rig, held: &[String])` makes the fake
// runner answer what a real rustup answers about a toolchain holding those components:
// the channel's compiler, the host and declared targets, and those components as installed.
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
    // the text `rustup component list --installed` prints for those components:
    // one line each, in the spelling `Component::listed_as` derives, ending with a newline.
    // Why:
    // The reconciler reads that text line by line, and a real listing ends each line.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const listing = held.map((name) => componentNamed(name).listedAs()).join('\n') + '\n';
    // ```
    let mut listing: String = String::new();
    for name in held {
        listing.push_str(&component_named(name).listed_as());
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
    // `x86_64-unknown-linux-gnu`, `aarch64-linux-android` and `x86_64-linux-android` are one line
    // each of `rustup target list --installed` on the toolchain this machine holds
    // (captured on 2026-11-27T17:35:46-05:00,
    // `keep/rustup-queries-2026-11-27.txt`).
    // Why:
    // The host target's spelling is what lets a component listed as `clippy-<triple>` count as the
    // declared `clippy`, and the two Android targets are the ones the row declares,
    // so answering them installs nothing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // rig.runner.answer('rustup target list --installed', 'x86_64-unknown-linux-gnu\naarch64-linux-android\nx86_64-linux-android\n', 0);
    // ```
    rig.runner.answer(
        "rustup target list --installed",
        "x86_64-unknown-linux-gnu\naarch64-linux-android\nx86_64-linux-android\n",
        0,
    );
}

// What:
// `#[test] fn the_component_names_the_stand_in_lists_are_the_ones_this_machines_rustup_printed()`
// compares the listing spelling this file derives with the real capture.
// Why:
// The two controls below answer `rustup component list --installed` with that derivation,
// and a spelling no rustup prints would test the reconciler against a shape that never occurs.
//
// In TS you'd write (pseudocode):
// ```ts
// test('the component names the stand-in lists are the ones this machines rustup printed', () => { ... });
// ```
#[test]
fn the_component_names_the_stand_in_lists_are_the_ones_this_machines_rustup_printed() {
    for component in &THE_COMPONENTS {
        // What:
        // `component.listed_as()` is the line this file's stand-in would print for it,
        // and `.as_str()` borrows that `String` so the capture's own `&str` entries can be searched.
        // Why:
        // A component listing that is not among the captured lines is a spelling rustup does not
        // print.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const listed = component.listedAs();
        // ```
        let listed: String = component.listed_as();
        assert!(
            THE_COMPONENT_LISTING_THIS_MACHINES_RUSTUP_PRINTS.contains(&listed.as_str()),
            "the stand-in lists {listed}, which the capture of this machine's `rustup component list --installed` does not hold"
        );
    }
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
    let programs: &'static [&'static str] = component_named(THE_COMPONENT_THIS_ISSUE_DECLARES).programs;
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
