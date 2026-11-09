# Clippy 0.1.98 configuration docs: unstable `clippy.toml` note hides stable Cargo lint levels

## Symptom

The stable Clippy configuration page opens with this warning:

```text
Note: The configuration file is unstable and may be deprecated in the future.
```

That warning appears before the page explains which file it means.
A reader trying to decide where to put a lint policy can reasonably infer one of these wrong conclusions:

- Clippy expects projects not to configure lint policy.
- Cargo's `[lints.clippy]` table might eventually absorb every `clippy.toml` option.
- `clippy.toml` might already be unsupported.

The actual split is narrower:

- Cargo's `[lints.clippy]` table is stable for lint levels such as `allow`,
   `warn`,
   `deny`,
   and `forbid`.
- Clippy's `clippy.toml` or `.clippy.toml` file still supplies lint-specific parameters such as
  `disallowed-methods`,
   `allow-unwrap-in-tests`,
   and `allow-unwrap-types`.
- Cargo currently does not forward arbitrary Clippy lint parameters from `[lints.clippy]`.

The surface error variants in a runnable harness are:

- `clippy::unwrap_used` errors when `[lints.clippy] unwrap_used = "deny"` sees `Result::unwrap`,
  `Result::unwrap_err`,
   or `Option::unwrap`.
- `clippy::disallowed_methods` errors when `clippy.toml` configures `std::result::Result::unwrap`.
- Cargo warns `unused manifest key` when `disallowed-methods` is put under
  `[lints.clippy.disallowed_methods]`,
   and then no method ban is enforced.

## Root cause

The confusion is documentation wording,
 not an indication that Clippy configuration has stopped working.
The page titled "Configuring Clippy" starts with an unstable-file warning before saying that the file is
`clippy.toml` or `.clippy.toml`.

From rust-clippy source,
commit `64c7431d6cd823d1a7663165c7e59d78e6dc726a`,
 cloned from
`https://github.com/rust-lang/rust-clippy.git`:

`rust-clippy/book/src/configuration.md:1`

```md
# Configuring Clippy
```

`rust-clippy/book/src/configuration.md:3`

```md
> **Note:** The configuration file is unstable and may be deprecated in the future.
```

Only after that note does the page name the specific file format.

`rust-clippy/book/src/configuration.md:5`

```md
Some lints can be configured in a TOML file named `clippy.toml` or `.clippy.toml`, which is searched for starting in the
```

The same page later does point to stable Cargo lint levels,
but the link is separated from the warning and uses a different subsection.

`rust-clippy/book/src/configuration.md:89`

```md
#### Lints Section in `Cargo.toml`
```

`rust-clippy/book/src/configuration.md:91`

```md
Finally, lints can be allowed/denied using [the lints
section](https://doc.rust-lang.org/nightly/cargo/reference/manifest.html#the-lints-section) in the `Cargo.toml` file:
```

Cargo's own docs describe `[lints]` as a level override mechanism.
They do not describe it as a general Clippy parameter file.

From Cargo source,
commit `a335d47ff8036918d3d548dabd513dc0444096a9`,
 cloned from
`https://github.com/rust-lang/cargo.git`:

`cargo/src/doc/src/reference/manifest.md:526`

```md
## The `[lints]` section
```

`cargo/src/doc/src/reference/manifest.md:528`

```md
Override the default level of lints from different tools by assigning them to a new level in a
```

`cargo/src/doc/src/reference/manifest.md:541`

```md
`level` corresponds to the [lint levels](https://doc.rust-lang.org/rustc/lints/levels.html) in `rustc`:
```

`cargo/src/doc/src/reference/manifest.md:552`

```md
To know which table under `[lints]` a particular lint belongs under, it is the part before `::` in the lint
```

The Cargo changelog records that `[lints]` stabilized in Rust 1.74 for reporting levels.
This source line is wrapped here for markdown width.

`cargo/src/doc/src/CHANGELOG.md:3797`

