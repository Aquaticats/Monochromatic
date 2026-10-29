# Unified-linter cutover handoff

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/rust-module/jsonc-edit.fuzz` is now `package/jsonc-edit/core-rust.fuzz`;
> `package/dev-script/file-enforcer` is now `package/file-enforcer/core`;
> `package/rust-module/jsonc-edit` is now `package/jsonc-edit/core-rust`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/rust-module/forbidden-regex` is now `package/forbidden-strings/core`.

## Purpose

Carry out step 7 of the numbered plan in
[`doc/planning/unified-linter.md`](../planning/unified-linter.md):
publish `monochromatic-lint`,
install it through mise,
switch every consumer,
and delete the five incumbent packages.
The main session delegated this on 2026-10-06,
after the owner approved crates.io publication for that session.
Step 8,
fixing the snippet findings,
is not part of this work.

This document is the evidence record,
updated after each step.
Work happens on branch `feat/linter-cutover`
in the linked worktree `.claude/worktrees/linter-cutover`.

## How to read and respond

Read `Blocker` first:
it names the one action only the owner can take.
Then read `Landing choreography`,
which lists what the main session must do at landing time and in which order,
because several steps change the production Git wrapper's behavior and cannot be done from the branch.
`Publications`,
`Publish workflow`,
and `Release targets` hold the crates.io and CI evidence.
Respond by completing the owner action in `Blocker`,
or with a correction to a named entry.

## Blocker

Resolved on 2026-10-10.
The user created a crates.io token with the publish-new,
publish-update and trusted-publishing scopes and ran `cargo login`.
`cargo publish --dry-run` (with the verifying build) passed, then `cargo publish --no-verify` from the cutover branch
published `monochromatic-lint` 0.1.0.
The trusted publisher (`Aquaticat/Monochromatic`, `cargo-publish.yml`, no environment, id 24613)
was added through `POST /api/v1/trusted_publishing/github_configs` with that token,
which the endpoint's source accepts for the trusted-publishing scope
(`src/controllers/trustpub/github_configs/create.rs`).
The token is still in `~/.cargo/credentials.toml`; revoking the bootstrap token is the user's step.

## Publications

### monochromatic-jsonc-edit 0.1.1

- Why:
   the repository source differed from the published 0.1.0 in `src/comment_merge.rs`
   (a comment body carrying a bare CR is emitted in block form)
   and `src/edit_apply.rs`
   (replacing a scalar document root is refused),
   measured by diffing the downloaded 0.1.0 `.crate` against the repository.
- Also in 0.1.1:
   five shadowed bindings renamed in `src/scan.rs`,
   `src/parse.rs`,
   and `src/parse_tests.rs`,
   because `mise run //package/rust-module/jsonc-edit:lint:types` failed at the branch base on the
   `shadow_reuse` and `shadow_unrelated` denials added on 2026-10-04.
   Behavior is unchanged;
   all 81 tests pass.
- Commits:
   `f313a8933` (version and lockfiles),
   `7af92c819` (renames),
   landed on `main` as merge `533574e1f`.
- Dry run on the exact landed commit:
   run 37530495220 on `533574e1f`,
   `je-publish-crate` concluded `success` with `Packaged 31 files, 308.8KiB (75.5KiB compressed)`,
   `Verifying monochromatic-jsonc-edit v0.1.1`,
   and `warning: aborting upload due to dry run`.
   An earlier dry run,
   37529342345 on `184ee5baf`,
   also passed;
   `main` moved before it could land.
- Publication:
   push run 37531066191 on `533574e1f`,
   job `je-publish-crate`,
   `Published monochromatic-jsonc-edit v0.1.1 at registry crates-io` at 2026-10-06T21:05:05Z.
- crates.io API (`/api/v1/crates/monochromatic-jsonc-edit/0.1.1`):
   license `LGPL-3.0-or-later`,
   `crate_size` 77285,
   checksum `77ab79da384484360cc6d2392921fe3bfb3e67df30416b17765f11b562d23d1c`,
   which equals the build-provenance attestation digest the run logged,
   and `trustpub_data` naming run 37531066191 and commit `533574e1f`.
