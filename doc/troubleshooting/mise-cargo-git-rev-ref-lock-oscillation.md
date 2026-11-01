# mise 2026.6.13 alternates cargo-git `rev:` and `ref:` lock entries for Slint tools

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

> Toolchain status:
>  `doc/decision/mpm-toolchain-adoption.md` (2026-10-26) moved tool provisioning off
>  Mise `[tools]` and `mise.lock`,
>  so this record is historical for the Mise-era toolchain it investigated.
>  Live provisioning runs `mise run toolchain:install` against `toolchain.lock.json`
>  (`doc/troubleshooting/mpm-toolchain-migration.md`).

mise 2026.6.13,
 cargo backend,
 `mise lock`,
 and lockfile-enabled config with a cargo-git
`rev:<commit>` tool can write both `rev:<commit>` and `ref:<commit>` entries for the same
resolved git revision.
Later lockfile update paths can drop one representation again,
 so
`mise.lock` appears to oscillate randomly across otherwise unrelated commits.

## Symptom

In this repo,
 `mise.lock` repeatedly changed around the Slint cargo-git tools without a matching
version decision:

```text
227917de79 duplicate ref present  chore(mise): update tool lockfile
4255b63912 only rev entries       chore(*): update pi catalog and lockfiles
cb5baf54c5 duplicate ref present  chore(*): refresh mise.lock
150e5e03ca only rev entries       chore(*): update pi catalog and tooling locks
c60f1d6440 duplicate ref present  chore(*): enable mise lockfile
c0ac426d1e only rev entries       chore(*): refresh mise and pi tooling
```

The concrete diff shape is a pair of extra lockfile entries like this:

```diff
# mise.lock
+[[tools."cargo:https://github.com/slint-ui/slint"]]
+version = "ref:85e3eb76819762cdcaa732fa87533ff896546bac"
+backend = "cargo:https://github.com/slint-ui/slint"
+
+[tools."cargo:https://github.com/slint-ui/slint".options]
+bin = "slint-lsp"
+crate = "slint-lsp"
```

Before the local mitigation,
 the same pattern also affected the second Slint tool key,
`cargo:https://github.com/slint-ui/slint.git`,
 used only to install `slint-viewer` from the same
upstream repository.

## Root cause

The first representation comes from our config.
`mise.no-env.toml` requested cargo-git `rev:`
versions for Slint:

```toml
# mise.no-env.toml
"cargo:https://github.com/slint-ui/slint" = { version = "rev:85e3eb76819762cdcaa732fa87533ff896546bac", crate = "slint-lsp", bin = "slint-lsp" }
"cargo:https://github.com/slint-ui/slint.git" = { version = "rev:85e3eb76819762cdcaa732fa87533ff896546bac", crate = "slint-viewer", bin = "slint-viewer" }
```

The second representation comes from mise's cargo-git normalization.
In `jdx/mise` commit
`d1fb2de4844d020a5fa3a17609432f79ef544897`,
 `latest_version_with_opts()` rewrites `tag:`,
`branch:`,
 and `rev:` requests into `ref:` strings:

`src/toolset/tool_version.rs:261-266`

```rust
// map cargo backend specific prefixes to ref
let version = match tv.request.version().split_once(':') {
    Some((_ref_type @ ("tag" | "branch" | "rev"), r)) => {
        format!("ref:{r}")
    }
    _ => tv.version,
};
```

`mise lock` then collects tools from more than one path.
In the first pass,
 it takes already
resolved versions from the active toolset and deduplicates by literal `(tool short, version)`:

`src/cli/lock.rs:541-548`

```rust
// Skip unresolved symbolic versions (e.g., a lockfile poisoned with "latest"
// as the version). Pass 2's fallback will resolve these to a concrete version.
if tv.version == "latest" {
    continue;
}
let key = (backend.ba().short.clone(), tv.version.clone());
if seen.insert(key) {
    all_tools.push((backend.ba().as_ref().clone(), tv));
}
```

In the second pass,
 it walks the config file requests.
If a resolved request matches,
 it again
uses the literal version string as part of the dedupe key:

`src/cli/lock.rs:570-578`

```rust
for tv in &resolved_tv.versions {
    if request_matches(&tv.request, request)
        && tv.version != "latest"
    {
        matched_resolved = true;
        let key = (ba.short.clone(), tv.version.clone());
        if seen.insert(key) {
            all_tools.push((ba.as_ref().clone(), tv.clone()));
        }
    }
}
```

