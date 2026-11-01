# Declarative configuration for the all-Rust monorepo manager

Merged into "Declarative configuration" in `doc/planning/monorepo-manager-from-scratch-design.md` on 2026-09-17.
This research read the configuration as forbidden from being Turing-complete;
the user only removed the requirement,
so G and H are disqualified only under that reading.

Research date: 2026-09-16.
Scope: read-only design research for the configuration of `./meow`
(placeholder name),
the single-file all-Rust monorepo manager accepted in `doc/decision/monorepo-manager-all-rust.md`.
This file replaces the configuration-hosting research in
`doc/planning/monorepo-manager-route-research/stack-all-rust-rewrite.md`
and reuses its measurements instead of repeating them.
Nothing under `/var/home/user/Monochromatic` was modified,
installed,
or committed.
No cgroup,
systemd unit,
or scope was created.

## Status and inputs

- Decisions carried in,
   all stated by the user on 2026-09-16
   (Verified: `doc/decision/monorepo-manager-all-rust.md` and
   `doc/planning/monorepo-manager-from-scratch-design.md`, sections "Configuration" and "Hashing", read this session):
  - The tool is all Rust,
     ships as one file run as `./meow`,
     and may unpack at run time.
  - file-enforcer is rewritten in Rust;
     FE14 (Cargo manifests) and FE15 (JetBrains) are plugins by definition;
     FE18 and FE19 may go to Meta Package Manager.
  - The configuration is declarative and not Turing-complete.
    The decision record lists this as decided,
     and rejects
     "Embedding a JavaScript, Lua, Starlark, Rhai, Rune, or WebAssembly host for a Turing-complete configuration".
  - `./meow` serves repositories other than Monochromatic.
  - 0.x release-blocking platforms are Linux on x86_64 and aarch64,
     each glibc-linked and static musl.
  - Cgroup sandboxing is a must;
     btrfs is the only accelerated filesystem.
- Relayed by the coordinating agent during this research:
  - The user named OpenTofu as the lead precedent:
     "I think opentofu does config files properly."
  - Content hashing uses `gxhash` 3 with `gxhash128`,
     not a cryptographic hash,
     under the constraints in `doc/troubleshooting/gxhash-aes-target-feature.md`.
  - Binary size and static builds are considered for both architectures.
- Documentation problems of runtimes and libraries are recorded and never cull an option.

## Conventions

- **Verified** means a file and line range read in this session,
   a page or file fetched in this session with the quoted text found in it,
   or a probe listed in "Probes run" with its command and output.
- **Unverified** means inference,
   arithmetic over measured numbers,
   or recall.
- Path abbreviations:
  - `SCR/` is `/tmp/claude-1000/-var-home-user-Monochromatic/e28ad59c-f3f5-46e9-8c8c-59a61c331610/scratchpad/`.
  - `DC/` is `SCR/declarative/`,
     this research's probe directory.
  - `AG/` is `~/temp/agent/`;
     every clone is `AG/<name>-2026-09-16`,
     made with `gh repo clone <repo> <dir> -- --depth 1`.
  - `FEC` is the repository's `file-enforcer.config.ts`.
  - `FE/` is `package/file-enforcer/core/src/`.
- Clone heads used for citations:
   `opentofu/opentofu` `bd2df274b`,
   `hashicorp/hcl` `00057cf06`,
   `opentofu/tofu-ls` `c37a827`,
   `martinohmann/hcl-rs` `f7b17594f`,
   `gruntwork-io/terragrunt` `3e574b36c`,
   `moonrepo/moon` `171d301f7`,
   `vercel/turborepo` `438c45b38`,
   `nrwl/nx` `e9ee96934`,
   `pantsbuild/pants` `511d4a7de`,
   `facebook/buck2` `b60de27af`,
   `bazelbuild/bazel` `8e90a0d8a`,
   `facebook/starlark-rust` `cc22bd6`,
   `jdx/mise` `bcb644309`,
   `go-task/task` `e165a4cfa`,
   `casey/just` `5969e396f`,
   `dprint/dprint` `89cc71a28`,
   `dprint/jsonc-parser` `e6e3837`,
   `evilmartians/lefthook` `1e23553ee`,
   `pre-commit/pre-commit.com` `2273402ec`,
   `renovatebot/renovate` `8763fe4cf`,
   `JamieMason/syncpack` `958d30689`,
   `copier-org/copier` `217e4f878`,
   `rust-lang/cargo` `495c385d0`,
   `projen/projen` `f98f6c2ba`,
   `kdl-org/kdl-rs` `ef1c325`,
   `tamasfe/taplo` `08f343be0`,
   `jelmer/yaml-edit` `543349e`,
   `rossmacarthur/upon` `9361a3d`,
   `Keats/tera` `6ead3c0`,
   `jdx/pklr` `681fee5`,
   `kcl-lang/kcl` `1970fb2`.
- HTTP requests used the User-Agent `declarative-config-research`;
   GitHub reads went through `gh`.

## Probes run

### Network

- `cargo generate-lockfile` in `DC/netprobe` with an `hcl-rs = "0.19"` dependency printed
   "Updating crates.io index" and "Locking 22 packages",
   so Cargo reached the index this session,
   unlike the earlier research
   (Verified, probe).
- Crate metadata:
   `node DC/meta/crates-meta.ts` fetched `https://crates.io/api/v1/crates/<name>` and `gh api repos/<owner>/<repo>`;
   results are in `DC/meta/crates-meta.json`.
  "Releases in the past year" counts non-yanked versions created since 2025-09-16.

### Format size, diagnostics, and write-back probe

- Crate `DC/sizes`,
   one Cargo feature per format,
   built by `node DC/sizes/run.ts` with
   `cargo build --release --target x86_64-unknown-linux-musl --jobs 4 --no-default-features --features <f>`
   on `rustc 1.100.0-nightly (0fc141305 2026-09-11)`,
   profile `lto = true`, `codegen-units = 1`, `strip = true`, `panic = "abort"`.
  Each binary parses a small configuration into one shared `serde` struct,
   prints a type error and a syntax error,
   and,
   for editing crates,
   changes one value and prints the document.
  `file` reported every binary as static
   (Verified, `DC/sizes/out/summary.txt`).
- Static musl sizes in bytes,
   with the increase over the 385,656-byte baseline
   (a `serde` derive and `println!` only)
   (Verified, probe; increases are Unverified arithmetic):
  - `serde_json` 1.0.151: 451,192 (+65,536).
  - `json5` 1.3.1: 496,248 (+110,592).
  - `ron` 0.12.2: 549,496 (+163,840).
  - `jsonc-parser` 0.33.2 with `cst` and `serde`: 565,880 (+180,224).
  - `upon` 0.11.0: 578,168 (+192,512).
  - `toml` 1.1.6: 594,552 (+208,896).
  - `yaml-edit` 0.3.1: 594,552 (+208,896).
  - `toml_edit` 0.25.15: 610,936 (+225,280).
  - `miette` 7.6.0 with `fancy`: 660,152 (+274,496).
  - `kdl` 6.7.1: 688,760 (+303,104).
  - `hcl-edit` 0.9.7: 729,720 (+344,064).
  - `handlebars` 6.4.4: 877,208 (+491,552).
  - `knus` 3.4.0 with `miette`: 1,155,800 (+770,144).
  - `tera` 2.4.0: 1,204,888 (+819,232).
  - `serde-saphyr` 1.3.0: 1,327,872 (+942,216).
  - `minijinja` 2.24.0: 1,413,984 (+1,028,328).
  - `hcl-rs` 0.19.8 deserializer plus evaluator: 1,438,432 (+1,052,776).
  - `serde_dhall` 0.13.0 without default features: 2,171,648 (+1,785,992).
  - `cel` 0.14.5: 3,048,480 (+2,662,824).
  - `regorus` 0.12.0: 7,424,608 (+7,038,952).
  - `askama` 0.16.1 (compile-time templates): 385,656 (+0).
- Build failures:
  - `serde_dhall` with default features failed on `openssl-sys` 0.9.117:
     "Could not find directory of OpenSSL installation"
     with `$TARGET = x86_64-unknown-linux-musl`;
     its only default feature is `reqwest`
     (Verified, `DC/sizes/out/dhall.build.txt` and `https://crates.io/api/v1/crates/serde_dhall/0.13.0`).
  - `starlark` 0.14.2 failed on the repository's nightly:
     "error[E0119]: conflicting implementations of trait `Allocative` for type `!`"
     at `allocative-0.3.6/src/impls/std/unsorted.rs:214`,
     an impl gated `#[cfg(rust_nightly)]`
     (Verified, probe and `gh api` read of `facebookexperimental/allocative`).
    `AG/buck2-2026-09-16/allocative/allocative/src/impls/std/unsorted.rs` no longer contains that impl
     (Verified, `rg`),
     and the `allocative` repository's newest commit is "Prepare to retire allocative and gazebo repositories"
     (Verified, `gh api`).
  - Built instead with `RUSTUP_TOOLCHAIN=stable` for glibc:
     5,593,360 bytes against a 311,856-byte glibc baseline,
     not a static binary
     (Verified, probe).
- C build steps:
   `node DC/sizes/cc-check.ts` ran `cargo tree` per feature;
   only `starlark` pulls `cc`,
   and the rest list no `cc`,
   `cmake`,
   `bindgen`,
   or C `-sys` crate
   (`linux-raw-sys` is pure Rust)
   (Verified, probe).
  So an aarch64 static build of every format crate except Starlark needs only the Rust target
   (Unverified: no aarch64 target is installed, so no aarch64 binary was built or sized).
- Write-back and diagnostics outputs are quoted where each format is described
   (Verified, `DC/sizes/out/<feature>.run.txt`).

### Round trip over repository files

- `DC/roundtrip`,
   glibc release build,
   parsed repository files read-only and compared the re-serialized text byte for byte
   (Verified, probe):
  - `hcl-edit`:
     `package/config/tofu/hetzner.tf` (27,689 bytes) identical;
     `DC/tofu/good/entry.tf` identical;
     `DC/tofu/good/main.tf` **not** identical.
    The difference:
     a `for` expression condition written over four lines joined by `&&`
     came back on one line
     (`diff` output in "HCL",
     Verified).
  - `toml_edit`:
     `mise.no-env.toml` (57,190 bytes) and `package/linter/rust/Cargo.toml` identical.
  - `jsonc-parser` CST:
     `package/config/dprint/index.json` and `package/config/typescript/tsconfig.options.json` identical.
  - `yaml-edit`:
     `pnpm-workspace.yaml` (18,343 bytes),
     `package/config/pnpr/config.yaml`,
     and `.github/workflows/pnpr-publish.yml` identical.

### OpenTofu probes

Run with the repository's `opentofu` 1.12.6 under Mise
(`~/.local/share/mise/installs/opentofu/1.12.6/tofu`,
`tofu version` printed "OpenTofu v1.12.6")
in throwaway directories under `DC/tofu/`,
with no providers downloaded:

- `DC/tofu/good/main.tf` expressed four units of today's configuration with built-in functions only:
   `CLAUDE.md` as a preamble plus `file("AGENTS.md")`,
   the skill mirror map with `fileset`,
   the SPDX-to-text mapping,
   and pnpr selection with a `templatefile` rendering;
   `tofu plan` printed the expected outputs
   (Verified, probe).
- `DC/tofu/good/entry.tf` expressed the whole of `pnprManifestHasEntryPoint`
   (`FEC:1924-1981`) without author-defined functions.
  The first attempt failed:
   "Error: Inconsistent conditional result types ...
   The 'true' value is object, but the 'false' value is string."
  After rewriting the `bin` branch with `try(tolist([tostring(m.bin)]), tolist(values(m.bin)), [])`,
   all nine fixture manifests matched the TypeScript semantics
   (`exports_string = true`, `exports_ts_only = false`, `main_dts = true`, `exports_and_main = false`, and so on)
   (Verified, probe).
- `tofu validate -json` on a malformed file returned diagnostics with byte ranges,
   for example
   summary "Invalid default value for variable",
   detail "This default value is not compatible with the variable's type constraint: a number is required.",
   and range `{"filename":"main.tf","start":{"line":3,"column":13,"byte":49},"end":{"line":3,"column":19,"byte":55}}`
   and "Call to unknown function ... There is no function named \"uppercase\""
   (Verified, probe).
- A failing `validation` block printed
   "jobs must exceed 4." and "This was checked by the validation rule at main.tf:4,3-13."
   (Verified, probe).
- `tofu fmt` kept both comments and aligned `a  = 1 # trailing` and `bb = "x"`
   (Verified, probe).
- A `main.tf.json` file with a `"//"` comment property,
   `"${sort(var.names)}"`,
   and `"${local.count + 1}"`
   planned `n = 3` and a sorted list
   (Verified, probe).

### Template recursion probes

In `DC/sizes` (Verified, probe outputs):

- `minijinja`:
   a macro calling itself printed `"321 x"`.
- `tera` 2:
   a component calling itself with `n={n - 1}` printed `"321"`.
- `handlebars`:
   a partial recursing over nested data printed `"a(b(c))"`;
   a partial including itself unconditionally returned `reason: CannotIncludeSelf`.
- `upon` and `askama` were exercised with loops only;
   their recursion rules come from documentation.

### Starlark and Pkl evaluation probes

- `starlark` 0.14.2 on stable glibc,
   `Dialect::Standard`,
   evaluated `def fact(n): return 1 if n <= 1 else n * fact(n - 1)` and printed
   "starlark recursion result: 120"
   (Verified, probe).
- `pklr` 2.0.4,
   default features,
   glibc release build of 5,226,752 bytes
   (Verified, probe):
  - `function double(n: Int): Int = n * 2` and a lambda evaluated to `{"result":42,"viaLambda":2}`.
  - A module-level function calling itself failed with "Eval error: undefined variable: fact".
  - A typed `Listing<Node>` property failed with "property 'children' expected Listing<Node>, got Object".

## 1. Logic inventory

Source:
`FEC` (2,329 lines),
read in full this session,
and the repository modules it imports:
`@monochromatic-dev/file-enforcer-core/ts`
(`FE/index.ts`,
with `FE/cargo/*.ts` and `FE/jetbrains/lsp4ij*.ts` read)
and `@monochromatic-dev/config-pnpr/ts`
(`isTypeScriptSourcePath`,
`package/config/pnpr/src/publish-plan.ts:156-193`)
(Verified).
Its other imports are `node:crypto`,
`node:fs/promises`,
`node:path`,
`nano-spawn`,
and a dynamic `import('browserslist')` at `FEC:622`
(Verified).

### Vocabulary used by the declarative forms

The forms below use the OpenTofu-shaped notation of option A ("7. Options"),
because OpenTofu is the lead precedent.
Options B to F transliterate the same blocks;
where a data-only format cannot express a form,
the unit says so.
Every block name is a proposal,
not an existing API.

- `absent "<name>"`:
   a path that must not exist,
   with `on_present = "error"` or `"remove"`.