```md
- 🎉 The `[lints]` table has been stabilized, allowing you to configure reporting levels for rustc and
  other tool lints in `Cargo.toml`.
```

Cargo's manifest schema has a place for per-lint config data,
but Cargo currently treats almost all such data as unexpected.
The schema accepts a lint either as a plain level or as a config object.

`cargo/crates/cargo-util-schemas/src/manifest/mod.rs:1604`

```rust
pub type TomlLints = BTreeMap<String, TomlToolLints>;
```

`cargo/crates/cargo-util-schemas/src/manifest/mod.rs:1608`

```rust
#[derive(Serialize, Debug, Clone)]
#[serde(untagged)]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
pub enum TomlLint {
    Level(TomlLintLevel),
    Config(TomlLintConfig),
}
```

`cargo/crates/cargo-util-schemas/src/manifest/mod.rs:1656`

```rust
pub struct TomlLintConfig {
    pub level: TomlLintLevel,
    #[serde(default)]
    pub priority: i8,
    #[serde(flatten)]
```

But Cargo validates extra keys against a small allow-list.
Today that list contains only `lints.rust.unexpected_cfgs.check-cfg`.

`cargo/src/cargo/util/toml/mod.rs:2706`

```rust
} else if let Some(config) = config.config() {
    for config_name in config.keys() {
        // manually report unused manifest key warning since we collect all the "extra"
        // keys and values inside the config table
        let expected = EXPECTED_LINT_CONFIG.contains(&(tool, name, config_name));
        if !expected {
            let message =
                format!("unused manifest key: `lints.{tool}.{name}.{config_name}`");
            warnings.push(message);
        }
    }
}
```

`cargo/src/cargo/util/toml/mod.rs:2724`

```rust
static EXPECTED_LINT_CONFIG: &[(&str, &str, &str)] = &[
    // forwarded to rustc/rustdoc
    ("rust", "unexpected_cfgs", "check-cfg"),
];
```

Cargo then converts lint entries to command-line lint-level flags.
It uses `level()` and `priority()` for each entry.
The only custom argument path in this function is the special `rust.unexpected_cfgs.check-cfg` case.

`cargo/src/cargo/util/toml/mod.rs:2761`

```rust
fn lints_to_rustflags(lints: &manifest::TomlLints) -> CargoResult<Vec<String>> {
```

`cargo/src/cargo/util/toml/mod.rs:2767`

```rust
lints.iter().map(move |(name, config)| {
    let flag = match config.level() {
        manifest::TomlLintLevel::Forbid => "--forbid",
        manifest::TomlLintLevel::Deny => "--deny",
        manifest::TomlLintLevel::Warn => "--warn",
        manifest::TomlLintLevel::Allow => "--allow",
    };
```

`cargo/src/cargo/util/toml/mod.rs:2794`

```rust
// Also include the custom arguments specified in `[lints.rust.unexpected_cfgs.check_cfg]`
if let Some(rust_lints) = lints.get("rust") {
```

Clippy still has its own config-file lookup and deserialization path.

`rust-clippy/clippy_config/src/conf.rs:956`

```rust
pub fn lookup_conf_file() -> io::Result<(Option<PathBuf>, Vec<String>)> {
    /// Possible filename to search for.
    const CONFIG_FILE_NAMES: [&str; 2] = [".clippy.toml", "clippy.toml"];
```

`rust-clippy/clippy_config/src/conf.rs:960`

```rust
// Start looking for a config file in CLIPPY_CONF_DIR, or failing that, CARGO_MANIFEST_DIR.
// If neither of those exist, use ".". (Update documentation if this priority changes)
let mut current = env::var_os("CLIPPY_CONF_DIR")
    .or_else(|| env::var_os("CARGO_MANIFEST_DIR"))
    .map_or_else(|| PathBuf::from("."), PathBuf::from)
    .canonicalize()?;
```

`rust-clippy/clippy_config/src/conf.rs:970`

