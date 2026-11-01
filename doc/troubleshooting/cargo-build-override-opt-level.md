# Cargo 1.100 nightly: build scripts' dependencies compile at `opt-level = 0`, so heavy `build.rs` work runs unoptimized

A `cargo build --release` of `forbidden-strings` spent most of its time
in a build script that ran unoptimized engine code,
even though the release profile asks for `opt-level = 3`.

## Symptom

`mise run //package/forbidden-strings/cli:build` (`cargo build --release`)
took about seven minutes on a 16-core machine,
almost all of it in `forbidden-strings`' `build.rs`,
which compiles the embedded 259-rule baseline through the `forbidden-regex` build-dependency
(`package/forbidden-strings/cli/build.rs:89`).
Nothing in the output says the build script is unoptimized:
the `Finished` line reads `` `release` profile [optimized] ``,
and the release profile in `package/forbidden-strings/cli/Cargo.toml` sets `opt-level = 3`.

In `cargo build --verbose` output,
the runtime `forbidden_regex` unit carries `-C opt-level=3`,
while the build-dependency unit for the same crate carries no `-C opt-level` flag at all,
which means rustc's default of 0.

## Root cause

Cargo treats build scripts, proc macros, and everything they depend on as host units
and resets their optimization level before applying any user override.
`src/workspace/profiles.rs:451` to `:464` in rust-lang/cargo at `495c385d0875c4ba51eb72ea0448a2d4c018b8d4`:

```rust
        // Next start overriding those settings. First comes build dependencies
        // which default to opt-level 0...
        if is_for_host {
            // For-host units are things like procedural macros, build scripts, and
            // their dependencies. For these units most projects simply want them
            // to compile quickly and the runtime doesn't matter too much since
            // they tend to process very little data. For this reason we default
            // them to a "compile as quickly as possible" mode which for now means
            // basically turning down the optimization level and avoid limiting
            // codegen units. This ensures that we spend little time optimizing as
            // well as enabling parallelism by not constraining codegen units.
            profile.opt_level = "0".into();
            profile.codegen_units = None;
```

The only way back is a `build-override` table,
merged after that reset,
`src/workspace/profiles.rs:489` to `:500`:

```rust
fn merge_toml_overrides(
    pkg_id: Option<PackageId>,
    is_member: bool,
    is_for_host: bool,
    profile: &mut Profile,
    toml: &TomlProfile,
) {
    if is_for_host {
        if let Some(build_override) = &toml.build_override {
            merge_profile(profile, build_override);
        }
    }
```