- Consumer build:
   a disposable crate depending on `monochromatic-jsonc-edit = "=0.1.1"`,
   built in `docker.io/library/rust:latest` with network,
   downloaded the crate from crates.io,
   and its probe exited 0 with
   `cr-comment-reparses=true` and `scalar-root-refused=true`.
   The same probe against `=0.1.0` printed `false` for both,
   so the probe can tell the versions apart.
- Gates before landing:
   `mise run //package/rust-module/jsonc-edit:test` (81 passed),
   `mise run //package/rust-module/jsonc-edit:lint:types` (clean),
   `mise run //package/rust-module/jsonc-edit:package`,
   `mise run //package/rust-module/jsonc-edit.fuzz:test` (13 passed),
   `mise run //package/git-policy/cli:native:check`,
   and `mise run //package/linter/monochromatic-lint:lint:container` with
   `MONOCHROMATIC_LINT_IMAGE_TAG=cutover` on `184ee5baf`:
   389 library tests passed with 1 ignored,
   13 executable tests passed,
   and Clippy passed.
   The gated directories are byte-identical between `184ee5baf` and `533574e1f`.

Every dependent's lockfile was regenerated through its owning task,
and each diff is the single `monochromatic-jsonc-edit` version line:
`native:lock:scanner` for `package/git-policy/cli`,
`lock:subject` for `package/git-policy/cli.fuzz`,
`lock` for the linter and its fuzz sidecar,
and `lint` and `lint:clippy`,
which run unlocked Cargo,
for `package/rust-module/jsonc-edit` and its fuzz sidecar.
`native:lock`,
the plain `cargo generate-lockfile --offline` of `package/git-policy/cli`,
was tried first and rewrote 608 lines,
so it was discarded.

### monochromatic-lint 0.1.0

Published on 2026-10-10 from the cutover branch.
The 0.1.0 release (dispatch run 38051569901, OIDC, eight archives) installs only by compiling,
because `cargo-binstall` does not look under a `monochromatic-lint-v<version>` tag.
0.1.1 adds `[package.metadata.binstall]` and is the version the repository pins:
dispatch run 38053428535 published it and created the release with eight archives,
and `MISE_CARGO_BINSTALL_ONLY=1 mise install cargo:monochromatic-lint` finished in 11.5 s.
Evidence and root cause:
[`doc/troubleshooting/mise-cargo-binstall-falls-back-to-compiling.md`](../troubleshooting/mise-cargo-binstall-falls-back-to-compiling.md).

## Publish workflow

`.github/workflows/cargo-publish.yml` gained `ml-detect`,
`ml-publish-crate`,
`ml-build-binary`,
and `ml-create-release`,
a `monochromatic-lint` dispatch choice,
and `package/linter/monochromatic-lint/Cargo.toml` as a push path.
They mirror the `forbidden-strings` jobs:
the same eight targets,
tag `monochromatic-lint-v<version>`,
and assets `monochromatic-lint-<version>-<target>.tar.gz` (`.zip` on Windows)
holding `monochromatic-lint-<version>-<target>/monochromatic-lint` and the README,
which is cargo-binstall's default layout.
Two differences:
the publish step skips a version already on crates.io,
because 0.1.0 is bootstrapped by hand,
and the binary matrix also runs in a dry run,
so a dry dispatch shows which targets build.
Attestation and the release stay behind `!inputs.dry-run`.
The matrix compiles with each runner's preinstalled rustup toolchain,
like the `forbidden-strings` matrix,
not with the repository's mise nightly.

The crate manifest now carries the release profile the incumbent Rust linter shipped with
(`lto = true`,
`codegen-units = 1`,
`opt-level = 3`,
`strip = true`),
and `file-enforcer.config.ts` maps `package/linter/monochromatic-lint` to that profile.
A control run set `opt-level = 2`,
ran `mise run file-enforcer`,
and saw it restored to 3.

### Incident in the jsonc-edit publish run

The push of `main` from `173e5fe2a` to `533574e1f` spanned two generations,
because local `main` held 16 commits another session had not pushed.
With `fetch-depth: 2`,
the pre-push head was not in the checkout,
and every detect job read its absence as a version bump.
The jobs for unchanged crates then ran:
`forbidden-strings` 0.4.1 was refused by crates.io as a duplicate,
`forbidden-regex` and `monochromatic-nested-wayland-session` skipped their existing versions,
both release jobs were refused because the tags exist,
and 13 build-provenance attestations were created for rebuilt bytes that were never published.
No crates.io version or GitHub release changed.
A `gh run cancel` was refused by this session's permission classifier.

