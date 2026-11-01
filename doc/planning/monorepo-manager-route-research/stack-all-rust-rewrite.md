# All-Rust monorepo manager with file-enforcer rewritten in Rust

Merged into "All-Rust tool" in `doc/planning/monorepo-manager-from-scratch-design.md` on 2026-09-16,
which records which of this file's disqualifications are inferences awaiting the user.
When copied into the repository,
a raw U+0001 character in "JSON" was replaced with the escape text ``.

Research date: 2026-09-16.
Scope: read-only design deep dive for the all-Rust option in
`doc/planning/monorepo-manager-from-scratch-design.md`,
section "Single-file shipping and a file-enforcer rewrite":
the daemon plus a Rust rewrite of `package/file-enforcer/core`,
with every plausible configuration host designed until disqualifying problems surfaced (rule `YKZ`).
Nothing under `/var/home/user/Monochromatic` was modified.
No cgroup, unit, or scope was created,
no toolchain target or system package was installed,
and nothing was committed.

## Status and inputs

- Requirements carried from the design doc,
   the vet checklist (`doc/audit/tech-monorepo-manager-vet-2026-09-16.md`, "File-enforcer functionality"),
   `doc/planning/monorepo-manager-route-research/stack-rust.md`,
   and `doc/planning/monorepo-manager-route-research/stack-rust-crates.md`.
- Stated by the user on 2026-09-16 and passed to this research:
  - The user is willing to rewrite file-enforcer in Rust.
  - The configuration does not have to be TypeScript.
  - FE18 and FE19 move to Meta Package Manager;
     FE14 and FE15 become plugins.
- Relayed to this research as user decisions on 2026-09-16 by the coordinating agent:
  - Shipping as a single file the user runs directly is a hard requirement;
     unpacking at run time is acceptable.
  - The TypeScript route is killed because a minimal top-level-await Node single executable measured 144 MiB;
     every shape that ships Node inside the file is out for the same reason.
  - The cross-shape comparison is replaced by a ranking among all-Rust configuration-hosting variants
     ("Ranking of configuration-hosting variants").
- Documentation problems are recorded and never cull,
   per the user's earlier decision recorded in the design doc.

## Conventions

- **Verified** means one of:
   a file and line range read in this session;
   a page or file fetched in this session with the quoted text found in it;
   or a probe listed in "Probes run" with its output.
- **Unverified** means inference,
   arithmetic,
   recall,
   or a figure published for a different build than the one this design needs.
- Path abbreviations:
  - `SCR/` is `/tmp/claude-1000/-var-home-user-Monochromatic/e28ad59c-f3f5-46e9-8c8c-59a61c331610/scratchpad/`.
  - `AR/` is `SCR/allrust/`,
     this research's work directory.
  - `REG/` is `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`.
  - `FE/` is `package/file-enforcer/core/src/`.
  - `TE/` is `package/toml-edit/core/src/`.
  - `STD/` is `~/.rustup/toolchains/nightly-2026-09-16-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/std/src/`.
- Crate metadata comes from `https://crates.io/api/v1/crates/<name>` with user agent `monochromatic-vet-discovery`
   and `gh api repos/<owner>/<repo>`,
   fetched today by `AR/crates-meta.ts` (results in `AR/crates-meta.json`).
  "Releases in the past year" counts non-yanked versions published since 2025-09-16.

## Probes run

### Network

- `cargo generate-lockfile` in `AR/netprobe` with a `rhai` dependency failed:
   "Could not connect to server (Failed to connect to index.crates.io:443 after 123 ms)".
  Per the caller's instruction the network policy was not worked around.
  Consequence:
   every probe below builds with `cargo build --offline` against crates already in the local registry cache,
   and crates that are not cached
   (`rhai`, `mlua`, `rquickjs`, `boa_engine`, `starlark`, `wasmtime`, `nickel-lang-core`, `jrsonnet-evaluator`,
   `browserslist-rs`, `ryu-js`)
   were evaluated from documentation,
   repository sources fetched with `curl` or `gh api`,
   and release metadata only.

### Rewrite scope

- `node AR/fe-count.ts` over `FE/`:
   137 `.ts` files;
   excluded 56 `*.test.*` files (11,126 lines, 8,452 code lines),
   `fuzz-budget.ts` (fuzz),
   `staleness-lock-regression-fixture.ts` (test fixture),
   `package/mise.container-test.ts` (container test),
   and `data/packages.generated.ts` (generated);
   77 production modules,
   13,396 lines,
   6,469 code lines.
  Code lines exclude blank lines and comment-only lines by a line heuristic.
- `node AR/toml-edit-count.ts` over `TE/`,
   minus the 7 `conformance/` files (1,231 lines, 547 code lines):
   47 production modules,
   7,954 lines,
   4,690 code lines.
- `node AR/config-features.ts`:
   Node `module.stripTypeScriptTypes` then `acorn` 8.18.0 over `file-enforcer.config.ts`
   (the repository's `typescript` package is 7.0.2 and has no JavaScript compiler API,
   so the TypeScript AST route failed with "Cannot read properties of undefined (reading 'Latest')").
  Output:
   37 function declarations,
   32 function expressions,
   27 async functions,
   56 `await` expressions,
   25 `if` statements,
   7 conditional expressions,
   32 logical expressions,
   7 `for...of` loops (1 `for await`),
   1 counter `for`,
   0 `while`,
   5 `try`,
   14 `throw`,
   2 classes,
   42 template literals (largest 2,970 characters),
   10 `Promise.all`-style `.all` calls,
   1 dynamic `import()`,
   6 import declarations.
  Host calls include `glob` 3,
   `lstat` 2,
   `chmod` 2,
   `readFile`,
   `unlink`,
   `mkdir`,
   `rm`,
   `createHash`,
   `nanoSpawn`,
   `process.platform`,
   and `process.cwd` 2.
  `wc --lines` reports 2,329 lines for the file.
  The prior research's counts (68 function declarations, 74 `await`) came from a different method and are not
  comparable.

### Byte parity

- `AR/parity/fixtures.ts` wrote shared fixtures:
   a drifted and a canonical `Cargo.toml`,
   a small TOML document,
   a JSON document with ordering and number hazards,
   and four LSP4IJ-like XML documents (well-formed, unknown entity `&nbsp;`, missing close tag, DOCTYPE with an
   internal entity).
- `AR/parity/parity-ts.ts` ran today's TypeScript primitives on them by importing repository sources read-only:
   `FE/cargo/apply-plan.ts` `applyCargoPlan`,
   `FE/pipeline/toml.ts` `editTomlKey` and `getTomlProperty`,
   `FE/pipeline/json.ts` `mergeFlatJson` and `formatJsonObject`,
   `FE/pipeline/xml.ts` `listXmlEntries` and `replaceOrInsertXmlEntry`,
   and `FE/io/glob-mirror.ts` `mirrorGlobPath`.
- `AR/parity/src/main.rs` (compiled under `#![forbid(unsafe_code)]`, offline)
   applied the same operations with `toml_edit` 0.25.13,
   `serde_json` 1.0.151 with `preserve_order`,
   `quick-xml` 0.41.0,
   and `roxmltree` 0.21.1,
   once with crate-default formatting and once with a repository-written emulation layer.
- `node AR/parity/compare.ts` compared outputs byte for byte;
   results are in "Byte-for-byte output parity".

### Sizes and linking

All binaries stripped,
`lto = true`,
`codegen-units = 1`,
built offline on the repository's nightly (`rustc 1.100.0-nightly`) for `x86_64-unknown-linux-gnu`:

- `AR/netprobe`,
   `println!` only:
   291,760 bytes.
- `AR/parity`,
   `toml_edit`, `serde_json`, `quick-xml`, `roxmltree`:
   850,192 bytes;
   `ldd` lists `libgcc_s.so.1` and `libc.so.6`.