```rust
loop {
    for config_file_name in &CONFIG_FILE_NAMES {
        if let Ok(config_file) = current.join(config_file_name).canonicalize() {
```

Clippy's `disallowed-methods` option is one of those config-file values.

`rust-clippy/clippy_config/src/conf.rs:640`

```rust
/// The list of disallowed methods,
/// written as fully qualified paths.
```

`rust-clippy/clippy_config/src/conf.rs:648`

```rust
#[disallowed_paths_allow_replacements = true]
#[lints(disallowed_methods)]
disallowed_methods: Vec<DisallowedPath> = Vec::new(),
```

The lint itself says it is driven by `clippy.toml`.

`rust-clippy/clippy_lints/src/disallowed_methods.rs:13`

```rust
/// ### What it does
/// Denies the configured methods and functions in clippy.toml
///
/// Note:
/// Even though this lint is warn-by-default,
/// it will only trigger if
/// methods are defined in the clippy.toml file.
```

For the original `Result::unwrap` policy question,
`clippy::unwrap_used` is broader than only `Result::unwrap`.

`rust-clippy/clippy_lints/src/methods/mod.rs:4673`

```rust
/// ### What it does
/// Checks for `.unwrap()` or `.unwrap_err()` calls on `Result`s and `.unwrap()` call on `Option`s.
```

`rust-clippy/clippy_lints/src/methods/mod.rs:4716`

```rust
#[clippy::version = "1.45.0"]
pub UNWRAP_USED,
restriction,
"using `.unwrap()` on `Result` or `Option`, which should at least get a better message using `expect()`"
```

Its config options are also `clippy.toml` values.

`rust-clippy/clippy_config/src/conf.rs:411`

```rust
/// Whether `unwrap` should be allowed in code always evaluated at compile time
#[lints(unwrap_used)]
allow_unwrap_in_consts: bool = true,
/// Whether `unwrap` should be allowed in test functions or `#[cfg(test)]`
#[lints(unwrap_used)]
allow_unwrap_in_tests: bool = false,
```

`rust-clippy/clippy_config/src/conf.rs:417`

```rust
/// List of types to allow `unwrap()` and `expect()` on.
```

`rust-clippy/clippy_lints/src/methods/unwrap_expect_used.rs:106`

```rust
if allow_unwrap_in_tests && is_in_test(cx.tcx, expr.hir_id) {
    return;
}
```

Earlier wrong hypothesis to avoid:
Rust is not expecting projects to avoid lint configuration.
The harness below disproves that reading because `[lints.clippy] unwrap_used = "deny"` caused
`clippy::unwrap_used` diagnostics for `Result::unwrap`,
 `Result::unwrap_err`,
 and `Option::unwrap`.

A second wrong hypothesis to avoid:
Cargo `[lints.clippy]` has not absorbed all `clippy.toml` functionality.
The harness below disproves that reading because putting `disallowed-methods` under
`[lints.clippy.disallowed_methods]` produced
`unused manifest key: lints.clippy.disallowed_methods.disallowed-methods` and no method ban was enforced.

## Verification

Versions under test:

```text
cargo 1.98.0-nightly (a595d0da2 2026-06-20)
clippy 0.1.98 (f28ac764c3 2026-06-23)
rustc 1.98.0-nightly (f28ac764c 2026-06-23)
```

Source under inspection:

```text
rust-clippy origin: https://github.com/rust-lang/rust-clippy.git
rust-clippy commit: 64c7431d6cd823d1a7663165c7e59d78e6dc726a
cargo origin: https://github.com/rust-lang/cargo.git
cargo commit: a335d47ff8036918d3d548dabd513dc0444096a9
```

Run this harness from any scratch-safe directory.
It creates four tiny crates under current `~/temp/agent` scratch space.

```shell
mkdir --parents "${HOME}/temp/agent"
chmod 700 "${HOME}/temp/agent"
scratch=$(mktemp --directory "${HOME}/temp/agent/clippy-config-harness-XXXXXXXXXX")
mkdir --parents \
  "$scratch/stable-lints/src" \
  "$scratch/disallowed-clippy-toml/src" \
  "$scratch/disallowed-cargo-lints/src" \
  "$scratch/test-allow/src"