If mise has to resolve the request directly,
 that branch uses the same literal key:

`src/cli/lock.rs:620-624`

```rust
match request.resolve(config, &resolve_options).await {
    Ok(tv) => {
        let key = (ba.short.clone(), tv.version.clone());
        if seen.insert(key) {
            all_tools.push((ba.as_ref().clone(), tv));
```

So `rev:85e3...` and `ref:85e3...` are considered different versions of the same tool during
`mise lock`,
 even though cargo-git treats them as the same commit identity for this request.

Later,
 lockfile update paths rebuild entries from the active toolset and merge by literal
`(version, options)`.
Existing entries are only preserved if their literal key appears in the new
entry set:

`src/lockfile.rs:1941-1967`

```rust
fn merge_tool_entries(
    entries: Vec<LockfileTool>,
    existing_tools: Option<&Vec<LockfileTool>>,
) -> Vec<LockfileTool> {
    // Group by (version, options) - the key for deduplication
    let mut by_key: HashMap<(String, BTreeMap<String, String>), LockfileTool> = HashMap::new();

    for tool in entries {
        let key = (tool.version.clone(), tool.options.clone());
        let entry = by_key.entry(key).or_insert_with(|| tool.clone());

        // Merge platforms - properly combine platform info to preserve URLs and prefer sha256
        for (platform, info) in tool.platforms {
            entry
                .platforms
                .entry(platform)
                .and_modify(|existing| *existing = info.merge_with(existing))
                .or_insert(info);
        }
    }

    // Merge with existing tools to preserve platform info
    if let Some(existing) = existing_tools {
        for existing_tool in existing {
            let key = (existing_tool.version.clone(), existing_tool.options.clone());
            if let Some(entry) = by_key.get_mut(&key) {
```

That explains the flip:

- `mise lock` can add `ref:<commit>` beside config-originated `rev:<commit>`.
- Other lockfile update paths can rebuild only the active representation and drop the unmatched
  existing representation.
- The sequence depends on which mise command touched the lockfile last,
   so the commit history looks
  random even though each command is deterministic.

## Verification

Versions under test:

- local mise:
  `2026.6.13 linux-x64 (2026-06-23)`;
- mise source:
  `jdx/mise@d1fb2de4844d020a5fa3a17609432f79ef544897`;
- repo config source before mitigation:
  `mise.no-env.toml` with both `slint-lsp` and
  `slint-viewer` cargo-git `rev:` tools;
- no-duplicate baseline commit:
  `205f14b845`.

### Reproduce the `mise lock` add side

Run this in a disposable worktree,
 not in the main checkout:

```bash
# /var/home/user/Monochromatic
mkdir --parents /tmp/agent
chmod 700 /tmp/agent
worktree=$(mktemp --directory /tmp/agent/mise-lock-repro.XXXXXXXX)
rmdir "$worktree"
git worktree add --detach "$worktree" 205f14b845
cd "$worktree"
export MISE_TRUSTED_CONFIG_PATHS="$worktree/mise.toml"
mise lock \
  'cargo:https://github.com/slint-ui/slint' \
  'cargo:https://github.com/slint-ui/slint.git'
git diff --unified=0 -- mise.lock
```

Observed output included four processed Slint lock targets:

```text
→ Processing 4 tool(s): cargo:https://github.com/slint-ui/slint@ref:85e3eb76819762cdcaa732fa87533ff896546bac, cargo:https://github.com/slint-ui/slint.git@ref:85e3eb76819762cdcaa732fa87533ff896546bac, cargo:https://github.com/slint-ui/slint@rev:85e3eb76819762cdcaa732fa87533ff896546bac, cargo:https://github.com/slint-ui/slint.git@rev:85e3eb76819762cdcaa732fa87533ff896546bac
```

The diff added exactly the two `ref:` entries that were absent from the baseline:

```diff
# mise.lock
+[[tools."cargo:https://github.com/slint-ui/slint"]]
+version = "ref:85e3eb76819762cdcaa732fa87533ff896546bac"
+backend = "cargo:https://github.com/slint-ui/slint"
+
+[tools."cargo:https://github.com/slint-ui/slint".options]
+bin = "slint-lsp"
+crate = "slint-lsp"
+
+[[tools."cargo:https://github.com/slint-ui/slint.git"]]
+version = "ref:85e3eb76819762cdcaa732fa87533ff896546bac"
+backend = "cargo:https://github.com/slint-ui/slint.git"
+
+[tools."cargo:https://github.com/slint-ui/slint.git".options]
+bin = "slint-viewer"
+crate = "slint-viewer"
```

