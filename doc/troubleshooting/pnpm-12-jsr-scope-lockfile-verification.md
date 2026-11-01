# pnpm 12.3.4 lockfile verification resolves the `@jsr` scope against registry.npmjs.org and fails every install

## Symptom

With pnpm 12.3.4 active (an uncommitted `mise.lock` bump from 11.21.0,
 observed 2026-09-07),
 every `pnpm install` and `pnpm install --lockfile-only` in this workspace failed,
 in the working checkout and in a pristine `git worktree add` of `HEAD` alike:

```text
? Verifying lockfile against supply-chain policies (781 entries)...
✗ Lockfile failed supply-chain policy check (781 entries in 991ms)
Error: ERR_PNPM_META_FETCH_FAIL

  × installing dependencies
  ╰─▶ ERR_PNPM_RESOLVING_NPM_RESOLVER_NETWORK_ERROR: Failed to fetch metadata
      from https://registry.npmjs.org/@jsr%2Fstd__path: HTTP status client
      error (404 Not Found) for url (https://registry.npmjs.org/
      @jsr%2Fstd__path)
```

`@jsr/std__path` reached the lockfile as `@std/path: npm:@jsr/std__path@^1.1.6`,
 a transitive dependency of `happy-opfs`,
 whose only workspace consumer was `module-fs-path`.
The lockfile entry already pointed at the JSR bridge:

```yaml
# pnpm-lock.yaml (before the fix)
'@jsr/std__path@1.1.6':
  resolution: {integrity: sha512-..., tarball: https://npm.jsr.io/~/11/@jsr/std__path/1.1.6.tgz}
```

The scope route was configured and visible to pnpm 12:

```text
$ pnpm config get @jsr:registry
https://npm.jsr.io/
$ pnpm config list --json | rg jsr
  "@jsr:registry": "https://npm.jsr.io/",
    "https://npm.jsr.io/": {
        "@jsr"
```

Yet the verification pass fetched the packument from `registry.npmjs.org`.

## Root cause

pnpm 12 re-verifies every lockfile entry against `minimumReleaseAge` and `trustPolicy` on install
 (`pnpm-workspace.yaml` sets `minimumReleaseAge: 1440`,
 `minimumReleaseAgeStrict: true`,
 `trustPolicy: no-downgrade`;
 the pass is the one `trustLockfile` documents skipping).
The verifier picks the registry per entry in
`pnpm/crates/resolving-npm-resolver/src/create_npm_resolution_verifier.rs:533` (tag `v12.3.4`,
 commit `666c35e`):

```rust
fn pick_registry(&self, name: &PkgName, tarball_url: Option<&str>) -> String {
    if let Some(url) = tarball_url {
        let normalized = canonical_tarball_url(url);
        for prefix in &self.named_registry_prefixes {
            if normalized.starts_with(&canonical_tarball_url(prefix)) {
                return prefix.clone();
            }
        }
    }
    pick_registry_for_package(&self.registries, &name.to_string(), None)
}
```

`named_registry_prefixes` holds only `namedRegistries` aliases (`gh:` and user-defined ones),
 so the `npm.jsr.io` tarball URL matches nothing there,
 and routing falls through to the scope map in
`pnpm/crates/lockfile/src/resolution.rs:716`:

```rust
pub fn pick_registry_for_package(
    registries: &HashMap<String, String>,
    pkg_name: &str,
    bare_specifier: Option<&str>,
) -> String {
    let scope = match bare_specifier.and_then(|spec| spec.strip_prefix("npm:")) {
        Some(target) => scope_of(target),
        None => scope_of(pkg_name),
    };
    if let Some(scope) = scope
        && let Some(url) = registries.get(scope)
    {
        return url.clone();
    }
    registries.get("default").cloned().unwrap_or_default()
}
```

The map comes from `config.resolved_registries()`
 (`pnpm/crates/package-manager/src/build_resolution_verifiers.rs:101`),
 which is `registries_by_scope` plus `default`
 (`pnpm/crates/config/src/lib.rs:2786`).
`registries_by_scope` is documented as "populated from `.npmrc` `@scope:registry=...`
and the scopes a `pnpm-workspace.yaml#registries` entry declares"
 (`pnpm/crates/config/src/lib.rs:1464`).
The fetch went to the default registry,
 so at verification time the map had no `@jsr` key even though `pnpm config get` reported the route.
The trace stops here:
 which config layer drops the user-level `~/.npmrc` scope route on the verification path was not isolated,
 because the workspace removed its only JSR edge instead (see the second workaround).