- `AR/skeleton`
   (`panic = "abort"`),
   which calls into `tokio` 1.53.1 (`rt-multi-thread`, `net`, `process`, `signal`, `io-util`, `sync`, `time`, `macros`,
   `fs`),
   `inotify` 0.11.4,
   `ignore` 0.4.33,
   `globset`,
   `blake3` 1.8.7,
   `sha2` 0.10.9,
   `tempfile` 3.27.0,
   `serde_json`,
   `toml_edit`,
   `quick-xml`,
   and `rustix` 1.1.4,
   and ran successfully (`skeleton run skeleton-run` printed the glob match, both hashes, and `exit status: 0`):
   2,055,536 bytes.
- `AR/skeleton-dbus`,
   the skeleton plus `zbus` 5.19.0 (`tokio` feature) calling `GetServerInformation` on the session bus
   (printed `notification server ("Plasma", "KDE", "6.7.4", "1.2")`):
   3,134,664 bytes,
   so `zbus` added 1,079,128 bytes.
- Static linking on this host:
  - No toolchain under `~/.rustup/toolchains/*/lib/rustlib/` contains a `musl` target (listing showed only
  - `x86_64-unknown-linux-gnu`).
  - `RUSTFLAGS='-C target-feature=+crt-static'` on `AR/netprobe` failed:
     "rust-lld: error: unable to find library -lm" and "unable to find library -lc";
     `/usr/lib64/libc.a` does not exist.
  - `command -v` found `zig` 0.15.2 (mise), `clang`, `gcc`;
     no `musl-gcc`.
  - Result:
     no static binary could be produced here without `rustup target add x86_64-unknown-linux-musl` or a layered static
     glibc,
     both of which change toolchain or system state and were not done.

### Edit loop for compiled configuration

- `node AR/rebuild-timing.ts` on `AR/parity` (about 400 lines in the leaf crate, four parser dependencies already
- built):
  - Debug:
     no-op builds 0.10 s ×3;
     one-constant edits 0.33, 0.33, 0.34, 0.33, 0.33 s.
  - Release with fat LTO:
     no-op builds 0.10 s ×3;
     one-constant edits 5.57, 5.64, 5.53, 5.54, 5.64 s.
- `cargo -Zscript --offline config.rs` (nightly) ran a single-file Rust script and printed `cargo script config ran`;
   stable `cargo 1.98.0 (797e8a9bc 2026-08-05)` refused with "running the file `config.rs` requires `-Zscript`".
- `AR/parity/target/release/parity.d` (Cargo dep-info) lists `src/edit.rs` and `src/main.rs` plus registry sources,
   so a compiled configuration's source-level inputs are enumerable after a build.

### Release and host binary sizes

`node AR/release-sizes.ts` (latest GitHub releases, Linux x86_64 assets, bytes) and `ls -lL`:

- `denoland/rusty_v8` v152.2.0:
   `librusty_v8_release_x86_64-unknown-linux-gnu.a.gz` 39,784,686;
   `librusty_v8_release_x86_64-unknown-linux-musl.a.gz` 40,034,235.
- Host `deno` 2.9.6 binary:
   95,600,728.
- `boa-dev/boa` v0.22:
   `boa-x86_64-unknown-linux-gnu` 33,804,544 (the CLI).
- `bytecodealliance/wasmtime` v48.0.2:
   `wasmtime-v48.0.2-x86_64-linux.tar.xz` 11,424,388;
   `...-c-api.tar.xz` 15,937,900.
- `tweag/nickel` 1.17.0:
   `libnickel_lang-x86_64-linux.a` 19,498,430;
   `nickel-x86_64-linux` 32,621,960;
   `nls-x86_64-linux` 20,084,432.
- `CertainLach/jrsonnet` v0.5.0-pre98 (prerelease):
   `jrsonnet-x86_64-linux-musl` 3,310,208.
- `quickjs-ng/quickjs` v0.16.2:
   `qjs-linux-x86_64` 2,587,888.
- `facebook/buck2` `latest`:
   `buck2-x86_64-unknown-linux-musl.zst` 39,427,424.
- `facebook/starlark-rust` v0.14.0,
   `rhaiscript/rhai` v1.26.0,
   `mlua-rs/mlua` v0.12.1:
   no Linux x86_64 assets;
   `DelSkayn/rquickjs`:
   no GitHub releases.
- `apple/pkl` 0.32.1:
   `pkl-linux-amd64` 101,977,360.
- `cue-lang/cue` v0.18.0-alpha.2:
   `cue_v0.18.0-alpha.2_linux_amd64.tar.gz` 10,786,691.
- `rune-rs/rune` `nightly` (2023-12-23):
   `rune-languageserver-linux-x86_64.gz` 7,574,069.
- `koto-lang/koto` v0.16.1:
   `koto-x86_64-unknown-linux-gnu.tar.gz` 2,387,154.

### Documentation and source loads

`node AR/docs-fetch.ts` saved pages under `AR/docs/`;
single `curl` and `gh api` loads are cited where used.

## Rule and decision conflicts

- `AD2` (`AGENTS.md:1737-1740`):
   "Switch from config-as-data to TypeScript when conf needs logic (`if`, `map`, `await`)."
  The root configuration has that logic (probe counts above).
  The user's statement that the configuration need not be TypeScript is recorded only in the vet doc's FE01 text;
   `AD2` itself is unchanged.
  Per variant:
  - TypeScript or JavaScript in an embedded engine keeps literal compliance.
  - Declarative data with Rust generators keeps the configuration as data and moves logic into tool code,
     which matches `AD2`'s wording but not its intent that logic stays author-editable.
  - Compiled Rust, Starlark, Lua, Rhai, Rune, WASM guests, Nickel, and Jsonnet put configuration logic in a language
  - `AD2` does not name.
  - If the user adopts one of those,
     `AD2` needs an edit through `file-enforcer.config.ts`'s `CLAUDE.md` generation path (rule `WC2`);
     this research did not edit it.
- `FE/../DECISION.rust-migration.md` ("Decision: no Rust migration for file-enforcer") rejects a rewrite for three
- reasons:
   the consumer surface is a TypeScript builder API,
   no-op cost is recomputation,
   and the residual is file I/O.
  None of the three addresses single-file shipping,
   the user's new driver.
  The decision record would need superseding once the user accepts a variant (rule `DRR`).
- `doc/planning/load-bearing-code-languages.md` (a proposal, "does not install new rules"):
   "An approved-language wrapper that sends project-specific code to an unapproved interpreter still violates the
   boundary"
   and "Keep TypeScript as default for repository automation".
  If installed,
   Starlark, Lua, Rhai, Rune, Nickel, and Jsonnet configurations need an explicit approval for this scope,
   and Rust needs approval for the tool itself (the design doc already says so).
- `MXR` (`AGENTS.md:1012-1020`, 300 code lines per `.rs` file) and `RDC` (`AGENTS.md:1022-1028`, rustdoc on every
- documentable item, private included)
   apply to every ported module and to any Rust configuration crate.
  They do not apply to Starlark, Lua, Rhai, JavaScript, or Nickel configuration files,
   which have no equivalent repository rule (inference from the absence of such rules in `AGENTS.md`).

## 1. Rewrite scope

### Measured size and what leaves

- Production:
   77 modules,
   13,396 lines,
   6,469 code lines.