### Reproduce the removal side

A separate disposable worktree confirmed the lockfile update path can remove the `ref:` entries.
The harness used disposable mise state so no installed tool state in the user's profile was changed:

```bash
# /var/home/user/Monochromatic
mkdir --parents /tmp/agent
chmod 700 /tmp/agent
worktree=$(mktemp --directory /tmp/agent/mise-lock-remove-repro.XXXXXXXX)
rmdir "$worktree"
git worktree add --detach "$worktree" 11d6da57b
scratch=$(mktemp --directory /tmp/agent/mise-state.XXXXXXXX)
cd "$worktree"
printf '' > "$scratch/global.toml"
export MISE_TRUSTED_CONFIG_PATHS="$worktree/mise.toml"
export MISE_GLOBAL_CONFIG_FILE="$scratch/global.toml"
export MISE_DATA_DIR="$scratch/data"
export MISE_CACHE_DIR="$scratch/cache"
export MISE_STATE_DIR="$scratch/state"
export MISE_CONFIG_DIR="$scratch/config"
mise unuse npm:pagefind
git diff --unified=0 -- mise.lock
```

The relevant diff removed the `ref:` Slint entries:

```diff
# mise.lock
-[[tools."cargo:https://github.com/slint-ui/slint"]]
-version = "ref:85e3eb76819762cdcaa732fa87533ff896546bac"
-backend = "cargo:https://github.com/slint-ui/slint"
-
-[tools."cargo:https://github.com/slint-ui/slint".options]
-bin = "slint-lsp"
-crate = "slint-lsp"
-
-[[tools."cargo:https://github.com/slint-ui/slint.git"]]
-version = "ref:85e3eb76819762cdcaa732fa87533ff896546bac"
-backend = "cargo:https://github.com/slint-ui/slint.git"
-
-[tools."cargo:https://github.com/slint-ui/slint.git".options]
-bin = "slint-viewer"
-crate = "slint-viewer"
```

### Catalog

Works cleanly:

- Removing a cargo-git tool from `mise.no-env.toml`,
   regenerating `mise.toml`,
   and running
  `mise lock` prunes that stale tool.
- Moving the Slint tools to crates.
  io entries avoids the cargo-git `rev:` / `ref:` identity split.
  `mise lock --dry-run 'cargo:slint-lsp' 'cargo:slint-viewer'` processes exactly
  `cargo:slint-lsp@1.17.0` and `cargo:slint-viewer@1.17.0`.

Still churns on the historical config:

- Keeping `slint-lsp` as a cargo-git `rev:` tool still leaves mise processing both
  `cargo:https://github.com/slint-ui/slint@ref:85e3...` and
  `cargo:https://github.com/slint-ui/slint@rev:85e3...` in `mise lock --dry-run`.
- Running `mise lock` from a no-duplicate baseline can recreate `ref:` entries.

## Verified workarounds

### Use crates.io Slint tool entries once a matching Slint release exists

This is the workaround now applied locally for both Slint tools:

```toml
# mise.no-env.toml
"cargo:slint-lsp" = "1.17.0"
"cargo:slint-viewer" = "1.17.0"
```

Then regenerate the generated config and refresh the lockfile:

```bash
# /var/home/user/Monochromatic
mise run sync:files
mise lock 'cargo:slint-lsp' 'cargo:slint-viewer'
```

Tradeoffs:

- `slint-viewer` is installed again because it no longer needs the duplicate
  cargo-git tool key.
- This removes both Slint cargo-git tool keys from config and lockfile,
  so future lock churn cannot include the `rev:` / `ref:` Slint entries.
- This became available only after Slint 1.17.0 shipped matching crates.
  io
  releases for the runtime crates and the tools.

## What does not work

- Treating this as random lockfile churn.
  The add and remove sides are deterministic,
   triggered by
  different mise lockfile code paths.
- Editing only `mise.toml`.
  It is generated from `mise.no-env.toml` by `file-enforcer.config.ts`,
   so
  the source file must change first,
   then `mise run sync:files` must regenerate it.
- Running plain `mise lock` while keeping a cargo-git `rev:` Slint tool.
  The dry-run still lists both
  `ref:` and `rev:` for `slint-lsp`.