A throwaway project with a project-level `.npmrc` holding `@jsr:registry=https://npm.jsr.io/` resolved
`pnpm add '@std/path@npm:@jsr/std__path@1.1.6' --lockfile-only` under pnpm 12.3.4 without error,
 so the route works when declared at project level;
 this workspace declares it only in `~/.npmrc`.

## Verification

Version under test:
 pnpm 12.3.4 (`mise.lock` entry
 `https://github.com/pnpm/pnpm/releases/download/v12.3.4/pnpm-linux-x64.tar.gz`,
 uncommitted),
 source tag `v12.3.4` at commit `666c35e`.
Harness,
 from the workspace root:

```bash
pnpm install --lockfile-only
```

Fails as quoted whenever the lockfile holds an `@jsr/*` entry and the `@jsr` route lives only in `~/.npmrc`.
The same command in a pristine `git worktree add "${HOME}/temp/agent/probe" HEAD` fails identically,
 so no local edit triggers it.

Patterns that work:

- `mise exec pnpm@11.21.0 -- pnpm install --lockfile-only` (the committed toolchain):
   `Done in 4.7s using pnpm v11.21.0`.
- pnpm 12.3.4 in a throwaway project with a project-level `.npmrc` scope route (above).

Patterns that fail:

- pnpm 12.3.4,
   `pnpm install` or `pnpm install --lockfile-only`,
   lockfile containing `@jsr/std__path@1.1.6`,
   route only in `~/.npmrc`.

## Verified workarounds

1.   Run the committed pnpm through mise for lockfile work:
     `mise exec pnpm@11.21.0 -- pnpm install --lockfile-only`.
     Tradeoff:
     the lockfile is written by pnpm 11 while the shell's default pnpm is 12;
     `pnpm --version` and `mise.lock` disagree until the bump is committed or reverted,
     and pnpm 12's verification pass is skipped entirely for that run.
2.   Remove the JSR edge:
     `happy-opfs` left `module-fs-path`,
     the catalog,
     and the lockfile in commit `e3badc3d7`
     (the OPFS backend now uses `navigator.storage.getDirectory()` directly).
     With no `@jsr/*` entry left,
     the verification pass has nothing to route to JSR.
     Measured after the release landed (`3e46e38df`):
     `pnpm install --lockfile-only` under pnpm 12.3.4 prints
     `✓ Lockfile passes supply-chain policies (775 entries in 5.3s)` and leaves the lockfile unchanged.
     Tradeoff:
     structural,
     not a fix of pnpm;
     the next npm package that carries a `@jsr/*` transitive reintroduces the failure.

## What does not work

- Passing the route on the command line:
   pnpm 12's CLI rejects `'--@jsr:registry=https://npm.jsr.io/'` with
   `Usage: pnpm install --lockfile-only --registry <REGISTRY>`.
- Restoring the catalog entry while keeping the package removed:
   the failure is in the verification pass over the existing lockfile,
   not in resolving the removed package,
   so the catalog line changes nothing.

## Upstream filing decision

`.out-of-scope/jsr.md` applies to the JSR side:
 a transitive `@jsr/*` edge belongs to the selected npm package (`happy-opfs`),
 and the workspace replaced that package,
 which is the remedy the policy prescribes.
The pnpm side is a separate question,
 walked here:

1.   Is it really upstream's fault?
     Unproven:
     the trace shows the verifier routing by `registries_by_scope`,
     but the layer that drops a `~/.npmrc` scope route on that path was not isolated.
2.   Can upstream fix it?
     Yes if it is a config-layer omission;
     the verifier already supports scope routing.
3.   Are they supporting this use case?
     Yes:
     `registries_by_scope` is documented as populated from `.npmrc` `@scope:registry`.
4.   Would the repo welcome our contribution?
     `pnpm/pnpm` has `CONTRIBUTING.md`,
     `AGENTS.md`,
     and `REVIEW_GUIDE.md` in the `v12.3.4` tree;
     no ban on AI-assisted reports was read for this note,
     and the files were not reviewed in full.
5.   Will they likely fix it?
     No signal either way:
     `gh search issues --repo pnpm/pnpm 'jsr registry scope 404'` and
     `'supply-chain policy check scoped registry'` returned no matching issue.
6.   Have we prototyped a minimal fix?
     No.

Decision:
 do not file.
Constraint 1 is unproven and constraint 6 is unmet;
 the local trigger is gone,
 so no prototype was attempted.
No draft is kept:
 a report without the isolated cause would not advance the tracker.
Re-open this note if a `@jsr/*` transitive returns and pnpm 12 is the committed toolchain;
 the next step is a project-level `.npmrc` probe in this workspace to confirm the layer,
 then the config-crate trace from `registries_by_scope` backward.