cat > "$scratch/stable-lints/Cargo.toml" <<'TOML'
[package]
name = "stable_lints"
version = "0.1.0"
edition = "2024"

[lints.clippy]
unwrap_used = "deny"
TOML
cat > "$scratch/stable-lints/src/main.rs" <<'RS'
fn result_value() -> Result<i32, &'static str> { Ok(1) }
fn option_value() -> Option<i32> { Some(2) }
fn main() {
    let a = result_value().unwrap();
    let b = result_value().unwrap_err();
    let c = option_value().unwrap();
    let d = result_value().expect("demo result must be present");
    println!("{a} {b} {c} {d}");
}
RS

cat > "$scratch/disallowed-clippy-toml/Cargo.toml" <<'TOML'
[package]
name = "disallowed_clippy_toml"
version = "0.1.0"
edition = "2024"

[lints.clippy]
disallowed_methods = "deny"
unnecessary_literal_unwrap = "allow"
TOML
cat > "$scratch/disallowed-clippy-toml/clippy.toml" <<'TOML'
disallowed-methods = [
  { path = "std::result::Result::unwrap", reason = "handle errors or explain panic with expect" },
]
TOML
cat > "$scratch/disallowed-clippy-toml/src/main.rs" <<'RS'
fn result_value() -> Result<i32, &'static str> { Ok(1) }
fn option_value() -> Option<i32> { Some(2) }
fn main() {
    let a = result_value().unwrap();
    let b = option_value().unwrap();
    let c = result_value().expect("demo result must be present");
    println!("{a} {b} {c}");
}
RS

cat > "$scratch/disallowed-cargo-lints/Cargo.toml" <<'TOML'
[package]
name = "disallowed_cargo_lints"
version = "0.1.0"
edition = "2024"

[lints.clippy.disallowed_methods]
level = "deny"
disallowed-methods = [
  { path = "std::result::Result::unwrap", reason = "handle errors or explain panic with expect" },
]

[lints.clippy]
unnecessary_literal_unwrap = "allow"
TOML
cat > "$scratch/disallowed-cargo-lints/src/main.rs" <<'RS'
fn result_value() -> Result<i32, &'static str> { Ok(1) }
fn main() {
    let a = result_value().unwrap();
    println!("{a}");
}
RS

cat > "$scratch/test-allow/Cargo.toml" <<'TOML'
[package]
name = "test_allow"
version = "0.1.0"
edition = "2024"

[lints.clippy]
unwrap_used = "deny"
TOML
cat > "$scratch/test-allow/clippy.toml" <<'TOML'
allow-unwrap-in-tests = true
TOML
cat > "$scratch/test-allow/src/lib.rs" <<'RS'
pub fn value() -> Option<i32> { Some(1) }
#[cfg(test)]
mod tests {
    use super::value;
    #[test]
    fn unwrap_is_allowed_in_tests_by_clippy_toml() {
        assert_eq!(value().unwrap(), 1);
    }
}
RS

for crate in stable-lints disallowed-clippy-toml disallowed-cargo-lints test-allow; do
  printf '\n===== %s =====\n' "$crate"
  (cd "$scratch/$crate" && cargo clippy --all-targets --message-format short)
  printf 'status=%s\n' "$?"
done
```

### Patterns that fail

Stable Cargo lint levels catch all forms covered by `clippy::unwrap_used`.
The `stable-lints` crate failed with these diagnostics:

```text
src/main.rs:4:13: error: used `unwrap()` on a `Result` value
src/main.rs:5:13: error: used `unwrap_err()` on a `Result` value
src/main.rs:6:13: error: used `unwrap()` on an `Option` value
status=101
```

This proves `[lints.clippy] unwrap_used = "deny"` is stable and active,
but it is too broad for "only `Result::unwrap`".

Putting the Clippy option payload under Cargo `[lints.clippy.disallowed_methods]` failed to enforce anything.
Cargo warned about the unused payload key and Clippy finished successfully:

```text
warning: Cargo.toml: unused manifest key: `lints.clippy.disallowed_methods.disallowed-methods`
Finished `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
status=0
```

