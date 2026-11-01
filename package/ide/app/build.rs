//! Validate UI declarations,
//!  require embedded resources in the native procedural-macro compilation,
//! and generate the table of language files the application binary carries inside itself.

/// What:
///  Include the shared digest function from the application's sources (`#[path]` names the
///       file;
///  `mod` makes it a module of this build script).
/// Why:
///  The digest recorded here must be computed exactly as the application recomputes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { fnv1a } from './src/content_digest';
/// ```
#[path = "src/content_digest.rs"]
mod content_digest;

/// What:
///  File system access and path types.
///  `Path` borrows a path,
///  `PathBuf` owns one.
/// Why:
///  The generator walks the prepared runtime and writes one generated source file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import fs from 'node:fs'; import path from 'node:path';
/// ```
use std::{
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
};

/// What:
///  The generated file's name below Cargo's `OUT_DIR`.
/// Why:
///  `src/main.rs` includes it by this name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const GENERATED = 'embedded_runtime.rs';
/// ```
const GENERATED: &str = "embedded_runtime.rs";

/// What:
///  Every regular file below `root`,
///  as (path relative to `root`,
///  absolute path) pairs.
///       `Vec<(String, PathBuf)>` is a growable list of pairs;
///  a work list replaces recursion.
/// Why:
///  Query and notice directories are nested;
///  a work list walks them without growing the stack.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function filesBelow(root: string): Array<[relative: string, absolute: string]>
/// ```
fn files_below(root: &Path) -> Vec<(String, PathBuf)> {
    let mut found: Vec<(String, PathBuf)> = Vec::new();
    // What: `vec![...]` builds a list holding the root; `to_path_buf()` copies it into an owned path.
    // Why: Each step takes one directory off this list and pushes its subdirectories.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const pending = [root];
    // ```
    let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];
    // What: `while let Some(directory) = pending.pop()` loops while the list still has an entry.
    // Why: The walk ends when every directory has been read.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // while (pending.length) { const directory = pending.pop()!; ... }
    // ```
    while let Some(directory) = pending.pop() {
        // What: a `match` whose failure arm calls `panic!(...)` stops the build with a message on failure.
        // Why: A build script reports problems by failing; there is no caller to hand an error to.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const entries = fs.readdirSync(directory, { withFileTypes: true }); // throws
        // ```
        let entries = match fs::read_dir(&directory) {
            Ok(listing) => listing,
            Err(error) => panic!("Cannot list {}: {error}", directory.display()),
        };
        for listed in entries {
            let entry = match listed {
                Ok(listed_entry) => listed_entry,
                Err(error) => panic!("Cannot list {}: {error}", directory.display()),
            };
            let path = entry.path();
            // `fs::metadata` follows symbolic links, so a linked file is embedded by its contents.
            let metadata = match fs::metadata(&path) {
                Ok(inspected) => inspected,
                Err(error) => panic!("Cannot inspect {}: {error}", path.display()),
            };
            if metadata.is_dir() {
                pending.push(path);
            } else if metadata.is_file() {
                // What: `strip_prefix(root)` gives the part of the path below the root.
                // Why: The table records each file by its place in the runtime layout.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const relative = path.relative(root, file);
                // ```
                let below = path
                    .strip_prefix(root)
                    .expect("walked paths lie below the root");
                let Some(text) = below.to_str() else {
                    panic!("{} is not valid Unicode", path.display());
                };
                let relative = text.to_string();
                found.push((relative, path));
            }
        }
    }
    return found;
}

/// What:
///  The grammar file names `manifest.json` lists,
///  such as `sql.so`.
///  `Vec<String>` owns them.
/// Why:
///  The runtime directory may still hold libraries of an earlier selection;
///  only listed ones ship.
///      The manifest is the runtime task's own `JSON.stringify` output,
///  so its `grammars` array is read
///      directly;
///  any entry that is not a plain `<name>.so` stops the build.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function listedGrammars(manifest: string): string[] { return JSON.parse(manifest).grammars; }
/// ```
fn listed_grammars(manifest: &str, location: &Path) -> Vec<String> {
    // What: `split_once("\"grammars\"")` divides the text at the key, returning `Option<(&str, &str)>`.
    // Why: Only the text after the key holds the list.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const after = manifest.split('"grammars"')[1];
    // ```
    let Some((_, after)) = manifest.split_once("\"grammars\"") else {
        panic!("{} has no grammars list", location.display());
    };
    let Some((_, opened)) = after.split_once('[') else {
        panic!("{} has no grammars array", location.display());
    };
    let Some((list, _)) = opened.split_once(']') else {
        panic!("{} has no grammars array", location.display());
    };
    let mut names: Vec<String> = Vec::new();
    // What: `split('"').skip(1).step_by(2)` takes the text between each pair of quotes.
    // Why: Every entry is one quoted file name.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const names = list.split('"').filter((_, index) => index % 2 === 1);
    // ```
    for name in list.split('"').skip(1).step_by(2) {
        let mut plain = true;
        for character in name.chars() {
            if !(character.is_ascii_alphanumeric() || "_-.".contains(character)) {
                plain = false;
                break;
            }
        }
        if !plain || !name.ends_with(".so") || name.starts_with('.') {
            panic!(
                "{} lists an unexpected grammar entry {name:?}",
                location.display()
            );
        }
        names.push(name.to_string());
    }
    if names.is_empty() {
        panic!("{} lists no grammar", location.display());
    }
    return names;
}

