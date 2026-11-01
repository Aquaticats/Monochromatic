# pnpm 12 `pnpm add <prefix>:<package>` reads the named-registry prefix as the package name and fails

## Symptom

pnpm `12.3.4` and `12.4.1` reject every alias-less `pnpm add` argument that routes a package through a named registry,
 although the `12.x` docs page
 ([dependency resolution settings](https://pnpm.io/settings/dependency-resolution),
 `docusaurus_version` `12.x`,
 checked 2026-09-14)
 documents `pnpm add work:@corp/lib@^2.0.0` for exactly that.
Observed 2026-09-14 while probing per-dependency routing for `doc/planning/private-npm-registry.md`.

The project declares the prefix in `pnpm-workspace.yaml`:

```yaml
# pnpm-workspace.yaml
registries:
  https://registry.npmmirror.com/:
    prefix: mirror
```

The error depends on whether the package inside the prefix is scoped,
 and whether a range follows it.
Neither `package.json` nor a lockfile is written in any failing case.

A scoped package (`mirror:@monochromatic-dev/logger-core@^0.4.0`,
 `mirror:@monochromatic-dev/logger-core`,
 `npmjs:@monochromatic-dev/logger-core@^0.4.0`,
 `gh:@pnpm/foo@^1.0.0`)
 fails with `ERR_PNPM_INVALID_DEPENDENCY_NAME`,
 naming the prefix as the dependency:

```text
Error: ERR_PNPM_INVALID_DEPENDENCY_NAME

  × adding a new package
  ╰─▶ Failed to resolve dependency tree: The current package contains a
      dependency with an invalid name: "mirror:". Dependency names must be a
      single package name or "@scope/name" — they cannot contain path-
      separator segments such as "..".
```

An unscoped package with a range (`mirror:is-number@^7.0.0`,
 `npmjs:is-number@^7.0.0`)
 fails with `ERR_PNPM_RESOLVING_NPM_RESOLVER_NETWORK_ERROR`,
 because pnpm looks up a package literally named `mirror:is-number` on the default registry:

```text
Error: ERR_PNPM_RESOLVING_NPM_RESOLVER_NETWORK_ERROR

  × adding a new package
  ╰─▶ Failed to fetch metadata from https://registry.npmjs.org/mirror%3Ais-
      number: HTTP status client error (404 Not Found) for url (https://
      registry.npmjs.org/mirror%3Ais-number)
```

An unscoped package without a range (`mirror:is-number`)
 fails with `ERR_PNPM_PACKAGE_MANAGER_ADD_RESOLVE_LATEST` for the same lookup:

```text
Error: ERR_PNPM_PACKAGE_MANAGER_ADD_RESOLVE_LATEST

  × adding a new package
  ╰─▶ Failed to resolve the latest version of mirror:is-number: Failed to
      fetch metadata from https://registry.npmjs.org/mirror%3Ais-number: HTTP
      status client error (404 Not Found) for url (https://registry.npmjs.org/
      mirror%3Ais-number)
```

The unscoped lookup ignores the prefix entirely:
 pointing `mirror` at `https://registry.invalid/` still produces the `registry.npmjs.org/mirror%3Ais-number` 404.

The same specifiers install when written into `package.json` by hand,
 and pnpm `11.25.0` accepts every failing `pnpm add` form listed here.

## Root cause

Source:
 `pnpm/pnpm` tag `v12.3.4` (commit `666c35e95c17f1a36414adc28bc25a73a8f1f67f`),
 the Rust CLI under `pnpm/crates`,
 unless a step names `main`.

1.   `resolve_added_dependency` splits the argument with `parse_wanted_dependency`,
     asks the alias-less resolvers only when no alias came back,
     and otherwise falls back to `split_name_spec`
     (`pnpm/crates/package-manager/src/add.rs:882`):

     ```rust
     let parsed = pnpm_resolving_parse_wanted_dependency::parse_wanted_dependency(package_selector);
     let aliasless = match (parsed.alias.as_deref(), parsed.bare_specifier.as_deref()) {
         (None, Some(specifier)) => {
             resolve_aliasless_specifier(specifier, config, http_client_arc, manifest).await?
         }
         _ => None,
     };
     let (package_name, explicit_spec) = match aliasless.as_ref() {
         Some(dep) => (dep.package_name.as_str(), Some(dep.manifest_specifier.as_str())),
         None => split_name_spec(package_selector),
     };
     ```

2.   `parse_wanted_dependency` takes the text before the first `@` after index 0 as an alias candidate
     and keeps it only when it is a valid old-style npm name
     (`pnpm/crates/resolving-parse-wanted-dependency/src/lib.rs:32`):

     ```rust
     let version_delimiter = find_version_delimiter(raw_wanted_dependency);
     if let Some(idx) = version_delimiter {
         let alias = &raw_wanted_dependency[..idx];
         if is_valid_old_npm_package_name(alias) {
             return ParsedWantedDependency {
                 alias: Some(alias.to_string()),
                 bare_specifier: Some(raw_wanted_dependency[idx + 1..].to_string()),
             };
         }
         return ParsedWantedDependency {
             alias: None,
             bare_specifier: Some(raw_wanted_dependency.to_string()),
         };
     }
     ```

     `mirror:` and `mirror:is-number` both contain `:`,
     which is outside the URL-safe set
     (`pnpm/crates/resolving-parse-wanted-dependency/src/validate_npm_package_name.rs:79`):

     ```rust
     fn is_url_friendly(string: &str) -> bool {
         string.chars().all(|ch| {
             ch.is_ascii_alphanumeric()
                 || matches!(ch, '-' | '_' | '.' | '!' | '~' | '*' | '\'' | '(' | ')')
         })
     }
     ```

     So the argument comes back alias-less,
     which is the correct reading.

3.   The alias-less resolvers claim only git specifiers,
     `http:`/`https:` tarballs,
     and local paths,
     so a named-registry argument falls through with `Ok(None)`
     (`pnpm/crates/package-manager/src/add.rs:1041`):

     ```rust
     if pnpm_resolving_git_resolver::parse_bare_specifier(specifier).is_some() {
         return resolve_aliasless_git(specifier, config, http_client).await.map(Some);
     }
     if specifier.starts_with("http:") || specifier.starts_with("https:") {
         return resolve_aliasless_tarball(specifier, config, http_client).await.map(Some);
     }
     resolve_aliasless_local(specifier, manifest).await
     ```

4.   `split_name_spec` then reads everything before the first `@` after index 0 as the package name
     (`pnpm/crates/package-manager/src/add.rs:1522`):

     ```rust
     fn split_name_spec(input: &str) -> (&str, Option<&str>) {
         match input.get(1..).and_then(|rest| rest.find('@')).map(|offset| offset + 1) {
             Some(idx) => (&input[..idx], Some(&input[idx + 1..])),
             None => (input, None),
         }
     }
     ```

     For `mirror:@monochromatic-dev/logger-core@^0.4.0` the name is `mirror:`;
     for `mirror:is-number@^7.0.0` it is `mirror:is-number`;
     for `mirror:is-number` (no `@`) it is the whole argument.

5.   An unscoped name then goes to `resolve_explicit_registry_spec` or the `latest` picker,
     which route by scope through `pick_registry_for_package`,
     so the default registry is asked for `mirror:is-number`
     (`pnpm/crates/package-manager/src/add.rs:1338`):

     ```rust
     let registries: std::collections::HashMap<String, String> =
         config.resolved_registries().into_iter().collect();
     let registry = pick_registry_for_package(&registries, package_name, None);
     ```

     That produces both unscoped error variants.
     A scoped argument finds no registry spec under the name `mirror:`,
     so the add keeps the manifest key `mirror:`,
     and the follow-up resolution rejects the key
     (`pnpm/crates/resolving-deps-resolver/src/resolve_dependency_tree.rs:493`,
     repeated at line 601 for importer specs):

     ```rust
     for (name, range) in manifest.dependencies(dependency_groups) {
         if !crate::is_valid_dependency_alias(name) {
             return Err(ResolveDependencyTreeError::InvalidDependencyName {
                 parent: "The current package".to_string(),
                 alias: name.to_string(),
             });
         }
     ```

The add path never consults the parser that already understands the syntax.
`parse_named_registry_specifier_to_registry_package_spec` splits `<prefix>:[@<scope>/]<name>[@<selector>]`
 against the configured and built-in prefixes
 (`pnpm/crates/resolving-npm-resolver/src/parse_bare_specifier.rs:197`):

```rust
pub fn parse_named_registry_specifier_to_registry_package_spec(
    raw_specifier: &str,
    known_registry_names: &HashSet<String>,
    package_alias: Option<&str>,
    default_tag: &str,
) -> Result<Option<NamedRegistryPackageSpec>, ParseNamedRegistrySpecifierError> {
    let Some(colon) = raw_specifier.find(':') else {
        return Ok(None);
    };
    if colon == 0 {
        return Ok(None);
    }
    let registry_name = &raw_specifier[..colon];
    if !known_registry_names.contains(registry_name) {
        return Ok(None);
    }
```

The install resolver uses it,
 which is why hand-written manifest entries work.

The TypeScript CLI (pnpm 11,
 kept in the same repository under `pnpm11/`)
 hands an alias-less argument to the resolver chain unchanged,
 and its `latest` resolution recognizes named-registry specifiers
 (`pnpm11/resolving/npm-resolver/src/index.ts:312`):

```ts
resolveLatestFromNamedRegistry: createResolveLatest(boundResolveFromNamedRegistry,
  (query) => isNamedRegistrySpec(query, ctx.namedRegistryNames)),
```

The Rust port replaced that dispatch with the `split_name_spec` fallback.

### The `jsr:` fix does not cover named registries

The same fallback broke `pnpm add jsr:@std/csv`
 ([pnpm/pnpm#14590](https://github.com/pnpm/pnpm/issues/14590)).
[pnpm/pnpm#14593](https://github.com/pnpm/pnpm/pull/14593)
 (merged 2026-09-06 as `091740e13317c1a5c9b6c3ce82f4a66c8b1ffadb`)
 fixed it with a `ProtocolSelector` that knows `npm:`,
 `jsr:`,
 and `workspace:` only.
`gh api repos/pnpm/pnpm/compare/091740e...v12.4.1` reports `v12.4.1` ahead by 118 commits and behind by 0,
 so `12.4.1` contains the fix,
 and `12.4.1` still fails every named-registry form in the verification catalog.
On `main` at `e2d0eaedcbcd8033544eb0b774db187db3bb4ef7` (2026-09-14)
 the selector still has no named-registry variant
 (`pnpm/crates/package-manager/src/add/specifier.rs:321`):

```rust
pub(super) enum ProtocolSelector {
    /// `npm:<name>[@<spec>]`. Nothing here is aliased — the install name is
    /// the real package name — so the entry is saved as the plain
    /// `<name>[@<spec>]` request would be.
    Npm { name: String, spec: Option<String> },
    /// `jsr:@<scope>/<name>[@<selector>]`.
    Jsr(JsrSpec),
    /// The alias form `workspace:<name>@<range>`. A version-only
    /// (`workspace:^1.2.3`) or path (`workspace:./pkg`, `workspace:C:\pkg`)
    /// specifier names no package, and [`WorkspaceSpec`] already tells the
    /// three apart.
    Workspace { name: String },
}
```

`main` then falls back to `split_name_spec(package_selector)` for every other alias-less argument
 (`pnpm/crates/package-manager/src/add/specifier.rs:101`).
A first hypothesis held that `12.4.1` would already carry a general protocol fix,
 because #14593 closed an `ERR_PNPM_INVALID_DEPENDENCY_NAME` report;
 the `12.4.1` catalog run and the enum disproved it.

## Verification

Versions under test:

- pnpm `12.3.4` and `12.4.1` and `11.25.0`,
   installed by mise from `aqua:pnpm/pnpm` (GitHub attestations verified by mise).
   `12.3.4` is what `pnpm = "latest"` in the repo `mise.toml` resolved to on 2026-09-14.
- An unpatched `main` build (`e2d0eae`),
   reporting `pnpm --version` `12.4.1`,
   built as described in the prototype section.

Harness,
 run once per catalog entry,
 each in a fresh directory.
`PNPM` selects the binary under test
 (a mise install path or a locally built `target/debug/pnpm`),
 `ARG` is the catalog argument,
 and `REGISTRY` is `https://registry.npmmirror.com/` unless an entry names the `registry.invalid` routing control:

```sh
# one catalog entry; PNPM, ARG, and REGISTRY set by the caller
cd -- "$(mktemp --directory)"
printf '%s\n' 'registries:' "  ${REGISTRY}:" '    prefix: mirror' > pnpm-workspace.yaml
printf '%s\n' '{ "name": "probe", "private": true, "type": "module" }' > package.json
CI=1 "${PNPM}" add --ignore-scripts "${ARG}"
cat package.json pnpm-lock.yaml
```

Manifest-written entries replace the `add` line with a `dependencies` object in `package.json`
 and `CI=1 "${PNPM}" install --ignore-scripts`.
The session drove these entries from a scratch Node script that did exactly this per entry
 and recorded exit status,
 stderr,
 `package.json` dependencies,
 and the lockfile importer entry.

### Fails on `12.3.4`, `12.4.1`, and unpatched `main`

`ERR_PNPM_INVALID_DEPENDENCY_NAME`:

- `pnpm add mirror:@monochromatic-dev/logger-core@^0.4.0`
- `pnpm add mirror:@monochromatic-dev/logger-core`
- `pnpm add npmjs:@monochromatic-dev/logger-core@^0.4.0`
- `pnpm add gh:@pnpm/foo@^1.0.0`

`ERR_PNPM_RESOLVING_NPM_RESOLVER_NETWORK_ERROR` (404 for `mirror%3Ais-number` on `registry.npmjs.org`):

- `pnpm add mirror:is-number@^7.0.0`
- `pnpm add npmjs:is-number@^7.0.0`

`ERR_PNPM_PACKAGE_MANAGER_ADD_RESOLVE_LATEST`:

- `pnpm add mirror:is-number`,
   also with `mirror` pointed at `https://registry.invalid/`

### Works on `12.3.4`, `12.4.1`, and unpatched `main`

- `pnpm add '@monochromatic-dev/logger-core@mirror:@monochromatic-dev/logger-core@^0.4.0'`
   saves `"@monochromatic-dev/logger-core": "mirror:@monochromatic-dev/logger-core@^0.4.0"`
   and locks `mirror:0.4.0`.
   Routing control:
    with `mirror` pointed at `https://registry.invalid/`,
    the same command fails fetching `https://registry.invalid/@monochromatic-dev%2Fmodule-logger`.
- `pnpm add 'logger@mirror:@monochromatic-dev/logger-core@^0.4.0'`
   saves under `logger` and locks `@monochromatic-dev/logger-core@mirror:0.4.0`.
- `pnpm add 'num@npm:is-number@^7.0.0'`
   (an `npm:` alias control).
- `pnpm install` with a hand-written
  `"@monochromatic-dev/logger-core": "mirror:@monochromatic-dev/logger-core@^0.4.0"`,
   `"logger": "mirror:@monochromatic-dev/logger-core@^0.4.0"`,
   or `"is-number": "npmjs:is-number@^7.0.0"`.
- `pnpm install` with the short form `"@monochromatic-dev/logger-core": "mirror:^0.4.0"`
   (checked on `12.3.4` and patched `main`),
   with the `registry.invalid` routing control failing as expected.

### Works on `11.25.0`

Every `pnpm add` form in the failing catalog except `gh:@pnpm/foo@^1.0.0`,
 which reaches `https://npm.pkg.github.com/@pnpm%2Ffoo` and gets `ERR_PNPM_FETCH_404`
 (the routing works;
 the package does not exist there).
pnpm 11 saves the short form:
 `mirror:@monochromatic-dev/logger-core@^0.4.0` records `"@monochromatic-dev/logger-core": "mirror:^0.4.0"`,
 and `npmjs:is-number@^7.0.0` records `"is-number": "npmjs:^7.0.0"`.

## Verified workarounds

### Name the package twice

```sh
pnpm add '@monochromatic-dev/logger-core@mirror:@monochromatic-dev/logger-core@^0.4.0'
```

The text before the first `@` is a valid alias,
 so the argument never reaches `split_name_spec`.
Tradeoffs:
 the manifest records the typed selector verbatim
 (`mirror:@monochromatic-dev/logger-core@^0.4.0`)
 instead of the pinned short form pnpm 11 writes (`mirror:^0.4.0`),
 so a typed `^0` stays `^0` rather than being rewritten to the resolved version's range,
 and the two pnpm majors produce different manifests for the same command.
Mistyping the second name installs a different package under the first name.

### Write the manifest entry, then install

```json
{
  "dependencies": {
    "@monochromatic-dev/logger-core": "mirror:^0.4.0"
  }
}
```

Then run `pnpm install`.
Tradeoffs:
 manual editing,
 and no resolved-version pinning:
 the range is whatever you write.
The short form works only when the manifest key equals the package name
 and the part after the prefix is a semver range;
 for anything else use the long form `mirror:<name>@<range>`.

### Route the whole scope instead

`@monochromatic-dev:registry=https://registry.example/` in `.npmrc`
 (or a `scopes` list on the `registries` entry)
 needs no prefix in the specifier,
 so a plain `pnpm add @monochromatic-dev/logger-core` works.
Tradeoff:
 every package in the scope now resolves from that registry,
 so it must proxy the scope members it does not host.
This is not a fix for the prefix syntax;
 it trades per-dependency routing for per-scope routing.

## What does not work

- Upgrading to `12.4.1`:
   it contains #14593 and still fails every named-registry form.
- The built-in prefixes:
   `npmjs:` and `gh:` fail exactly like a configured prefix,
   so the failure is not about how `registries` is read.
- Suspecting that `pnpm add` ignores the `registries` setting:
   disproved by the alias-form control,
   whose fetch goes to `https://registry.invalid/` when the prefix points there.
- Omitting the range (`mirror:@scope/pkg`):
   still `ERR_PNPM_INVALID_DEPENDENCY_NAME`.

## Upstream prototype

A minimal fix,
 prototyped against `main` at `e2d0eaedcbcd8033544eb0b774db187db3bb4ef7`
 in a disposable clone whose `origin` was verified as `https://github.com/pnpm/pnpm.git`:
 [pnpm-add-registry-prefix.patch](pnpm-add-registry-prefix.patch).

What it changes,
 all in `pnpm/crates/package-manager/src/add`:

- New `named_registry.rs`:
   `NamedRegistrySelector::parse` merges the configured `registries_by_prefix` over the built-ins
with `merge_named_registries`
   (the validation the install uses)
   and runs `parse_named_registry_specifier_to_registry_package_spec`.
   `save_specifier` picks the version from the prefix's registry
   and renders `<prefix>:<range>`,
   the pnpm 11 manifest shape.
- `specifier.rs`:
   `AddSelector` tries the named-registry parse for alias-less arguments that match no protocol,
   before the alias-less resolvers and `split_name_spec`,
   keys the entry by the package name inside the prefix,
   and saves the named-registry specifier.
- `registry.rs`:
   the version pick and range rendering move out of `resolve_explicit_registry_spec` into `pick_saved_registry_range`,
   which takes the registry explicitly,
   so the named path reuses the same pick policy:
   `minimumReleaseAge`,
   preferred versions,
   and range style.
   The metadata cache key already includes the registry URL
   (`metadata_cache_key(&scope, opts.registry, &spec.name, ..)`
   in `pnpm/crates/resolving-npm-resolver/src/pick_package.rs`),
   so the two registries do not share cache entries.
- `add.rs`:
   `ParseNamedRegistrySpecifier` and `InvalidNamedRegistry` error variants,
   both `#[diagnostic(transparent)]`,
   so the add reports the codes an install of the same specifier reports.
- Tests:
   `add_keys_a_named_registry_selector_by_the_package_inside_the_prefix`
   runs `Add` against a mockito registry declared under the `work` prefix,
   for `work:@corp/lib`,
   `work:@corp/lib@^1.0.0`,
   `work:lib`,
   and `work:lib@^1.0.0`,
   asserting `work:^1.0.0` is saved and the default registry receives no request.

Build and test bounds:
 `podman run --rm --memory=2g --cpus=2` with `docker.io/library/rust:1.97-bookworm`,
 only the clone and a throwaway `CARGO_HOME` mounted,
 no credentials in the environment,
 `RUSTUP_TOOLCHAIN=1.97.1` (the image's toolchain;
 upstream pins `1.97.0`),
 and `CARGO_PROFILE_DEV_DEBUG=0`.
The clone's `.cargo/config.toml` block that replaces crates.io
with the pnpm-managed `.pnpm/crates` vendor directory was deleted locally,
 because that directory is populated by a `pnpm install` the container did not run;
 the patch does not include that edit.

Build command,
 run once on the unpatched clone (binary copied aside) and again after applying the patch:

```sh
podman run --rm --memory=2g --cpus=2 \
  --volume "${CLONE}:/work:Z" --volume "${CARGO_CACHE}:/cargo:Z" \
  --env CARGO_HOME=/cargo --env CARGO_TARGET_DIR=/work/target \
  --env CARGO_PROFILE_DEV_DEBUG=0 --env RUSTUP_TOOLCHAIN=1.97.1 \
  --workdir /work docker.io/library/rust:1.97-bookworm \
  cargo build --package pnpm-cli --bin pnpm --jobs 2
```

Verification command:
 the harness from "Verification",
 with `PNPM` set to each built binary in turn,
 over the whole catalog.

Unpatched `main`:
 the failing catalog fails with the same three error variants.
Patched `main`:

```text
add mirror:@monochromatic-dev/logger-core@^0.4.0  exit=0  {"@monochromatic-dev/logger-core":"mirror:^0.4.0"}  lock mirror:0.4.0
add mirror:is-number@^7.0.0                         exit=0  {"is-number":"mirror:^7.0.0"}                         lock mirror:7.0.0
add mirror:is-number                                exit=0  {"is-number":"mirror:^7.0.0"}                         lock mirror:7.0.0
add mirror:@monochromatic-dev/logger-core         exit=0  {"@monochromatic-dev/logger-core":"mirror:^0.4.0"}  lock mirror:0.4.0
add npmjs:is-number@^7.0.0                          exit=0  {"is-number":"npmjs:^7.0.0"}                          lock npmjs:7.0.0
add npmjs:@monochromatic-dev/logger-core@^0.4.0   exit=0  {"@monochromatic-dev/logger-core":"npmjs:^0.4.0"}  lock npmjs:0.4.0
add gh:@pnpm/foo@^1.0.0                             exit=1  ERR_PNPM_RESOLVING_NPM_RESOLVER_NETWORK_ERROR: 404 from https://npm.pkg.github.com/@pnpm%2Ffoo
add mirror:is-number, mirror -> registry.invalid    exit=1  ERR_PNPM_RESOLVING_NPM_RESOLVER_NETWORK_ERROR: https://registry.invalid/is-number
```

The saved manifests match pnpm `11.25.0` byte for byte on these forms,
 the `gh:` row now reaches GitHub Packages as pnpm 11 does,
 and the `registry.invalid` row proves the prefix routes the lookup.
The alias forms,
 the `npm:` control,
 and the hand-written manifest installs are unchanged.

Unit tests,
 same container bounds:

```text
cargo test --package pnpm-package-manager --lib --jobs 2 -- add::tests::installation
test add::tests::installation::add_keys_a_named_registry_selector_by_the_package_inside_the_prefix ... ok
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 690 filtered out; finished in 0.93s
```

With `add.rs`,
 `specifier.rs`,
 and `registry.rs` put back to `main` while the new test stays,
 the new test fails for the reported reason:

```text
test add::tests::installation::add_keys_a_named_registry_selector_by_the_package_inside_the_prefix ... FAILED
add work:@corp/lib should succeed: Failed to resolve dependency tree: The current package contains a dependency with an invalid name: "work:". [...]
```

`cargo clippy --package pnpm-package-manager --lib --tests` reports no warnings on the patched crate.
Stable `rustfmt` was not a usable check:
 upstream formats with the complexity-aware configuration from pnpm/pnpm#14875,
 and `cargo fmt --check` on untouched `main` files already reports differences.

### Pre-existing test interference found while prototyping

The full `add::` test module failed in some patched runs
 (`add_resolves_package_selectors_concurrently_and_reports_in_selector_order`:
 "the selectors' latest requests never overlapped within 15s",
 or `add_reports_resolution_errors_in_selector_order`:
 `Expected 1 request(s) to: GET /@second%2Fb/latest ...but received 0`).
The cause is not the patch.
`add_does_not_wait_for_a_slower_later_resolution_after_an_error`
 (`pnpm/crates/package-manager/src/add/tests/installation.rs`)
 leaves a mock handler sleeping for 20 seconds,
 and mockito 1.7.2 returns every dropped server to a FIFO free list
 (`ServerPool::recycle` in `mockito-1.7.2/src/server_pool.rs` calls `server.reset()` and `push_back`),
 with nothing in that function waiting for in-flight handlers.
The reproduction is consistent with a later test drawing the still-busy server and stalling;
 the mockito server loop itself was not traced.
Running only
that test plus `add::tests::reporting::` fails the concurrency test in every run on unpatched `main` (2 of 2)
and on the patched tree (2 of 2),
 while whole-module runs that skip the new test pass on both (4 of 4 each);
 adding any test that draws pool servers changes which test receives the busy one.

This belongs in a separate upstream report,
 and is not part of the draft here.

## Upstream filing decision

`.out-of-scope/` was checked:
 it holds entries for `bun-install`,
 `cargo-workspace`,
 `claude-code-upstream-bugs`,
 `codex-harness`,
 `jsr`,
 `lightningcss`,
 `low-impact-typescript-formatting`,
 `module-es-monolith`,
 `pi-gpt55-long-context`,
 `terminal-title-fork-parity-tests`,
 and `typescript-project-references`.
None covers pnpm;
 `jsr.md` concerns JSR packages,
 not pnpm's named-registry prefixes.

Duplicate search (2026-09-14,
 `gh search issues --repo pnpm/pnpm --include-prs`,
 open and closed):
 `named registry add`,
 `ERR_PNPM_INVALID_DEPENDENCY_NAME`,
 `invalid name gh:`,
 `pnpm add gh:`,
 `named registries add`,
 `named registry`,
 plus `gh search prs --state open 'named registry'`.
Closest hits:

- [pnpm/pnpm#14590](https://github.com/pnpm/pnpm/issues/14590) (closed):
   same fallback,
   `jsr:` only.
- [pnpm/pnpm#14593](https://github.com/pnpm/pnpm/pull/14593) (merged):
   the `jsr:`/`npm:`/`workspace:` fix,
   which leaves named registries out.
- [pnpm/pnpm#13393](https://github.com/pnpm/pnpm/issues/13393) (closed):
   `update --latest` not rewriting named-registry specifiers,
   a different command.

No issue or open PR covers `pnpm add <prefix>:<package>`,
 so the artifact is a new-issue draft.

1.   Is it really upstream's fault?
     Yes.
     The `12.x` docs show the failing command,
     pnpm `11.25.0` runs it,
     and the Rust add path drops the named-registry dispatch the TypeScript CLI has
     (root-cause steps 1 to 5).
2.   Can upstream fix it?
     Yes:
     the parser and resolver exist,
     and the prototype wires them into the add selector.
3.   Are they supporting this use case?
     Yes:
     the `registries` `prefix` setting,
     the built-in `gh:` and `npmjs:` aliases,
     and the `pnpm add work:@corp/lib@^2.0.0` example are documented for `12.x`,
     and `pnpm/crates/cli/tests/suite/named_registry_install.rs` tests manifest-written named-registry installs.
4.   Would the repo welcome our contribution?
     Yes,
     with disclosure.
     `CONTRIBUTING.md` "AI-assisted contributions" (read in the `v12.3.4` tree) welcomes agent-made contributions,
     requires the contributor to check linked PRs,
     understand the change,
     run the relevant tests,
     and keep the diff scoped,
     and requires agent-written PRs,
     issues,
     and comments to end with a footer naming the agent and model.
     `pnpm/CONTRIBUTING.md` requires parity with the TypeScript CLI,
     which the prototype matches.
     The bug is a regression,
     so the `.github/ISSUE_TEMPLATE/regression-report.yaml` fields apply.
5.   Will they likely fix it?
     Likely:
     #14590 was fixed and closed the day it was reported by the same class of change.
6.   Have we prototyped a minimal fix compatible with their architecture?
     Yes:
     see "Upstream prototype".
     The new unit test fails without the fix and passes with it,
     clippy is clean,
     and the built CLI matches pnpm `11.25.0` on the catalog.
     Not yet run:
     upstream's `just lint` formatting,
     dylint,
     and the CLI end-to-end suite,
     which the contribution guide expects before a PR.

Decision:
 all six constraints hold,
 so the draft is fileable.
It is not filed from this session:
 the owner files it,
 and a human must confirm the reproduction and patch before the disclosure footer is truthful.

~~~md
Title: `pnpm add <prefix>:<pkg>` fails for named registries in v12 (reads the prefix as the package name)
Template: Regression Report
Labels: regression

### Last pnpm version that worked

11.25.0

### pnpm version

12.3.4, 12.4.1, and main at e2d0eaedcbcd8033544eb0b774db187db3bb4ef7

### Code to reproduce the issue

```sh
cd -- "$(mktemp --directory)"
printf '%s\n' 'registries:' '  https://registry.npmmirror.com/:' '    prefix: mirror' > pnpm-workspace.yaml
printf '%s\n' '{ "name": "probe", "private": true }' > package.json
pnpm add 'mirror:@monochromatic-dev/logger-core@^0.4.0'
pnpm add 'mirror:is-number@^7.0.0'
pnpm add 'npmjs:is-number@^7.0.0'
```

### Expected behavior

As documented for 12.x (https://pnpm.io/settings/dependency-resolution, `pnpm add work:@corp/lib@^2.0.0`)
and as pnpm 11.25.0 does: the package inside the prefix is resolved from the prefix's registry,
and the manifest records `"@monochromatic-dev/logger-core": "mirror:^0.4.0"` / `"is-number": "mirror:^7.0.0"`.

### Actual behavior

Scoped package (also `npmjs:@scope/pkg`, `gh:@scope/pkg`):

```
ERR_PNPM_INVALID_DEPENDENCY_NAME
Failed to resolve dependency tree: The current package contains a dependency with an invalid name: "mirror:".
```

Unscoped package: the default registry is asked for a package literally named `mirror:is-number`
(`ERR_PNPM_RESOLVING_NPM_RESOLVER_NETWORK_ERROR`, 404 for `https://registry.npmjs.org/mirror%3Ais-number`;
without a range, `ERR_PNPM_PACKAGE_MANAGER_ADD_RESOLVE_LATEST`). This happens even when the prefix points at an unreachable URL.

Writing the same specifier into package.json and running `pnpm install` works, and so does
`pnpm add '@monochromatic-dev/logger-core@mirror:@monochromatic-dev/logger-core@^0.4.0'`.

### Additional information

In `pnpm/crates/package-manager/src/add/specifier.rs`, an alias-less argument that is not `npm:`/`jsr:`/`workspace:`
(`ProtocolSelector`, added by #14593 for #14590) and not git/tarball/local falls back to `split_name_spec`,
which takes the text before the first `@` as the package name (`mirror:` or `mirror:is-number`).
The add path never calls `parse_named_registry_specifier_to_registry_package_spec`, which the install resolver uses.
The TypeScript CLI routes these through `resolveLatestFromNamedRegistry` (`pnpm11/resolving/npm-resolver/src/index.ts`).

Suggested fix (prototyped against main, patch available): in `add/specifier.rs`, parse alias-less arguments with
`parse_named_registry_specifier_to_registry_package_spec` over `merge_named_registries(config.registries_by_prefix)`
before the alias-less resolvers; key the manifest entry by the parsed package name; pick the version from the prefix's
registry through the same pick path `resolve_explicit_registry_spec` uses (factored out to take the registry explicitly);
save `<prefix>:<range>`. A mockito test in `add/tests/installation.rs` covers scoped/unscoped, with/without range,
and asserts the default registry gets no request. With the patch, the saved manifests match pnpm 11.25.0 for every form in this report.

Written by an agent (Claude Code, claude-opus-5).
~~~
