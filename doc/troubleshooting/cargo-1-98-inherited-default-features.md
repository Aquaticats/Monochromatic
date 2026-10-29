# Cargo 1.98: `default-features = false` beside `workspace = true` is a hard error in Edition 2024 when the workspace entry keeps defaults

The Rust dependency catalog migration (commit `fd0bd03540`) passed on host nightly cargo 1.101
but broke every Edition 2024 crate that disables default features on an inherited dependency
when built with cargo 1.98.0, the toolchain in the `localhost/monochromatic/ide` image.
It blocked the IDE 0.1.3 release until commit `d1477a13ba`.

## Symptom

`cargo metadata` (and any build) exits 101 while loading the manifest, with this cause line:

```text
  `default-features = false` cannot override workspace's `default-features`
```

The trigger is a member entry such as

```toml
# package/desktop-app/ide/Cargo.toml (before d1477a13ba)
[dependencies]
futures-util = { workspace = true, default-features = false, features = ["std"] }

[workspace.dependencies]
futures-util = ">=0.3.31"
```

where the workspace entry is a bare version string (default features unspecified, so on)
or sets `default-features = true`.
The same manifest loads cleanly on cargo 1.99 and later, including nightly 1.101,
so a host-only check misses it.

Affected crates at `fd0bd03540`, by site count:
`package/desktop-app/ide` (7),
`package/music-player/truepeak-core` (5),
`package/cli/nested-wayland-session` (3),
`package/forbidden-strings/cli` (2),
`package/desktop-app/terminal` (1),
`package/linter/monochromatic-lint` (1),
and `package/forbidden-strings/core` (1).
All 24 first-party manifests declare `edition = "2024"`
(`rg --only-matching '^edition\s*=.*' --glob Cargo.toml package` returns 24 lines, all `"2024"`).

## Root cause

Cargo 1.98.0 is cargo tag `0.99.0`, commit `797e8a9bca276c1c9f9f738d2a20f484fa4eea9d`
(matching `cargo --version` output `cargo 1.98.0 (797e8a9bc 2026-08-05)` in the image).
While merging an inherited dependency, `inner_dependency_inherit_with`
(`src/cargo/util/toml/mod.rs:1182`) compares the member's `default-features` with the workspace entry's
and calls `deprecated_ws_default_features` when the member turns defaults off but the workspace does not
(`src/cargo/util/toml/mod.rs:1221` to `:1227`):

```rust
        (Some(false), Some(true)) => {
            deprecated_ws_default_features(name, Some(true), edition, warnings)?;
        }
        // member: default-features = false and
        // workspace: dep = "1.0" should ignore member default-features
        (Some(false), None) => {
            deprecated_ws_default_features(name, None, edition, warnings)?;
```

`deprecated_ws_default_features` (`src/cargo/util/toml/mod.rs:1247`) bails for Edition 2024
and only warns for older editions:

```rust
    if Edition::Edition2024 <= edition {
        anyhow::bail!("`default-features = false` cannot override workspace's `default-features`");
    } else {
        warnings.push(format!(
            "`default-features` is ignored for {label}, since `default-features` was \
                {ws_def_feat} for `workspace.dependencies.{label}`, \
                this could become a hard error in the future"
        ));
    }
```

This is the documented Edition 2024 rule
([edition guide: reject unused inherited default-features][edition-guide]).

[rust-lang/cargo#17126][cargo-17126] ("feat(toml): allow overriding inherited default-features in 2024",
merged 2026-08-07, after the 1.98 branch point) implements RFC 3945.
At cargo `06413ecf8cd3ec5b10b93ac2b85f6fad34f8597c` the merge lives in
`src/workspace/parser/mod.rs:1240` to `:1265`:

```rust
    // RFC 3945: Allow workspace members to override the workspace dependency's
    // `default-features` setting.
    if edition >= Edition::Edition2024 {
        merged_dep.default_features = default_features.or(merged_dep.default_features);
    } else {
```

Older editions still ignore the member key with a warning that reads
"overriding workspace `default-features` to false requires Rust 1.99+ and the 2024 edition"
(`src/workspace/parser/mod.rs:1292` to `:1297`).
The [Cargo workspaces reference][workspaces] states the same:
"`default-features` declared on a package dependency overrides the value set here in Edition 2024 packages
(requires Rust 1.99+)."