/// What:
///  Collect every file the application binary embeds,
///  as (path in the former application
///       directory layout,
///  absolute source path) pairs,
///  sorted by path.
///  `crate_licenses` is the list
///       the `notices` task writes with cargo-about.
/// Why:
///  One list feeds both the generated table and the build's change tracking.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function embeddedFiles(runtime: string, packageRoot: string, crateLicenses: string): Array<[string, string]>
/// ```
fn embedded_files(runtime: &Path, package: &Path, crate_licenses: &Path) -> Vec<(String, PathBuf)> {
    let manifest = runtime.join("manifest.json");
    let text = match fs::read_to_string(&manifest) {
        Ok(read) => read,
        Err(error) => panic!(
            "The application binary embeds its language runtime, but {} cannot be read ({error}). Prepare it with `mise run //package/ide/app:runtime` (the build, build:debug, lint, and test tasks run it first), then build again.",
            manifest.display()
        ),
    };
    let mut files: Vec<(String, PathBuf)> = vec![
        ("runtime/manifest.json".to_string(), manifest.clone()),
        (
            "runtime/Helix-LICENSE".to_string(),
            runtime.join("Helix-LICENSE"),
        ),
    ];
    for (relative, path) in files_below(&runtime.join("queries")) {
        files.push((format!("runtime/queries/{relative}"), path));
    }
    for library in listed_grammars(&text, &manifest) {
        let name = library.trim_end_matches(".so");
        let library_path = runtime.join("grammars").join(&library);
        // A listed library that is absent would otherwise fail later as a bare read error without a remedy.
        if !library_path.is_file() {
            panic!(
                "The language manifest {} lists {library}, but {} is missing, so the runtime is incomplete. Prepare it again with `mise run //package/ide/app:runtime`, then build again.",
                manifest.display(),
                library_path.display()
            );
        }
        files.push((format!("runtime/grammars/{library}"), library_path));
        let notices = files_below(&runtime.join("licenses").join(name));
        if notices.is_empty() {
            panic!(
                "The runtime holds no license notice for the bundled grammar {name}; prepare it again with the runtime task."
            );
        }
        for (relative, path) in notices {
            files.push((format!("runtime/licenses/{name}/{relative}"), path));
        }
    }
    for (relative, path) in files_below(&package.join("LICENSES")) {
        files.push((format!("LICENSES/{relative}"), path));
    }
    for notice in ["Inter-LICENSE.txt", "JetBrainsMono-OFL.txt"] {
        files.push((
            format!("LICENSES/font/{notice}"),
            package.join("asset/font").join(notice),
        ));
    }
    // What: `is_file()` tests that the crate license list exists before it is embedded.
    // Why: Without it the executable would carry no crate's terms; the build stops and names the task.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!isFile(crateLicenses)) throw new Error('... run the notices task ...');
    // ```
    if !crate_licenses.is_file() {
        panic!(
            "The application binary embeds the license texts of its Rust crates, but {} is missing. Write it with `mise run //package/ide/app:notices` (the build, build:debug, lint, and test:cli tasks run it first), or copy it into this target directory, then build again.",
            crate_licenses.display()
        );
    }
    files.push((
        "LICENSES/crates.json".to_string(),
        crate_licenses.to_path_buf(),
    ));
    // What: `sort_by(by_path)` orders pairs by their first element.
    // Why: The application finds files by halving this sorted list.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // files.sort(([left], [right]) => compare(left, right));
    // ```
    files.sort_by(by_path);
    return files;
}

/// Orders two embedded files by their path in the runtime layout.
fn by_path(left: &(String, PathBuf), right: &(String, PathBuf)) -> std::cmp::Ordering {
    return left.0.cmp(&right.0);
}