Every detect step now fetches the missing commit and fails closed when it cannot.
Reproduction,
root cause,
and the before-and-after evidence:
[`doc/troubleshooting/cargo-publish-shallow-before-sha.md`](../troubleshooting/cargo-publish-shallow-before-sha.md).

## Release targets

Both releases carry eight archives:
`aarch64-apple-darwin`, `aarch64-pc-windows-msvc`, `aarch64-unknown-linux-gnu`, `aarch64-unknown-linux-musl`,
`x86_64-apple-darwin`, `x86_64-pc-windows-msvc`, `x86_64-unknown-linux-gnu` and `x86_64-unknown-linux-musl`
(`gh release view monochromatic-lint-v0.1.1`).
Only `x86_64-unknown-linux-gnu` was installed and run.

## Landing choreography

Landed on 2026-10-10 as a fast-forward to merge `b111c0ca39`, pushed.
The order that avoided a window in which commits failed:

1. Built the wrapper's `dist` in the branch worktree from the branch tree
   (`mise run //package/git-policy/cli:build:js:node`).
   The installed wrapper runs the repository's `dist`,
   whose old Markdown policy runs the deleted CLI,
   so it had to be replaced
   at the same moment the deletion landed.
2. Committed two files other sessions had left uncommitted in the main checkout
   (`pnpm-lock.yaml`, `package/config/pnpr/config.yaml`),
   with the user's approval, because the fast-forward touched both.
3. Fast-forwarded `main`, copied the built `dist` over the live one by rename, ran `git cli-git trust --yes`
   for the changed `cli-git.config.ts` (the user approved this on 2026-10-10) and ran the pinned tool install.

The user approved the trust step.
Commits made on the branch used `--no-enforce-markdown/autofix` while the installed wrapper still ran the deleted CLI.

## Cutover change

The `markdown/autofix` options in `cli-git.config.ts` now take the policy defaults:
`monochromatic-lint` on `PATH` and the `markdown/lfs-image-url` rule, with `package/ssg/` excluded.
A smoke test in a disposable repository with a disposable home passed (`git add` of a Markdown file exits 0).
The tool is declared as `"cargo:monochromatic-lint" = "0.1.1"` in `mise.no-env.toml`
with `binstall_native = false` under `[settings.cargo]`.
Leftovers: `package/cli/markdown-lint/dist` and similar ignored build directories of the deleted packages remain on disk
in the main checkout and are not tracked.

## AGENTS.md rules

The plan names `MXR` and `RDC`.
Neither code exists in `AGENTS.md` at the branch base or in `forbidden-strings.append.txt`
or the main checkout's `forbidden-strings.append.local.txt`.
Commit `d03ec673e` (`docs(AGENTS.md): apply rule-by-rule optimization`, 2026-09-29) removed both:
`MXR`,
which named `monochromatic-rust-linter`'s `max-lines` rule,
its 300-line budget,
and its exemptions,
was folded into the tool-agnostic `MXL` rule,
which now reads "Over max-lines (TS, Rust): split into sibling files/modules";
`RDC`,
which required rustdoc on every documentable item,
was dropped without a replacement.
No current `AGENTS.md` rule names an incumbent linter,
so the cutover changes no rule.
Recording the two retired codes in `forbidden-strings.append.local.txt`,
as the `CRN` rule asks,
is left to the main session,
because that file is local to the main checkout.

## Consumer differential

Both incumbents ran read-only against this worktree,
 without `--fix`,
on the file set the new linter walks.
The Markdown incumbent is `package/cli/markdown-lint/src/cli.ts` from the main checkout,
run with `--format=json --lfs-image-exclude=package/ssg/`,
 the flags of the `lint:markdown` task.
The Rust incumbent is `package/linter/rust`,
built from `main` source with `cargo build --release --locked --offline` in a scratch directory,
because the prebuilt binary in the main checkout predates two `main` commits
(`76b72979d` and `365a68255`);
the fresh build and the prebuilt binary print identical findings.
The new linter is `monochromatic-lint .` with the worktree configuration.

### Totals before the configuration change