The Cargo book documents the same default under "Build Dependencies"
(<https://doc.rust-lang.org/cargo/reference/profiles.html>):

```text
To compile quickly, all profiles, by default, do not optimize build
dependencies (build scripts, proc macros, and their dependencies), [...]
[profile.release.build-override]
opt-level = 0
codegen-units = 256
```

The same page says profiles are read only from the workspace root manifest
("Profile settings defined in dependencies will be ignored"),
and that "Overrides cannot specify the panic, lto, or rpath settings."
Settings other than those three,
such as `overflow-checks = true` from the `forbidden-strings` release profile,
still carry into the build script's units.

The Cargo clone is newer than the installed Cargo (`7941be6fb`, 2026-09-11);
the pre-change `--verbose` build log confirms the installed Cargo behaves the same way:
the build-dependency `forbidden_regex` unit was invoked with `--crate-type lib -C overflow-checks=on`
and no `-C opt-level`,
while the runtime unit had `-C opt-level=3` and `-C linker-plugin-lto`.

The `forbidden-regex` DFA construction is CPU-heavy,
so the unoptimized build of it is several times slower than the optimized one.

## Verification

Versions under test:

- `cargo 1.100.0-nightly (7941be6fb 2026-09-11)`
- `rustc 1.100.0-nightly (215a8af4b 2026-09-15)`
- Linux `7.2.0-ogc6.1.fc44.x86_64`, 16 logical CPUs

### Emulating the build-script profile

A throwaway harness crate includes the same `#[path]` parser modules as `build.rs`
and times `RegexSet::new` over `data/builtin-rules.txt`.
Its `buildemu` profile mirrors what Cargo gives the build-dependency:

```toml
# frx-timing/Cargo.toml
[package]
name = "frx-timing"
version = "0.0.0"
edition = "2024"
publish = false

[dependencies]
forbidden-regex = { path = "/var/home/user/Monochromatic/package/forbidden-strings/core" }

[profile.buildemu]
inherits = "release"
opt-level = 0
codegen-units = 256
lto = false
debug = false
overflow-checks = true

[profile.release]
opt-level = 3
lto = false
overflow-checks = true
panic = "unwind"

[profile.opt3cgu256]
inherits = "release"
codegen-units = 256

[workspace]
```

```rust
// frx-timing/src/main.rs
#![allow(dead_code)]

use forbidden_regex::RegexSet;
use std::time::Instant;

#[path = "/var/home/user/Monochromatic/package/forbidden-strings/cli/src/rule/frx/error.rs"]
mod error;
#[path = "/var/home/user/Monochromatic/package/forbidden-strings/cli/src/rule/frx/escape.rs"]
mod escape;
#[path = "/var/home/user/Monochromatic/package/forbidden-strings/cli/src/rule/frx/format.rs"]
mod format;
#[path = "/var/home/user/Monochromatic/package/forbidden-strings/cli/src/rule/frx/sections.rs"]
mod sections;

fn main() {
    let mode = std::env::args().nth(1).expect("mode: whole | each");
    let text = std::fs::read_to_string(
        "/var/home/user/Monochromatic/package/forbidden-strings/cli/data/builtin-rules.txt",
    )
    .expect("read rules");
    let rules = format::parse_rules(&text).unwrap_or_else(|e| panic!("parse: {e}"));
    let patterns: Vec<String> = rules.iter().map(|r| r.pattern.clone()).collect();
    eprintln!("rules: {}", patterns.len());
    if mode == "whole" {
        let start = Instant::now();
        let set = RegexSet::new(&patterns).unwrap_or_else(|e| panic!("compile: {e}"));
        let built = start.elapsed();
        let bytes = set.to_bytes().expect("serialize");
        if let Some(out) = std::env::args().nth(2) {
            std::fs::write(out, &bytes).expect("write bytes");
        }
        eprintln!(
            "whole: {:.3}s, bytes {}, seedless {}, groups {}, line_start {}, sha-ish len {}",
            built.as_secs_f64(),
            bytes.len(),
            set.seedless_count(),
            set.seedless_group_count(),
            set.line_start_count(),
            bytes.iter().map(|&b| b as u64).sum::<u64>(),
        );
        return;
    }
    let threads: usize = std::env::args().nth(2).map_or(1, |s| s.parse().expect("threads"));
    let next = std::sync::atomic::AtomicUsize::new(0);
    let times = std::sync::Mutex::new(vec![0f64; patterns.len()]);
    let start = Instant::now();
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if i >= patterns.len() {
                    break;
                }
                let t = Instant::now();
                RegexSet::new(std::slice::from_ref(&patterns[i])).expect("compile one");
                times.lock().unwrap()[i] = t.elapsed().as_secs_f64();
            });
        }
    });
    let wall = start.elapsed().as_secs_f64();
    let mut times: Vec<(f64, usize)> =
        times.into_inner().unwrap().into_iter().enumerate().map(|(i, t)| (t, i)).collect();
    let sum: f64 = times.iter().map(|t| t.0).sum();
    times.sort_by(|a, b| b.0.total_cmp(&a.0));
    eprintln!("each: threads {threads}, wall {wall:.3}s, sum {sum:.3}s");
    for (t, i) in times.iter().take(15) {
        eprintln!("  rule {i}: {t:.3}s");
    }
    let over = |limit: f64| times.iter().filter(|t| t.0 > limit).count();
    eprintln!(
        "  >1s: {}, >0.1s: {}, >0.01s: {}",
        over(1.0),
        over(0.1),
        over(0.01)
    );
}
```

```sh
cargo build --profile buildemu && ./target/buildemu/frx-timing whole opt0.bin
cargo build --release && ./target/release/frx-timing whole opt3.bin
sha256sum opt0.bin opt3.bin
```

### Configurations measured

With the engine building rules one at a time (before `forbidden-regex` commit `2d7f341f4`):

- `opt-level = 0`, 256 codegen units (Cargo's build-script default): 390.8s
- `opt-level = 3`, 16 codegen units: 97.5s, peak resident memory 0.88 GB

With the engine building rules on 16 worker threads (from `2d7f341f4`):

- `opt-level = 0`, 256 codegen units: 77.2s
- `opt-level = 3`, 16 codegen units: 21.3s and 18.2s on two runs, peak resident memory 5.8 GB
- `opt-level = 3`, 256 codegen units: 18.7s, peak resident memory 5.9 GB

Every configuration serialized the same bytes:
SHA-256 `d5cdce69254b5fc2f546fe69dc2522dd9da1a9451cad92226b2d71f2920bbd2f`.

### End-to-end clean builds

Each build used a fresh `CARGO_TARGET_DIR`
and `mise run //package/forbidden-strings/cli:build -- --timings --verbose`.

- before (`9b3c13a82`: no build-override, one-at-a-time engine): 7m 01s total, build script run 400.07s
- after (`f515ed927` plus later comment and documentation commits;
  build-override `opt-level = 3`, threaded engine):
  50.86s total, build script run 19.58s

## Verified workarounds

### Set `build-override` in the workspace root manifest

```toml
# package/forbidden-strings/cli/Cargo.toml
[profile.release.build-override]
opt-level = 3

[profile.dev.build-override]
opt-level = 3
```

The dev override matters because `cargo check`, clippy, `cargo nextest`, and debug builds all run the build script.
`package/forbidden-strings/cli.fuzz/Cargo.toml` repeats both tables,
because it is its own workspace root and Cargo ignores the scanner manifest's profiles there.

Tradeoffs:

- every host unit is optimized,
  including proc macros such as `serde_derive` and `clap_derive`,
  which lengthens their compile time for no runtime benefit:
  in the end-to-end clean builds, `serde_derive` went from 1.43s to 4.23s and `clap_derive` from 0.74s to 3.75s
  (both builds ran units concurrently, so single-unit durations are approximate)
- the build-dependency copy of `forbidden-regex` and its dependencies still compiles separately from the runtime copy,
  because the runtime release profile uses `lto = true` and `codegen-units = 1`,
  which an override cannot match (`lto` is not allowed in overrides)
- a crate that later depends on `forbidden-strings` from another workspace gets unoptimized build scripts again
  unless its own root sets the override

### Leave codegen units at the override default

16 versus 256 codegen units measured 18.2s versus 18.7s,
inside the 18.2s to 21.3s spread of two identical 16-unit runs,
so the manifest sets only `opt-level`.

## What does not work

- Setting `build-override` only in a dependency's manifest:
  Cargo reads profiles from the workspace root only,
  which is why the fuzz crate needs its own copy.
- `lto` in a `build-override` table:
  the Cargo book lists `lto` among the settings overrides cannot specify.
- Threading the engine alone, without the override:
  77.2s unoptimized versus 18.2s to 21.3s optimized,
  so the optimization level is the larger of the two wins.

## Upstream filing artifact

### Upstream filing decision

Checked `.out-of-scope/`:
`.out-of-scope/cargo-workspace.md` covers not adding a root Cargo workspace,
not this default,
so it does not exempt the topic.

1. **Is it really upstream's fault?**
   No.
   The default is deliberate and documented,
   in both the source comment at `src/workspace/profiles.rs:451` to `:464`
   and the Cargo book's "Build Dependencies" section.
   Build scripts that do heavy computation are the exception the override exists for.
2. **Can upstream fix it?**
   Not applicable: there is no defect.
3. **Are they supporting this use case?**
   Yes, through `[profile.<name>.build-override]`, which is the workaround.
4. **Would the repo welcome our contribution?**
   Not evaluated, because constraint 1 fails.
5. **Will they likely fix it?**
   Not applicable.
6. **Have we prototyped a minimal fix?**
   Not applicable.

Duplicate search:
`gh search issues --repo rust-lang/cargo 'build-override'` returned
rust-lang/cargo#16193 ("Allow specifying build dependencies `opt-level` with `[project.dev.build-override.package]`")
and rust-lang/cargo#9351 ("Precedence issue with `build-override` and `package."*"`"),
both about override granularity, not the default.
Nothing to add to either.

Nothing to file, and no comment to post.