/// What:
///  Write the Rust source of the embedded table into `OUT_DIR` and tell Cargo what to watch.
/// Why:
///  `include_bytes!` in the generated file copies each file into the executable at compile time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function generateEmbeddedRuntime(outDir: string, packageRoot: string): void
/// ```
fn generate_embedded_runtime(out_directory: &Path, package: &Path) {
    // What: The nearest ancestor named `build` is Cargo's build-script directory, and its parent is `target/<profile>`.
    //       That holds for `target/<profile>/build/ide-<hash>/out` and for the layout of newer Cargo,
    //       `target/<profile>/build/ide/<hash>/out`, where a fixed count of levels reaches `build` instead.
    // Why: The runtime task prepares `target/debug/runtime` and `target/release/runtime`, one per profile.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const profileDirectory = path.dirname(nearestAncestorNamed(outDir, 'build'));
    // ```
    let Some(profile) = out_directory
        .ancestors()
        .find(|ancestor| ancestor.file_name().is_some_and(|name| name == "build"))
        .and_then(Path::parent)
    else {
        panic!(
            "Unexpected Cargo output directory {}: no ancestor is named build",
            out_directory.display()
        );
    };
    let runtime = profile.join("runtime");
    // What: `parent()` climbs from `target/<profile>` to `target`, where the `notices` task writes.
    // Why: The crate license list is the same for every profile.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const crateLicenses = path.join(path.dirname(profileDirectory), 'crate-licenses.json');
    // ```
    let crate_licenses = profile
        .parent()
        .unwrap_or(profile)
        .join("crate-licenses.json");
    println!("cargo:rerun-if-changed={}", runtime.display());
    println!("cargo:rerun-if-changed={}", crate_licenses.display());
    println!(
        "cargo:rerun-if-changed={}",
        package.join("LICENSES").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        package.join("asset/font").display()
    );
    let files = embedded_files(&runtime, package, &crate_licenses);
    let mut table = String::new();
    // What: `let mut key: u64` starts the digest over all paths and file digests.
    // Why: The key names the cache directory, so it changes whenever any embedded file changes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let keyInput = '';
    // ```
    let mut key_input: Vec<u8> = Vec::new();
    let mut total: usize = 0;
    for (relative, path) in &files {
        let bytes = match fs::read(path) {
            Ok(read) => read,
            Err(error) => panic!("Cannot read {}: {error}", path.display()),
        };
        let digest = content_digest::fnv1a(&bytes);
        total += bytes.len();
        key_input.extend_from_slice(relative.as_bytes());
        key_input.push(0);
        key_input.extend_from_slice(&digest.to_le_bytes());
        let Some(absolute) = path.to_str() else {
            panic!("{} is not valid Unicode", path.display());
        };
        // What: `writeln!(table, ...)` appends one formatted line; `{relative:?}` writes the text as a
        //       quoted, escaped Rust string literal. `.expect` stops on the impossible write failure.
        // Why: Paths become literals in generated code and must stay valid whatever characters they hold.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // table += `EmbeddedFile { path: ${JSON.stringify(relative)}, ... },\n`;
        // ```
        writeln!(
            table,
            "        EmbeddedFile {{ path: {relative:?}, bytes: include_bytes!({absolute:?}), digest: 0x{digest:016x} }},"
        )
        .expect("writing to a String cannot fail");
    }
    let key = format!("{:016x}", content_digest::fnv1a(&key_input));
    let generated = format!(
        "// Generated by build.rs from {runtime}: {count} files, {total} bytes. Do not edit.\n\
         use ide_app::runtime::embedded::{{EmbeddedFile, EmbeddedRuntime}};\n\
         /// The language runtime and license texts this build embeds, sorted by path.\n\
         pub static EMBEDDED_RUNTIME: EmbeddedRuntime = EmbeddedRuntime {{\n    key: {key:?},\n    files: &[\n{table}    ],\n}};\n",
        runtime = runtime.display(),
        count = files.len(),
    );
    let output = out_directory.join(GENERATED);
    if let Err(error) = fs::write(&output, generated) {
        panic!("Cannot write {}: {error}", output.display());
    }
}

/// What:
///  Build entry point invoked by Cargo,
///  separate from application main.
/// Why:
///  The non-GUI tests can exercise document logic without opening a window or embedding files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (features.has('gui')) { compileSlint('ui/app.slint'); generateEmbeddedRuntime(outDir, root); }
/// ```
fn main() {
    // Cargo passes this to the Slint procedural macro used by the native module.
    // The standalone compile_with_config validation uses the same embedding policy.
    println!("cargo:rustc-env=SLINT_EMBED_RESOURCES=true");
    // What: Read an optional Cargo feature flag; is_ok means it exists.
    // Why: Headless document tests need no generated UI bindings and no embedded runtime.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (process.env.CARGO_FEATURE_GUI !== undefined) { ... }
    // ```
    if std::env::var("CARGO_FEATURE_GUI").is_ok() {
        // What: Configure fonts and other imported resources as embedded bytes.
        // Why: Runtime rendering must not require the build machine's font files.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const config = compilerConfig({ embedResources: true });
        // ```
        let config = slint_build::CompilerConfiguration::new()
            .embed_resources(slint_build::EmbedResourcesKind::EmbedFiles);
        // What: expect terminates the build on a returned compiler error.
        // Why: A malformed UI must fail compilation rather than ship no window.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // compileSlint('ui/app.slint', config); // throws on failure
        // ```
        slint_build::compile_with_config("ui/app.slint", config).expect("compile IDE UI");
        // What: `PathBuf::from(std::env::var_os(...)...)` reads Cargo's directories as owned paths.
        // Why: The generated table goes to `OUT_DIR`; license texts are read from the package.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // generateEmbeddedRuntime(process.env.OUT_DIR, process.env.CARGO_MANIFEST_DIR);
        // ```
        let out_directory = PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo sets OUT_DIR"));
        let package = PathBuf::from(
            std::env::var_os("CARGO_MANIFEST_DIR").expect("Cargo sets CARGO_MANIFEST_DIR"),
        );
        generate_embedded_runtime(&out_directory, &package);
    }
}