This proves Cargo `[lints.clippy]` has not absorbed `clippy.toml`'s `disallowed-methods` functionality.

### Patterns that work cleanly

The `clippy.toml` configuration for `disallowed-methods` only flagged `std::result::Result::unwrap` in the harness.
It did not flag `Option::unwrap` or `Result::expect`:

```text
src/main.rs:4:28: error: use of a disallowed method `std::result::Result::unwrap`
status=101
```

The `clippy.toml` option `allow-unwrap-in-tests = true` suppressed `unwrap_used` for a test target while the Cargo
manifest still denied the lint level:

```text
Checking test_allow v0.1.0 ([SCRATCH]/test-allow)
Finished `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s
status=0
```

## Verified workarounds

### Use Cargo `[lints.clippy]` when a whole lint matches policy

Patch:

```diff
# Cargo.toml
+[lints.clippy]
+unwrap_used = "deny"
```

This is the stable path for lint levels.
The harness above verifies that Clippy 0.1.98 enforces it.

Tradeoff:
`clippy::unwrap_used` covers `Result::unwrap`,
 `Result::unwrap_err`,
 and `Option::unwrap`.
It is not suitable when the policy is only "ban `Result::unwrap`".

### Use `clippy.toml` for lint parameters that Cargo does not forward

Patch:

```diff
# Cargo.toml
+[lints.clippy]
+disallowed_methods = "deny"
+unnecessary_literal_unwrap = "allow"
```

```diff
# clippy.toml
+disallowed-methods = [
+  { path = "std::result::Result::unwrap", reason = "handle errors or explain panic with expect" },
+]
```

This is the precise current path for "ban only `Result::unwrap`".
The harness verifies that it flags `Result::unwrap` and leaves `Option::unwrap` and `Result::expect` alone.

Tradeoff:
the Clippy config-file format is explicitly marked unstable,
so future Clippy versions may require a migration.
Keep this use narrow and easy to search.

### Use `clippy.toml` test allowances when the whole lint is otherwise useful

Patch:

```diff
# Cargo.toml
+[lints.clippy]
+unwrap_used = "deny"
```

```diff
# clippy.toml
+allow-unwrap-in-tests = true
```

The harness verifies this combination succeeds for a test target using `Option::unwrap`.

Tradeoff:
this allows every `unwrap` covered by `unwrap_used` in tests,
not just specific fixture setup paths.
Use local `#[allow]` with a justification when the exception is narrower.

## What does not work

### Cargo `[lints.clippy]` does not carry `disallowed-methods`

This manifest shape parses,
but Cargo warns and ignores the `disallowed-methods` payload:

```toml
# Cargo.toml
[lints.clippy.disallowed_methods]
level = "deny"
disallowed-methods = [
  { path = "std::result::Result::unwrap", reason = "handle errors or explain panic with expect" },
]
```

Observed output:

```text
warning: Cargo.toml: unused manifest key: `lints.clippy.disallowed_methods.disallowed-methods`
status=0
```

### `unwrap_used = "deny"` does not mean only `Result::unwrap`

This manifest shape is stable and works,
but it flags `Option::unwrap` too:

```toml
# Cargo.toml
[lints.clippy]
unwrap_used = "deny"
```

Observed output:

```text
error: used `unwrap()` on a `Result` value
error: used `unwrap_err()` on a `Result` value
error: used `unwrap()` on an `Option` value
```

### Treating `clippy.toml` as unsupported is wrong

Clippy 0.1.98 still reads `.clippy.toml` / `clippy.toml`,
and the harness verifies both `disallowed-methods` and `allow-unwrap-in-tests` affect diagnostics.
The instability warning is a migration-risk warning about that file format,
not a present-tense unsupported warning.