- Leaves the rewrite:
  - FE18 and FE19 to Meta Package Manager:
     `package/` and `data/` production modules,
     10 modules,
     943 code lines,
     plus the 22,608-line generated Repology index.
    Meta Package Manager is a Python program run as an external command,
     so package provisioning is not inside the single file (inference from the vet doc's delegation text).
  - FE21 to FE23 file-enforcer watch mode:
     `watch/`,
     9 modules,
     1,027 code lines,
     replaced by the daemon watcher designed in `stack-rust-crates.md` (C2-A);
     only the watch-set,
     protection,
     and echo rules are re-derived.
- Plugins (Rust modules compiled into the tool, exposed to the configuration host):
  - FE14 `cargo/`:
     4 modules,
     241 code lines.
  - FE15 `jetbrains/`:
     5 modules,
     711 code lines.
- Core library to port:
   49 modules,
   7,487 lines,
   3,547 code lines,
   of which FE09 staleness is 20 modules and 1,420 code lines.
- Dependency behavior that must be ported with it:
  - `module-toml-edit`'s canonical emitter and splice rules,
     from a 47-module,
     4,690-code-line package;
     the subset file-enforcer calls (`parseTomlEdit`, `tomlSet`, `tomlHas`, `tomlGetValue`, `tomlStringify`) was not
     measured separately.
  - `@lezer/xml`'s error-tolerant ranges,
     `dot-prop`,
     `nano-spawn`,
     `tiny-readdir-glob`,
     and `chokidar` (`FE/../package.json` dependencies, verified).
- Tests:
   56 test files,
   11,126 lines;
   property tests (`*.property.unit.test.ts`) and regression suites for the staleness lock need Rust equivalents.

### FE checklist with crate candidates

Maintenance figures are from `AR/crates-meta.json`.

- FE01 configuration with author control over sequencing and parallelism:
   see "3. Configuration hosting".
- FE02 upward discovery of the configuration file:
   `std::path::Path::ancestors`;
   CLI parsing with `clap` 4 (an incumbent in `CARGO_SHARED_DEPENDENCIES`, `file-enforcer.config.ts:1498`).
- FE03 reads and globs with provenance:
  - `globset` 0.4.20 (2026-08-04, 4 releases, BurntSushi/ripgrep) with an `ignore` 0.4.33 walk,
     both repository incumbents (`stack-rust-crates.md`, "Repository incumbents").
  - `glob` 0.3.4 (2026-07-21, 1 release).
  - `wax` 0.7.0 (2026-01-29, 1 release, 760,211 recent downloads),
     which also offers wildcard captures.
  - Semantics to preserve:
     `tiny-readdir-glob` in the library and Node `fs.glob` in the configuration.
    `fd --hidden --no-ignore --max-depth 2 --type directory '^\.' package` and the same over `.agents/skills` printed
    nothing,
     so dot-leading directories do not affect today's outputs,
     but the dotfile rules of the engines were not compared (unverified).
- FE04 in-memory read cache with invalidation:
   a repository `HashMap` keyed by path;
   no crate needed.
- FE05 skip-identical atomic writes:
  - Today:
     temp file `${filePath}.${pid}.${uuid}.tmp`,
     write,
     fsync,
     rename,
     directory fsync with unsupported-error degradation,
     then `floor(mtimeMs)` (`FE/io/write-atomic.ts:158`).
  - `tempfile` 3.27.0 `NamedTempFile::persist` plus explicit `sync_all` and directory fsync through `rustix`.
  - `atomic-write-file` 0.3.1 (2026-08-11, BSD-3-Clause, 35 stars):
     its docs show temp file,
     `fsync`,
     `rename`,
     and directory file descriptors;
     directory fsync after rename is not stated in the text read (unverified).
  - `atomicwrites` 0.4.4 (2024-09-19, 0 releases in the past year).
- FE06 create-only atomic writes:
   `rustix::fs::linkat` of a written temp file or `OpenOptions::create_new` plus rename semantics,
   ported from `FE/io/write-if-absent-atomic.ts`.
- FE07 glob mirroring:
   the substitution is string code,
   `split('*')` with leftmost `indexOf` capture (`FE/io/glob-mirror.ts:179`),
   not a glob engine capture;
   `wax` captures would change semantics for `**`.
  A direct port matched byte for byte (parity probe).
- FE08 lazy builders:
   closures or host-callable values,
   depending on the configuration host.
- FE09 staleness manifest and lock:
  - Manifest JSON with SHA-256 digests:
     `sha2` 0.11.0 (2026-03-25) to stay compatible with existing manifests;
     `blake3` would invalidate them once (prior research).
  - Lock:
     `mkdir <manifest>.lock`,
     `owner.json` published by renaming `owner.pending.json`,
     retry 10 ms,
     timeout 5 s,
     stale after 60 s with a `kill(pid, 0)` liveness check
     (`FE/io/staleness-manifest-lock.ts:21-31`, `FE/io/staleness-manifest-lock-recovery.ts:14`).
  - `fd-lock` 4.0.4 and `fs4` 1.1.0 use advisory `flock`-style locks (recall; not re-read),
     a different protocol;
     while TypeScript and Rust writers coexist,
     only a port of the `mkdir` protocol interoperates (inference).
  - Liveness:
     `rustix::process::test_kill_process` or a pidfd (recall; not re-read).
- FE10 JSON transforms:
   `serde_json` with `preserve_order` (`indexmap` 2.14.2) plus a repository serializer (parity probe);
   `jsonc-parser` 0.33.2 and `json5` 1.3.1 are unnecessary because today's code uses strict `JSON.parse`.
- FE11 TOML reads and comment-preserving edits:
   `toml_edit` 0.25.15 (2026-09-11, 22 releases, TOML 1.1) with a ported canonical emitter (parity probe);
   `taplo` 0.14.0 (2025-05-22, 0 releases in the past year, 240 open issues) as the alternative formatter-preserving
   tree.
- FE12 XML entries:
   `quick-xml` 0.42.0 (2026-08-22, 10 releases) or `roxmltree` 0.21.1 (2025-10-12, 1 release) for element ranges,
   with the entity decoder and attribute encoder ported from `FE/pipeline/xml-coding.ts:120`, `:177`;
   `xmlparser` 0.13.6 (2023), `xml-rs` 1.0.0, `xot` 0.31.2 (2025-04-09, 0 releases), and `xmltree` 0.12.0 do not expose
   splice ranges as directly (unverified beyond metadata).
- FE13 `dedup`:
   one line of repository code (`FE/pipeline/transform.ts:17`).
- FE14 Cargo manifest plans:
   plugin on the FE11 layer.
- FE15 LSP4IJ settings:
   plugin on the FE10 and FE12 layers plus options-directory discovery.
- FE16 command execution:
   `std::process::Command` or `tokio::process` (incumbent);
   `nano-spawn`'s error text must be matched only if configurations inspect it (inference).
- FE17 platform predicates:
   `std::env::consts::OS` and an executable lookup;
   0.x is Linux only per the design doc.
- FE20 tracking:
   host-level API;
   whether it is sound depends on the configuration host ("Watch tracking" in each variant).
- FE22 desktop notifications:
   `zbus` 5.19.0 incumbent (added 1,079,128 bytes in the size probe) or `notify-rust` 4.18.0 (2026-06-16, 9 releases).
- FE24 structured logging:
   `tracing` 0.1.44,
   already a shared dependency (`file-enforcer.config.ts:1520-1522`).
- Configuration-only dependency:
   `browserslist` 4.28.9 with `caniuse-lite` 1.0.30001810 (`pnpm-lock.yaml:6103`, `:6119`)
   becomes `browserslist-rs` 0.21.2 (2026-09-16),
   whose data lives in `browserslist-data` 0.3.1 (2026-09-16, 268,800-byte crate).
  `browserslist-rs`'s repository `package.json` pins `browserslist` `^4.28.9` and `caniuse-lite` `^1.0.30001810` for
  data generation (verified by `gh api`),
   and its README lists only custom-stats queries as unsupported;
   the root `.browserslistrc` uses none.
  Output parity was not probed (the crate is not cached),
   and the data then updates with crate releases instead of the pnpm lock (inference).

## 2. Byte-for-byte output parity

### What produces bytes today

- String templates and concatenation,
   which any Rust `format!` reproduces exactly:
   `CLAUDE.md`,
   `LICENSE`,
   `mise.toml`,
   `.cache/forbidden-strings.rules.txt`,
   the seeded `forbidden-strings.append.local.txt`,
   `package/config/pnpr/config.yaml`,
   package `LICENSES/*.txt` copies,
   and the git-policy source mirrors with `replaceAll` and `replace` (`file-enforcer.config.ts:2204-2329`).
  JavaScript `replace` with a string pattern replaces only the first occurrence (`file-enforcer.config.ts:2291`,
  `:2309`),
   so the port must use `replacen(.., 1)` there (recall of `String.prototype.replace`; unverified by probe).
- `JSON.stringify(value, null, 2)`:
   `.browserslistrc.resolved.local.json`,
   skill mirror manifests,
   and LSP4IJ embedded JSON.
- Structured edits:
   Cargo manifests through `module-toml-edit`
   and LSP4IJ XML through `@lezer/xml` splices.

### TOML

Crate defaults (`toml_edit` 0.25.13 with `TableLike::insert`) against today's `applyCargoPlan` and `editTomlKey`:

- Identical:
   the already-canonical manifest (527 bytes),
   a string with `"`, tab, U+0001, `é`, and an emoji,
   creating a key inside an existing table,
   a dotted key,
   a top-level string,
   and `getTomlProperty`.
- Different:
  - Drifted manifest (TypeScript 821 bytes, Rust 771 bytes):
     `TableLike::insert` on an existing key deleted the comment line above it (`# comment above clap`);
     arrays render `["derive"]` instead of `[ "derive", ]`;
     inline tables drop the trailing comma (`{ ... }` instead of `{ ..., }`).
    Cause:
     `Table::insert` calls `entry.key_mut().fmt()` on an occupied entry
     (`REG/toml_edit-0.25.13+spec-1.1.0/src/table.rs:429-443`),
     which resets the key's decoration,
     where the comment above a key is stored.
  - Five-element array:
     `module-toml-edit` goes multi-line above 4 elements or 80 columns (`TE/types.ts:166-171`, `TE/emit-value.ts:280`);
     `toml_edit` stays inline.
  - Re-setting an equal array:
     TypeScript reformats to `[ 1, 2, ]`,
     `toml_edit` keeps `[1, 2]`.
  - Creating a top-level key:
     TypeScript inserts it after the blank line and directly before `[t]`;
     `toml_edit` appends it after `a.b = 1`,
     leaving the blank line attached to `[t]`.
- Emulation layer (repository code in the probe):
   emit canonical text with a port of `assembleArrayParts`,
   parse it with `str::parse::<toml_edit::Value>` so the raw representation is kept,
   copy the old value's decoration (keeps `# clap trailing`),
   replace through `get_mut` instead of `insert` (keeps the comment above),
   and for top-level creation move the first table's prefix onto the new key with `insert_formatted`.
  Result:
   drifted manifest identical (821 bytes),
   long array identical (111 bytes),
   equal-array re-set identical (92 bytes),
   top-level creation identical (102 bytes).
- Consequence:
   `toml_edit` supplies the lossless tree and TOML 1.1 syntax (`InlineTable::set_trailing_comma`,
   `REG/toml_edit-0.25.13+spec-1.1.0/src/inline_table.rs:94`),
   but every placement and formatting rule of `module-toml-edit` that file-enforcer reaches must be ported and tested
   against the TypeScript output.
  Only the cases above were probed.

### JSON

- `serde_json::to_string_pretty` against `JSON.stringify(JSON.parse(x), null, 2)`:
   different (TypeScript 212 bytes, Rust 219 bytes).
  - Key order:
     JavaScript places array-index keys first in ascending order (`"1"`, `"2"`, then `"b"`, `"a"`);
     `preserve_order` keeps source order.
  - Numbers:
     JavaScript prints `100`, `12345678901234567000`, `0`, `1e+21`, `5e-7`, and `1` for `1.0`;
     `serde_json` printed `100.0`, `12345678901234567890`, `-0.0`, and `1.0`.
  - Strings:
     both left U+007F and U+2028 raw and escaped U+0001 as `\u0001`
     (checked with `String.prototype.includes` on both outputs).
- Emulation layer:
   numbers through `f64` formatted by ECMAScript `Number::toString` rules on Rust's shortest `{:e}` digits,
   and array-index keys ordered first.
  Result:
   round trip identical (212 bytes) and `mergeFlatJson` output identical (273 bytes).
- `ryu-js` 1.0.3 (Boa's ECMAScript number formatter) is the crate candidate for the number part;
   it is not cached and was not probed.

### XML

Splice logic ported from `FE/pipeline/xml.ts:279`, `:421` with a partial decoder port
(the probe's decoder omitted numeric entities,
which no fixture used):

- Well-formed document:
   `quick-xml` and `roxmltree` both identical for listing (417 bytes),
   replacement (311 bytes),
   and insertion (651 bytes).
- Unknown entity `&nbsp;`:
   `quick-xml` identical (raw attribute bytes plus the ported decoder);
   `roxmltree` failed with "unknown entity reference 'nbsp' at 9:54".
- DOCTYPE with an internal entity:
   `quick-xml` identical;
   `roxmltree` failed with "XML with DTD detected" under default options.
- Missing close tag:
   both failed ("ill-formed document: expected `</entry>`, but `</map>` was found at 562";
   "expected 'entry' tag, not 'map' at 15:7"),
   while `@lezer/xml` recovered and the TypeScript code spliced anyway,
   listing the unclosed entry as `<entry key='srv-2'><value />\n      `.
- Consequence:
   `quick-xml` reproduces today's bytes for well-formed and entity-bearing JetBrains files;
   malformed files fail closed in Rust where TypeScript writes a best-effort splice.

### Glob mirror and browserslist

- `mirrorGlobPath` port:
   identical for four cases,
   including the `**` error text.
- `browserslist-rs`:
   not probed ("FE checklist with crate candidates").

### Parity plan

- A differential harness in a throwaway worktree runs the TypeScript configuration and the Rust port on the same tree
   and compares every destination byte for byte,
   then repeats after scripted drift (edited `Cargo.toml` keys, reordered JSON, reformatted XML).
- Until it passes,
   the first Rust run can rewrite managed files across the repository,
   including `CLAUDE.md`,
   which carries agent instructions (inference).

## 3. Configuration hosting

### What the root configuration needs

From the AST probe and a read of `file-enforcer.config.ts`:

- Functions and callbacks:
   69 function definitions,
   and callbacks passed into the library (`spec: buildCargoManifestPlan`, `file-enforcer.config.ts:1777`).
- Async and author-controlled concurrency:
   27 async functions,
   56 `await`,
   a top-level `Promise.all` over 10 generator calls and 18 mirrored-file writes (`:2204-2329`).
- Control flow:
   25 `if`,
   7 `for...of` (one `for await` over `glob`),
   one counter loop,
   5 `try` and 14 `throw` with 2 error classes,
   `ENOENT` handling by error code (`:287-299`).
- Text:
   42 template literals,
   the largest 2,970 characters (the `CLAUDE.md` preamble).
- Data:
   `Set`,
   `Map`,
   sorting with a comparator (`:1223`),
   `Object.fromEntries`,
   `Reflect.ownKeys`,
   JSON parse and stringify.
- Host effects:
   `glob`,
   `lstat`,
   `chmod`,
   `mkdir`,
   `rm`,
   `unlink`,
   `readFile`,
   SHA-256,
   spawning the forbidden-strings scanner (`:774-781`),
   `process.platform` (`:182`),
   and `process.cwd()` (`:1328`, `:1332`).
- Imports:
   `node:crypto`,
   `node:fs/promises`,
   `node:path`,
   `nano-spawn`,
   the file-enforcer API,
   `@monochromatic-dev/config-pnpr/ts` for `isTypeScriptSourcePath`
   (whose `publish-plan.ts:1` imports the `yaml` npm package),
   and a dynamic `import('browserslist')` (`:622`).
- Known soundness defect to avoid repeating:
   `doc/troubleshooting/file-enforcer-staleness.md` shows a lazy builder skipped after an untracked `node:fs` read,
   and counts direct `glob` and `lstat` sites in the root configuration.

### Cross-cutting design choice for embedded engines

- In-process evaluation:
   the daemon writes destinations itself,
   so echo suppression registers the expected hash before each write;
   but an in-process evaluation is not in a task cgroup,
   so the design doc's pause (`cgroup.freeze`) and end (`cgroup.kill`) cannot target it,
   and cancellation depends on the engine's interrupt hook.
- Self re-exec child:
   the same single file runs `/proc/self/exe __evaluate <config>` in a task cgroup
   (the launcher pattern verified in `stack-rust-crates.md`, C1-A),
   restoring freeze,
   kill,
   and memory limits,
   at the price of the two-channel echo ordering that the prior research solves with content hashes.
  Rust event types are shared inside one binary,
   so no cross-language schema exists.
- Either choice keeps the single file.

### Variant A: TypeScript or JavaScript on an embedded engine

#### A1: `rquickjs` (QuickJS-ng)

- Design:
   `rquickjs` 0.13.0 with `futures` and `loader`;
   an `AsyncRuntime` on `tokio`;
   a module loader that serves built-in modules
   `@monochromatic-dev/file-enforcer-core/ts`,
   `node:fs/promises`,
   `node:crypto`,
   `node:path`,
   `nano-spawn`,
   and `browserslist` from Rust host functions,
   and loads repository `.ts` files after type stripping.
  The configuration file keeps its name and most of its text.
- Type stripping:
   either embed an `oxc` parser, transformer, and code generator
   (`oxc_transformer` 0.150.0, 61 releases in the past year),
   or author `file-enforcer.config.js` with JSDoc types checked by the repository's TypeScript toolchain.
  No `oxc` size was measured;
   the repository's `rolldown-binding.linux-x64-gnu.node` 1.2.7 (which contains `oxc` plus the bundler) is 19,312,448
   bytes (`ls -lL`), an upper bound only.
- Expressiveness:
   async and `await`,
   `Promise.all`,
   classes,
   `try`/`catch`,
   and template literals are native JavaScript.
  `Array.fromAsync` exists in QuickJS-ng as `builtin-array-fromasync.js` and `toSorted` appears in its tests
   (`gh search code` on `quickjs-ng/quickjs`, default branch);
   the commit vendored by `rquickjs` (submodule `5301314c`) was not checked.
  QuickJS is single-threaded,
   so generator CPU work runs serially while host I/O can overlap (inference from the `AsyncRuntime` docs text).
- npm and workspace imports:
   `browserslist` is CommonJS with data files,
   and `config-pnpr` pulls in `yaml`,
   so both become host functions or inlined helpers;
   generic npm loading is out of scope (inference).
- Sandboxing:
   no file, process, or network access unless the host registers it
   (inference: the optional features read on the crate page are module loading, native module loading, and async
   support,
   and no file-system module was found there; unverified against the crate source).
- Watch tracking:
   exact,
   because every read,
   glob,
   `lstat`,
   and module load passes through host functions or the loader.
- Error messages:
   JavaScript exceptions with stack traces;
   quality relative to V8 unverified.
- Editor tooling:
   the repository's TypeScript toolchain with a `.d.ts` for host modules;
   keeping that declaration file in step with Rust host functions is repository work (a generator was not evaluated).
- Documentation (recorded):
   the `rquickjs` crate page warns that `Runtime` and `Context` locks are "discouraged" in async code and points to
   `AsyncRuntime`;
   no contradiction found in the sections read.
   The 367 KiB figure on bellard.org describes Bellard's QuickJS,
   not the QuickJS-ng fork `rquickjs` vendors.
- Maintenance:
   0.13.0 (2026-09-08),
   6 releases,
   63 open issues,
   1,009 stars,
   no GitHub releases.
- Binary size:
   bellard.org:
   "367 KiB of x86 code for a simple hello world";
   QuickJS-ng `qjs-linux-x86_64` is 2,587,888 bytes;
   the added size for this design is unmeasured.
- Static linking:
   `rquickjs-sys` compiles QuickJS-ng C sources and ships pregenerated bindings including
   `x86_64-unknown-linux-musl.rs` (`gh api repos/DelSkayn/rquickjs/contents/sys/src/bindings`),
   so a musl build needs a musl-targeting C compiler (inference; `zig` 0.15.2 is present, unverified for this build).
- Rules:
   keeps TypeScript (`AD2` literal compliance and the proposal's TypeScript default).
- Disqualifying problems found:
   none.
  Problems:
   a repository-written Node-API shim layer;
   TypeScript stripping adds an unmeasured `oxc` dependency or changes authoring to JavaScript;
   C sources complicate the static musl build;
   engine feature coverage for the exact APIs used is only partly verified.

#### A2: `boa_engine`

- Design:
   as A1 with Boa's module loader and job queue.
- Expressiveness:
   as A1;
   `Array.fromAsync` exists (`core/engine/src/builtins/array/from_async.rs`, `gh search code` on `boa-dev/boa`).
- Static linking:
   expected pure Rust,
   so a musl build would need no C cross compiler
   (inference: the crate page lists `intl`, `boa_runtime`, and `tag_ptr` features and mentions no C build; the
   dependency tree was not checked).
- Binary size:
   the Boa CLI release asset is 33,804,544 bytes,
   which presumably includes its REPL and default features (unverified);
   an embedding without `intl` is unmeasured.
- Maintenance:
   0.22.0 (2026-08-28),
   3 releases,
   217 open issues,
   7,556 stars.
- Documentation (recorded):
   no problem found in the crate page sections read.
- Disqualifying problems found:
   none.
  Problems:
   A1's shim and stripping work,
   plus a size risk that no evidence bounds below the 33.8 MB CLI.

#### A3: `deno_core` (V8)

- Design:
   `deno_core` 0.412.0 ops for host functions.
- Binary size:
   V8 alone ships as a 39,784,686-byte gzip-compressed static library (glibc) and 40,034,235 bytes (musl);
   the host `deno` binary is 95,600,728 bytes.
- Maintenance:
   54 releases in the past year.
- Disqualifying problem:
   V8 is the engine that dominates Node's size (inference),
   and the user killed every Node shape for size (144 MiB),
   so the same size class disqualifies this variant.

### Variant B: Starlark (`starlark` 0.14.2)

- Design:
   `file-enforcer.star` evaluated with a `Globals` set exposing the file-enforcer API as Rust functions;
   `load()` resolved by a host `FileLoader`.
- Expressiveness:
  - `async`, `await`, `class`, `except`, `raise`, `try`, `while`, `with`, and `yield` are reserved and unusable
     (`starlark_syntax/src/lexer.rs:1266-1286` on `main`, fetched today).
  - Top-level `for` and `if` need `enable_top_level_stmt`,
     and f-strings need `enable_f_strings`,
     both "Disabled by default" (`starlark_syntax/src/dialect.rs:58-66`).
  - So `Promise.all` becomes a host `parallel()` primitive whose thread safety over Starlark values is unverified;
     `ENOENT` handling becomes host functions returning `None`;
     error classes become `fail()` messages.
- Sandboxing:
   hermetic;
   the crate docs call Starlark "a deterministic version of Python".
- Watch tracking:
   exact through host functions and `load()`.
- Error messages:
   span-annotated diagnostics (recall; unverified in this session).
- Editor tooling:
   `starlark_lsp` is a library and `starlark_bin` is not published as a release asset (v0.14.0 has no Linux assets),
   so an LSP would ship inside the tool or be built separately;
   type annotations are optional through `DialectTypes`.
- Documentation (recorded):
   no problem found in the docs.rs root or `dialect.rs` comments read.
- Maintenance:
   2 releases,
   40 open issues,
   pushed 2026-09-16;
   Buck2 embeds it (recall).
- Binary size:
   unmeasured;
   crate sources are 696,353 and 275,356 compressed bytes (`starlark`, `starlark_syntax`);
   Buck2's 39.4 MB compressed binary contains far more than the interpreter.
- Static linking:
   pure Rust (inference from the absence of C sources in the files read).
- Rules:
   new language for `AD2` and the approved-language proposal.
- Disqualifying problems found:
   none.
  Problems:
   no async,
   no exceptions,
   no `while`;
   the 2,329-line configuration is a full rewrite into a restricted language.

### Variant C: Rhai (`rhai` 1.26.1)

- Design:
   `file-enforcer.rhai` with registered Rust functions and a module resolver.
- Expressiveness:
   functions,
   loops,
   `try`/`catch`/`throw`,
   and object maps exist (features page);
   no async:
   the maintainer answered "All calls with Rhai are blocking and single-threaded" and "there is no `await` keyword"
   (issue `rhaiscript/rhai#215`, comments read with `gh issue view`).
  Parallel generators need a host primitive over the `sync` feature's `Send + Sync` engine (features page: "Re-entrant
  scripting Engine can be made Send + Sync").
- Sandboxing:
   no I/O unless registered;
   limits for "runaway scripts" (features page).
- Watch tracking:
   exact through registered functions and the resolver.
- Error messages:
   positions on errors (recall; unverified).
- Editor tooling:
   `rhaiscript/lsp` last pushed 2023-03-17 (78 stars).
- Documentation (recorded):
   the features page lists "Closures that can capture shared variables",
   while the "What Rhai Isn't" page says "No first-class closures" and "No first-class functions" before describing
   simulated closures and function pointers;
   the minimal-build page gives no size figures.
- Maintenance:
   7 releases,
   15 open issues,
   5,682 stars.
- Binary size:
   unmeasured;
   no documented figure.
- Static linking:
   pure Rust (inference).
- Rules:
   new language.
- Disqualifying problems found:
   none.
  Problems:
   no async,
   a stale language server,
   and a full rewrite into a new language.

### Variant D: Lua (`mlua` 0.12.1)

- Design:
   `file-enforcer.lua` with `lua54` or `luau` and `vendored`;
   host functions registered with `create_async_function`;
   only safe standard libraries loaded.
- Expressiveness:
   closures,
   `pcall`/`error`,
   long strings for templates,
   and async:
   "`mlua` supports async/await for all Lua versions including Luau" (`mlua` README line 75);
   concurrent host operations are coroutines driven as Rust futures (docs "Async/await support").
- Sandboxing:
   `StdLib` flags select which standard libraries load (docs);
   Luau adds a sandbox mode (recall).
- Watch tracking:
   exact when `io` and `os` are not loaded.
- Error messages:
   message with source position and traceback (recall).
- Editor tooling:
   LuaLS with annotation files (recall; not checked).
- Documentation (recorded):
   no problem found in the README or crate page sections read.
- Maintenance:
   7 releases,
   60 open issues,
   2,865 stars.
- Binary size:
   lua.org:
   "the Lua interpreter built with all standard Lua libraries takes 293K and the Lua library takes 484K";
   the `mlua` embedding is unmeasured.
- Static linking:
   `vendored` builds "static Lua(JIT) libraries from sources during `mlua` compilation using lua-src or luajit-src"
   (README line 49),
   so a musl build needs a musl-targeting C compiler (inference).
- Rules:
   new language.
- Disqualifying problems found:
   none.
  Problems:
   full rewrite into a new language;
   C sources in the static build.

### Variant E: Rune (`rune` 0.14.2)

- Design:
   as C with Rune modules.
- Expressiveness:
   "First-class async support with Generators" (README line 43).
- Editor tooling:
   `rune-languageserver` exists only as a 2023-12-23 nightly asset.
- Maintenance:
   1 release in the past year (0.14.2, 2026-05-22),
   19,371 recent downloads,
   69 open issues.
- Binary size:
   unmeasured;
   the crate source is 2,771,441 compressed bytes.
- Rules:
   new language.
- Disqualifying problems found:
   none.
  Problems:
   low adoption,
   one release a year,
   dated tooling,
   full rewrite into a new language.

### Variant F: Rust compiled into the tool

- Design:
   the configuration is a Rust module inside the tool's crate;
   the single file contains the repository's configuration.
  An edit schedules a `cargo build` of the tool as a task,
   then the daemon restarts on the new binary,
   either at once (dropping subscriptions and running tasks)
   or when no task is running.
- Expressiveness:
   complete,
   including `tokio` concurrency.
- Sandboxing:
   none;
   configuration code can call `std::fs` directly.
- Watch tracking:
   source inputs from Cargo dep-info (probe);
   runtime reads are sound only if configuration code uses the tracked API,
   enforced at lint level with `clippy::disallowed_methods`,
   which the repository already denies fleet-wide (`file-enforcer.config.ts:1434-1439`).
- Daemon continuity:
   keeping running tasks across the restart needs file descriptor handoff;
   adopting inherited descriptors uses `FromRawFd::from_raw_fd`,
   an `unsafe fn` (`STD/os/fd/raw.rs:114`),
   while passing them with `SCM_RIGHTS` keeps repository code safe but cannot transfer parenthood of running children
   (inference from Unix process semantics; unexercised).
- Edit latency:
   a one-constant edit rebuilt a small leaf crate in 5.5 to 5.6 s with fat LTO (probe);
   the full daemon crate is larger and unmeasured.
- Error messages and tooling:
   `rustc` diagnostics and `rust-analyzer`.
- Binary size:
   no engine.
- Single-file fit:
   running needs nothing else,
   but every configuration edit needs `cargo` and `rustc` on the machine.
- Rules:
   Rust logic in configuration (`AD2`);
   `MXR` and `RDC` apply.
- Disqualifying problem:
   by the user's own precedent that the Rust core with TypeScript children passes only if Node travels inside the file,
   a configuration that needs an external compiler to take effect fails the single-file requirement,
   and the Rust toolchain cannot travel inside the file (inference; an open question for the user).
  Also every configuration edit restarts the long-running process.

### Variant G: Rust configuration crate run as a child binary

- Design:
   an `enforce/` crate in the repository depends on a published file-enforcer library crate;
   the daemon builds it with `cargo` as a task and runs it in a task cgroup;
   events flow over descriptor 3 with types shared through a protocol crate.
  Nightly `cargo -Zscript` could shrink the crate to one `file-enforcer.config.rs` file (probe),
   but stable Cargo 1.98.0 refuses it.
- Expressiveness:
   complete.
- Sandboxing:
   cgroups only.
- Watch tracking:
   as F.
- Edit latency:
   0.33 s debug rebuild of a small leaf crate (probe);
   a cold build of the library and its dependencies is unmeasured
   (the forbidden-strings clean release build took 50.86 s, `doc/troubleshooting/cargo-build-override-opt-level.md:247`
   as cited in `stack-rust.md`, R8; not re-read).
- Version skew:
   the prebuilt single file and the repository-built configuration binary can carry different protocol or library
   versions (inference).
- Single-file fit:
   worse than F:
   even an unedited configuration needs `cargo` on a fresh clone.
- Disqualifying problem:
   F's toolchain dependency,
   at every first run instead of only at edit time.

### Variant H: WASM guest (`wasmtime`)

- Design:
   the configuration compiles to a WASM component;
   the daemon hosts it with `wasmtime` 48.0.2 through a WIT interface that exposes only file-enforcer host functions;
   a new `.wasm` hot-loads without restarting the daemon.
- Expressiveness:
   the guest language's,
   for example Rust;
   parallelism comes from host-side concurrency.
- Sandboxing:
   strongest;
   no ambient file access without WASI preopens.
- Watch tracking:
   exact through host functions.
- Binary size:
   the docs' minimal C-API builds measured 2.0 to 2.1 MB for `libwasmtime.so` with `--no-default-features`,
   with Pulley as the execution engine;
   the page names no Wasmtime version (0 matches for "version");
   the CLI release is 11,424,388 compressed bytes.
  A component-model embedding for this design is unmeasured.
- Static linking:
   `crates/wasmtime/build.rs` compiles `src/runtime/vm/helpers.c` with `cc` under the `runtime` feature
   (`build.rs:112-135` at `v48.0.2`),
   so a musl build needs a musl-targeting C compiler (inference).
- Maintenance:
   71 releases in the past year,
   842 open issues.
- Documentation (recorded):
   the minimal-embedding size figures are not tied to a version,
   and they measure the C API rather than a Rust embedding.
- Disqualifying problem:
   F's toolchain dependency for edits (`wasm32-wasip2` target plus `cargo`),
   unless a compiled `.wasm` is committed,
   which puts a build artifact under version control.

### Variant I: declarative data with built-in Rust generators

- Design:
   `file-enforcer.toml` (or `.kdl` through `kdl` 6.7.1, which "preserves formatting")
   lists rules;
   generators such as `concat`, `mirror`, `cargo-manifests`, `lsp4ij`, `license-texts`, `skill-mirror`, `pnpr-config`,
   and `browserslist-targets` are Rust modules in the tool.
- Expressiveness:
   the root configuration's bespoke logic
   (pnpr selection and name validation, skill-mirror ownership pruning, SPDX expression mapping, scanner spawning)
   has no data form,
   so each becomes a repository-specific generator;
   the data file holds constants such as `CARGO_SHARED_DEPENDENCIES` and `PNPR_EXCLUDED_PACKAGES`.
- Watch tracking:
   exact,
   because generators are repository code that uses the tracked API.
- Single-file fit:
   data edits need nothing else;
   logic edits are variant F.
- Rules:
   literal `AD2` compliance;
   the approved-language proposal's "Native declarative workflow/task wiring is allowed; custom algorithms belong in
   approved source".
- Disqualifying problems:
   F's toolchain dependency and restart for every logic edit,
   plus a schema invented per generator.

### Variant J: pure configuration languages

- Nickel (`nickel-lang-core` 0.18.0):
   pure evaluation without effects (recall),
   so glob-read-decide flows need host precomputation;
   the static library asset is 19,498,430 bytes and the CLI 32,621,960 bytes;
   `nls` is a separate 20,084,432-byte binary.
  Documentation (recorded):
   the docs.rs crate root has no crate-level prose,
   going straight to the module list.
  Disqualifying problem:
   no effects for the configuration's discovery logic,
   and a size class near the one the user rejected.
- Jsonnet (`jrsonnet-evaluator`):
   newest stable release 0.4.2 on 2021-07-12,
   with only prereleases since (`0.5.0-pre98`);
   lazy and pure,
   so writes become a returned plan and reads become native callbacks (recall of Jsonnet's native-function mechanism);
   musl CLI 3,310,208 bytes.
  Disqualifying problem:
   no stable release in five years for a load-bearing dependency.

### Excluded before design

- Pkl through `rpkl` 0.8.0:
   "Requires the pkl binary to be available on your path" (README line 9),
   and that binary is 101,977,360 bytes.
- CUE:
   no Rust implementation found in the crates.io search terms used;
   embedding means a Go library,
   and the user excluded Go.
- Dhall through `serde_dhall` 0.13.0:
   0 releases in the past year and a total language without host effects (recall).
- Dynamic Rust plugins through `libloading` 0.9.0 or `abi_stable` 0.11.3 (last release 2023-10-12):
   musl's static `dlopen` is a stub that sets "Dynamic loading not supported" (`src/ldso/dlopen.c`, fetched from
   `git.musl-libc.org`),
   so a static single file cannot load them.
- Koto 0.16.1 (1,331 recent downloads),
   Steel 0.8.3 (a Scheme),
   and `piccolo` 0.3.3 (0 releases in the past year):
   adoption or activity below the other embedded languages.
- HCL (`hcl-rs`) and RON (`ron`):
   data formats with expressions but no author-defined functions (recall),
   folded into variant I.

## 4. Single-file shipping

### Static linking

- Target choice:
   `x86_64-unknown-linux-musl`,
   "64-bit Linux with musl 1.2.5" on the Rust platform support page,
   is the static target;
   the glibc target needs static `libc.a`,
   which this host lacks (probe).
- This host cannot build either today without a state change (probe);
   CI provisioning of the musl target is unverified.
- Pure Rust dependencies need only the Rust target.
- C sources need a musl-targeting C compiler:
   `rquickjs-sys` (QuickJS-ng),
   `mlua` `vendored` (Lua),
   and `wasmtime`'s `helpers.c`.
  `zig` 0.15.2 is on this host but was not tried as the compiler.
- `btrfs-uapi` 0.13.0:
   its `build.rs` runs `bindgen` on bundled `src/raw/btrfs_tree.h`,
   which includes `<linux/types.h>`, `<linux/ioctl.h>`, and `<linux/fs.h>` (`src/raw/btrfs.h:27-29`),
   and emits no link directives,
   so it adds no C library to link and does not break static linking.
  It needs `libclang` (present: `/usr/lib64/libclang.so.22.1.8`) and kernel UAPI headers (present under
  `/usr/include/linux/`) at build time;
   whether `bindgen` finds them when targeting musl is unverified.
- Engines and static linking:
   Boa, Starlark, Rhai, Rune, Nickel, and Jsonnet are pure Rust (inference);
   V8 ships a prebuilt musl static library (release asset above);
   dynamic plugin loading is impossible (musl `dlopen` stub).
- The `musl` allocator's multi-threaded performance is a known concern (recall; unverified), usually addressed with a
- Rust or C allocator crate.

### Size evidence

- Measured floor for a no-engine daemon on glibc:
   2,055,536 bytes,
   or 3,134,664 bytes with `zbus` (probes).
  The ported file-enforcer,
   scheduler,
   RPC,
   cgroup code,
   and `browserslist-data` add unmeasured size;
   a musl build's size is unmeasured.
- Added by the configuration host (all unmeasured for this design; nearest evidence):
  - F, G, I:
     none.
  - D (Lua):
     hundreds of kilobytes by lua.org's library figure.
  - A1 (QuickJS-ng):
     under 2.6 MB by the `qjs` CLI asset,
     plus type stripping if embedded.
  - H (`wasmtime`):
     about 2 MB minimal by the docs,
     more with the component model.
  - B, C, E:
     no evidence.
  - A2 (Boa):
     up to the 33.8 MB CLI.
  - J (Nickel):
     up to the 19.5 MB static library.
  - A3 (V8):
     a 39.8 MB compressed static library.
- Unpacking at run time is allowed by the user,
   but no variant needs it:
   every candidate links into one executable.

## 5. Migration

### Translating the root configuration

- A1 or A2:
   keep `file-enforcer.config.ts`;
   keep the import of `@monochromatic-dev/file-enforcer-core/ts`, served by the host loader;
   replace `node:` imports with host modules of the same names or inline equivalents;
   replace `import('browserslist')` with a host function backed by `browserslist-rs`;
   inline `isTypeScriptSourcePath` (an 8-line function, `package/config/pnpr/src/publish-plan.ts:186-193`) instead of
   loading `config-pnpr`, which imports `yaml`.
  Most of the 2,329 lines stay as they are (inference from the import surface).
- B, C, D, E:
   a full rewrite of 69 functions and 42 templates into the new language.
- F, G:
   a full rewrite into Rust modules under `MXR` and `RDC`,
   with the `CLAUDE.md` preamble as `include_str!` or a raw string.
- I:
   constants move into the data file;
   each generator becomes Rust code in the tool.

### Consumers of the TypeScript API

`rg --line-number 'dev-script-file-enforcer'` outside the package, `node_modules`, `dist`, and `doc`:

- `file-enforcer.config.ts:30`.
- `package.json:35` (root dependency) and `pnpm-lock.yaml:430`, `:1830`, `:4190`.
- `package/vm-builder/cli/src/import.ts:9` and `src/build-and-import.ts:20` import `exec`,
   called 14 times (`rg 'exec\('`),
   and `package/vm-builder/cli/package.json:16`.
  These need a TypeScript replacement for `exec` (`nano-spawn` or `node:child_process`) whatever the host.
- `package/test-fixture/file-enforcer-perf/src/perf.config.ts:21` and `src/perf.bench.test.ts:21`
   import `cat`, `dedup`, `getJsonProperty`, `overwrite`, `overwriteEach`, `readCache`, `reset`, `classifyEvent`,
   `expandGlob`, `mirrorGlobPath`, `trackDest`, `trackRead`, `trackWriteTime`;
   the fixture benchmarks the TypeScript implementation and would be retired or rewritten.
- `package/config/pnpr/config.yaml:69` lists the package as published,
   so external consumers of the registry package may exist (unverified).
- `package/oxlint-plugin/prefer-readonly-parameter-type/src/workspace-source-effect.unit.test.ts:22` reads
- `FE/cargo/apply-plan.ts` as a fixture.
- Task wiring:
   `mise.no-env.toml:1120-1127` (`sync:files`, `watch:sync:files`) runs
   `node package/file-enforcer/core/src/cli.ts`;
   `package/config/pnpr/src/publish-plan.ts:44` and `src/publish-missing-versions.ts:205` tell users to run
   `mise run sync:files`.
- Documents that describe the TypeScript design:
   `FE/../DECISION.rust-migration.md`,
   `FE/../README.md` (547 lines),
   `FE/../TODO.md`,
   `FE/../HANDOVER.fuzzing.md`,
   `doc/planning/cargo-toml-file-enforcer.md`,
   and `doc/troubleshooting/file-enforcer-staleness.md`.

### Interim coexistence

- Until parity,
   both implementations may write the same destinations and the same manifest;
   the Rust lock must follow the `mkdir` plus `owner.json` protocol and the SHA-256 manifest format,
   or the two must never run on one tree (inference).
- The prior research's "the rewrite's interim state is Option 2" assumed Node children;
   with Node shapes killed,
   the interim is the existing standalone TypeScript CLI beside a daemon that does not yet enforce files (inference).

## Documentation problems recorded

- `toml_edit` `Table::insert`:
   docs.rs says only "Inserts a key-value pair into the map.",
   while the implementation resets an existing key's formatting (`REG/toml_edit-0.25.13+spec-1.1.0/src/table.rs:434`),
   which deleted a comment line in the parity probe.
- Rhai:
   closures described as a feature on one page and as absent on another ("Variant C").
- Rhai minimal-build page:
   no size figures.
- Wasmtime minimal-embedding page:
   figures without a version,
   measured on the C API.
- `nickel-lang-core`:
   no crate-level documentation on docs.rs.
- bellard.org's QuickJS size figure does not describe the QuickJS-ng fork that `rquickjs` vendors
   (an evidence mismatch rather than a page error).
- Shared-core documentation problems from `stack-rust-crates.md` still apply
   (`tokio::process` and `ctrl_c`, `LinesCodec`, `btrfs-uapi` permission notes, systemd freezer pages).

### Yikes

Ranked by severity,
most severe first.
Shared items apply to every variant;
variant items follow.

#### Shared by every variant

1.  **Byte parity needs repository-written emulation over every candidate crate.**
    `toml_edit`, `serde_json`, and both XML parsers differed from today's bytes with default behavior;
    emulation layers matched every probed case except malformed XML,
    but `module-toml-edit`'s placement and formatting rules reached by file-enforcer are only partly ported,
    and a mismatch rewrites managed files including `CLAUDE.md`.
2.  **No static binary is buildable on this host as configured.**
    The musl target is not installed and static glibc is missing;
    every C-based engine or `wasmtime` also needs a musl C compiler;
    CI provisioning is unverified.
3.  **The rewrite is large and conflicts with a recorded decision.**
    49 core modules (3,547 code lines),
    9 plugin modules (952 code lines),
    a port of `module-toml-edit` behavior,
    and 56 test files (11,126 lines),
    against `DECISION.rust-migration.md`,
    `AD2`,
    and `MXR`/`RDC` for every Rust file.
4.  **Shared Rust core risks remain** (`stack-rust-crates.md`, "Remaining yikes"):
    unexercised cgroup migration, freeze, and kill;
    delegation prerequisites;
    repository-written watcher, subscriptions, and batching.
5.  **`browserslist-rs` data follows crate releases**, and its output parity is unprobed.
6.  **Package provisioning leaves the single file** through Meta Package Manager, an external Python tool.

#### A1 TypeScript on QuickJS-ng

1.  Repository-written Node-API shims and a `.d.ts` that must track Rust host functions.
2.  Type stripping needs an unmeasured `oxc` dependency or a switch to JavaScript with JSDoc.
3.  C sources in the static build.
4.  Exact API coverage of the vendored QuickJS-ng commit is unverified.
5.  Single-threaded engine: generator CPU work runs serially.

#### A2 TypeScript on Boa

1.  Size risk bounded only by the 33.8 MB CLI.
2.  A1 items 1, 2, and 5.

#### A3 TypeScript on V8

1.  Disqualifying: V8's 39.8 MB compressed static library puts it in the size class the user rejected.

#### B Starlark

1.  No async, exceptions, or `while` (reserved keywords); a full rewrite into a restricted language.
2.  New language under `AD2` and the approved-language proposal.
3.  No released language-server binary; unmeasured size.

#### C Rhai

1.  No async (maintainer statement).
2.  New language; language server last pushed 2023-03-17.
3.  Contradictory closure documentation; unmeasured size.

#### D Lua

1.  New language and a full rewrite.
2.  C sources in the static build.

#### E Rune

1.  New language, one release a year, low adoption, 2023 tooling.

#### F Rust compiled into the tool

1.  Disqualifying by the user's precedent: configuration edits need `cargo` and `rustc` outside the single file.
2.  Every configuration edit restarts the daemon or waits for idle; task handoff is unexercised.
3.  Untracked `std::fs` reads are prevented only at lint level.

#### G Rust child binary

1.  Disqualifying by the user's precedent: every fresh clone needs `cargo` before the configuration runs.
2.  Tool and library version skew.
3.  F item 3.

#### H WASM guest

1.  Disqualifying by the user's precedent for edits, or a committed binary artifact.
2.  71 `wasmtime` releases a year; C helper in the static build; unmeasured component-model size.

#### I Declarative data with built-in generators

1.  Disqualifying for logic edits, which are F.
2.  A schema invented per bespoke generator.

#### J Nickel or Jsonnet

1.  Nickel: no effects and a 19.5 MB static library.
2.  Jsonnet: no stable release since 2021.

## Ranking of configuration-hosting variants

Per the user's decision,
this ranking replaces the comparison against the Rust core with TypeScript children and the TypeScript daemon with a
Rust addon,
both of which ship Node and are out.

A1 > A2 > D > B > C > E > H > I > F > G > J (Jsonnet) > J (Nickel) > A3.

- A1 over A2:
   both keep the configuration in TypeScript with a near-mechanical migration,
   but QuickJS-ng has size evidence under 2.6 MB while Boa's only bound is a 33.8 MB CLI,
   and size decided the Node shapes;
   Boa's pure-Rust static build does not outweigh that.
- A2 over D:
   A2 keeps TypeScript,
   satisfying `AD2` and the approved-language proposal,
   and keeps most of the 2,329 lines,
   while D rewrites everything into an unapproved language;
   D's smaller engine does not outweigh two rule conflicts.
- D over B:
   D keeps async and exceptions and has small, documented engine size,
   while B lacks async, exceptions, and `while`.
- B over C:
   B has a maintained language-server library and a specification,
   while C's language server has not moved since 2023 and neither has async.
- C over E:
   C has 7 releases a year and 5,682 stars,
   while E has 1 release and 19,371 recent downloads, despite E's async support.
- E over H:
   E evaluates without an external compiler,
   while H needs `cargo` and a WASM target for every edit,
   which the user's single-file precedent treats as disqualifying.
- H over I:
   both need a compiler for logic edits,
   but H hot-loads without restarting the daemon and sandboxes the configuration,
   while I restarts the daemon and invents a schema per generator.
- I over F:
   I's data-only edits need no compiler or restart,
   while every F edit does.
- F over G:
   F's single file runs an unedited configuration without a compiler,
   while G needs `cargo` on every fresh clone and adds version skew.
- G over Jsonnet:
   G has a maintained toolchain,
   while Jsonnet's evaluator has had no stable release since 2021 and needs the configuration redesigned as a pure plan.
- Jsonnet over Nickel:
   Jsonnet's native callbacks can supply reads and its CLI is 3.3 MB,
   while Nickel has no effects and a 19.5 MB static library.
- Nickel over A3:
   Nickel's size problem is bounded at 19.5 MB,
   while V8 is in the size class that already killed Node.

Fewest and least severe disqualifying problems:
A1 (TypeScript on `rquickjs`) has no disqualifying problem found;
its worst items are repository-written shims and an unmeasured type-stripping dependency.

## Open questions

- Does a configuration that needs the repository's Rust toolchain to take effect count against the single-file
- requirement,
   as this research inferred from the user's Node precedent?
  A "no" moves F, G, H, and I above the embedded interpreters,
   because they avoid shims and a second language.
- Should the configuration stay TypeScript (keeping `AD2`),
   or should `AD2` be amended for a different configuration language?
- Is in-process evaluation acceptable without cgroup pause and end,
   or should evaluations run as a re-executed child of the single file?
- Next measurements, each needing network access for Cargo or an installed musl target:
   build A1, A2, B, C, and D embeddings with the file-enforcer host API stub and record static musl sizes;
   probe `browserslist-rs` output against `.browserslistrc.resolved.local.json`;
   and run the differential parity harness on a throwaway worktree.