- `file "<name>"`:
   one managed destination with `path` and `content`,
   plus `if_exists` (`"overwrite"` default or `"skip"`),
   `mode`,
   `sensitive` (never print contents or diffs, after OpenTofu's `sensitive` outputs),
   and protection against external edits by default.
- `mirror "<name>"`:
   copies files matched by `from` to the positional pattern `to`,
   with optional `header`,
   literal `replace` pairs,
   and `prune`.
- `prune`:
   `"ledger"` removes only destinations this rule wrote on an earlier run and no longer declares;
   `"owned"` with `owns = [globs]` removes every undeclared file matching the globs.
- `toml_keys "<name>"`:
   guarded key enforcement over TOML files,
   the FE14 plugin surface.
- `lsp4ij_server "<name>"`:
   the FE15 plugin surface.
- `task "<name>"`:
   an ordinary program with `command`,
   declared `inputs` and `outputs`,
   `depends_on`,
   and `managed = true` when its outputs are protected destinations.
- `check "<name>"`:
   assertions evaluated before any write,
   after OpenTofu `check` blocks.
- Functions used:
   `file`,
   `fileset`,
   `jsondecode`,
   `tomldecode`,
   `join`,
   `concat`,
   `distinct`,
   `sort`,
   `lookup`,
   `try`,
   `can`,
   `dirname`,
   `startswith`,
   `strcontains`,
   plus proposed `spdx_ids(expression)` and `lockfile(package_name)`.
- Variables:
   `path.root` (repository root),
   `each.key` and `each.value` under `for_each`,
   and a per-match iterator such as `manifest.dir` inside rules that visit many files.

### LI01 forbidden root `CONTEXT.md`

- Source:
   `FEC:44`, `:251-270`, `:578-596`, `:2202`.
- Reads:
   `lstat("./CONTEXT.md")`,
   registered as a watched path.
- Writes:
   nothing;
   throws `ForbiddenRootContextFileError` before any other write.
- Placement:
   general built-in feature `absent`,
   with the message as plain data.
  "Before any write" becomes the built-in rule that checks and `absent` errors run before enforcement.

```hcl
# meow.hcl
absent "root_context" {
  path          = "CONTEXT.md"
  on_present    = "error"
  error_message = "CONTEXT.md is forbidden. Do not create cached context files; read source code directly and use doc/agent/domain.md for this repo policy."
}
```

### LI02 root `LICENSE`

- Source:
   `FEC:2205-2209`.
- Reads `LICENSES/LGPL-3.0-or-later.txt`;
   writes `LICENSE` (tracked).
- Placement:
   general built-in `file`.

```hcl
# meow.hcl
file "license" {
  path    = "LICENSE"
  content = file("LICENSES/LGPL-3.0-or-later.txt")
}
```

### LI03 `CLAUDE.md`

- Source:
   `FEC:2211-2246`,
   a 2,970-character template literal preamble
   (the earlier research's AST probe)
   followed by `AGENTS.md`.
- Reads `AGENTS.md`;
   writes `CLAUDE.md` (tracked).
- Placement:
   general built-in `file` with concatenated parts;
   the preamble becomes a checked-in text file.
- Awkward:
   the header "Generated from `AGENTS.md` by file-enforcer." changes wording if the tool name appears in it,
   which touches the design's open byte-identity question;
   and where the preamble file lives
   (for example `doc/agent/claude-code-preamble.md`)
   is a placement choice,
   not a measured fact.
  Inline heredocs would also work in HCL, TOML, KDL, and YAML,
   but not in JSONC,
   which has no multi-line strings.

```hcl
# meow.hcl
file "claude_md" {
  path = "CLAUDE.md"
  content = join("", [
    file("doc/agent/claude-code-preamble.md"),
    file("AGENTS.md"),
  ])
}
```

### LI04 generated `mise.toml`

- Source:
   `FEC:656-724`.
- Reads `mise.no-env.toml` and the directory glob `package/*/*/node_modules/.bin`
   (install state, not workspace metadata;
   the generated comment points at issue #335).
- Writes `mise.toml` (tracked):
   a header,
   `mise.no-env.toml`,
   and an `[env]` section whose `_.path` lists `node_modules/.bin` then the sorted glob.
- Placement:
   retirement with Mise.
  Until then the decision record keeps Mise on macOS and Windows CI,
   so an interim form is needed
   (Verified: `doc/decision/monorepo-manager-all-rust.md`, "Decision").
- Forced and awkward:
   OpenTofu's `fileset` "enumerates a set of regular file names"
   (Verified: `website/docs/language/functions/fileset.mdx:3`),
   so a directory glob needs a new function;
   the list-to-TOML-array text needs `join` over a `for` expression,
   which data-only formats cannot write
   and would need either a bespoke built-in or a task.
  Deriving the list from workspace metadata,
   as issue #335 proposes,
   removes the install-state dependency.

```hcl
# meow.hcl, interim until Mise leaves CI
locals {
  mise_bin_paths = distinct(concat(
    ["node_modules/.bin"],
    [for p in project_dirs("pnpm") : "${p}/node_modules/.bin"], # workspace metadata, issue #335
  ))
}

file "mise_toml" {
  path = "mise.toml"
  content = join("", [
    "# Generated from mise.no-env.toml by meow.\n",
    file("mise.no-env.toml"),
    "\n",
    file("config/meow/mise-env-head.toml"),
    join(",\n", [for d in local.mise_bin_paths : "  \"${d}\""]),
    file("config/meow/mise-env-tail.toml"),
  ])
}
```

### LI05 forbidden-strings local appendix seed

- Source:
   `FEC:812-834`.
- Writes `forbidden-strings.append.local.txt` only when absent
   (gitignored by `.gitignore:11` `*.local.*`, Verified),
   then always `chmod 0600`.
- Placement:
   general built-in `file` with `if_exists = "skip"` and `mode`,
   after Terragrunt's `if_exists = "skip"`
   ("Precedent").

```hcl
# meow.hcl
file "forbidden_strings_local_seed" {
  path      = "forbidden-strings.append.local.txt"
  content   = file("config/meow/forbidden-strings-local-seed.txt")
  if_exists = "skip"
  mode      = "0600"
  sensitive = true
}
```

### LI06 forbidden-strings runtime rules

- Source:
   `FEC:835-859`.
- Reads `forbidden-strings.append.txt` and `forbidden-strings.append.local.txt`;
   `cat` joins files with `'\n'`
   (Verified: `FE/io/cat.ts:171`).
- Writes `.cache/forbidden-strings.rules.txt`
   (gitignored by `.gitignore:68`, Verified),
   creates `.cache`,
   sets mode 0600.
- Placement:
   general built-in `file`.
  Ordering against LI05 comes from path overlap:
   this rule reads the path LI05 writes.

```hcl
# meow.hcl
file "forbidden_strings_rules" {
  path = ".cache/forbidden-strings.rules.txt"
  content = join("", [
    file("config/meow/forbidden-strings-rules-header.txt"),
    join("\n", [
      file("forbidden-strings.append.txt"),
      file("forbidden-strings.append.local.txt"),
    ]),
  ])
  mode      = "0600"
  sensitive = true
}
```

### LI07 eager scanner cache compilation

- Source:
   `FEC:182-184`, `:736-782`, `:860`.
- Reads the rules file and the release scanner binary;
   when the binary is absent it logs a warning and skips.
- Runs `package/forbidden-strings/cli/target/release/forbidden-strings compile-rules`
   with `--rules ./.cache/forbidden-strings.rules.txt`,
   which "Compiles one authoritative runtime rules file into derived user cache"
   (Verified: `package/forbidden-strings/cli/src/cli.rs:127`),
   content-keyed
   (Verified: `src/runtime_cache/mod.rs:10`, "Registers content key and platform cache path resolution").
- Placement:
   repository task in `package/forbidden-strings/cli`,
   depending on the scanner's release build task.
- Awkward:
   its output is outside the repository,
   so the task declares no outputs and relies on the scanner's own content key;
   and "skip with a warning when the scanner is absent" becomes "build the scanner first",
   a behavior change on fresh clones
   (the earlier research cites a 50.86-second clean release build of the scanner, not re-measured).
  The `win32` executable suffix retires with 0.x being Linux only.

```hcl
# package/forbidden-strings/cli/meow.hcl
task "compile_rules" {
  command    = ["target/release/forbidden-strings", "compile-rules", "--rules", "${path.root}/.cache/forbidden-strings.rules.txt"]
  inputs     = ["${path.root}/.cache/forbidden-strings.rules.txt"]
  depends_on = [task.build_release]
  cache      = false # output lives in the user cache, keyed by content inside the scanner
}
```

### LI08 retired root rules output

- Source:
   `FEC:861-866`,
   `rm -f ./forbidden-strings.local.txt`.
- Placement:
   retirement;
   it is a migration shim for a pre-2026 output.
  If kept,
   the general built-in `absent` covers it.

```hcl
# meow.hcl, only if the shim is kept
absent "retired_forbidden_strings_local" {
  path       = "forbidden-strings.local.txt"
  on_present = "remove"
}
```

### LI09 git-policy source mirrors

- Source:
   `FEC:2252-2296`.
- Reads 18 listed source files in three packages;
   writes 18 files under `package/git-policy/cli/src/optional/` (tracked),
   each with a one-line header and import-specifier rewrites.
- The first group uses `replaceAll`,
   the other two `replace`,
   which replaces only the first occurrence.
  Every mirrored file in those two groups contains the specifier at most once
   (Verified: `rg --count-matches` over the three `src` directories),
   so a single all-occurrences replacement reproduces today's bytes.
- Why the mirrors exist:
   "Cli-git statically bundles generated source mirrors into its single import and executable artifact"
   (Verified: `package/git-policy/forbidden-strings/README.md:6`).
- Placement:
   general built-in `mirror` with `header` and `replace`.
  Retiring the mirrors through bundler path aliases in cli-git is possible in principle
   (Unverified: not tried).
- Awkward:
   explicit file lists,
   because `package/git-policy/markdown-lint/src/markdown-lint-fixture.ts` exists and is not mirrored
   (Verified: `rg` output).

```hcl
# meow.hcl
locals {
  git_policy_mirrors = {
    repository-policy = {
      source  = "package/git-policy/repository/src"
      files = [
        "bump-dependents-worktree.ts", "dependent-bump-workflow.ts", "dependent-version-bump-policy.ts",
        "dependent-version-bump.ts", "index.ts", "manifest-text.ts", "publishable-names.ts", "source-imports.ts",
      ]
      replace = {
        "@monochromatic-dev/git-policy-api/ts"           = "../../api/index.ts"
        "@monochromatic-dev/git-policy-markdown-lint/ts" = "../markdown-lint/index.ts"
      }
    }
    forbidden-strings = {
      source  = "package/git-policy/forbidden-strings/src"
      files = [
        "cache-warning.ts", "errors.ts", "index.ts", "materialize-candidates.ts", "scan-candidates.ts", "scanner-output.ts",
      ]
      replace = { "@monochromatic-dev/git-policy-api/ts" = "../../api/index.ts" }
    }
    markdown-lint = {
      source  = "package/git-policy/markdown-lint/src"
      files   = ["errors.ts", "full-content-patch.ts", "index.ts", "rewrite-candidates.ts"]
      replace = { "@monochromatic-dev/git-policy-api/ts" = "../../api/index.ts" }
    }
  }
}

mirror "git_policy" {
  for_each = local.git_policy_mirrors
  from     = [for f in each.value.files : "${each.value.source}/${f}"]
  to       = "package/git-policy/cli/src/optional/${each.key}/*"
  header   = "// Generated from `${mirror.source}` by meow; edit canonical source owner.\n"
  replace  = each.value.replace
}
```

### LI10 package license texts

- Source:
   `FEC:89-130`, `:301-566`.
- Reads every `package/*/*/package.json` string `license`
   and every `package/*/*/Cargo.toml` string `package.license`
   (a workspace-inherited table is ignored).
- Maps each SPDX expression by substring:
   `LGPL-3.0-or-later` yields GPL and LGPL texts,
   otherwise `GPL-3.0-or-later` yields GPL,
   and `CC-BY-SA-4.0` yields CC,
   unioned per package directory.
- Writes `<package>/LICENSES/<id>.txt` copies of root `LICENSES/<id>.txt`
   (312 tracked today, Verified: `git ls-files`),
   and deletes known-id texts a package no longer needs,
   but only in packages that have at least one license expression.
- Placement:
   general built-in,
   either as a REUSE-style `license_texts` feature
   or composed from generic parts:
   a real SPDX parser function,
   `for_each` over `file` rules,
   and `prune = "owned"`.
- Awkward:
   substring matching and SPDX parsing agree on today's expressions only by inspection
   (Unverified: not compared across all manifests);
   and `owns` over `package/*/*/LICENSES/{known ids}.txt` would also prune packages with no license expression,
   which today's code leaves alone.
  A data-only format needs the `license_texts` built-in,
   because the per-package union is a computation.

```hcl
# meow.hcl, composed form
locals {
  license_text_sources = {
    "CC-BY-SA-4.0"      = "LICENSES/CC-BY-SA-4.0.txt"
    "GPL-3.0-or-later"  = "LICENSES/GPL-3.0-or-later.txt"
    "LGPL-3.0-or-later" = "LICENSES/LGPL-3.0-or-later.txt"
  }
  license_implies = { "LGPL-3.0-or-later" = ["GPL-3.0-or-later"] }

  license_expressions = concat(
    [for p in fileset(path.root, "package/*/*/package.json") : { dir = dirname(p), expr = try(tostring(jsondecode(file(p)).license), null) }],
    [for p in fileset(path.root, "package/*/*/Cargo.toml") : { dir = dirname(p), expr = try(tostring(tomldecode(file(p)).package.license), null) }],
  )

  license_files = merge([
    for e in local.license_expressions : {
      for id in distinct(flatten([for i in spdx_ids(e.expr) : concat([i], lookup(local.license_implies, i, []))])) :
      "${e.dir}/LICENSES/${id}.txt" => local.license_text_sources[id]
      if contains(keys(local.license_text_sources), id)
    } if e.expr != null
  ]...)
}

file "package_license_text" {
  for_each = local.license_files
  path     = each.key
  content  = file(each.value)
  prune    = "owned"
  owns = [
    "package/*/*/LICENSES/CC-BY-SA-4.0.txt",
    "package/*/*/LICENSES/GPL-3.0-or-later.txt",
    "package/*/*/LICENSES/LGPL-3.0-or-later.txt",
  ]
}
```

### LI11 Cargo manifest enforcement (FE14 plugin)

- Source:
   `FEC:1382-1779` and `FE/cargo/{types,apply-plan,manage-cargo-manifests}.ts`.
- Reads and writes each first-party `Cargo.toml` in place
   (19 tracked, Verified: `git ls-files`),
   discovered by `package/*/*/Cargo.toml` and `package/*/*/*/Cargo.toml`
   and filtered by `/target/` and `/node_modules/`
   (Verified: `manage-cargo-manifests.ts:28-31`).
- Logic:
   guarded enforcements
   (edition under `[package]`;
   license and publish only when present;
   three clippy keys under `[lints.clippy]`;
   21 shared dependency requirements only when the dependency is present;
   repository only when present;
   readme and a path-derived homepage when repository is present),
   a directory-to-profile-preset map over nine crates,
   and two block insertions when `[lints.clippy]` or `[workspace]` is absent.
- Placement:
   general built-in plugin `toml_keys` plus plain data.
- Awkward:
   the homepage and profile lookups are per-manifest derivations,
   which need a per-match iterator in HCL
   or a placeholder microsyntax such as `{dir}` in data-only formats.

```hcl
# meow.hcl
locals {
  cargo_shared_dependencies = {
    dependencies = {
      aho-corasick          = "1"
      anyhow                = "1"
      arbitrary             = { version = "1", features = ["derive"] }
      clap                  = { version = "4", features = ["derive"] }
      gxhash                = "3"
      i-slint-backend-winit = "1.17.0"
      ignore                = "0.4"
      libfuzzer-sys         = { version = "0.4", features = ["arbitrary-derive"] }
      memchr                = "2"
      opus                  = { git = "https://github.com/SpaceManiac/opus-rs", rev = "559876660603dc8079a053e03e6438766f669e69" }
      rayon                 = "1"
      regex                 = "1"
      ringbuf               = "0.4"
      serde                 = { version = "1", features = ["derive"] }
      serde_json            = "1"
      slint                 = { version = "1.17.0", features = ["backend-winit", "renderer-femtovg", "renderer-software"] }
      symphonia             = { version = "0.6", features = ["all"] }
      tracing               = "0.1"
      tracing-appender      = "0.2"
      tracing-subscriber    = { version = "0.3", features = ["env-filter"] }
    }
    build-dependencies = {
      slint-build = "1.17.0"
    }
  }

  cargo_profiles = {
    scanner  = { release = { lto = true, codegen-units = 1, opt-level = 3, panic = "unwind", overflow-checks = true, strip = true } }
    overflow = { release = { lto = true, codegen-units = 1, opt-level = 3, overflow-checks = true, strip = true } }
    linter   = { release = { lto = true, codegen-units = 1, opt-level = 3, strip = true } }
    bench    = { release = { lto = true, codegen-units = 1, opt-level = 3 } }
    music    = { release = { strip = "symbols", lto = true } }
    fuzz     = { release = { panic = "unwind" }, dev = { panic = "unwind" } }
  }

  cargo_profile_by_dir = {
    "package/forbidden-strings/cli"             = "scanner"
    "package/forbidden-strings/core"       = "scanner"
    "package/nested-wayland-session/cli"        = "overflow"
    "package/linter/rust"                       = "linter"
    "package/forbidden-strings/core.bench" = "bench"
    "package/music-player/truepeak-core.bench"  = "bench"
    "package/music-player/desktop-app"          = "music"
    "package/forbidden-strings/core.fuzz"  = "fuzz"
    "package/forbidden-strings/cli.fuzz"            = "fuzz"
  }
}

toml_keys "cargo_manifests" {
  files    = ["package/*/*/Cargo.toml", "package/*/*/*/Cargo.toml"]
  exclude  = ["**/target/**", "**/node_modules/**"]
  iterator = manifest # manifest.path and manifest.dir, evaluated once per matched file

  # Same order and guards as FEC:1642-1657 and :1741-1759, so new keys land where they land today.
  enforcements = concat(
    [{ guard = ["package"], path = ["package", "edition"], value = "2024" }],
    [for k, v in { license = "LGPL-3.0-or-later", publish = false } : { guard = ["package", k], path = ["package", k], value = v }],
    [for k, v in { disallowed_methods = "deny", implicit_return = "deny", needless_return = "allow" } : { guard = ["lints", "clippy"], path = ["lints", "clippy", k], value = v }],
    flatten([for table, deps in local.cargo_shared_dependencies : [for name, value in deps : { guard = [table, name], path = [table, name], value = value }]]),
    [{ guard = ["package", "repository"], path = ["package", "repository"], value = "https://github.com/Aquaticat/Monochromatic.git" }],
    [{ guard = ["package", "repository"], path = ["package", "readme"], value = "README.md" }],
    [{ guard = ["package", "repository"], path = ["package", "homepage"], value = "https://github.com/Aquaticat/Monochromatic/tree/main/${manifest.dir}" }],
    flatten([
      for profile, keys in lookup(local.cargo_profiles, lookup(local.cargo_profile_by_dir, manifest.dir, ""), {}) :
      [for k, v in keys : { guard = ["profile", profile], path = ["profile", profile, k], value = v }]
    ]),
  )

  insert_if_absent {
    path = ["lints", "clippy"]
    text = file("config/meow/cargo-lints-block.toml")
  }
  insert_if_absent {
    path = ["workspace"]
    text = file("config/meow/cargo-workspace-block.toml")
  }
}
```

The `enforcements` list is today's `CargoManifestPlan` written as an expression.
A data-only format can hold the static entries,
but the homepage and profile entries depend on `manifest.dir`,
so data-only formats need `{dir}` placeholders and a `profile_by_dir` table interpreted by the plugin
(Unverified: proposed semantics, not built).

### LI12 pnpr registry configuration

- Source:
   `FEC:1781-2200`.
- Reads every `package/*/*/package.json`
   (sorted);
   writes `package/config/pnpr/config.yaml` (tracked).
- Logic:
   every manifest needs a string name,
   else an error;
   publishable means not under `package/test-fixture/`,
   a string `version`,
   an entry point per the exports, main, module, and bin rules with TypeScript-source exclusion,
   and not excluded;
   stale exclusions are an error;
   names must be in the scope and use only `a-z0-9-._~`;
   exclusion reasons must be one line;
   the count is logged;
   the YAML is rendered from a template with a loop.
- The same package already parses `config.yaml` at publish time and owns `isTypeScriptSourcePath`
   (Verified: `package/config/pnpr/src/publish-plan.ts:1`, `:186-193`).
- Placement:
   repository task in `package/config/pnpr`,
   a TypeScript program run by Node,
   with the reviewed exclusions,
   identity,
   and origin as data in that package.
- Awkward:
   a task output that is tracked and must stay protected like a managed file;
   the design needs `managed = true` on task outputs.
  The pure-expression alternative works
   (Verified: `DC/tofu/good/entry.tf`),
   but repeats the TypeScript-source predicate inline four times because there are no author-defined functions.

```hcl
# package/config/pnpr/meow.hcl
task "generate_config" {
  command = ["node", "src/generate-config.ts"]
  inputs = [
    "${path.root}/package/*/*/package.json",
    "src/generate-config.ts", "src/publish-plan.ts", "src/pnpr-policy.ts",
  ]
  outputs = ["config.yaml"]
  managed = true
}
```

### LI13 resolved Browserslist targets

- Source:
   `FEC:47-77`, `:598-650`, `:869-910`.
- Reads `.browserslistrc`,
   picks the `production` section or `defaults`,
   and resolves with the installed `browserslist` package using `path: false` and empty stats;
   writes `.browserslistrc.resolved.local.json` (gitignored).
- The consumer `package/config/rolldown/src/browserslist-targets.ts:905-945` reads that file when present,
   and otherwise imports `browserslist` and resolves `undefined` queries with `{ path: process.cwd() }`
   (Verified),
   so the fallback differs in configuration selection.
- Placement:
   repository task in `package/config/rolldown`,
   which already owns the consumer,
   with the lockfile slices of `browserslist` and `caniuse-lite` as inputs.
  Alternative:
   retirement,
   by making the consumer resolve deterministically with the generator's options.
  A built-in on `browserslist-rs` was rejected earlier because its data follows crate releases rather than the pnpm lock
   (Unverified inference carried from `stack-all-rust-rewrite.md`).

```hcl
# package/config/rolldown/meow.hcl
task "resolve_browserslist" {
  command = ["node", "src/resolve-browserslist-targets.ts"]
  inputs = [
    "${path.root}/.browserslistrc", lockfile("browserslist"), lockfile("caniuse-lite"),
    "src/resolve-browserslist-targets.ts",
  ]
  outputs = ["${path.root}/.browserslistrc.resolved.local.json"]
}
```

### LI14 Harper LSP4IJ settings (FE15 plugin)

- Source:
   `FEC:1270-1372` and `FE/jetbrains/lsp4ij*.ts`, `options-dir.ts`.
- Reads and writes `LanguageServersSettings.xml` and `UserDefinedLanguageServerSettings.xml`
   in the latest `IntelliJIdea*` or `IdeaIC*` options directory under `XDG_CONFIG_HOME` or the home directory,
   outside the repository;
   absence of the product,
   files,
   or base server logs a warning and skips
   (Verified: `FE/jetbrains/lsp4ij.ts:82-171`, `options-dir.ts:326-336`).
- Logic in the configuration:
   `harperLintersDisabled` turns rule names into `harper-ls.linters.<rule> = false`,
   and `process.cwd()` builds absolute exclude patterns.
- Placement:
   built-in plugin with plain data;
   `process.cwd()` becomes `path.root`.
- Awkward:
   a repository configuration writing user-scope editor state outside the repository,
   which also sits outside the watched tree and the task sandbox's intent.

```hcl
# meow.hcl
lsp4ij_server "harper" {
  product_prefixes = ["IntelliJIdea", "IdeaIC"]

  base_server {
    command_line_includes = "harper-ls"
    server_name_equals    = "Harper Language Server"
    template_id           = "harper-ls"
  }

  config {
    set = { for rule in ["UseTitleCase", "SplitWords", "PhrasalVerbAsCompoundNoun"] : "harper-ls.linters.${rule}" => false }
    array_union = {
      "harper-ls.excludePatterns" = ["**/AGENTS.md", "**/CLAUDE.md", "${path.root}/AGENTS.md", "${path.root}/CLAUDE.md"]
    }
  }

  schema_defaults = {
    "harper-ls.linters.UseTitleCase" = { type = "boolean", default = true, description = "Prompts you to use title case in relevant headings." }
  }

  scoped_server "harper-ls-agents-claude-md" {
    name             = "Harper Language Server (AGENTS.md and CLAUDE.md)"
    file_names       = ["AGENTS.md", "CLAUDE.md"]
    language_id      = "markdown"
    copy_options = [
      "commandLine", "installAlreadyDone", "installerConfigurationContent", "serverUrl", "templateId", "workingDir",
      "workspaceFolderStrategyConfiguration",
    ]
    config_omit_keys = ["harper-ls.excludePatterns"]
    config {
      set = { for rule in ["MissingTo", "LongSentences", "OxfordComma"] : "harper-ls.linters.${rule}" => false }
    }
  }
}
```

### LI15 skill mirror with ownership manifests

- Source:
   `FEC:186-229`, `:912-1268`.
- Reads `.agents/skills/*/*.md` and each destination's prior `.agents-mirror-manifest.json`;
   validates manifest keys as `.agents/skills/<name>/<file>.md` without traversal or backslashes.
- Writes mirrors under `.claude/skills` and `.factory/skills`
   plus a manifest mapping canonical path to SHA-256
   (`FEC:1102-1106`),
   and deletes only destinations the prior manifest owned.
  Destination-only skills survive by construction
   (`FEC:1239`).
- Both destination roots are gitignored
   (Verified: `.gitignore:124` `/.claude/`, `.gitignore:125` `/.factory/skills/`),
   so a fresh clone has no mirrors to prune.
- Placement:
   general built-in `mirror` with `prune = "ledger"`,
   the ledger kept in the tool's state and hashed with `gxhash128`.
  Precedents for a ledger:
   OpenTofu state,
   Terragrunt's `remove_terragrunt` signature,
   projen's `.projen/files.json`
   ("Precedent").
- Awkward:
   `prune = "owned"` would delete destination-only skills,
   so this unit needs the ledger mode,
   while LI10 needs the glob mode.

```hcl
# meow.hcl
mirror "agent_skills" {
  for_each = toset([".claude/skills", ".factory/skills"])
  from     = ".agents/skills/*/*.md"
  to       = "${each.key}/*/*.md"
  prune    = "ledger"
}
```

### LI16 sequencing and parallelism

- Source:
   `FEC:2202` awaits the `CONTEXT.md` check first;
   `FEC:2204-2329` runs everything else in one `Promise.all`.
- Placement:
   general built-in scheduling.
  Checks and `absent` errors run before writes;
   rule order comes from declared reads and writes
   (LI06 after LI05, LI07 after LI06)
   and explicit `depends_on`;
   unrelated rules run in parallel.
- Awkward:
   FE01 says "with author control over sequencing and parallelism"
   (Verified: `doc/audit/tech-monorepo-manager-vet-2026-09-16.md:97-101`);
   derived ordering replaces author control,
   which follows from the declarative decision but changes FE01's wording.

No declarative form beyond `depends_on`.

### LI17 filesystem error helpers

- Source:
   `errorHasCode`, `unlinkIfExists`, `ENOENT` branches, `SkillMirrorManifestError`
   (`FEC:132-299`, `:412-434`, `:912-948`).
- Placement:
   retirement;
   built-ins own absent-path semantics and diagnostics.

### LI18 logging calls

- Source:
   `tagged` loggers at `FEC:766-771`, `:2060-2063`, `:2142`.
- Placement:
   built-in structured events (FE24);
   nothing to configure.

### LI19 staleness, lazy builders, and watch registration

- Source:
   `addWatchedPaths` at `FEC:579`,
   and the library's staleness manifest and lazy builders (FE08, FE09).
- Placement:
   built-in daemon behavior;
   inputs come from declarations and function calls,
   so nothing is registered by hand.

### LI20 trailing Oxlint comment

- Source:
   `FEC:2327-2328`,
   a comment with no code.
- Placement:
   retirement.

### LI21 module imports

- Source:
   `FEC:1-33`, `:622`.
- Placement:
   retirement.
  `isTypeScriptSourcePath` moves with LI12 into its own package,
   `browserslist` with LI13,
   `nano-spawn` with LI07,
   and `node:` modules with the built-ins.

### LI22 platform executable suffix

- Source:
   `FEC:182-184`.
- Placement:
   retirement;
   0.x is Linux only,
   and LI07 names the build task instead of a platform path.

### Plain data units

- LD01 Cargo canonical values, shared dependencies, profile presets, and directory map
   (`FEC:1374-1657`):
   plain data in the root configuration,
   shown in LI11.
- LD02 pnpr identity, origin, scope, fixture prefix, name characters, and reviewed exclusions with reasons
   (`FEC:1783-1889`):
   plain data in `package/config/pnpr`,
   read by the LI12 task.
- LD03 license text sources, manifest globs, and the LGPL-implies-GPL rule
   (`FEC:79-130`):
   plain data,
   shown in LI10.
- LD04 Harper LSP4IJ policy values
   (`FEC:1306-1372`):
   plain data,
   shown in LI14.
- LD05 fixed paths, modes, mirror roots, and message texts
   (`FEC:35-229`, `:1434-1453`, and the text blocks at `:660-717`, `:814-829`, `:842-849`):
   plain data or checked-in text files.

### Classification counts

Primary placement of the 27 units:

- General built-in features:
   13
   (LI01 `absent`, LI02, LI03, LI05, LI06 `file`, LI09 and LI15 `mirror`, LI10 license texts,
   LI11 `toml_keys`, LI14 `lsp4ij_server`, LI16 scheduling, LI18 events, LI19 daemon tracking).
- Repository tasks:
   3
   (LI07 scanner compile, LI12 pnpr configuration, LI13 Browserslist).
- Plain data:
   5
   (LD01 to LD05).
- Retirement:
   6
   (LI04 with Mise, LI08, LI17, LI20, LI21, LI22).

### Forced or awkward classifications

- LI04:
   retirement is blocked while Mise stays on macOS and Windows CI;
   the interim form needs a directory listing and list-to-text assembly,
   which only an expression language writes without a bespoke built-in.
- LI07:
   output outside the repository,
   and "skip when absent" becomes "build first".
- LI10:
   composed or dedicated built-in;
   substring semantics versus SPDX parsing;
   glob pruning is broader than today's pruning.
- LI11:
   per-manifest derivations need an iterator or a placeholder microsyntax.
- LI12:
   a task whose committed output must be protected;
   the expression form is possible but repeats a predicate.
- LI13:
   task versus retirement,
   and the consumer fallback resolves differently today.
- LI14:
   writes outside the repository.
- LI15:
   needs ledger pruning,
   not glob pruning.
- LI16:
   author-controlled ordering in FE01 is replaced by derived ordering.
- LI03:
   header wording and preamble file location.

## 2. Precedent

### OpenTofu, the lead precedent

Sources:
`AG/opentofu-2026-09-16/website/docs/`,
`AG/opentofu-2026-09-16/internal/`,
`AG/hcl-2026-09-16`,
`AG/tofu-ls-2026-09-16`,
and the probes under `DC/tofu/`.

#### Two syntaxes over one model

- HCL "has both a _native syntax_, intended to be pleasant to read and write for humans,
   and a JSON-based variant that is easier for machines to generate and parse"
   (Verified: `AG/hcl-2026-09-16/README.md:13-15`).
- OpenTofu's JSON syntax "is useful when generating portions of a configuration programmatically"
   and "Everything that can be expressed in native syntax can also be expressed in JSON syntax"
   (Verified: `website/docs/language/syntax/json.mdx`, opening section).
- Comments in JSON syntax use a property named `"//"`,
   "ignored by OpenTofu entirely" in block bodies,
   while the docs "do not recommend hand-editing of JSON syntax configuration files"
   (Verified: `syntax/json.mdx:270-294`).
  The probe planned a `.tf.json` with `"//"` and `"${local.count + 1}"`
   (Verified, probe).
- File extensions `.tf`, `.tofu`, `.tf.json`, and `.tofu.json`,
   with `.tofu` taking precedence over a same-named `.tf`
   (Verified: `language/files/index.mdx:12-27`).

#### Blocks and attributes

- "An _argument_ assigns a value to a particular name",
   and a block is a container with a type and labels
   (Verified: `syntax/configuration.mdx:29-63`).
- HCL leaves the schema to the application:
   "The application defines which attribute names and nested block types are expected,
   and HCL parses the configuration file,
   verifies that it conforms to the expected structure"
   (Verified: `AG/hcl-2026-09-16/README.md:25-29`).
- Unknown arguments produce the diagnostic "Unsupported argument"
   (Verified: `AG/hcl-2026-09-16/hclsyntax/structure.go:87`),
   and HCL suggests near names with a Levenshtein distance under 3
   (Verified: `AG/hcl-2026-09-16/didyoumean.go:19-25`).

#### Expressions and functions

- "A _function_ is an operation that has been assigned a symbolic name.
   Functions are made available for use in expressions by the calling application,
   by populating the _function table_"
   (Verified: `AG/hcl-2026-09-16/hclsyntax/spec.md:401-403`).
  The HCL language itself defines no functions.
- OpenTofu documents 124 built-in functions
   (Verified: 125 files in `website/docs/language/functions/`, one of them `index.mdx`).
- Author-defined functions:
  - Stable OpenTofu has none;
     the functions page lists built-in functions and provider-defined functions only
     (Verified: `language/functions/index.mdx:1-60`).
  - Providers,
     which are separate plugin binaries,
     may register functions under `provider::<provider_name>::<function_name>`,
     and the notes for provider authors point to "the experimental Lua and Go providers"
     (Verified: `language/functions/index.mdx:36-71`).
  - OpenTofu 1.13 adds experimental Symbol Libraries in `*.sym.hcl` files with `function`,
     `typedef`,
     and `values` blocks,
     "subject to *significant* changes even in patch releases"
     (Verified: `language/symbol-libraries/index.mdx`, and the v1.13.0-beta1 `CHANGELOG.md:32-34` via `gh api`).
  - Those functions reject recursion at call time:
     "Recursive call to %s detected, call stack: %s"
     (Verified: `internal/configs/symlib/functions.go:255-259`).
    So OpenTofu keeps evaluation terminating even when it adds author-defined functions.
  - HCL ships an optional `userfunc` extension for applications that want `function` blocks
     (Verified: `AG/hcl-2026-09-16/ext/userfunc/README.md:1-16`).
- `templatefile` may recurse,
   with "a limited call depth (1024 by default)"
   (Verified: `language/functions/templatefile.mdx:44-50`).

#### Variables, locals, outputs

- `variable` blocks take `type`,
   `default`,
   `description`,
   and `validation` blocks with `condition` and `error_message`
   (Verified: `language/values/variables.mdx:84`, `:163-176`).
- The probe's failing validation printed the message and "This was checked by the validation rule at main.tf:4,3-13."
   (Verified, probe).
- Local values,
   "if overused they can also make a configuration hard to read by future maintainers by hiding the actual values"
   (Verified: `language/values/locals.mdx:81-82`).
- Outputs support `sensitive` to suppress values in CLI output
   (Verified: `language/values/outputs.mdx:87`, `:110-120`).

#### Modules and composition across directories

- "A _module_ is a collection of one or many `.tf`, `.tf.json`, `.tofu`, `.tofu.json` files kept together
   in a directory",
   "nested directories are treated as completely separate modules",
   and splitting blocks across files "has no effect on the module's behavior"
   (Verified: `language/files/index.mdx:37-50`).
- Child modules are included explicitly with module calls,
   from local directories or registries
   (Verified: `language/files/index.mdx:52-56`).
- Override files named `*_override.tf` merge into existing objects,
   processed "in lexicographical order";
   otherwise two files defining the same object is an error
   (Verified: `language/files/override.mdx:9-40`).

#### Repetition: `for_each`, `count`, `dynamic`

- "The `for_each` meta-argument accepts a map or a set of strings",
   exposing `each.key` and `each.value`
   (Verified: `language/meta-arguments/for_each.mdx:31-54`).
- HCL implements `dynamic` as an extension,
   used "to dynamically generate blocks of other types by iterating over collection values"
   (Verified: `AG/hcl-2026-09-16/ext/dynblock/README.md:1-12`).
- OpenTofu's guidance:
   "Overuse of `dynamic` blocks can make configuration hard to read and maintain ...
   Always write nested blocks out literally where possible"
   (Verified: `language/expressions/dynamic-blocks.mdx:142-146`).

#### Type constraints, validation, checks

- The repository's own OpenTofu module uses `type = map(string)`,
   `list(number)`,
   `check` blocks with `assert`,
   `for_each`,
   `dynamic "rule"`,
   `moved`,
   and `data "external"`
   (Verified: `package/config/tofu/hetzner.tf:1-120`, `:525-540`, `:900-941`).
- "check blocks do _not_ affect OpenTofu's execution of an operation",
   while a validation block halts plan or apply
   (Verified: `language/checks/index.mdx:75-85`).

#### Diagnostics with source ranges

- `tofu validate -json` reports `range.filename`,
   `start` and `end` line,
   column,
   and byte,
   plus the source snippet
   (Verified, probe in "Probes run").

#### Formatting

- "The `tofu fmt` command is used to rewrite OpenTofu configuration files to a canonical format and style",
   it "is intentionally opinionated and has no customization options",
   and new formatting rules are not considered breaking
   (Verified: `website/docs/cli/commands/fmt.mdx:7-33`).
- The probe kept comments and aligned `=` signs
   (Verified, probe).

#### Language server

- `tofu-ls` is "[WIP]",
   derived from `terraform-ls`
   (Verified: `AG/tofu-ls-2026-09-16/README.md:1-45`),
   and depends on `github.com/hashicorp/hcl-lang` replaced by an OpenTofu fork
   (Verified: `AG/tofu-ls-2026-09-16/go.mod:5`, `:16`).
  Its completion and hover come from a Go schema,
   not from JSON Schema
   (Unverified: inferred from the `hcl-lang` dependency and the feature matrix; the schema code was not read).

#### Where bespoke logic lives

- Providers are separate plugins speaking a provider protocol;
   they supply resources,
   data sources,
   and functions
   (Verified: `language/functions/index.mdx:36-71`).
- The external data source is the documented escape hatch:
   "This mechanism is provided as an 'escape hatch' for exceptional situations
   where a first-class Terraform provider is not more appropriate"
   (Verified: `hashicorp/terraform-provider-external` `docs/data-sources/external.md:13`, via `gh api`).
  The program receives a JSON object of strings and must print one
   (Verified: same file, lines 38-44).
- This repository uses that escape hatch four times,
   running TypeScript programs with `bun run`
   (Verified: `package/config/tofu/hetzner.tf:81-100`, `:284-285`).

#### Versioning the language

- The `language` block holds `compatible_with { opentofu = ">= 1.12" }`,
   an `edition` such as `tofu2024`,
   and `experiments`
   (Verified: `language/settings/index.mdx:40-75`).
- Linting is experimental and opt-in through `-lint`
   (Verified: `language/linting/index.mdx:9-20`).

#### Lessons for meow

- One model,
   two syntaxes,
   with JSON reserved for machine generation.
- The application owns the schema and the function table;
   configuration authors get expressions but no functions of their own in the stable language.
- Bespoke logic goes to separately built plugins or to an explicitly labeled escape hatch,
   not into the configuration language.
- When OpenTofu does add author functions,
   it blocks recursion.
- A directory is one document;
   composition is explicit.
- Checks and validations are first-class and carry source ranges.
- The formatter is opinionated and not configurable.

### HCL in Rust

- `hcl-rs` 0.19.8 parses through `hcl-edit`
   (`pub use hcl_edit as edit`,
   Verified: `AG/hcl-rs-2026-09-16/crates/hcl-rs/src/lib.rs:42`),
   deserializes with `serde` "according to the HCL JSON Specification",
   formats,
   and evaluates expressions and templates
   (Verified: `crates/hcl-rs/README.md:13-27`, `src/eval/mod.rs:1-60`).
- It has no built-in function library;
   functions are declared on `hcl::eval::Context`
   (Verified: `src/eval/mod.rs`, "Function calls in expressions").
  Issue #484 "Provide built-in functions" is open;
   the maintainer wrote that these functions "are specific to Terraform"
   and could live in an auxiliary crate
   (Verified: `gh issue view 484`).
- Errors lose source locations:
   the deserializer printed `invalid type: string "many", expected u32` with no line,
   and the evaluator printed ``eval error: undefined function `lower` in expression `lower("A")` ``
   (Verified, `DC/sizes/out/hcl.run.txt`).
  moon's HCL loader passes `span: None` in all three of its error paths
   (Verified: `AG/moon-2026-09-16/crates/config-loader/src/formats/hcl.rs:66-93`).
- `hcl-edit` 0.9.7 "Parse[s] and modify HCL documents while preserving whitespace and comments"
   but warns "**Expect breaking changes at any time**"
   (Verified: `crates/hcl-edit/README.md:9-21`).
  It has no formatter module
   (Verified: `ls crates/hcl-edit/src`),
   no `dynamic` block expansion appears in either crate
   (Verified: `rg --ignore-case 'dynamic|for_each|dynblock'` found only iterator calls),
   and no parser for the HCL JSON syntax was found
   (Unverified: `rg` for JSON in `hcl-rs/src` found only the serde mapping).
- Round trip:
   byte-identical on `hetzner.tf`,
   but it joined a multi-line `&&` condition onto one line in `DC/tofu/good/main.tf`
   (Verified, probe):

```diff
# DC/roundtrip: diff DC/tofu/good/main.tf main.roundtrip.tf
48,51c48
<     if !startswith(p, "package/test-fixture/")
<     && can(m.version)
<     && length([for k in keys(try(m.exports, {})) : k if k != "./ts" && !startswith(k, "./ts/")]) > 0
<     && !contains(keys(var.pnpr_excluded), m.name)
---
>     if !startswith(p, "package/test-fixture/") && can(m.version) && length([for k in keys(try(m.exports, {})) : k if k != "./ts" && !startswith(k, "./ts/")]) > 0 && !contains(keys(var.pnpr_excluded), m.name)
```

- Maintenance:
   both crates 6 releases in the past year,
   185 stars,
   11 open issues,
   pushed 2026-09-05
   (Verified, `DC/meta/crates-meta.json`).
- `SamuelMarks/hashicorp-configuration-language-rs`,
   created 2026-09-09 with 0 stars,
   claims an HCL2 and `cty` reimplementation with "70+ Functions"
   (Verified: `gh api`);
   too new to evaluate.
- Editors:
   no `JsonSchemaEnabler` for HCL was found in `JetBrains/intellij-plugins`,
   where GitHub code search returned only ESLint and Prettier enablers
   (Verified: `gh api search/code`, subject to GitHub's search coverage).

### Terragrunt `generate` blocks

- A `generate` block writes a file with `path`,
   `contents`,
   and `if_exists` of `overwrite`,
   `overwrite_terragrunt` ("overwrite the existing file if it was generated by terragrunt; otherwise, error"),
   `skip`,
   or `error`;
   `if_disabled` supports `remove_terragrunt`;
   a signature in a `comment_prefix` comment marks ownership
   (Verified: `AG/terragrunt-2026-09-16/docs/src/content/docs/04-reference/01-hcl/02-blocks.mdx:1466-1497`).
- Lesson:
   HCL-based file generation with ownership-aware overwrite and removal already exists,
   and `if_exists = "skip"` is LI05's create-only write.

### Bazel

- "Recursion is not allowed.",
   top-level `for` and `if` statements are not allowed,
   and `BUILD` files cannot declare functions
   (Verified: `AG/bazel-2026-09-16/docs/rules/language.mdx:113-130`).
- Starlark "is not Turing-complete,
   which discourages some (but not all) users from trying to accomplish general programming tasks within the language"
   (Verified: `docs/contribute/codebase.mdx:427-431`).
- Buildfiles are "a declarative manifest describing a set of artifacts to build" rather than
   "an imperative set of commands in a Turing-complete scripting language"
   (Verified: `docs/basics/artifact-based-builds.mdx:15-20`).
- Lesson:
   the restriction is enforced by the interpreter;
   starlark-rust does not enforce it by default ("Starlark" in "3. Configuration formats").

### Buck2

- BXL "is a Starlark-based script that enables integrators to inspect and interact with the Buck2 graph",
   with actions and dynamic outputs
   (Verified: `AG/buck2-2026-09-16/docs/bxl/index.md:6-26`).
- Lesson:
   graph introspection and generation get a separate extension language instead of widening build files.

### Pants

- "BUILD files are very hermetic in nature with no support for using `import` or other I/O operations"
   (Verified: `AG/pants-2026-09-16/docs/docs/using-pants/key-concepts/targets-and-build-files.mdx:50`).
- Macros "cannot import other modules",
   and anything "more complex than a macro allows" becomes a new target type in a plugin
   (Verified: `docs/docs/writing-plugins/macros.mdx:9-33`).
- Options live in `pants.toml`
   (Verified: `docs/docs/using-pants/key-concepts/options.mdx:24`).
- Lesson:
   data for options in TOML,
   restricted declarations,
   logic in plugins.

### Nx

- Tasks come from `package.json` scripts,
   `project.json` targets,
   or plugins that "read the tool configuration and create tasks for you"
   (Verified: `AG/nx-2026-09-16/astro-docs/src/content/docs/concepts/mental-model.mdoc:26-37`).
- Sync generators "ensure that your repository is maintained in a correct state",
   run in dry-run mode in CI where out-of-sync files fail the task
   (Verified: `concepts/sync-generators.mdoc:9-38`).
- Lesson:
   file-enforcer-style synchronization is a named feature there,
   implemented as plugin code in TypeScript,
   and tasks are inferred from native tool configuration.

### Turborepo

- Package `turbo.json` files extend the root with `"extends": ["//"]`
   (Verified: `AG/turborepo-2026-09-16/apps/docs/content/docs/reference/package-configurations.mdx:20-41`).
- `turbo.jsonc` adds comments "with IDE support"
   (Verified: `reference/configuration.mdx:18`).
- `"dependsOn": ["^build"]` runs the task in dependencies first
   (Verified: `crafting-your-repository/configuring-tasks.mdx:66-78`);
   `$TURBO_DEFAULT$` and `$TURBO_EXTENDS$` are string microsyntax inside JSON values
   (Verified: same file, `:211-219`, `:291`).
- Custom generators are Plop configurations in `turbo/generators/config.ts`
   (Verified: `guides/generating-code.mdx:51-82`).
- Lesson:
   root-plus-package composition by `extends`,
   microsyntax where JSON has no expressions,
   and code generation in a separate TypeScript escape hatch.

### moon

- moon 2.0 reads JSON,
   JSONC,
   HCL,
   Pkl,
   TOML,
   and YAML,
   and validates all of them with JSON Schema through a `$schema` property pointing into `.moon/cache/schemas`
   (Verified: `AG/moon-2026-09-16/website/docs/config/overview.mdx:10-88`).
- Its HCL support evaluates only `locals` blocks and one `concat` function,
   then deserializes
   (Verified: `crates/config-loader/src/formats/hcl.rs:16-56`).
- Pkl "must exist in the environment" as the `pkl` binary
   (Verified: `overview.mdx:100-104`).
- Extensions are WASM plugins with "whitelisted access to the file system"
   (Verified: `website/docs/guides/extensions.mdx:9-13`);
   code generation templates use Tera
   (Verified: `website/docs/guides/codegen.mdx:68-102`).
- moon was excluded from the market vet for documentation
   (Verified: `doc/planning/mise-removal-coverage.md:46-48`);
   it is cited here only as a design precedent.
- Lesson:
   one schema,
   many syntaxes;
   escape hatches as sandboxed WASM.

### mise

- Configuration values use Tera expressions
   (Verified: `AG/mise-2026-09-16/docs/templates.md:2-7`, `:45`),
   including `exec(command) -> String`,
   which "Runs a shell command and returns its output as a string"
   (Verified: `docs/templates.md:302`).
- Tasks declare `sources` and `outputs` for up-to-date checks
   (Verified: `docs/tasks/task-configuration.md:467-509`),
   and file tasks carry `#MISE` comment headers
   (Verified: `docs/tasks/file-tasks.md:48`).
- mise itself still depends on `serde_yaml = "0.9"`,
   `taplo`,
   `tera` 1 and 2,
   and `toml_edit`
   (Verified: `gh api repos/jdx/mise/contents/Cargo.toml`).
- Lesson:
   a template engine inside configuration values brought arbitrary command execution into the configuration.

### Taskfile

- Templating is Go `text/template` with slim-sprig functions
   (Verified: `AG/task-2026-09-16/website/src/latest/docs/reference/templating.md:4-15`).
- A variable with `sh:` "is considered a dynamic variable",
   evaluated by running a shell command
   (Verified: `website/src/latest/docs/guide.md:1558-1560`).
- `sources` and `generates` drive checksum-based skipping
   (Verified: `guide.md:830-870`).
- Lesson:
   declarative YAML with a shell escape hatch inside values.

### just

- "`just` is a command runner,
   not a build system"
   (Verified: `AG/just-2026-09-16/README.md:50`).
- Backticks evaluate commands into values
   (Verified: `README.md:2434-2436`),
   and user-defined functions are "currently unstable"
   (Verified: `README.md:2497-2510`).
- Lesson:
   even a deliberately small language grew functions,
   and shell evaluation is its escape hatch.

### dprint

- "Wasm plugins are compiled to a `.wasm` file and run sandboxed.
   Process plugins are compiled to an executable file and do NOT run sandboxed",
   and process plugins require a checksum
   (Verified: `AG/dprint-2026-09-16/website/src/plugins.md:11-16`).
- Configurations `extends` files and URLs
   (Verified: `website/src/config.md:405`).
- Lesson:
   JSONC configuration,
   logic in versioned,
   checksummed plugins.

### lefthook

- Accepts YAML,
   TOML,
   JSON,
   and JSONC,
   and merges a `lefthook-local` file
   (Verified: `AG/lefthook-2026-09-16/docs/configuration.md:7-23`).
- `scripts` run through a declared `runner`
   (Verified: `docs/configuration/runner.md:5-20`).
- Lesson:
   bespoke logic as script files with a declared runner.

### pre-commit

- Repository-local hooks use `repo: local` with an `entry` command,
   useful when "The scripts are tightly coupled to the repository"
   (Verified: `AG/pre-commit.com-2026-09-16/sections/advanced.md:268-300`).
- Lesson:
   the configuration names programs;
   it does not contain them.

### Renovate presets

- Presets are JSON or JSON5,
   and `.jsonc` is recommended for comments
   (Verified: `AG/renovate-2026-09-16/docs/usage/configuration-options.md:30`, `:44-47`).
- Custom managers use `regex` or JSONata queries
   (Verified: `configuration-options.md:1068-1108`),
   and `postUpgradeTasks` run allowlisted commands on self-hosted instances
   (Verified: `docs/usage/self-hosted-configuration.md:35-63`).
- `configMigration` rewrites configurations,
   with the warning "Renovate may downgrade JSON5 content to plain JSON ...
   Renovate may also remove the JSON5 comments"
   (Verified: `configuration-options.md:879-903`).
- Lesson:
   an automated configuration migration must use a comment-preserving editor.

### syncpack

- Reads `.syncpackrc` as JSON,
   YAML,
   JavaScript,
   or TypeScript;
   "JSON is preferred as it is the fastest,
   without the overhead of calling out to Node.js and TypeScript"
   (Verified: `AG/syncpack-2026-09-16/site/src/content/docs/config/syncpackrc.mdx`).
- `versionGroups` are ordered,
   first match wins,
   and a `pinned` group sets `pinVersion` for listed dependencies
   (Verified: `site/src/content/docs/version-groups.mdx`, `version-groups/pinned.mdx`).
- Lesson:
   a declarative dependency-version policy,
   the same shape as `CARGO_SHARED_DEPENDENCIES`.

### copier

- Jinja extensions,
   migrations,
   and tasks "allow arbitrary code execution"
   and are refused unless `--trust` or `--UNSAFE` is passed
   (Verified: `AG/copier-2026-09-16/docs/configuring.md:1830-1845`).
- Lesson:
   the escape hatch is explicit and gated.

### Cargo build scripts

- Build scripts write to `OUT_DIR` and "should not modify any files outside of that directory"
   (Verified: `AG/cargo-2026-09-16/doc/book/src/reference/build-scripts.md:84-89`).
- `cargo::rerun-if-changed=PATH` declares inputs,
   compared by "mtime"
   (Verified: `build-scripts.md:409-415`).
- Lesson:
   manifests stay data;
   logic is an ordinary program with declared inputs and a confined output directory.

### projen

- Managed files are "marked as read-only on the file system",
   carry a "Generated by projen" header,
   and are recorded in `.projen/files.json`
   (Verified: `AG/projen-2026-09-16/docs/introduction/the-projen-workflow.md:30-34`,
   `docs/custom/custom-components.md:102-105`).
- Lesson:
   an ownership ledger for generated files,
   the analog of LI15's manifests.

### Cross-cutting lessons

- Every surveyed tool keeps bespoke logic outside its declarative files:
   providers and external programs (OpenTofu),
   plugins (Pants, Nx, dprint, moon),
   build scripts (Cargo),
   or script entries (pre-commit, lefthook).
- Where a tool put a template engine or shell evaluation into values
   (mise `exec`, Taskfile `sh:`, just backticks, copier Jinja extensions),
   the configuration regained arbitrary execution.
- Ownership of generated files is tracked by signature (Terragrunt),
   ledger (projen, OpenTofu state),
   or glob.
- Package-level configuration composes by `extends` (Turborepo, lefthook, dprint, Renovate)
   or by directory modules (OpenTofu).

## 3. Configuration formats

Maintenance figures are from `DC/meta/crates-meta.json`
(Verified, fetched 2026-09-16).
Sizes are the isolated static musl increases from "Probes run";
the daemon skeleton already links `serde_json` and `toml_edit`
(Verified: `stack-all-rust-rewrite.md:168-182`),
so their real increase is smaller
(Unverified: shared crates were not measured inside the skeleton).
No format crate except Starlark needs a C compiler
(Verified: `DC/sizes/cc-check.ts`),
so aarch64 static builds need only the Rust target
(Unverified: no aarch64 build).

### Repository incumbents

- TOML:
   `mise.no-env.toml` (1,240 lines),
   178 package `mise.toml` files,
   19 tracked `Cargo.toml` files,
   and `package/rust-module/rust-linter-core`,
   whose `rust-linter.toml` loader has `extends` chains with cycle detection,
   upward discovery collecting "outermost first",
   `[[overrides]]`,
   and `#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]`
   (Verified: `wc`, `rg --files`, `git ls-files`,
   `src/config/load.rs:24`, `:148`, `:197`,
   `src/config/file.rs:35`, `:65`).
  `package/toml-edit/core` is the TypeScript comment-preserving editor.
- JSON and JSONC:
   `package/jsonc-edit/core-ts` (TypeScript, hand-written parser, comments as data),
   `package/config/dprint/index.json` with `"$schema": "https://dprint.dev/schemas/v0.json"`,
   `node.config.json`,
   and `package/config/typescript/tsconfig.options.json`
   (Verified: `rg '"\$schema"'` and `package/jsonc-edit/core-ts/README.md`).
- YAML:
   `pnpm-workspace.yaml`,
   15 GitHub workflow files,
   the generated `package/config/pnpr/config.yaml`,
   and the `yaml` npm package in `config-pnpr`
   (Verified: `rg --files`, `publish-plan.ts:1`).
- HCL:
   `package/config/tofu/hetzner.tf` (941 lines) and `.terraform.lock.hcl`,
   run with Mise-provided OpenTofu 1.12.6
   (Verified: `wc`, `ls`, `mise ls`).
- KDL, RON, Dhall, Starlark, KCL, CEL, Rego, Pkl:
   no files or dependencies found
   (Verified: `rg --files` for their extensions and `rg` over `Cargo.toml` files returned nothing).
- JSON Schema generation:
   `@valibot/to-json-schema` in `package/mcp/stdio`
   (Verified: `package/mcp/stdio/src/tool-schema.ts:7`);
   no Rust schema generator is in use.

### TOML

- Libraries:
   `toml` 1.1.6 (21 releases in the past year) for `serde`,
   `toml_edit` 0.25.15 (22) for lossless editing,
   `serde_spanned` for spans,
   `taplo` 0.14.0 and `taplo-lsp` 0.8.0 (0 releases, 240 open issues) for formatting and a language server.
- Diagnostics:
   type and syntax errors carry line,
   column,
   and a caret snippet:
   "TOML parse error at line 2, column 8 ... invalid type: string \"many\", expected u32"
   and "unclosed array table, expected `]`"
   (Verified, probe).
  Unknown keys pass silently without `deny_unknown_fields`
   (Verified, probe: "toml unknown key accepted without deny_unknown_fields: true").
- Schema and editors:
   taplo reads `#:schema ./foo-schema.json` directives and a root `$schema` key
   (Verified: `AG/taplo-2026-09-16/site/site/configuration/directives.md:16`, `using-schemas.md:7`);
   JetBrains' TOML plugin registers a `JsonSchemaEnabler` behind the registry key `org.toml.json.schema`,
   `defaultValue="true"`
   (Verified: `JetBrains/intellij-community` `plugins/toml/core/src/main/resources/intellij.toml.core.xml:36`,
   `plugins/toml/json/src/main/kotlin/org/toml/ide/json/TomlJsonSchemaEnabler.kt`, via `gh api`).
- Comments and write-back:
   `toml_edit` kept a header comment,
   an inner comment,
   and a trailing comment when `version` changed
   (Verified, probe),
   and round-tripped `mise.no-env.toml` and a `Cargo.toml` byte for byte
   (Verified, probe).
  The earlier research found `Table::insert` deletes a comment above a replaced key
   (Verified there: `toml_edit` `src/table.rs:429-443`),
   so writers must replace through `get_mut`.
- Composition:
   no include syntax;
   composition is an application convention,
   such as `extends` in `rust-linter.toml` or Cargo's `workspace = true` inheritance.
- Per-package:
   a package `meow.toml` extends the root's templates,
   after Turborepo's `extends: ["//"]` and mise's `task_templates` with `extends`
   (Verified: `mise.no-env.toml:510-600`, package `mise.toml` files use `extends = "build"`).
- Size:
   `toml` +208,896 bytes;
   `toml_edit` +225,280 bytes.
- Turing status:
   data only.

### KDL

- Libraries:
   `kdl` 6.7.1 (4 releases, 517 stars),
   which "preserves formatting when editing"
   and supports KDL v2 and v1
   (Verified: `AG/kdl-rs-2026-09-16/README.md:1-20`);
   typed decoding through `knus` 3.4.0
   (1 release, 22 open issues; the maintainer "may not have much time to fix issues")
   or `facet-kdl` 0.42.0 (302 recent downloads)
   (Verified: `gh api repos/TheLostLambda/knus/readme`, crates.io metadata).
  Zellij uses `kdl` 4.5 with the `span` feature
   (Verified: `gh api repos/zellij-org/zellij/contents/zellij-utils/Cargo.toml`).
- Diagnostics:
   `kdl` returns `KdlDiagnostic` values with `SourceSpan` offsets and labels such as "No closing '}' for child block"
   (Verified, probe);
   `knus` rendered a miette snippet at the right span,
   but with the text "expected string scalar, found string" for a string given to an integer field
   (Verified, probe).
- Schema and editors:
   `kdl-lsp` "only supports diagnostics";
   KDL Schema support is future work
   (Verified: `AG/kdl-rs-2026-09-16/tools/kdl-lsp/README.md:1-7`).
  No JSON Schema route exists
   (Unverified: no editor integration found in this session).
- Comments and write-back:
   the probe changed `version 1` to `version 2` and kept both comments
   (Verified, probe).
- Composition:
   none in the language;
   application convention.
- Per-package:
   as TOML.
- Size:
   `kdl` +303,104 bytes;
   `knus` with `miette` +770,144 bytes.
- Turing status:
   data only.
- Fit:
   node arguments and properties are scalars,
   so nested TOML values such as `{ version = "4", features = ["derive"] }` become child nodes
   (Unverified: modeled, not probed).

### JSON

- Library:
   `serde_json` 1.0.151 (6 releases).
- Diagnostics:
   "invalid type: string \"many\", expected u32 at line 3 column 16";
   a comment fails with "key must be a string at line 3 column 3"
   (Verified, probe).
- Schema and editors:
   JSON Schema everywhere.
- Comments:
   none,
   so as a hand-edited format it is out;
   OpenTofu keeps its JSON variant for generated files
   ("Precedent").
- Size:
   +65,536 bytes.

### JSONC

- Library:
   `jsonc-parser` 0.33.2 (14 releases, dprint's author, 1 open issue),
   with a `cst` feature for comment-preserving manipulation and a `serde` feature
   (Verified: `AG/jsonc-parser-2026-09-16/Cargo.toml:20-27`, `src/lib.rs` crate docs).
- Diagnostics:
   "invalid type: string \"many\", expected u32 on line 3 column 11" and "Unexpected close brace on line 4 column 1",
   without a snippet
   (Verified, probe).
- Schema and editors:
   JSON Schema through `$schema` in JetBrains and other editors
   (Unverified for JSONC specifically:
   Turborepo documents "IDE support" for `turbo.jsonc`, Verified,
   but no editor source was read).
- Comments and write-back:
   the CST changed `"version": 1` to `2` and kept the comment and trailing commas
   (Verified, probe),
   and round-tripped `dprint/index.json` and `tsconfig.options.json`
   (Verified, probe).
- Composition:
   application `extends` (Turborepo, dprint, Renovate).
- Size:
   +180,224 bytes.
- Fit:
   no multi-line strings,
   so LI03, LI05, and LI06 texts must be separate files.

### JSON5

- Libraries:
   `json5` 1.3.1 (5 releases);
   `json5format` 0.2.6 preserves comments but last released 2022-06-17;
   `serde_json5` is archived
   (Verified, metadata).
- Diagnostics:
   "expected number at line 3 column 9"
   (Verified, probe).
- Write-back:
   Renovate warns its migration "may also remove the JSON5 comments"
   (Verified: "Renovate presets").
- Size:
   +110,592 bytes.
- Result:
   folded into JSONC;
   no maintained comment-preserving editor.

### YAML

- Libraries and maintenance:
   `serde_yaml` 0.9.34 is marked `+deprecated` and archived;
   `serde_yml` 0.0.13 is archived;
   `serde_norway` 0.9.42 has 0 releases in the past year;
   `serde-saphyr` 1.3.0 has 35 releases and 221 stars;
   `saphyr` 0.0.12 and `yaml-rust2` 0.13.0 are parsers;
   `yaml-edit` 0.3.1 is a rowan-based lossless editor with 10 stars
   (Verified, metadata).
- Diagnostics:
   `serde-saphyr` printed snippets such as
   "error: line 2 column 7: invalid u32 ... 2 | jobs: many ... ^ invalid u32",
   and rejected `jobs: 010` for a `u32`
   (Verified, probe).
- Schema and editors:
   JetBrains' YAML plugin registers `YamlJsonEnabler`
   (Verified: `JetBrains/intellij-community` `plugins/yaml/backend/src/schema/YamlJsonEnabler.java`,
   via `gh api search/code`).
- Comments and write-back:
   `yaml-edit` kept comments when changing a value,
   but `Document::from_str` refused a leading comment with
   "Input contains stream-level comments outside the document. Use YamlFile::from_str() to preserve them."
   (Verified, probe);
   `YamlFile` round-tripped three repository YAML files
   (Verified, probe).
- Composition:
   anchors and aliases inside one file;
   includes are application convention (lefthook `extends` and remotes, moon `extends`).
- Size:
   `serde-saphyr` +942,216 bytes;
   `yaml-edit` +208,896 bytes.
- Turing status:
   data only.

### RON

- Library:
   `ron` 0.12.2 (3 releases, 4,016 stars).
- Diagnostics:
   "3:14: Expected integer"
   (Verified, probe).
- Schema, editors, write-back:
   no JSON Schema route and no comment-preserving editor found
   (Unverified: not searched beyond crates.io metadata).
- Size:
   +163,840 bytes.
- Result:
   excluded before design;
   Rust-specific syntax and no editor path for other repositories.

### HCL

Covered in depth under "HCL in Rust" in "2. Precedent".

- Libraries:
   `hcl-edit` 0.9.7 and `hcl-rs` 0.19.8 (6 releases each, one maintainer).
- Diagnostics:
   syntax errors carry position and snippet
   ("HCL parse error in line 2, column 10 ... expected `]`");
   deserialization and evaluation errors carry none
   (Verified, probe).
- Schema and editors:
   no JSON Schema route;
   a completion-capable language server would be repository work
   (Unverified inference from "HCL in Rust").
- Comments and write-back:
   byte-identical on `hetzner.tf`,
   one defect on a multi-line condition
   (Verified, probe).
- Composition:
   directory modules and explicit module calls (OpenTofu), implemented by the application.
- Size:
   `hcl-edit` +344,064 bytes;
   `hcl-rs` with evaluator +1,052,776 bytes.
- Turing status:
   expressions without author functions terminate;
   recursion enters only through functions the application adds,
   as OpenTofu's `templatefile` and Symbol Libraries show
   ("Precedent").

### KCL

- Implementation:
   Rust workspace `kcl-lang/kcl` (2,414 stars),
   not published on crates.io under its crate names;
   `kcl-lib` on crates.io is KittyCAD's unrelated language
   (Verified: crates.io search for `kcl`, `AG/kcl-2026-09-16/Cargo.toml:1-40`).
  The newest GitHub release is v0.11.2 from 2025-04-18,
   while the workspace version is 0.12.5
   (Verified: `gh api repos/kcl-lang/kcl/releases/latest`, `Cargo.toml`).
  It vendors `serde_yaml` and copies of `rustc_span` and `rustc_errors`
   (Verified: `Cargo.toml` path dependencies).
- Size:
   `kclvm-v0.11.2-linux-musl-amd64.tar.gz` is 8,936,515 bytes compressed
   (Verified: release assets).
- Turing status:
   schemas recurse;
   the language tour computes Fibonacci "using the recursive schema config"
   (Verified: `kcl-lang/kcl-lang.io` `docs/reference/lang/tour.md:2313-2327`, via `gh api`).
- Result:
   disqualified ("7. Options", option H).

### Dhall

- Library:
   `serde_dhall` 0.13.0,
   released 2025-09-10,
   0 releases since 2025-09-16
   (Verified, metadata).
- Turing status:
   "Dhall is not Turing-complete.
   Evaluation always terminates, no exceptions"
   (Verified: `dhall-lang/dhall-lang` README lines 12, 143, via `gh api`).
- Diagnostics:
   "invalid type: string \"many\", expected u32" with no location
   (Verified, probe).
- Static build:
   default feature `reqwest` pulls OpenSSL and failed for musl;
   without default features it built at +1,785,992 bytes
   (Verified, probe).
- Write-back:
   no comment-preserving Rust editor found
   (Unverified: not searched beyond crates.io).
- Result:
   designed as option I.

### Starlark

- Library:
   `starlark` 0.14.2 (2 releases, 40 open issues) and `starlark_lsp` 0.14.2 as a library.
- Turing status:
   Bazel's interpreter forbids recursion
   (Verified: "Bazel" in "2. Precedent"),
   but `starlark` 0.14.2 with `Dialect::Standard` evaluated a recursive factorial to 120
   (Verified, probe).
- Build:
   fails on the repository's nightly through `allocative` 0.3.6;
   5,593,360 bytes on stable glibc against a 311,856-byte baseline;
   pulls `cc`
   (Verified, probe).
- Result:
   designed as option G.

### CEL

- Library:
   `cel` 0.14.5 (13 releases; formerly `cel-interpreter`).
- Turing status:
   "CEL evaluates in linear time, is mutation free, and not Turing-complete"
   (Verified: `google/cel-spec` README line 17, via `gh api`).
- Diagnostics:
   "ERROR: <input>:1:17: Syntax error: mismatched input '<EOF>' ..." with a caret line
   (Verified, probe).
- Size:
   +2,662,824 bytes.
- Role:
   an expression sublanguage inside a data format,
   not a file format;
   designed as option F.

### Rego

- Library:
   `regorus` 0.12.0 (Microsoft, 6 releases).
- Turing status:
   "policy evaluation should be known to _terminate_ ...
   `while` loops, or recursive references ... are therefore not allowed in Rego"
   (Verified: `open-policy-agent/opa` `docs/docs/errors/rego-recursion-error/rule-name-is-recursive.md:8-12`,
   via `gh api`).
- Size:
   +7,038,952 bytes.
- Result:
   designed as option J.

### Excluded before design

- Pkl,
   recorded as an exclusion as instructed:
  - The instructed reason,
     an external binary,
     no longer holds alone:
     `pklr` 2.0.4 by jdx is "A pure Rust parser and evaluator for Apple's Pkl configuration language.
     No external binary or CLI required",
     with 32 releases in the past year
     (Verified: `AG/pklr-2026-09-16/README.md:1-12`, metadata).
    moon's loader still uses `rpkl`,
     which needs the `pkl` binary
     (Verified: `overview.mdx:100-104`, schematic's `Cargo.toml` via `gh api`).
  - Pkl permits recursion:
     "Like methods, anonymous functions can be recursive"
     (Verified: `apple/pkl` `docs/modules/language-reference/pages/index.adoc:4470`, via `gh api`),
     which conflicts with the not-Turing-complete decision.
  - `pklr` is partial:
     self-recursive module functions and a typed `Listing<Node>` failed
     (Verified, probe);
     its default features pull `rustls` with `aws-lc-rs`
     (Verified: `AG/pklr-2026-09-16/Cargo.toml:37`).
- CUE:
   the implementation is Go,
   which the user excluded
   (Verified: earlier research found no Rust implementation; not re-searched).
- Nickel:
   "Starlark forbids recursion and side-effects which are allowed in Nickel"
   (Verified: `nickel-lang/nickel` `RATIONALE.md:299`, via `gh api`),
   plus the 19,498,430-byte static library measured earlier.
- Jsonnet:
   "All listed languages but Jsonnet forbid general recursion"
   (Verified: `RATIONALE.md:155`),
   and no stable `jrsonnet-evaluator` release since 0.4.2 in 2021
   (Verified, metadata).
- RON:
   see "RON".
- JSON5:
   see "JSON5".
- Hjson (`deser-hjson` 2 releases, `serde-hjson` 0),
   EDN (`edn-rs` 1 release),
   and UCL (`libucl` 0.2.3 from 2019):
   adoption or maintenance below every designed option
   (Verified, metadata).

## 4. Generated text

### What builds text today

- Whole-file concatenation with fixed headers:
   LI02,
   LI03,
   LI05,
   LI06.
- One header line per mirrored file:
   LI09.
- Text assembled from a computed list:
   LI04 (`_.path` entries) and LI12 (published names and exclusion comments).
- Serialized data:
   LI13 and LI15 (`JSON.stringify(value, null, 2)`),
   LI11 and LI14 (structured edits).

### File includes and concatenation

- Form:
   `content` is a list of parts,
   each a literal or a file,
   joined by a declared separator
   (LI03, LI06).
- Covers LI02, LI03, LI05, LI06, and LI09's header with a per-file source variable.
- Terminates by construction:
   no loops,
   no references between parts.
- Keeps long prose in Markdown or text files that editors,
   Harper,
   and `rg` already handle,
   instead of in configuration strings.

### Template engines

- `minijinja` 2.24.0:
   macros recurse
   (Verified, probe `"321 x"`),
   and loops accept a `recursive` modifier
   (Verified: `AG/minijinja-2026-09-16/minijinja/src/syntax.rs:289-291`).
  Reintroduces unbounded recursion,
   bounded only by an engine limit.
  +1,028,328 bytes.
- `tera` 2.4.0:
   components recurse with arithmetic arguments
   (Verified, probe `"321"`);
   mise builds `exec(command)` on Tera
   ("Precedent").
  Reintroduces recursion.
  +819,232 bytes.
- `handlebars` 6.4.4:
   partials recurse over data
   (Verified, probe `"a(b(c))"`)
   and reject an unconditional self-include with `CannotIncludeSelf`
   (Verified, probe);
   helpers are host Rust functions.
  Recursion is bounded by data depth unless a host helper manufactures data
   (Unverified: no such helper tested).
  +491,552 bytes.
- `upon` 0.11.0:
   `if`, `for`, `with`, and `include`;
   "Self-referential templates and include cycles are allowed but the maximum include depth is restricted"
   (Verified: `AG/upon-2026-09-16/SYNTAX.md:277-279`).
  No macros or author functions
   (Verified: `SYNTAX.md` section list at `:3-30`).
  With includes disabled,
   loops run only over supplied values,
   so rendering terminates
   (Unverified: inferred from the syntax list).
  1 release in the past year,
   64 stars.
  +192,512 bytes.
- `askama` 0.16.1:
   templates compile into the binary
   (Verified, probe: identical size to the baseline),
   so a repository cannot edit them without rebuilding `meow`.
  Unusable for configuration-owned templates.
- HCL templates (`%{ for }`, `%{ if }`):
   loops over finite collections;
   OpenTofu's `templatefile` recursion is depth-limited to 1024
   ("Precedent").
  In an HCL configuration,
   forbidding a template from calling `templatefile` keeps rendering terminating
   (Unverified: proposed rule).

### Result

- 0.x needs no general template engine:
   concatenation covers the whole-file texts,
   LI12 moves into a TypeScript task that keeps its template literal,
   and LI04 retires.
- If an expression-capable format is chosen,
   its own template directives cover list-to-text needs
   such as LI04's interim form.
- If a data-only format later needs list rendering,
   `upon` with includes disabled is the smallest measured engine without author-defined recursion.
- Jinja-family engines (`minijinja`, `tera`) conflict with the not-Turing-complete decision.

## 5. Other repositories

### Discovery

- Precedents:
  - `rust-linter-core` collects `rust-linter.toml` files "from `start` upward, outermost first"
     (Verified: `package/rust-module/rust-linter-core/src/config/load.rs:197`).
  - Mise marks the root with `monorepo_root = true` and lists `config_roots`
     (Verified: `mise.no-env.toml:13-23`).
  - OpenTofu runs in one root module directory
     (Verified: `language/files/index.mdx:58-60`).
  - Turborepo names the root `//` in `extends`
     ("Precedent").
- Proposal:
   walk up from the working directory to the nearest `meow` configuration that declares a `workspace`;
   configuration files below it without `workspace` are project files;
   `--root` overrides the walk;
   a second `workspace` below the root is an error naming both paths.
  The walk stops at the filesystem root and never at a `.git` boundary,
   because `doc/planning/mise-removal-coverage.md` records that
   "A package-workspace boundary and a Git-checkout boundary are not interchangeable"
   (Verified: `mise-removal-coverage.md`, "Repository-root discovery").
- Environment variable names follow the tool's final name,
   per the design's `MONOCHROMATIC_JOBS` note
   (Verified: `monorepo-manager-from-scratch-design.md:234`).

### Schema versioning

- Precedents:
  - OpenTofu's `language` block with `compatible_with`,
     `edition`,
     and `experiments`
     ("Precedent").
  - Taskfile files start with `version: '3'`
     (Verified: `AG/task-2026-09-16/website/src/latest/docs/reference/schema.md:25`).
  - moon generates JSON Schemas into `.moon/cache/schemas` and deprecated its hosted URLs
     "as they do not support dynamic settings"
     (Verified: `overview.mdx:90-95`).
  - Renovate's `configMigration` rewrites configurations and can drop JSON5 comments
     ("Precedent").
- Proposal:
  - A required `language` block (or table) with `compatible_with = ">= 0.1"` and an `edition`.
  - Unknown keys and blocks are errors with a near-name suggestion,
     after HCL's "Unsupported argument" and `nameSuggestion`.
  - `meow schema` writes JSON Schema generated from the Rust configuration types with `schemars` 1.2.2 (5 releases)
     into the cache directory,
     after moon,
     for TOML, YAML, and JSONC editors;
     HCL and KDL get no schema route ("3. Configuration formats").
  - `meow migrate` rewrites the files through the comment-preserving editor of the chosen format,
     and refuses to write when the edited document does not re-parse to the intended value.

### Documentation needs

- Insufficient documentation is why Mise is being replaced
   ("not documented well enough",
   Verified: `mise-removal-coverage.md:27-28`),
   and the documentation confusion rule culls tool candidates
   (Verified: `mise-removal-coverage.md:44-48`),
   so `meow`'s own configuration reference is a release deliverable.
- Needed pages:
   every block and argument with its type and default,
   every built-in function with examples runnable in a console
   (OpenTofu documents each function this way, Verified: `language/functions/index.mdx:24-32`),
   evaluation and write order,
   ownership and pruning semantics,
   each diagnostic by code
   (OPA publishes one page per error, Verified: `rule-name-is-recursive.md`;
   Turborepo publishes `messages/` pages,
   Verified: `apps/docs/content/docs/messages/missing-root-task-in-turbo-json.mdx` exists),
   and migration from `file-enforcer.config.ts` and `mise.toml`.
- Generated reference:
   the same Rust types that produce the JSON Schema produce the argument reference,
   so the reference and the parser cannot drift
   (Unverified: proposed).

## 6. One configuration for file enforcement and tasks

### Shared model

- One module system:
   the root file holds `workspace`,
   repository-wide rules,
   and task templates;
   a project file beside a native manifest holds that project's tasks and project-owned rules
   (LI07, LI12, LI13).
- One graph:
   every rule instance and every task is a node with declared reads and writes.
  - A node that reads a path another node writes depends on it
     (LI06 on LI05, LI07 on LI06).
  - `depends_on` covers relations that no path shows
     (LI07 on the scanner build).
  - A task whose `outputs` are tracked files sets `managed = true`,
     and the watcher protects those outputs like rule destinations (FE22).
- One cache key design:
   rules and tasks share the cache key in the design's "Cache" section,
   with `gxhash128` over input contents
   (Verified: design, "Cache").
- Evaluation order:
   parse,
   evaluate locals and function reads,
   run `check` blocks and `absent` errors,
   then schedule rules and tasks by the graph.
  File-enforcement work,
   including configuration evaluation,
   runs in the re-executed child inside a task cgroup,
   as adopted in the design
   (Verified: design, "Adopted from settled requirements").
- Default work:
   on change,
   affected rules and affected tasks in the default set run;
   test tasks exclude files matching `*.expensive.*.test.*`
   (Verified: design, "Further answers on 2026-09-16").

### Tasks from templates

- Mise today:
   `task_templates` in `mise.no-env.toml:510` onward,
   referenced by `extends = "build"` in package files
   (Verified: `package/config/pnpr/mise.toml:1-24`),
   with fanout implemented in inline Node scripts
   (Verified: `mise-removal-coverage.md`, "Task discovery and execution").
- Proposal:
   `task_template` blocks at the root select projects by native manifest kind and file presence,
   after moon's `inheritedBy`
   (Verified: `AG/moon-2026-09-16/website/docs/config/tasks.mdx:128-137`)
   and Nx inference from tool configuration ("Precedent");
   a project file may override a templated task by name.

```hcl
# meow.hcl
task_template "test_unit" {
  applies_to = project.kind == "pnpm" && length(fileset(project.dir, "src/**/*.unit.test.ts")) > 0
  command    = ["node", "--test", "src"] # illustrative; today the runner is the inline run_test_files script in mise.no-env.toml
  inputs     = ["src/**", "package.json", lockfile(project)]
  exclude    = ["**/*.expensive.*.test.*"]
}
```

The `applies_to` expression is the HCL form;
a data-only format needs fixed selector keys such as `kind = "pnpm"` and `has_files = "src/**/*.unit.test.ts"`
(Unverified: proposed).

### Affected work from native manifests

- Edges come from the manifests the design names:
   pnpm workspace dependencies,
   Cargo path dependencies,
   and Gradle projects,
   "with extra rules for relationships between ecosystems"
   (Verified: design, "Decisions on 2026-09-16").
- Measured inputs:
   19 `Cargo.toml` files under `package/` contain a `path = "` key,
   a count that mixes path dependencies with target paths such as `[lib] path`
   (Verified: `rg --count-matches 'path\s*=\s*"'` without a line cap),
   two Gradle builds have `settings.gradle.kts`
   (`package/linter/kotlin`, `package/music-player/android-app`, Verified: `rg --files`),
   and pnpm workspaces come from `pnpm-workspace.yaml`
   (Verified: file read).
- Topological task dependencies follow Turborepo's `^` form
   ("Precedent"):
   `depends_on = [dependencies.build]` in a template means the same task in every upstream project.
- Cross-ecosystem edges are declared data:

```hcl
# package/music-player/android-app/meow.hcl (illustrative edge)
project {
  depends_on_projects = ["package/music-player/android-app/rust"]
}
```

- A changed file maps to the project whose directory is the longest prefix;
   affected projects are the reverse-dependency closure;
   a change to a rule's declared read re-runs only that rule
   (Unverified: proposed).
- Configuration files are inputs too:
   a change to a project file re-evaluates that module,
   and a change to the root file re-evaluates everything
   (Unverified: proposed).

## 7. Options

Every option shares "6. One configuration for file enforcement and tasks",
the built-in features named in "1. Logic inventory",
the three repository tasks,
`gxhash128` ledgers,
and evaluation in the re-executed child inside a task cgroup.
They differ in syntax,
expression power,
libraries,
and tooling.

Two decisions inside the options are separable
("9. Open questions"):
the syntax,
and how much expression power the configuration has.

### A: OpenTofu-shaped HCL

#### Design

- Files:
   `meow.hcl` plus any `*.meow.hcl` in the same directory form one module,
   after OpenTofu's directory modules;
   the suffix avoids claiming other tools' `.hcl` files.
  `*.meow.json` is the machine-generated variant,
   deferred past 0.x.
- Root module:
   `language`,
   `workspace`,
   `locals`,
   `check`,
   repository-wide rules,
   and `task_template` blocks.
- Project modules:
   discovered from native manifests,
   not from explicit `module` calls,
   because pnpm,
   Cargo,
   and Gradle already define projects;
   this is the one structural departure from OpenTofu.
- Expressions:
   HCL expressions and template directives;
   an application-owned function table limited to what the inventory needs
   (`file`, `fileset`, `dirset`, `jsondecode`, `tomldecode`, `join`, `concat`, `flatten`, `merge`, `distinct`,
   `sort`, `keys`, `values`, `lookup`, `contains`, `length`, `try`, `can`, `tostring`, `tolist`, `toset`,
   `dirname`, `startswith`, `endswith`, `strcontains`, `replace`, `spdx_ids`, `lockfile`, `project_dirs`);
   no author-defined functions in 0.x;
   no function that evaluates another template file.
- Termination:
   HCL expressions iterate only over finite values,
   and every function is host-defined and total,
   so evaluation terminates
   (Unverified: argued from the HCL specification and OpenTofu's recursion guard, not proved).
- Reads:
   only through `file`,
   `fileset`,
   `dirset`,
   and `lockfile`,
   so every read is a tracked input.
  A rule may not read another node's output during evaluation;
   that keeps OpenTofu's "known after apply" machinery out of 0.x.
- Per-match iterators:
   rules visiting many files expose a named iterator
   (`manifest.dir` in LI11, `mirror.source` in LI09).
- Implementation:
   parse with `hcl-edit` for spans and comment-preserving edits;
   evaluate with a repository evaluator over `hcl-edit` spans,
   because `hcl-rs` evaluation and deserialization drop locations;
   render with `miette` or `annotate-snippets`;
   `meow fmt` normalizes `hcl-edit` decor;
   `meow lsp` inside the binary serves diagnostics,
   completion from the Rust schema types,
   and hover from the generated reference,
   reachable from JetBrains through LSP4IJ,
   which the repository already configures (LI14).
- Versioning:
   `language { compatible_with { meow = ">= 0.1" } edition = meow2026 }`.

```hcl
# meow.hcl (root module, abbreviated; the rule blocks are LI01 to LI15)
language {
  compatible_with {
    meow = ">= 0.1"
  }
  edition = meow2026
}

workspace {
  heavy_test_pattern = "*.expensive.*.test.*"
  concurrency_env    = "MONOCHROMATIC_JOBS"
}

check "pnpr_exclusions_are_single_line" {
  assert {
    condition     = alltrue([for reason in values(local.pnpr_excluded) : !strcontains(reason, "\n")])
    error_message = "A reviewed pnpr exclusion reason spans more than one line."
  }
}

# absent "root_context" { ... }        LI01
# file "license" { ... }               LI02
# file "claude_md" { ... }             LI03
# file "forbidden_strings_rules" {...} LI05, LI06
# mirror "git_policy" { ... }          LI09
# file "package_license_text" { ... }  LI10
# toml_keys "cargo_manifests" { ... }  LI11
# lsp4ij_server "harper" { ... }       LI14
# mirror "agent_skills" { ... }        LI15
```

#### Pros

- Matches the user's precedent and the repository's own OpenTofu module,
   including `check`,
   `for_each`,
   `dynamic`,
   and the external-program escape hatch
   ("Precedent").
- Per-match derivations stay in one general expression language:
   LI10's license union,
   LI11's homepage and profile lookup,
   LI14's rule-name mapping,
   and LI04's interim list,
   instead of a placeholder syntax per built-in.
  This keeps built-ins general for other repositories.
- Shown expressible:
   OpenTofu evaluated LI03,
   LI10's mapping,
   LI15's ownership map,
   and LI12's selection,
   entry-point rules,
   name check through `regex`,
   and YAML rendering with built-in functions
   (Verified, probe);
   LI12's stale-exclusion and missing-name errors were not written.
- Directory modules,
   editions,
   validation,
   and checks have a worked-out precedent to copy.
- `hcl-edit` round-tripped a 27,689-byte real module byte for byte
   (Verified, probe).
- Pure Rust;
   `hcl-edit` adds 344,064 bytes
   (Verified, probe).

#### Cons

- The Rust HCL stack is thin:
   no built-in functions (issue #484 open),
   no locations on evaluation or deserialization errors,
   no formatter,
   no HCL JSON parser,
   and no dynamic-block expansion,
   so the evaluator,
   function table,
   formatter,
   JSON variant,
   and language server are repository code
   (Verified: "HCL in Rust").
- `hcl-edit` has one maintainer,
   warns "Expect breaking changes at any time",
   and joined a multi-line condition onto one line in the probe
   (Verified).
  Owning a fork is plausible;
   the defect would need an upstream report or a fix.
- No JSON Schema route for editors;
   completion exists only once `meow lsp` exists
   (Verified for JetBrains' HCL enabler search; Unverified for other editors).
- Expressions let logic drift back into configuration:
   `hetzner.tf` has 941 lines with 64 `for` keywords,
   and its `locals` blocks at lines 104,
   324,
   and 561 to 898 compute CIDR lists,
   rule chunks,
   and firewall balancing
   (Verified: `rg --count-matches '\bfor\b'`, block-start `rg`, and reads of `:520-620`, `:700-720`),
   and OpenTofu warns about overusing `locals` and `dynamic`
   ("Precedent").
- Less familiar than TOML or YAML to repositories outside infrastructure work
   (Unverified: no survey).

#### Disqualifying problems

None found.

### B: TOML data with built-in features

#### Design

- Files:
   root `meow.toml` with `[language]`,
   `[workspace]`,
   and rules;
   `include = ["meow/cargo.toml"]` splits large roots,
   after `rust-linter.toml`'s `extends`;
   project `meow.toml` files hold `[tasks.<name>]` with `extends = "<template>"`,
   after package `mise.toml` files.
- No expressions.
  Per-match values use placeholders defined per field,
   such as `{dir}` in LI11's homepage and `{source}` in LI09's header,
   after Turborepo's `$TURBO_DEFAULT$` microsyntax
   ("Precedent").
- Computations with no data form become dedicated built-ins:
   `license_texts` (LI10)
   and a `profile_by_dir` table interpreted by the Cargo plugin (LI11).
- LI04's interim list needs a bespoke part such as `{ list = "workspace_bin_dirs" }`,
   or the TypeScript file-enforcer keeps generating `mise.toml` until Mise leaves CI.
- Implementation:
   `toml` with `serde_spanned` and `deny_unknown_fields`,
   `toml_edit` for writes and `meow migrate`,
   `schemars` for `meow schema`,
   and taplo or JetBrains for completion.

```toml
# meow.toml (root, abbreviated)
#:schema ./.cache/meow/schema/root.json

[language]
compatible_with = ">= 0.1"
edition = 2026

[workspace]
heavy_test_pattern = "*.expensive.*.test.*"
include = ["meow/cargo.toml", "meow/lsp4ij.toml"]

[[absent]]
path = "CONTEXT.md"
on_present = "error"
error_message = "CONTEXT.md is forbidden. Do not create cached context files; read source code directly and use doc/agent/domain.md for this repo policy."

[[file]]
path = "CLAUDE.md"
parts = [{ file = "doc/agent/claude-code-preamble.md" }, { file = "AGENTS.md" }]

[[file]]
path = ".cache/forbidden-strings.rules.txt"
parts = [
  { file = "config/meow/forbidden-strings-rules-header.txt" },
  { file = "forbidden-strings.append.txt" },
  { text = "\n" },
  { file = "forbidden-strings.append.local.txt" },
]
mode = "0600"
sensitive = true

[[mirror]]
name = "git-policy-repository-policy"
source_dir = "package/git-policy/repository/src"
files = ["bump-dependents-worktree.ts", "dependent-bump-workflow.ts", "index.ts"] # abbreviated
to = "package/git-policy/cli/src/optional/repository-policy/*"
header = "// Generated from `{source}` by meow; edit canonical source owner.\n"
replace = { "@monochromatic-dev/git-policy-api/ts" = "../../api/index.ts" }

[[mirror]]
name = "claude-skills"
from = ".agents/skills/*/*.md"
to = ".claude/skills/*/*.md"
prune = "ledger"

[license_texts.packages]
manifests = ["package/*/*/package.json", "package/*/*/Cargo.toml"]
dest = "{package}/LICENSES/{id}.txt"
implies = { "LGPL-3.0-or-later" = ["GPL-3.0-or-later"] }
texts = { "CC-BY-SA-4.0" = "LICENSES/CC-BY-SA-4.0.txt", "GPL-3.0-or-later" = "LICENSES/GPL-3.0-or-later.txt", "LGPL-3.0-or-later" = "LICENSES/LGPL-3.0-or-later.txt" }
```

```toml
# meow/cargo.toml (abbreviated)
[toml_keys.cargo]
files = ["package/*/*/Cargo.toml", "package/*/*/*/Cargo.toml"]
exclude = ["**/target/**", "**/node_modules/**"]

[[toml_keys.cargo.enforce]]
guard = ["package"]
path = ["package", "edition"]
value = "2024"

[[toml_keys.cargo.enforce]]
guard = ["package", "repository"]
path = ["package", "homepage"]
value = "https://github.com/Aquaticat/Monochromatic/tree/main/{dir}"

[toml_keys.cargo.enforce_entries_if_present.dependencies]
anyhow = "1"
clap = { version = "4", features = ["derive"] }

[toml_keys.cargo.profiles.scanner.release]
lto = true
codegen-units = 1
opt-level = 3
panic = "unwind"
overflow-checks = true
strip = true

[toml_keys.cargo.profile_by_dir]
"package/forbidden-strings/cli" = "scanner"
"package/forbidden-strings/core" = "scanner"
```

#### Pros

- The most mature Rust libraries of any option:
   `toml_edit` has 22 releases in the past year and is already in the daemon skeleton;
   type and syntax errors carry snippets
   (Verified).
- Editor completion without repository code:
   taplo `#:schema` and JetBrains' TOML JSON Schema support,
   on by default
   (Verified).
- Largest set of repository incumbents:
   Mise files,
   Cargo manifests,
   and the `rust-linter.toml` loader with `extends`,
   discovery,
   and `deny_unknown_fields`
   (Verified).
- Aligns with Cargo's split:
   manifests are data,
   logic is a build script or task
   ("Precedent").
- Byte-identical round trips on repository files
   (Verified).

#### Cons

- No expressions:
   every per-match value is a field-specific placeholder,
   and every computation without a data form becomes a dedicated built-in
   (LI10, LI11's `profile_by_dir`, LI04's list).
  Those placeholder syntaxes multiply per built-in.
- TOML arrays of tables nest poorly:
   `[[toml_keys.cargo.enforce]]` belongs to whichever table header precedes it,
   and deep inline values repeat long key prefixes.
- No references:
   presets are looked up by string key inside each plugin.
- Composition (`include`, `extends`) is invented by the tool.

#### Disqualifying problems

None found.

### C: KDL data with built-in features

#### Design

As B,
with KDL node syntax,
`kdl` for parsing and edits,
and a repository mapping layer from `KdlDocument` to Rust types
(or `knus` derives).

```kdl
// meow.kdl (abbreviated)
language compatible-with=">= 0.1" edition=2026

absent "CONTEXT.md" on-present="error" message="CONTEXT.md is forbidden. Do not create cached context files; read source code directly and use doc/agent/domain.md for this repo policy."

file "CLAUDE.md" {
    part file="doc/agent/claude-code-preamble.md"
    part file="AGENTS.md"
}

toml-keys "cargo" {
    files "package/*/*/Cargo.toml" "package/*/*/*/Cargo.toml"
    enforce guard="package" path="package.edition" value="2024"
    enforce-if-present table="dependencies" {
        dependency "clap" version="4" {
            features "derive"
        }
    }
}
```

#### Pros

- Nesting reads naturally,
   comments are preserved on edit,
   and parse errors carry spans
   (Verified).
- Pure Rust at +303,104 bytes
   (Verified).

#### Cons

- No schema-driven completion or validation in any editor found;
   `kdl-lsp` reports syntax diagnostics only
   (Verified).
- No repository incumbent,
   and the least familiar format for other repositories
   (Unverified: no survey).
- Typed decoding needs repository mapping code or `knus`,
   whose error text misdescribed a type mismatch
   (Verified).
- Scalar-only arguments turn nested Cargo values into child nodes,
   a second schema for the same TOML data.
- B's placeholder and dedicated-built-in cons.

#### Disqualifying problems

None found.

### D: YAML data with built-in features

#### Design

As B,
with YAML after moon,
Taskfile,
and lefthook;
`serde-saphyr` for parsing,
`yaml-edit` (`YamlFile`) for writes,
and JSON Schema for editors.

```yaml
# meow.yaml (abbreviated)
language:
  compatible_with: ">= 0.1"
  edition: 2026
absent:
  - path: CONTEXT.md
    on_present: error
    error_message: >-
      CONTEXT.md is forbidden. Do not create cached context files;
      read source code directly and use doc/agent/domain.md for this repo policy.
file:
  - path: CLAUDE.md
    parts:
      - file: doc/agent/claude-code-preamble.md
      - file: AGENTS.md
```

#### Pros

- Familiar to most monorepo tool users
   (moon, Taskfile, lefthook, pre-commit, GitHub Actions).
- JetBrains JSON Schema support through `YamlJsonEnabler`
   (Verified).
- `serde-saphyr` gives snippet diagnostics and strict scalar typing
   (Verified).
- `yaml-edit` round-tripped three repository YAML files
   (Verified).

#### Cons

- Serde layer churn:
   `serde_yaml` and `serde_yml` are archived,
   `serde_norway` had no release in a year
   (Verified).
- `yaml-edit` has 10 stars,
   and its `Document` entry point rejects files with leading comments
   (Verified).
- Indentation-sensitive structure and implicit scalar typing are authoring hazards
   (Unverified: general YAML knowledge).
- `serde-saphyr` adds 942,216 bytes
   (Verified).
- B's placeholder and dedicated-built-in cons.

#### Disqualifying problems

None found.

### E: JSONC data with built-in features

#### Design

As B,
with JSONC after Turborepo and dprint;
`jsonc-parser` with `serde` and `cst`;
`$schema` in the file.

```jsonc
// meow.jsonc (abbreviated)
{
  "$schema": "./.cache/meow/schema/root.json",
  "language": { "compatible_with": ">= 0.1", "edition": 2026 },
  "absent": [
    {
      "path": "CONTEXT.md",
      "on_present": "error",
      "error_message": "CONTEXT.md is forbidden. Do not create cached context files; read source code directly and use doc/agent/domain.md for this repo policy.",
    },
  ],
  "file": [
    { "path": "CLAUDE.md", "parts": [{ "file": "doc/agent/claude-code-preamble.md" }, { "file": "AGENTS.md" }] },
  ],
}
```

#### Pros

- JSON Schema completion in essentially every editor
   (Unverified beyond Turborepo's documented IDE support).
- `jsonc-parser` is maintained by dprint's author and its CST round-tripped repository files
   (Verified).
- +180,224 bytes
   (Verified).
- Incumbents:
   dprint configuration,
   `tsconfig` files,
   and `package/jsonc-edit/core-ts`
   (Verified).

#### Cons

- No multi-line strings,
   so every long text is a separate file.
- Quoting and bracket noise across a configuration of LI11's size.
- Diagnostics give line and column without a snippet unless the tool renders one
   (Verified).
- B's placeholder and dedicated-built-in cons.

#### Disqualifying problems

None found.

### F: TOML data with CEL expressions

#### Design

B's files,
plus fields ending in `_expr` evaluated by `cel` with host-provided variables
(`manifest.dir`, `project.kind`)
and host functions;
reads stay in built-ins.

```toml
# meow/cargo.toml (abbreviated)
[[toml_keys.cargo.enforce]]
guard = ["package", "repository"]
path = ["package", "homepage"]
value_expr = "'https://github.com/Aquaticat/Monochromatic/tree/main/' + manifest.dir"

[[toml_keys.cargo.enforce]]
guard_expr = "manifest.dir in profile_by_dir"
path = ["profile"]
value_expr = "profiles[profile_by_dir[manifest.dir]]"
```

#### Pros

- Expressions that are "not Turing-complete" and evaluate "in linear time"
   (Verified: CEL specification).
- Keeps every TOML tooling advantage for the data around the expressions.
- Span diagnostics for CEL syntax errors
   (Verified).

#### Cons

- Expressions live inside strings,
   so editors neither highlight nor complete them,
   and JSON Schema sees only strings.
- Two syntaxes in one file.
- CEL produces values,
   not blocks,
   so repetition such as `for_each` over rules still needs built-ins.
- `cel` adds 2,662,824 bytes,
   about eight times `toml`
   (Verified; ratio Unverified arithmetic).

#### Disqualifying problems

None found.

### G: Starlark with recursion blocked

#### Design

`meow.star` files evaluated by `starlark`,
host functions that return rule and task declarations,
`load()` resolved by the host,
and a repository-written check that rejects recursive calls,
after OpenTofu's Symbol Libraries guard.

```python
# meow.star (abbreviated)
file(path = "CLAUDE.md", parts = [read("doc/agent/claude-code-preamble.md"), read("AGENTS.md")])

for dest in [".claude/skills", ".factory/skills"]:
    mirror(src = ".agents/skills/*/*.md", dest = dest + "/*/*.md", prune = "ledger")
```

#### Pros

- Bazel and Buck2 precedent,
   author-defined functions for abstraction,
   and a `starlark_lsp` library.

#### Cons

- `starlark` 0.14.2 evaluates recursion under `Dialect::Standard`
   (Verified, probe),
   so non-Turing-completeness depends on repository code.
- Does not build on the repository's nightly
   (Verified, probe);
   the fix sits in Buck2's repository,
   which absorbed `allocative`
   (Verified).
- 5,593,360 bytes on glibc against a 311,856-byte baseline,
   and it pulls `cc`
   (Verified).
- Top-level `for` needs `enable_top_level_stmt`
   (Verified in the earlier research: `starlark_syntax/src/dialect.rs:58-66`).
- The most program-like option,
   against the direction that logic moves into code.

#### Disqualifying problems

- As shipped,
   `starlark` gives a Turing-complete configuration,
   which the decision record rejects
   ("Embedding a ... Starlark ... host for a Turing-complete configuration").
  A recursion-blocked dialect escapes the wording of that rejection but not its direction.

### H: KCL

#### Design

`kcl-lang/kcl` as a Git dependency,
configuration as KCL schemas,
output converted to Rust types.

#### Pros

- Validation-oriented schemas and a formatter in the project
   (Unverified: not built).

#### Cons

- Not on crates.io under its own names;
   newest release 2025-04-18;
   vendored `rustc_span`, `rustc_errors`, and `serde_yaml`
   (Verified).
- Release archive 8,936,515 bytes compressed
   (Verified).

#### Disqualifying problems

- Recursive schemas make it Turing-complete
   (Verified: KCL language tour).

### I: Dhall

#### Design

`meow.dhall` evaluated by `serde_dhall` without default features into Rust types;
imports for composition.

```dhall
-- meow.dhall (abbreviated)
{ files =
  [ { path = "CLAUDE.md", parts = [ "doc/agent/claude-code-preamble.md", "AGENTS.md" ] } ]
}
```

#### Pros

- Total by design:
   "Evaluation always terminates, no exceptions"
   (Verified).
- Functions,
   types,
   and imports without a recursion escape.

#### Cons

- `serde_dhall` had no release in the past year
   (Verified).
- Errors lack source locations
   (Verified, probe).
- The default feature breaks static musl builds;
   without it the crate adds 1,785,992 bytes
   (Verified).
- No comment-preserving editor,
   so no `meow migrate`
   (Unverified: none found).
- Unfamiliar syntax for most repositories
   (Unverified).

#### Disqualifying problems

None found.

### J: Rego

#### Design

Policy modules evaluated by `regorus`,
with rules such as `files[path] := content` producing the rule and task sets,
and host built-ins for reads.

```rego
# meow.rego (abbreviated)
package meow

files["CLAUDE.md"] := concat("", [read("doc/agent/claude-code-preamble.md"), read("AGENTS.md")])
```

#### Pros

- Recursion is a compile-time error,
   so evaluation terminates
   (Verified: OPA documentation).
- Functions without recursion,
   comprehensions,
   and a large policy-as-code precedent.

#### Cons

- `regorus` adds 7,038,952 bytes
   (Verified).
- A query language whose results are documents,
   so tasks and rules become derived sets rather than declarations,
   which is harder to document and to edit with a tool.
- No comment-preserving editor found
   (Unverified).

#### Disqualifying problems

None found.

### Excluded before design

Pkl,
CUE,
Nickel,
Jsonnet,
RON,
JSON5,
Hjson,
EDN,
and UCL,
with reasons in "3. Configuration formats".

## 8. Ranking

A > B > F > E > D > C > I > J > G > H.

- A over B:
   A's expressions and `for_each` keep per-match derivations
   (LI10, LI11, LI14, the LI04 interim)
   in one general language,
   which follows the user's OpenTofu precedent and the repository's OpenTofu incumbent;
   B needs a placeholder syntax per built-in or a dedicated built-in per computation,
   a language gap that closes only by adding expressions.
  A's worst problems are repository code
   (evaluator, function table, formatter, language server)
   and an `hcl-edit` defect,
   which the user's acceptance of "writing much more code" covers;
   B's library and editor maturity does not outweigh that.
- B over F:
   F's CEL expressions sit inside strings without editor support,
   add a second syntax,
   produce no blocks,
   and cost 2,662,824 bytes,
   while B covers the same inventory with placeholders and built-ins;
   expressions pay off only when native to the syntax, as in A.
- F over E:
   F keeps TOML's multi-line strings,
   repository incumbents,
   JetBrains schema support,
   and the `toml_edit` already in the daemon,
   and adds terminating expressions;
   E has neither multi-line strings nor expressions.
- E over D:
   both have JSON Schema editors,
   but `jsonc-parser` is maintained by dprint's author with 14 releases and round-tripped repository files,
   while YAML's serde layer went through two archived crates,
   its lossless editor has 10 stars,
   and YAML adds indentation and implicit-typing hazards.
- D over C:
   YAML has JetBrains JSON Schema support and repository incumbents;
   KDL has no schema-driven editor support,
   no incumbent,
   and a typed decoder whose error text misdescribed a mismatch.
- C over I:
   `kdl` is maintained with formatting-preserving edits and span diagnostics;
   `serde_dhall` had no release in a year,
   reports errors without locations,
   and has no comment-preserving editor.
- I over J:
   both terminate by design,
   but Dhall is a configuration language producing typed records at 1,785,992 bytes,
   while Rego is a policy query language whose results are derived sets,
   at 7,038,952 bytes.
- J over G:
   Rego rejects recursion at compile time;
   `starlark` evaluates recursion by default,
   fails to build on the repository's nightly,
   and needs a C compiler.
- G over H:
   both allow recursion,
   but `starlark` is on crates.io with a maintained language-server library and Bazel's non-recursive dialect to copy,
   while KCL is not published on crates.io and its newest release is 2025-04-18.

Worst problem per option:

- A:
   the Rust HCL stack lacks built-in functions,
   error locations in evaluation,
   a formatter,
   and editor schemas,
   and `hcl-edit` changed bytes on a multi-line condition.
- B:
   no expressions,
   so each derivation becomes a per-built-in placeholder or a dedicated built-in.
- F:
   expressions inside strings,
   invisible to editors,
   for 2.66 MB.
- E:
   no multi-line strings.
- D:
   an archived-crate history and a 10-star lossless editor.
- C:
   no schema-driven editor support.
- I:
   an unreleased-for-a-year crate with location-free errors.
- J:
   7.04 MB and a query model that does not map onto declarations.
- G:
   Turing-complete by default,
   against the decision record.
- H:
   Turing-complete through recursive schemas.

## 9. Open questions for the user

Settled answers adopted without asking,
open to veto:

- FE01's author control over sequencing becomes derived ordering from declared reads,
   writes,
   and `depends_on`
   (LI16).
- LI15's skill-mirror ownership moves from per-directory manifest files to the tool's ledger,
   hashed with `gxhash128`;
   both mirror roots are gitignored,
   so no clone depends on the manifests.
- LI12 and LI13 become TypeScript tasks in their owning packages,
   since those packages already hold the related code
   (`publish-plan.ts`, `browserslist-targets.ts`).
- No general template engine in 0.x ("4. Generated text").

Questions whose answers change the design:

1.  Is HCL syntax itself wanted,
    or OpenTofu's model
    (application-owned schema and functions, expressions without author functions, checks, directory modules,
    and an escape hatch to programs)
    regardless of syntax?
    - HCL syntax, or both:
       A stays first.
       Pro: native expressions and the incumbent `hetzner.tf`.
       Con: the thinnest Rust tooling among the leading options.
    - The model only,
       with tooling maturity weighed above native expressions:
       B or F moves above A.
       Pro: `toml_edit`, taplo, and JetBrains schema support exist today.
       Con: expressions either disappear (B) or move into strings (F).
    Ranking:
     HCL syntax or both > model only,
     because only HCL carries OpenTofu's model natively,
     and the tooling gap is code the user accepted writing.
2.  How much expression power should 0.x have?
    This is separable from the syntax.
    - None:
       data plus built-ins and tasks (B, C, D, E).
       Pro: nothing to evaluate or document beyond fields.
       Con: placeholders per built-in and more dedicated built-ins.
    - Expressions with application-owned functions and no author functions (A, F):
       Pro: per-match derivations stay general.
       Con: an evaluator and a function reference to build and document.
    - Also author-defined functions with recursion rejected,
       after OpenTofu's experimental Symbol Libraries:
       Pro: removes repeated predicates such as LI12's inline TypeScript-source test.
       Con: OpenTofu itself marks this experimental and "subject to *significant* changes".
    Ranking:
     expressions without author functions > none > with author functions,
     because the first covers every inventory derivation,
     none pushes derivations into built-ins,
     and author functions add the feature OpenTofu has not stabilized.
3.  May a repository configuration write outside the repository,
    as LI14 writes JetBrains settings under the user's config directory?
    - Yes, in the repository file:
       Pro: parity with today.
       Con: user-scope state in a repository file, outside the watched tree.
    - Move user-scope rules to a per-user `meow` configuration:
       Pro: the repository file only touches the repository.
       Con: a second configuration file to discover and document.
    Ranking:
     per-user file > repository file,
     because other repositories should not edit a contributor's editor settings by cloning.
4.  On a fresh clone,
    should LI07 build the forbidden-strings scanner before compiling rules,
    or keep today's "skip with a warning when the scanner is absent"?
    - Build first:
       Pro: the cache is always warm.
       Con: a Rust release build on first preparation.
    - Skip when absent:
       Pro: today's cost.
       Con: an existence condition on a task,
        which the graph otherwise avoids.
    Ranking:
     build first > skip when absent,
     because the daemon builds affected work anyway and conditions on file existence are undeclared inputs.
5.  Until Mise leaves the macOS and Windows runners,
    which tool writes `mise.toml` (LI04)?
    - `meow`'s interim rule:
       Pro: one enforcer.
       Con: LI04's awkward form ships in 0.x.
    - The TypeScript file-enforcer for that file only:
       Pro: no interim design.
       Con: two enforcers writing one repository.
    Ranking:
     `meow`'s interim rule > TypeScript file-enforcer,
     because two enforcers on one tree need the lock interoperability the earlier research flagged.
6.  Byte identity remains the design's existing open question;
    headers that name the generator (LI03, LI09) change bytes under any option.

## Documentation problems recorded

Recorded, never culling:

- `hcl-edit` README:
   "The documentation as well as usage examples are scarce"
   (Verified: `crates/hcl-edit/README.md:15`).
- `starlark` evaluates recursion,
   while Bazel's Starlark documentation says "Recursion is not allowed";
   the starlark-rust documentation searched in this session does not mention recursion
   (Verified: `rg --ignore-case recurs` over `AG/starlark-rust-2026-09-16/docs` and `starlark/src/eval`
   found no policy text).
- `tera` 2 documentation has no statement on component recursion
   (Verified: `rg --ignore-case recurs` over `AG/tera-2026-09-16/docs/content` returned nothing),
   although components recurse
   (Verified, probe).
- `knus` error text "expected string scalar, found string" for a string given to an integer field
   (Verified, probe).
- `yaml-edit`'s quick start uses `Document::from_str`,
   which rejects a file whose first line is a comment
   (Verified: README and probe).
- moon's configuration overview lists HCL as supported
   while its loader evaluates only `locals` and a `concat` function and reports no spans
   (Verified: `overview.mdx:10-19`, `hcl.rs:16-93`).
- `tofu-ls` README marks the server "[WIP]"
   (Verified).
- OpenTofu Symbol Libraries:
   "the documentation is still a work in-progress"
   (Verified: `language/symbol-libraries/index.mdx:18`).
- Renovate documents that its migration may remove JSON5 comments
   (Verified).