- The new linter reported 56,404 errors and 3,347 warnings.
  Of the errors,
   397 are `rust/require-rustdoc` and 56,007 are Markdown.
- The incumbent Markdown linter reported 55,144 findings:
  53,448 `semantic-line-breaks`,
  1,684 `MD034`,
  5 `MD001`,
  2 `MD054`,
  2 `MD026`,
  1 `no-pipe-tables`,
  1 `MD053`,
  and 1 `MD040`.
- The incumbent Rust linter reported 651 findings:
  646 `builtin(require-rustdoc)`
  and 5 `builtin(max-lines)`,
  all in `.rs` files.
- The 402 errors from the earlier partial count are the 397 Rust rustdoc errors
  and the 5 `markdown/heading-increment` errors.
  The Markdown rules that make up the rest of the count are not in that figure.

### Why the incumbents did not gate the Rust findings

- `package/git-policy/cli/src/native`,
   268 errors:
  the incumbent task `native:lint:rust` (`package/git-policy/cli/mise.toml:43` on `main`) was never reached
  by any aggregate task.
  The handover `doc/handover/cli-git-native-policy-engine.md`,
   section
  "The repository's Rust linter is not part of the gate",
   records that decision.
  The incumbent reports the same 268 findings when run on that path.
- `package/linter/monochromatic-lint/src`,
   83 errors:
  the incumbent task `lint:rust` (`package/linter/monochromatic-lint/mise.toml:21` on `main`) exists,
  but that package's `lint` is `cargo check` (`mise.toml:13` to `15`),
  and the root `lint` fanout visits only `//package/` tasks (`mise.toml:335` on `main`).
  The incumbent reports the same 83 findings when run on that path.
- `doc/troubleshooting/slint-listview-random-seek-probe.rs`,
   46 errors:
  no package owns `doc/`,
   so the root fanout (`mise.toml:335` on `main`) never reached it.
  The incumbent reports the same 46 findings when run on that path.
- Fuzz sidecars (`*.fuzz`) and `doc/audit/resharp-fuzz-2026-06-19`:
  the incumbent reports 249 `require-rustdoc` and 5 `max-lines` findings there.
  The new configuration already exempts these paths (`rust-exemptions`),
  and none of the four fuzz packages has a `lint:rust` task on `main`.

### Markdown differences

- The new linter reports 662 more `semantic-line-breaks` and 201 more `no-bare-urls` than the incumbent.
  All 201 bare-URL differences and 539 of the 662 semantic differences are prose findings in
  10 files that contain supplementary-plane characters (emoji and similar).
  The incumbent stops reporting some later findings after such a character.
  Reproduction:
   `${HOME}/temp/agent/momoa-probe/e1.md` has an emoji and a later bare URL;
  the incumbent prints nothing for it,
   while `e2.md` (the same text with an ASCII character) gets a finding.
  The new linter is correct on these,
   so the incumbent under-reports them.
- The remaining 123 semantic differences are `///` doc-comment lines inside fenced Rust blocks
  in 32 Markdown files.
  The incumbent treats fenced code as code and reports nothing there;
  the new linter reads the doc comments as prose.
  Reproduction:
   `${HOME}/temp/agent/momoa-probe/fence.md`.
  The new linter is wrong on these,
   and `monochromatic-lint --rules` lists no options for
  `markdown/semantic-line-breaks`,
   so no configuration key expresses the exception and they stay open.
- Every other per-file,
   per-rule Markdown count is identical between the two linters.

### Configuration change this differential produced

- A new block `old-gate-scope` in `monochromatic-lint.config.jsonc` turns `rust/require-rustdoc` off
  for `package/git-policy/cli/src/native/**/*.rs` and `doc/troubleshooting/**/*.rs`,
  the two paths the incumbent gate never ran.
  Its comment cites the handover and the root `lint:rust` fanout filter.
  No severity changed.
- After the change,
   errors fall from 56,404 to 56,090,
   the 314 removed being the 268 git-policy
  and 46 probe findings.
  The 83 `monochromatic-lint` findings stay,
   because the owner has to decide whether that package
  is inside the gate (see below).

## Deleted paths

The five incumbent packages,
 deleted by commit `5bc05682e` on this branch:

- `package/cli/markdown-lint` (71 files)
- `package/linter/rust` (23 files)
- `package/rust-linter-plugin/builtin` (12 files)
- `package/rust-module/rust-linter-core` (34 files)
- `package/rust-module/rust-linter-pattern` (13 files)

The same commit also edited `package/config/pnpr` and `package/dev-script/file-enforcer`
to drop their registrations.

The sweep ran `rg --no-config --hidden --line-number --fixed-strings` over the worktree
for each package path,
 each crate and package name,
and the names `rust-linter.toml` and `markdown-lint/`,
excluding `doc/handover/**`,
 `.git`,
 `node_modules`,
 `target` build output,
 and `*.local.*` files.
The deleted directories are absent,
 so they need no exclusion.

The sweep found one live reference:

- `cli-git.config.ts:38` runs `node package/cli/markdown-lint/src/cli.ts` for the `markdown/autofix` policy.
  This is the production commit wrapper,
  which still calls the deleted linter until the landing step.
  It is left for the landing choreography,
   not changed on the branch.

The sweep found references that do not run the deleted code:

- Six Rust doc comments in `package/linter/monochromatic-lint/src/markdown_lfs_*_tests.rs`
  cite deleted `*.unit.test.ts` files as provenance.
- `package/linter/monochromatic-lint/fixtures/semantic-break-parity.json` and
  `fixtures/lfs-url-parity.json` record hashes of the deleted sources.
  Tests read them with `include_str!`,
   so they stay as measured evidence.
- `package/linter/monochromatic-lint/README.md:7` says the crate replaces the deleted packages.
- `package/rust-module/forbidden-regex/counting-automaton.md:1155` names the old tool.
- 199 matching lines under `doc/` (outside `doc/handover`) are historical planning,
   audit,
  troubleshooting,
   and decision records.
  Some of those lines name the unrelated `package/git-policy/markdown-lint`.
  Records keep their text under the durable-record rules,
   so none were edited here.

The git-policy packages named `markdown-lint` (`package/git-policy/markdown-lint` and its
`package/git-policy/cli/src/optional/markdown-lint` copy) are different packages and were not deleted.

## Left for step 8

- Snippet warnings:
   1,314 `rust/require-rustdoc` warnings in Rust fences inside Markdown,
  under the `snippet-burn-down` block.
  The plan names 905 snippet findings,
  and this run measures a different count,
   so step 8 has to reconcile the two.
- `rust/no-anonymous-functions`:
   2,033 warnings,
   1,812 in `.rs` files and 221 in Markdown fences.
  The rule has no incumbent counterpart,
   and the owner decision of 2026-10-05 keeps it a warning.
- `markdown/semantic-line-breaks` inside fenced Rust doc comments:
   123 errors in 32 files.
  This is the new-linter over-report described above.
- Shared Markdown errors,
   reported by the incumbent too (group b,
   not fixed here):
  53,987 prose `semantic-line-breaks`,
   of which 539 are incumbent false negatives,
  so 53,448 are shared;
  1,885 `no-bare-urls`,
   of which 1,684 are shared and 201 are incumbent false negatives;
  5 `heading-increment`,
   at `doc/audit/tech-monorepo-manager-vet-2026-09-16/rescreen-result-1.md:20`,
  `screening-primary-chunk-1.md:21`,
   `screening-primary-chunk-2.md:17`,
  `screening-registry-chunk-1.md:17`,
   and `screening-registry-chunk-2.md:31`;
  and 2 `no-trailing-punctuation`,
   2 `link-image-style`,
   1 `no-pipe-tables`,
  1 `link-image-reference-definitions`,
   and 1 `fenced-code-language`,
   identical to the incumbent.
- `rust/require-rustdoc` errors in `package/linter/monochromatic-lint/src`:
   83 errors,
   unexempted.
  The owner has to decide whether this package is inside the gate.
  If it is,
   the 83 items need rustdoc.
  If it is not,
   an `old-gate-scope` path can cover it,
   as it does for git-policy.
- `package/git-policy/cli/src/native`:
   268 rustdoc errors are exempted by `old-gate-scope`,
  and they stay real findings the owner has not fixed (the older handover counted 263).
- `doc/troubleshooting/slint-listview-random-seek-probe.rs`:
   46 rustdoc errors are exempted by `old-gate-scope`.
  The probe is not a package,
   so the exemption follows the existing `doc/audit` precedent.
