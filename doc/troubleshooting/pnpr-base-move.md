# pnpr 0.1.0-alpha.16 refuses an unmarked store at startup and answers npm only under /npm

Moving our pnpr fork base from `0.1.0-alpha.11` to `0.1.0-alpha.16`
 (patched image `ghcr.io/aquaticat/pnpr:0.1.0-alpha.16-oidc-workload-discovery.01dc1384276b`,
 upstream `cbe5accc7ace6812e1770f6301d2742c91381c85`) brings two independent breaking changes:

1. The storage layout gate aborts startup on an unmarked store that holds data,
   which crash-loops the container and serves Caddy 502s.
2. The npm registry surface answers only under `/npm`,
   so every root URL (`/~monochromatic-dev/...`) 404s once the new base is deployed.

Both fired in the 2026-10-21 deploy of that base;
 the outage and its rollback are recorded in
[the OIDC discovery topic's fork update](pnpr-oidc-github-actions-discovery.md#fork-update-2026-10-21).

## Symptom

### Storage gate: startup abort and 502

Deploying the new base over the existing store,
 the container exits before `pnpr listening` and restarts;
 Caddy answers every registry URL with `502`.
 The container log holds:

```text
Error:   × Storage is temporarily unavailable: the hosted store has no layout.json
│ and holds data in the earlier layout. Back it up and run `pnpr upgrade-`
│ storage` to convert it
```

The same gate runs at every startup of this base,
 so the container cannot stay up until the store is converted.

### npm URL move: root npm URLs answer 404

With the new base running against a converted store,
 the URLs this repo documents and publishes to,
 for example `GET /~monochromatic-dev/@monochromatic-dev%2fmodule-or-throw`,
 answer `404` with an empty body.
 The same packument answers `200` at `/npm/~monochromatic-dev/@monochromatic-dev/or-throw-core`.
 An unauthenticated publish probe at the root URL answers `404` (empty body)
 instead of the `401 Authentication required for package "..."` the old base returns.

## Root cause

### Startup refuses an unmarked store that holds data

`serve` and `serve_listener` call `recover_publish_journal` before binding
 (`pnpr/crates/pnpr/src/server/lifecycle.rs:23`),
 which runs the layout gate as its first storage operation:

```rust
// pnpr/crates/pnpr/src/server/lifecycle.rs:23
pub async fn recover_publish_journal(config: &Config) -> pnpr_error::Result<()> {
    let storage = Storage::new(
        &config.storage.hosted_backend,
        config.storage.hosted_dir.clone(),
        config.storage.cache_dir.clone(),
    )?;
    storage.ensure_hosted_layout().await?;
```

`ensure_hosted_layout` refuses a store in the earlier layout
 (`pnpr/crates/storage/src/layout_state.rs:86`):

```rust
// pnpr/crates/storage/src/layout_state.rs:86
pub async fn check_hosted_layout(&self) -> Result<HostedLayout> {
    match self.hosted_layout().await? {
        HostedLayout::Unmarked { has_data: true } => Err(RegistryError::StorageUnavailable {
            reason: format!(
                "the hosted store has no {LAYOUT_FILE} and holds data in the earlier layout. \
                 Back it up and run `pnpr upgrade-storage` to convert it",
            ),
        }),
```

`has_data` is true when any top-level directory name does not start with `.`
 (`pnpr/crates/storage/src/layout_state.rs:47`),
 which every store written by `0.1.0-alpha.11` is:
 packages sit as `<store root>/<name>/package.json` with sibling blobs.
 That path shape is the earlier layout's reader
 (old `pnpr/crates/storage/src/local_store.rs:257` at base `8f20a3f`):

```rust
// pnpr/crates/storage/src/local_store.rs:257 (base 8f20a3f, the 0.1.0-alpha.11 reader)
!name.is_empty() && fs::try_exists(self.root.join(name).join(DOCUMENT_FILE)).await?;
```

The current layout instead writes `layout.json` markers and encoded package trees
 (`pnpr/crates/storage/src/layout.rs:22`):

```rust
// pnpr/crates/storage/src/layout.rs:22
pub const LAYOUT_VERSION: u32 = 1;
pub const LAYOUT_FILE: &str = "layout.json";
pub(crate) const PACKAGES_DIR: &str = "packages";
pub(crate) const DOCUMENT_FILE: &str = "document.json";
pub(crate) const BLOBS_DIR: &str = "blobs";
const DEFAULT_NAMESPACE: &str = "default";
```

`pnpr upgrade-storage` converts the earlier layout into that tree and writes the marker last,
 so an interrupted conversion resumes
 (documents move after their blobs,
`pnpr/crates/pnpr/src/upgrade_storage.rs`).
 It drops the earlier `.package-index` and `.pnpr-uploads` records
 (`const DROPPED: [&str; 2] = [".package-index", ".pnpr-uploads"];`,
 `pnpr/crates/pnpr/src/upgrade_storage/plan.rs:19`),
 and refuses stores it cannot convert unambiguously
 (`check_unambiguous`,
`pnpr/crates/pnpr/src/upgrade_storage/plan.rs:49`).

### npm routes always mount under the ecosystem base

The earlier base mounted the npm surface at the site root whenever npm was the only ecosystem
 (`pnpr/crates/pnpr/src/server/routing.rs` at base `8f20a3f`,
lines 232-236):

```rust
// pnpr/crates/pnpr/src/server/routing.rs (base 8f20a3f)
if registries.is_only_ecosystem(Ecosystem::Npm) {
    router = router.merge(npm);
} else {
    router = router.nest("/npm", account_routes().merge(npm));
}
```

Production declares only npm,
 so its URLs sit at the root:
 a forged workload publish probe against `https://pnpr.c.aquati.cat/~monochromatic-dev/@monochromatic-dev%2fprobe-nonexistent`
 answered `401 Authentication required for OIDC credentials` on 2026-10-21,
 after the rollback restored `0.1.0-alpha.11`.

The current base always nests under the ecosystem prefix
 (`pnpr/crates/pnpr/src/server/routing.rs:222` and `:231`):

```rust
// pnpr/crates/pnpr/src/server/routing.rs:222
fn registry_routes(state: &AppState, router: Router<AppState>) -> Router<AppState> {
    let registries = &state.inner.config.routing.registries;
    let npm = npm_registry_routes();
    ...
    if registries.has_ecosystem(Ecosystem::Npm) {
        router = router.nest(&Ecosystem::Npm.base_path(), account_routes().merge(npm));
    }
```

`base_path` is the ecosystem name under a slash
 (`pnpr/crates/package-name/src/lib.rs:39`):

```rust
// pnpr/crates/package-name/src/lib.rs:39
pub fn base_path(self) -> String {
    format!("/{self}")
}
```

The registry directory endpoint confirms the mount
 (`GET /-/pnpr/v0/registries` on the new base):

```json
{"registries":[{"name":"monochromatic-dev","kind":"hosted","ecosystem":"npm"}],
 "defaultRegistries":{"npm":"monochromatic-dev"},
 "ecosystems":{"npm":{"available":true,"prefixed":true}}}
```

`"prefixed": true` is the npm surface living at `/npm`,
 including the path-less form (`/npm/@scope/name`) and the named form (`/npm/~<registry>/...`).

### Readings ruled out

- "The conversion lost or corrupted the migrated documents."
  Ruled out on the converted fixture:
  `pnpr verify` reports `Packages checked: 1` and `Versions checked: 1`,
  `pnpr export --out` reports `Packages: 1`,
  and `GET /npm/~monochromatic-dev/@monochromatic-dev/or-throw-core` answers `200`
  with the exact document body.
  The earlier `404`s were probes against the root surface.
- "The `404` is a claim or gate mask (`RegistrySource::Unclaimed`,
  `hosted_gate::MaskNotFound`)."
  The read path resolves:
  the `@monochromatic-dev/*` pattern claims the name
  (`PackagePattern::Scope`,
  `pnpr/crates/registry/src/package_pattern.rs:145`),
  the gate allows `$all` and reads the `default` namespace
  (`pnpr/crates/pnpr/src/server/package_reads.rs:369`),
  and the document path matches the converted tree
  (`pnpr/crates/storage/src/local_store.rs:253`).
  The surface difference settled it.
- "`0.1.0-alpha.11` would reject the migrated `ui:` config block."
  Retracted earlier and wrong:
  that base drops unknown top-level fields.
  Recorded in
  [the OIDC discovery topic's config schema change](pnpr-oidc-github-actions-discovery.md#config-schema-change).

## Verification

### Versions and images under test

- Patched image `ghcr.io/aquaticat/pnpr:0.1.0-alpha.16-oidc-workload-discovery.01dc1384276b@sha256:09a9818eaebba777848f3d01382c97143e232d95085ea4950408ad436505109c`,
  built by `.github/workflows/pnpr-patched-image.yml` run `39646035147`;
  port commit `01dc1384276b91f2b5a3664b6bf8532617e4e6f9` on upstream `cbe5accc7ace6812e1770f6301d2742c91381c85`.
  Local runs used `localhost/config-pnpr:dev` built from `package/config/pnpr/Containerfile`.
- Comparison base `8f20a3f` (`0.1.0-alpha.11` reader and router),
  and the `@pnpm/pnpr@0.1.0-alpha.16` release binary for `pnpr config-check` and `upgrade-storage` output.
- Production behavior quoted here was measured against `https://pnpr.c.aquati.cat`
  on 2026-10-21 after rollback restored the `0.1.0-alpha.11` image.

Config under test is the generated `package/config/pnpr/config.yaml`
 (`pnpr config-check`):

```text
Registries (1):
  - monochromatic-dev (hosted, 1 patterns, org: "")
Default registry: monochromatic-dev
Storage: local filesystem (/pnpr/storage)
Cache: /pnpr/storage/.pnpr-cache
Auth: htpasswd (/pnpr/storage/htpasswd, registration: disabled)
Surfaces: registry=true, resolver=true, artifacts=false
```

### Harness

Runs on the host with podman;
 `IMAGE` is the patched image pinned above.
 The container runs as uid 999,
 so fixture directories are opened with `chmod 0777` for the bind mount.

```bash
# fixture: one package in the earlier layout
export REPRO="${HOME}/temp/agent/pnpr-base-move-repro"
mkdir --parents "${REPRO}/st-legacy/@monochromatic-dev/or-throw-core"
printf '%s' '{"name":"@monochromatic-dev/or-throw-core","dist-tags":{"latest":"1.0.0"},"versions":{"1.0.0":{"name":"@monochromatic-dev/or-throw-core","version":"1.0.0"}}}' > "${REPRO}/st-legacy/@monochromatic-dev/or-throw-core/package.json"
printf 'synthetic-blob' > "${REPRO}/st-legacy/@monochromatic-dev/or-throw-core/index.tgz"
chmod 0777 "${REPRO}" "${REPRO}/st-legacy" "${REPRO}/st-legacy/@monochromatic-dev" "${REPRO}/st-legacy/@monochromatic-dev/or-throw-core"
```

```bash
# startup on the unmarked store: exits before "pnpr listening"
podman run --rm --name pnpr-t-crash --env PNPR_SECRET=local-throwaway-secret-0123456789abcdef --volume "${REPRO}/st-legacy:/pnpr/storage:z" "${IMAGE}" --config /pnpr/config.yaml --storage /pnpr/storage
```

```bash
# the remedy: convert, then re-run to see the idempotent message
podman run --rm --name pnpr-t-upg --env PNPR_SECRET=local-throwaway-secret-0123456789abcdef --volume "${REPRO}/st-legacy:/pnpr/storage:z" "${IMAGE}" --config /pnpr/config.yaml --storage /pnpr/storage upgrade-storage
podman run --rm --name pnpr-t-upg2 --env PNPR_SECRET=local-throwaway-secret-0123456789abcdef --volume "${REPRO}/st-legacy:/pnpr/storage:z" "${IMAGE}" --config /pnpr/config.yaml --storage /pnpr/storage upgrade-storage
```

```bash
# serve the converted store and probe both surfaces
podman run --detach --name pnpr-t-srv --env PNPR_SECRET=local-throwaway-secret-0123456789abcdef --volume "${REPRO}/st-legacy:/pnpr/storage:z" --publish 127.0.0.1:17690:7677 "${IMAGE}" --config /pnpr/config.yaml --storage /pnpr/storage
curl --silent --write-out ' %{http_code}\n' 'http://127.0.0.1:17690/-/ping'
curl --silent --write-out ' %{http_code}\n' 'http://127.0.0.1:17690/npm/~monochromatic-dev/@monochromatic-dev/or-throw-core'
curl --silent --write-out ' %{http_code}\n' 'http://127.0.0.1:17690/npm/@monochromatic-dev/or-throw-core'
curl --silent --write-out ' %{http_code}\n' 'http://127.0.0.1:17690/~monochromatic-dev/@monochromatic-dev/or-throw-core'
curl --silent --request PUT --header 'content-type: application/json' --data '{}' --write-out ' %{http_code}\n' 'http://127.0.0.1:17690/npm/~monochromatic-dev/@monochromatic-dev%2fmodule-or-throw'
podman stop pnpr-t-srv
```

`PNPR_SECRET` above is a local throwaway value for the harness only;
 the deployed instance reads its 64-character secret from the Coolify host.

### Store contents that survive the conversion

Works cleanly,
 measured on the fixture above:

- `upgrade-storage --dry-run` prints `Would move 1 documents` and `Would move 1 other files`
  and changes nothing.
- `upgrade-storage` prints `Moved 1 documents` and `Moved 1 other files`.
- The converted tree is `default/packages/!40monochromatic-dev!2fmodule-or-throw/{document.json,blobs/index.tgz}`
  plus `layout.json` (`{"layout":1}`) at the store root
  and `.pnpr-cache/layout.json`.
  `!40` and `!2f` are the escaped `@` and `/` of the package name
  (`ESCAPE = b'!'`,
  `pnpr/crates/storage/src/layout.rs:29`).
- `pnpr verify` passes the document:
  `Packages checked: 1`,
  `Versions checked: 1`,
  `Missing blobs: 0`,
  `Corrupted blobs: 0`.
- `pnpr export --out <dir>` round-trips it:
  `Export completed successfully: Packages: 1, Versions: 1`.
  The synthetic version declares no `dist.tarball`,
  so `verify` reports its blob as `ORPHANED` and `export` reports `Blobs: 0`;
  both are fixture artifacts,
  not conversion losses.

### Requests that answer on the new base

Works cleanly:

- `GET /-/ping` → `{}`.
- `GET /-/ready` → `{"status":"ok","storage":"ok","auth":"ok"}`.
- `GET /npm/~monochromatic-dev/@monochromatic-dev/or-throw-core` → `200` and the packument JSON.
- `GET /npm/@monochromatic-dev/or-throw-core` → `200` and the same packument
  (path-less base addresses the default registry).
- `PUT /npm/~monochromatic-dev/@monochromatic-dev%2fmodule-or-throw` unauthenticated
  → `401 Authentication required for package "@monochromatic-dev/or-throw-core"`.

### Requests and boots that fail

Failing catalog,
 split by variant:

- Startup on an unmarked store holding data
  → the `Storage is temporarily unavailable` error quoted in Symptom,
  no `pnpr listening` line,
  container exit.
  In production this surfaced as Caddy `502` on every URL.
- `GET /~monochromatic-dev/@monochromatic-dev/or-throw-core` and the `%2f` spelling
  → `404` empty body on the new base
  (on `0.1.0-alpha.11` the same paths answer `200` or `401`).
- `PUT /~monochromatic-dev/@monochromatic-dev%2fmodule-or-throw` unauthenticated
  → `404` empty body on the new base
  instead of `401 Authentication required for package "..."`.
- `upgrade-storage` on a store marked with another layout version
  → `the hosted store uses layout <n>, and this pnpr converts only the unmarked layout to layout 1`.
- Running the earlier reader (`0.1.0-alpha.11`) against a converted store
  → it looks for `<store root>/<name>/package.json`
  (old `pnpr/crates/storage/src/local_store.rs:257`),
  which the conversion moved away
  (`Moved 1 documents`),
  so it serves nothing there.

## Verified workarounds

### Run `pnpr upgrade-storage` before the base move

Measured end to end with the harness above:
 dry run previews,
 the run converts and writes the marker last,
 and a second run prints `The hosted store already has a current layout.json; nothing to do.`

Tradeoffs:

- The conversion is one-way for the live store:
  the earlier reader cannot see the converted tree.
  The error message says `Back it up and run \`pnpr upgrade-storage\``,
  but this repo keeps its standing no-backup decision
  (owner decision 2026-10-21:
   no pre-conversion snapshot,
   recovery republishes current versions),
  so a rollback after the conversion empties the converted store
  and lets the `pnpr-publish` workflow republish
  (runbook section "Move to a newer pnpr base").
- The CLI mandates stopping every pnpr process that uses the store first
  (`pnpr upgrade-storage --help`),
  so it runs in a deploy window with the container stopped.
- The conversion drops `.package-index` and `.pnpr-uploads`
  (`pnpr/crates/pnpr/src/upgrade_storage/plan.rs:19`);
  anything that relied on those records must rebuild them.

### Move client registry URLs under /npm in the same deploy

The repo's own URLs move together with the repin,
 in one lockstep commit
 so no intermediate state publishes against the wrong surface:

- Publish tooling sets its registry target to `${origin}/~${registryName}/`
  (`package/config/pnpr/src/publish-steps.ts:299`) and reads packuments at `${origin}/<name>`
  (`package/config/pnpr/src/registry-read.ts:144`).
  Both become `/npm/...` forms.
- Consumer route becomes `@monochromatic-dev:registry=https://pnpr.c.aquati.cat/npm/~monochromatic-dev/`
  (the root form recorded in `doc/decision/private-npm-registry.md` stays as the historical value,
  with the move noted there).
- Test fixtures quoting root URLs,
  for example `package/config/pnpr/src/publish-retry.unit.test.ts`,
  move with them.

Tradeoffs:

- Consumer lockfiles written before the move record root tarball URLs,
  because packument responses rewrite `dist.tarball` to the serving base
  (`pnpr/crates/pnpr/src/server/package_reads.rs:268`).
  Those entries need re-resolution after the move;
  frozen-lockfile installs fail until they do.
- The change is only correct on the new base:
  `0.1.0-alpha.11` answers the `/npm/...` forms with `404`,
  so it must not land before the repin.

### Keep root URLs with a reverse-proxy rewrite (proposed, host-side)

The host's Caddy block (`doc/runbook/deploy-pnpr-registry.md`,
Stage C) is a plain `reverse_proxy`.
 A path rewrite or redirect there can map `/~<registry>/` onto `/npm/~<registry>/`,
 keeping old URLs and old lockfiles working.

Tradeoffs:

- Not run here (no agent access to the Coolify host),
  so treat it as a proposal and verify against the host's Caddy version before relying on it.
- It keeps two public spellings alive forever and hides the real surface from new clients.

## What does not work

- Repinning `package/config/pnpr/Containerfile` without converting the store first.
  Tried on 2026-10-21;
  the container crash-loops and production answers `502`.
- Serving the converted store from the `0.1.0-alpha.11` image.
  The earlier reader looks for `<store root>/<name>/package.json`,
  which the conversion moved.
- Deciding whether the migration lost documents by probing the root URLs.
  Wrong surface;
  `pnpr verify`,
  `pnpr export --out`,
  and the `/npm/...` reads all see the documents.
  Named here so the reading is not re-derived.
- `pnpr export <dir>` (positional).
  The subcommand takes `--out <DIR>` (`unexpected argument '/export' found`).
  The directory must also exist and be writable by the container uid
  (`Permission denied (os error 13)` otherwise).
- Config with the old `web:` block on the new base.
  `pnpr config-check` rejects it (`unknown field \`web\`,
  expected one of ... ui ...`).

## Upstream filing decision

`.out-of-scope/` holds no pnpr entry
 (checked 2026-10-21),
 and `gh search issues --repo pnpm/pnpm 'upgrade-storage'`
 plus searches for `authorization_endpoint` and the URL surface
 returned no matching issue or pull request
 (checked 2026-10-21;
 hits were unrelated:
pnpm/pnpm#8002,
#14758,
#5797,
#2694,
#11131).
 The standing 2026-09-15 decision that agents do not file upstream themselves also applies.
 Both behaviors are deliberate upstream designs,
 so the walk ends at constraint 1 and nothing is filed.

### Storage gate

1. **Is it really upstream's fault?**
   No.
   The gate is a deliberate fail-closed design with its remedy named in the error text.
2. **Can upstream fix it?**
   Not applicable;
   there is no defect to fix.
3. **Are they supporting this use case?**
   Yes:
   `pnpr upgrade-storage` exists for exactly this conversion,
   and its help text covers the sequencing.
4. **Would the repo welcome our contribution?**
   Not applicable at this constraint,
   since the walk stopped at constraint 1.
   No contribution is proposed.
5. **Will they likely fix it?**
   Not applicable;
   nothing is broken on their side.
6. **Have we prototyped a minimal fix?**
   Not required.
   Constraints 1-5 do not hold,
   so the auto-prototype trigger does not fire.

### npm URL move

1. **Is it really upstream's fault?**
   No.
   The always-nest mount is a routing simplification,
   and the directory endpoint announces it (`"prefixed": true`).
   Our URLs are the stale surface.
2. **Can upstream fix it?**
   Not applicable;
   nothing is broken on their side.
3. **Are they supporting this use case?**
   Yes;
   both surfaces are declared
   (`mount_bases`,
   `pnpr/crates/pnpr/src/server/ecosystem.rs:44`).
4. **Would the repo welcome our contribution?**
   Not applicable at this constraint.
5. **Will they likely fix it?**
   Not applicable.
6. **Have we prototyped a minimal fix?**
   Not required;
   the auto-prototype trigger does not fire.

Nothing to add upstream:
 no new-issue draft and no comment draft are kept for these two behaviors.
 The OIDC discovery defect that the patched image carries is tracked separately in
[its own topic](pnpr-oidc-github-actions-discovery.md#upstream-filing-decision).