## A container lint that mounts only the crate silently drops the ban

### Symptom

A package whose lint task falls back to podman passes where the same source on the host is denied.
A planted `Result::unwrap` in `package/terminal/app/src/main.rs` passed
`mise run //package/terminal/app:lint:clippy:container` before the dispatch handed the lint the root file,
and failed it afterwards with the reason text of that file:

```text
error: use of a disallowed method `std::result::Result::unwrap`
   --> src/main.rs:506:64
    |
506 |     let _planted_unwrap = std::env::var("PI_ISSUE_1756_PLANT").unwrap();
    |                                                                ^^^^^^
    |
    = note: handle errors with ? or explain intentional panic with expect
```

### Root cause

Clippy starts its configuration lookup in `CLIPPY_CONF_DIR`,
else `CARGO_MANIFEST_DIR`,
and walks up the directories from there.
A dispatch that mounts only the package directory at `/work` leaves no `clippy.toml` anywhere in that chain,
and Clippy says nothing about the missing file:
`disallowed_methods = "deny"` in `[lints.clippy]` switches on a ban whose method list is empty.

### What reaches the lint

- A read-only bind mount at `/clippy.toml`,
  as `package/music-player/desktop-app` does.
- A read-only bind mount at `/clippy-conf/clippy.toml` with `CLIPPY_CONF_DIR=/clippy-conf`,
  as `package/ide/app`,
  `package/terminal/app` and `package/nested-wayland-session/cli` do.
  A mount over `/work/clippy.toml` is worse:
  podman leaves an empty path in the host package directory.
- A `COPY clippy.toml /work/clippy.toml` in the image,
  for the mount-free snapshots of
  `package/linter/monochromatic-lint` and the git-policy native test images.

A bind mount whose source is missing fails loudly instead of linting without the file:

```text
Error: statfs /path/to/clippy.toml: no such file or directory
```

### Guard

The container start sites ledger at `package/container-limit/core/src/container-start-sites.unit.test.ts` lists the
package task files that pass a clippy argument to a container program,
and fails,
naming the file,
when one of them carries neither the mount nor the directory variable.

## Upstream filing artifact

### Upstream filing decision

`.out-of-scope/` was checked before drafting.
The only Cargo-adjacent entry was `.out-of-scope/cargo-workspace.md`,
which covers this repository not using a root Cargo workspace.
It does not exempt Clippy or Cargo documentation issues.
No Clippy-specific exemption matched.

Duplicate search was run against open and closed rust-clippy issues and PRs with these terms:

```text
"configuration file is unstable" "lints.clippy"
clippy.toml Cargo.toml lints configuration unstable
configuration.md Cargo.toml lints clippy.toml
```

All returned `[]`.
Cargo issue and PR searches for `clippy.toml lints configuration options` also returned `[]`.
No duplicate thread was found.

Constraint check:

- Is it really upstream's fault?
  Yes,
   but as wording rather than behavior.
  The Clippy source page puts a broad unstable-file warning at
  `rust-clippy/book/src/configuration.md:3` and only later links Cargo `[lints]` at line 91.
- Can upstream fix it?
  Yes.
  A documentation-only patch can clarify that the warning applies to `clippy.toml` and link stable Cargo lint levels.
- Are they supporting this use case?
  Yes.
  The same page documents both `clippy.toml` configuration and Cargo's lints section.
- Would the repo welcome our contribution?
  Yes.
  `CONTRIBUTING.md:5` says to ask or submit an issue or pull request anyway,
  `CONTRIBUTING.md:9` says Clippy welcomes contributions from everyone,
  and `.github/PULL_REQUEST_TEMPLATE.md:1` says "Thank you for making Clippy better!
  ".
  Searches of `README.md`,
   `CONTRIBUTING.md`,
   `.github`,
   and `book/src/development` found no ban on
  AI-assisted reports or patches.