So the generated per-crate `[workspace.dependencies]` table must itself carry `default-features = false`
for any cargo older than 1.99, and for any edition older than 2024 on every cargo version,
or the member's request is rejected or silently ignored.

## Verification

Harness (repo root; mounts the checkout read-only with SELinux labeling disabled,
because `:Z` relabeling of the whole checkout fails with `lsetxattr ... operation not permitted`
on files under `node_modules/.monochromatic/`):

```sh
# Reproduce on HEAD's old manifest (control) inside the cargo 1.98 image.
git show d1477a13ba~1:package/desktop-app/ide/Cargo.toml > "${HOME}/temp/agent/ide-old-Cargo.toml"
podman run --rm --memory=2g --cpus=2 --security-opt label=disable \
  --volume "${PWD}:/work:ro" \
  --volume "${HOME}/temp/agent/ide-old-Cargo.toml:/work/package/desktop-app/ide/Cargo.toml:ro" \
  localhost/monochromatic/ide \
  sh -c 'cargo metadata --locked --format-version 1 --manifest-path /work/package/desktop-app/ide/Cargo.toml >/dev/null'
```

### Fails on cargo 1.98.0

- Member `{ workspace = true, default-features = false }` with workspace entry `">=0.3.31"`:
  exit 101, "`default-features = false` cannot override workspace's `default-features`".
- The old manifest for each of the seven affected crates, before `d1477a13ba`
  (reported by the session that built the IDE image; the ide manifest was re-run here as the control).

### Works on cargo 1.98.0 and nightly 1.101

- Member `{ workspace = true, default-features = false }` with workspace entry
  `{ version = ">=0.3.31", default-features = false }`:
  `cargo metadata --locked` passes for all seven affected crates at `d1477a13ba`,
  on host nightly 1.101 (`--offline`) and inside `localhost/monochromatic/ide`.
- Member `{ workspace = true }` with any workspace entry.

## Verified workarounds

### Disable default features in the workspace entry (adopted)

`package/dev-script/file-enforcer/src/cargo/catalog/default-features.ts` renders
`default-features = false` into a crate's generated entry when every catalog site of that dependency
in the crate writes `default-features = false`; sites keep their `features` lists.
Tradeoffs:

- The choice is per crate, not per site.
  When one site of a dependency disables defaults and another (for example in `[dev-dependencies]`)
  does not, the sync fails with a named issue instead of guessing,
  because a workspace entry with defaults off would silently turn them off for the other site.
  The remedy is to make the sites agree.
- The generated table differs between crates for the same catalog entry,
  which is fine because each crate is its own workspace root.

### Raise the minimum cargo to 1.99

Rejected for now: the IDE image pins cargo 1.98.0, and the catalog must not depend on which
toolchain a consumer has. It also would not help pre-2024-edition crates, which still ignore the member key.

## What does not work

- Removing `default-features = false` from the member sites: cargo then enables the dependency's
  default features, which changes what each crate compiles and links.
- Verifying only on host nightly: nightly 1.101 includes rust-lang/cargo#17126, so it accepts the old manifests.

## Upstream filing decision

Nothing to file.

1. **Is it really upstream's fault?** No. The 1.98 error is the documented Edition 2024 rule,
   and upstream already relaxed it in rust-lang/cargo#17126 for 1.99.
2. **Can upstream fix it?** Already fixed for 1.99; 1.98 is a released toolchain.
3. **Are they supporting this use case?** Yes, from 1.99 with Edition 2024.
4. **Would the repo welcome our contribution?** Not evaluated: there is nothing to contribute.
5. **Will they likely fix it?** Already fixed.
6. **Have we prototyped a fix?** Not applicable.

`.out-of-scope/cargo-workspace.md` covers the root-workspace question, not this behavior.
Duplicate search: `gh search prs --repo rust-lang/cargo --merged 3945` found rust-lang/cargo#17126;
there is no gap for a new issue or comment to fill.

[cargo-17126]: https://github.com/rust-lang/cargo/pull/17126
[edition-guide]: https://doc.rust-lang.org/edition-guide/rust-2024/cargo-inherited-default-features.html
[workspaces]: https://doc.rust-lang.org/cargo/reference/workspaces.html