- Removing only the `ref:` entries by hand.
  `mise lock` can recreate them from the installed
  `ref:` representation.

## Upstream filing decision

`.out-of-scope/` has no exemption matching mise,
 cargo-git lockfiles,
 or Slint.
Checked files:

- `.out-of-scope/bun-install.md`
- `.out-of-scope/cargo-workspace.md`
- `.out-of-scope/claude-code-upstream-bugs.md`
- `.out-of-scope/codex-harness.md`
- `.out-of-scope/jsr.md`
- `.out-of-scope/lightningcss.md`
- `.out-of-scope/low-impact-typescript-formatting.md`
- `.out-of-scope/module-es-monolith.md`
- `.out-of-scope/pi-gpt55-long-context.md`
- `.out-of-scope/terminal-title-fork-parity-tests.md`
- `.out-of-scope/typescript-project-references.md`

Duplicate search found no matching issue or pull request:

```bash
gh search issues --repo jdx/mise 'cargo git rev ref lockfile mise lock' --state open --limit 20
gh search issues --repo jdx/mise 'cargo git rev ref lockfile mise lock' --state closed --limit 20
gh search prs --repo jdx/mise 'cargo git rev ref lockfile mise lock' --state open --limit 20
gh search prs --repo jdx/mise 'cargo git rev ref lockfile mise lock' --state closed --limit 20
```

The commands returned no rows.

Walking the six constraints:

- Really upstream's fault?
  Yes for the inconsistent lock identity.
  Our config is valid cargo-git `rev:` syntax,
   and mise itself writes both
  `rev:` and `ref:` forms.
- Can upstream fix it?
  Yes.
  The identity can be normalized in `mise lock`,
   in lockfile merge keys,
   or in
  cargo-git version canonicalization.
- Are they supporting this use case?
  Mostly yes.
  mise documents lockfiles and has a cargo backend that accepts cargo-git
  `rev:` requests.
- Would the repo welcome our contribution?
  Yes for a discussion-first contribution path,
   not for a direct issue.
  `.github/ISSUE_TEMPLATE/config.yml` sets `blank_issues_enabled: false` and
  directs features,
   bug reports,
   questions,
   configuration,
   features,
   and behavior
  to GitHub Discussions.
  `CONTRIBUTING.md` points to the web guide,
   and the guide says to start a
  discussion or mention the plan in Discord before non-obvious PRs.
  It welcomes public pull requests after direction is settled.
- Will they likely fix it?
  Soft yes.
  No matching issue or PR was found,
   and no maintainer text was found declining
  this class of lockfile fix.
- Prototyped a minimal fix?
  Yes.
  The prototype patch is saved beside this document as
  [mise-cargo-git-rev-ref-lock-oscillation.patch](mise-cargo-git-rev-ref-lock-oscillation.patch).

### Prototype result

The prototype introduces `ToolVersion::lockfile_version()`.
It preserves the request spelling when a cargo-git `tag:`,
 `branch:`,
 or `rev:`
request resolves to the equivalent `ref:` install identity,
 then uses that value
when `mise lock` deduplicates,
 prunes stale versions,
 resolves lock data,
 and
serializes `LockfileTool` records.

The disposable upstream clone was created fresh under `/tmp/agent/`:

```bash
# /var/home/user/Monochromatic
mkdir --parents /tmp/agent
chmod 700 /tmp/agent
fresh=$(mktemp --directory /tmp/agent/mise-upstream-prototype.XXXXXXXX)
rmdir "$fresh"
gh repo clone jdx/mise "$fresh" -- --depth 1
cd "$fresh"
git remote get-url origin
# https://github.com/jdx/mise.git
git rev-parse HEAD
# d1fb2de4844d020a5fa3a17609432f79ef544897
git apply /var/home/user/Monochromatic/doc/troubleshooting/mise-cargo-git-rev-ref-lock-oscillation.patch
git diff --stat
# src/cli/lock.rs             | 92 +++++++++++++++++++++++++++++++++++++++++----
# src/lockfile.rs             |  4 +-
# src/toolset/tool_version.rs | 54 ++++++++++++++++++++++++++
# 3 files changed, 140 insertions(+), 10 deletions(-)
```

Targeted verification ran in a secret-free environment outside this repository,
with Cargo writing build artifacts under `/tmp/agent/`:

```bash
# /tmp/agent
env -i \
  HOME=/tmp/agent/mise-proto-home \
  CARGO_HOME=/tmp/agent/mise-proto-home/.cargo \
  RUSTUP_HOME=/home/user/.rustup \
  RUSTUP_TOOLCHAIN=nightly-x86_64-unknown-linux-gnu \
  CARGO_TARGET_DIR=/tmp/agent/mise-20260624-lock-oscillation/target \
  PATH=/home/user/.cargo/bin:/usr/bin:/bin \
  cargo test --manifest-path /tmp/agent/mise-upstream-prototype.DzUfrrWU/Cargo.toml cargo_git
```

The first prototype test run caught an over-broad unit-test seam that called
backend registry state before initialization.
That test was moved to `toolset::tool_version`,
 where it exercises the new
normalization helper without needing registry state.
The corrected fresh-clone run passed:

```text
running 4 tests
test toolset::tool_version::tests::lockfile_version_keeps_non_matching_cargo_git_ref ... ok
test toolset::tool_version::tests::lockfile_version_preserves_requested_cargo_git_rev ... ok
test cli::lock::tests::test_current_versions_preserve_requested_cargo_git_rev ... ok
test cli::lock::tests::test_prune_stale_versions_removes_cargo_git_ref_duplicate ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 1409 filtered out; finished in 0.00s
```

Decision:
do not file a GitHub issue because the repo disables blank issues and routes bug
reports to Discussions.
The draft below is appropriate for a Discussion or for a PR description after a
human decides to upstream the prototype.

### Discussion draft, do not post as-is

~~~md
Title: `mise lock` alternates cargo-git `rev:` and `ref:` entries for the same locked revision

I have a project lockfile with a cargo-git tool configured as:

```toml
"cargo:https://github.com/slint-ui/slint" = { version = "rev:85e3eb76819762cdcaa732fa87533ff896546bac",
crate = "slint-lsp", bin = "slint-lsp" }
```

With mise 2026.6.13, different lockfile update paths alternate between one entry and two entries for
the same commit:

```toml
[[tools."cargo:https://github.com/slint-ui/slint"]]
version = "rev:85e3eb76819762cdcaa732fa87533ff896546bac"
backend = "cargo:https://github.com/slint-ui/slint"

[[tools."cargo:https://github.com/slint-ui/slint"]]
version = "ref:85e3eb76819762cdcaa732fa87533ff896546bac"
backend = "cargo:https://github.com/slint-ui/slint"
```

Source trace from `jdx/mise@d1fb2de`:

- `src/toolset/tool_version.rs:261-266` maps cargo-git `rev:`, `tag:`, and `branch:` requests to
  `ref:<value>` in `latest_version_with_opts()`.
- `src/cli/lock.rs:541-548`, `src/cli/lock.rs:570-578`, and `src/cli/lock.rs:620-624` deduplicate
  `mise lock` targets by literal `(tool short, version)`, so `rev:<sha>` and `ref:<sha>` survive as
  distinct versions.
- `src/lockfile.rs:1941-1967` merges lockfile entries by literal `(version, options)` and preserves
  only existing entries whose literal key appears in the new entry set, so other update paths can
  drop one representation again.

Reproduction from a repo state with only `rev:` entries:

```bash
git worktree add --detach "$WT" 205f14b845
cd "$WT"
export MISE_TRUSTED_CONFIG_PATHS="$WT/mise.toml"
mise lock 'cargo:https://github.com/slint-ui/slint'
git diff --unified=0 -- mise.lock
```

Observed: `mise lock` adds the matching `ref:<sha>` entry beside the existing `rev:<sha>` entry.

I prototyped a minimal fix that adds `ToolVersion::lockfile_version()` and uses it in the lockfile
paths that deduplicate, prune stale versions, resolve lock data, and serialize lockfile tools.
The targeted tests pass:

```text
running 4 tests
test toolset::tool_version::tests::lockfile_version_keeps_non_matching_cargo_git_ref ... ok
test toolset::tool_version::tests::lockfile_version_preserves_requested_cargo_git_rev ... ok
test cli::lock::tests::test_current_versions_preserve_requested_cargo_git_rev ... ok
test cli::lock::tests::test_prune_stale_versions_removes_cargo_git_ref_duplicate ... ok
```

Question: should cargo-git lock identity be normalized so `rev:<sha>` and `ref:<sha>` for the same
resolved commit do not appear as separate lockfile versions?
~~~