- Will they likely fix it?
  Plausibly yes.
  It is a small docs clarification,
  no duplicate or maintainer rejection was found,
  and the contribution policy welcomes documentation changes.
- Have we prototyped a minimal fix compatible with their architecture?
  Yes.
  A disposable clone at commit `64c7431d6cd823d1a7663165c7e59d78e6dc726a` was edited after confirming origin.
  The pre-patch check failed because the clarifying note was absent:

```text
Pre-patch docs clarity check:
missing clippy.toml-only note beside unstable warning
pre_status=1
```

The post-patch check passed:

```text
patched note distinguishes clippy.toml options from stable Cargo lint levels
post_status=0
```

Prototype diff,
captured with `git diff --unified=0` to avoid fenced-diff trailing whitespace on blank context lines:

```diff
diff --git a/book/src/configuration.md b/book/src/configuration.md
index cb2ac67..abca254 100644
--- a/book/src/configuration.md
+++ b/book/src/configuration.md
@@ -3 +3,3 @@
-> **Note:** The configuration file is unstable and may be deprecated in the future.
+> **Note:** The configuration file described in this section (`clippy.toml` / `.clippy.toml`) is unstable and may be
+> deprecated in the future. Use the [`Cargo.toml` lints section](#lints-section-in-cargotoml) below for stable
+> lint levels.
```

All six constraints hold.
The filing artifact below is fileable,
but no external issue or pull request was posted from this repository session.

### Draft upstream issue

~~~md
Title: Clarify that the unstable configuration-file note applies to clippy.toml, not Cargo lint levels

Labels: A-documentation

Description:

The Clippy configuration page currently opens with:

```md
> **Note:** The configuration file is unstable and may be deprecated in the future.
```

Source trace:

- `book/src/configuration.md:3` contains the warning.
- `book/src/configuration.md:5` then introduces `clippy.toml` and `.clippy.toml`.
- `book/src/configuration.md:89` names the Cargo.toml lints subsection.
- `book/src/configuration.md:91` links Cargo's `[lints]` section for `allow` / `warn` / `deny` lint levels.

The warning is true for the Clippy configuration file described by the page,
but the placement makes it easy to read as applying to all Clippy lint policy.
Cargo's `[lints]` table is stable for lint levels,
so readers can think Rust discourages lint configuration or that Cargo `[lints]` should absorb every `clippy.toml`
option.

Reproduction:

1.  Open <https://doc.rust-lang.org/stable/clippy/configuration.html>.
2.  Read the first warning before reaching the "Lints Section in Cargo.toml" subsection.
3.  Try to decide whether this is stable:

```toml
[lints.clippy]
unwrap_used = "deny"
```

Expected docs behavior:

the warning should say it applies to the `clippy.toml` / `.clippy.toml` file format,
and it should point readers who only need lint levels to Cargo's stable `[lints]` section.

Suggested fix,
captured with `git diff --unified=0`:

```diff
diff --git a/book/src/configuration.md b/book/src/configuration.md
index cb2ac67..abca254 100644
--- a/book/src/configuration.md
+++ b/book/src/configuration.md
@@ -3 +3,3 @@
-> **Note:** The configuration file is unstable and may be deprecated in the future.
+> **Note:** The configuration file described in this section (`clippy.toml` / `.clippy.toml`) is unstable and may be
+> deprecated in the future. Use the [`Cargo.toml` lints section](#lints-section-in-cargotoml) below for stable
+> lint levels.
```

Verification performed:

- Confirmed `cargo 1.98.0-nightly` accepts `[lints.clippy] unwrap_used = "deny"` and Clippy reports
  `Result::unwrap`, `Result::unwrap_err`, and `Option::unwrap`.
- Confirmed `clippy.toml` still drives `disallowed-methods` and `allow-unwrap-in-tests`.
- Confirmed putting `disallowed-methods` under `[lints.clippy.disallowed_methods]` produces
  `unused manifest key: lints.clippy.disallowed_methods.disallowed-methods` and does not enforce the method ban.
~~~
