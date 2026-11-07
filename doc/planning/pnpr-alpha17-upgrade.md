# pnpr alpha.17 upgrade

## Completed upgrade at 2026-11-05T05:29:26Z

The upgrade landed on `main` as `1f6a6f8dc8`.
The owner selected repairing publishing and rebuilding current versions into a fresh store.
Archive transfer was not selected.
Production publication and anonymous verification of all 169 current archives completed successfully.
The [registry decision](../decision/private-npm-registry.md#production-verification-at-2026-11-05t052926z)
holds the deployment checks and their limits.
The source reviews in this document retain the evidence and alternatives considered before implementation.

## Source review at 2026-11-04T19:11:37-05:00

The owner asked,
"Bump pnpr to 0.1.0-alpha.17 too?"
This records the source findings needed to make that upgrade concrete.
It does not claim deployment or runtime verification has finished.
The Cargo registry proposal remains separate from this authorized pnpr upgrade.

The published tag `pnpr@0.1.0-alpha.17` resolves to
`6b5b8ac53a6120a644c3693ecac8aaed8af88174`.
The GitHub release was published at `2026-10-27T00:33:17Z`.
Both facts were read through `gh api` and the cloned Git tag;
the npm registry also serves `@pnpm/pnpr/0.1.0-alpha.17`.
The comparison baseline is our pinned fork
`01dc1384276b91f2b5a3664b6bf8532617e4e6f9`.

The existing [registry decision](../decision/private-npm-registry.md)
calls for returning to upstream when a release verifies GitHub workload tokens.
It retains anonymous reads,
authenticated publishing,
no stored CI publishing token,
and recovery by republishing current versions.

## The GitHub discovery fix is upstream

Alpha.17 separates workload signing keys from browser-login metadata.
The [discovery document type][oidc-fields] contains `issuer` and `jwks_uri`.
The [discovery implementation][oidc-discovery] fetches those keys
and only parses the full login metadata when `login` is configured.
This removes the missing-`authorization_endpoint` requirement that caused our fork.

The source supports testing the official release instead of porting the fork patch again.
An actual GitHub workload publish still has to pass before the return to upstream is considered verified.
The existing immutable subject and exact repository,
owner,
and workflow claim values remain the intended trust boundary.
The [claims field is a string-keyed map][oidc-claims],
so the configuration spelling change does not rename JWT keys such as `repository_id` or `workflow_ref`.

The official image is [documented as `ghcr.io/pnpm/pnpr`][image-doc].
The [release workflow][image-workflow] builds the image from the same release's Linux musl binaries,
checks their SHA-256 files before staging them,
and publishes `linux/amd64` and `linux/arm64` images with build provenance and an SBOM.
The deployment must pin the verified alpha.17 OCI index digest.
This source review does not invent that digest or treat a mutable tag as verification.

## The existing volume cannot be reused as the new store

Alpha.17 changes both namespace and package paths.
The [layout implementation][layout] adds the ecosystem directory:
the default npm namespace becomes `default/npm/`.
It splits scoped npm names into readable directories
and drops the package basename from versioned blob filenames.
For example,
the alpha.16 shape for `@scope/pkg@1.0.0` is
`default/packages/!40scope!2fpkg/blobs/pkg-1.0.0.tgz`;
the alpha.17 shape is
`default/npm/packages/@scope/pkg/blobs/1.0.0.tgz`.

Both source revisions declare `LAYOUT_VERSION = 1`.
The [marker reader][layout-marker] returns `HostedLayout::Current`
when the marker's number equals that constant.
Therefore the marker is not evidence that the new reader can consume the older paths.
This is source evidence;
the upgrade's disposable-store probe must establish the observed startup and lookup results.

The [alpha.17 command enum][commands] has no `upgrade-storage` command.
The older alpha.11-to-alpha.16 conversion runbook is not an alpha.17 migration procedure.

Use a new storage volume for alpha.17 while retaining the alpha.16 volume intact.
Keep the registry name `monochromatic-dev` and its public URL.
Rollback then selects the older image and its untouched volume together.
No deletion of the older volume is required by the upgrade.

### Preserving existing tarball bytes

The ordinary npm publishing path can accept the original archives.
The [attachment writer][attachments] decodes the submitted base64 bytes,
checks them against the submitted distribution integrity,
and writes the decoded archive.
This suggests replaying captured package documents and tarballs into the new empty registry
when preserving existing consumers' integrity values matters.
It is an implementation option awaiting a disposable end-to-end check,
not a verified migration tool.

Rebuilding the same package version from today's sources does not establish byte equality
with the previously published archive.
The existing no-backup decision permits recovery by republishing current versions,
but it does not prove that rebuilt archives satisfy an older consumer lockfile.
The upgrade must record which recovery method was exercised and its integrity results.

## Configuration and consumer checks

The generated pnpr configuration currently spells the registration cap `max_users`.
Alpha.17's [htpasswd configuration type][htpasswd] requires camelCase,
rejects unknown fields,
and retains `-1` as the disabled-registration sentinel.
The generator must emit `maxUsers` with that same value,
then regenerate its managed `config.yaml`.

The [npm route construction][npm-routes] still mounts npm under `/npm`
and supports the named-registry route.
The existing consumer URL
`https://pnpr.c.aquati.cat/npm/~monochromatic-dev/`
therefore requires no path change in the proposed upgrade.
Anonymous read and authenticated publish behavior still need consumer checks on the released image.

Alpha.17 [serves metrics on the main listener only when explicitly configured][metrics].
The current configuration contains no `metrics` block.
An absent public `/-/metrics` endpoint is therefore expected after this upgrade;
it is not by itself a failed deployment.
Do not enable public metrics merely to preserve the old version probe.
The upgrade must verify the running version through its deployment artifact or another tested surface.

The [CLI source][commands] retains the `config` command group
and removes the standalone `config-check` spelling.
Any verification command must use the actual alpha.17 command interface.

## Cargo remains a separate authentication change

Alpha.17's [workload validation and request check][cargo-auth]
still require a hosted npm registry and exact npm package names.
The request check permits only `PUT` to the named npm package URL.
It does not authorize Cargo's publication endpoint.

The source diagnostic remains:
`OIDC workloads require a hosted npm registry and explicit packages`.
Upgrading alone therefore does not supply keyless Cargo publishing.
The [Cargo protocol note](self-hosted-cargo-registry-protocol.md)
records the other integration boundaries,
including this repository's alternate-registry policy checks.

## Evidence limits and completion checks

This review inspected immutable source revisions and current release metadata.
It did not modify either source checkout,
build a binary,
run a registry,
or change production data.
Runtime evidence belongs in the upgrade's deployment and troubleshooting records.

The upgrade still needs evidence from the selected artifact for:

- Correct generated configuration and an empty-store startup.
- The older-store behavior,
  using a disposable fixture rather than the production volume.
- Anonymous package metadata and tarball reads,
  including distribution integrity.
- Rejection of unauthenticated and malformed-token publishing.
- A successful real GitHub OIDC publish after deployment.
- The selected preservation or republish path,
  including a consumer using an existing lockfile when archives are preserved.

The linked source selections were opened in `agent-browser`
and matched against their rendered source text.
The browser was closed after those checks.

Scoped Markdown lint passed with
`mise run lint:monochromatic-lint -- doc/planning/pnpr-alpha17-upgrade.md`.
Pandoc rendered the GFM document to HTML with `--fail-if-warnings` and exited successfully.
The rendered headings,
links,
code spans,
and lists were inspected.

## Offline migration source review at 2026-11-04T19:28:02-05:00

This archive-transfer alternative was not selected.
The completed upgrade rebuilt current versions instead.

Migration must inventory the stored packages,
including historical names and versions,
rather than use the current publishing workflow's package list.
The proposed migration keeps production publishing trust unchanged:
an isolated temporary alpha.17 server accepts the replay,
then the production server starts with its normal configuration.
This section records source analysis;
the main upgrade work owns the disposable runtime checks and implementation.

### The alpha.16 exporter reads the source store

The [old binary's command dispatch][old-main]
sends maintenance commands directly to `run_maintenance`,
which loads configuration and calls `run_command`.
The export branch calls `export_storage`.
It does not start the server or run server startup recovery.

The [export implementation][old-export] constructs `Storage`
and calls `check_hosted_layout`.
The [local store constructor][old-store] only constructs paths and in-memory locks.
The filesystem export path enumerates package directories,
reads documents,
opens referenced archive files,
and writes its output under `--out`.
It does not call `ensure_hosted_layout`,
`check_readiness`,
`AuthState::load`,
or `recover_publish_journal`.
A read-only mount for the existing filesystem store is therefore supported by the inspected call path.
The actual old-image invocation still needs the read-only-mount check.

The exporter enumerates every stored npm package in each configured hosted namespace,
without using package publish rules or OIDC package lists as an inventory filter.
For the current flat registry,
the namespace is the default namespace.
Its output is `<out>/<package-name>/package.json`,
where `package.json` contains the complete stored packument.
A scoped name becomes nested directories such as `<out>/@scope/name/`.
Referenced tarball files and declared Sigstore bundles sit beside that JSON file.

The exporter takes an archive filename from the final slash-separated component of `dist.tarball`,
after stripping a query or fragment.
It does not URL-decode that filename.
The replay must validate these names and resolve them within the exported package directory.

The [blob export path][old-export-blobs] silently skips a missing source blob
and skips a destination blob that already exists.
An exit status of zero and its package count therefore do not prove a complete export.
Use fresh export scratch on a new attempt,
then compare every declared version and archive against the inventory and declared integrity.
Do not reuse an unchecked partial export.

### A stopped source is necessary for a coherent export

Export does not lock out another registry process,
drain requests,
or replay a pending journal.
A read-only source mount in the migration container does not prevent an older container
from writing through another mount.
The deployment must establish that the old server has finished its writes before export starts.
The new server's dependency on successful migration does not by itself establish that earlier shutdown.

An uncleanly stopped old server can leave a journal that export does not replay.
Its recovery has to be resolved with the old binary before the source is treated as a complete snapshot.
The migration must not silently write to the retained source to perform that recovery.

### Replay and resumption

Node built-ins cover the required operations:
filesystem enumeration and reads,
JSON parsing,
base64 encoding,
hashing,
subprocess management,
and HTTP requests to the local server.
The migration does not require package installation or rebuilding historical packages.
An implementation still has to validate those operations against exported fixtures.

The temporary server and replay client must share one network namespace
if the server binds only to loopback.
The proposed one-shot container has no external network,
mounts the old volume read-only,
and writes only export scratch and the new store.
Its temporary anonymous publishing rules must not become production configuration.

Alpha.17's [publish check][resume-check]
rejects an attachment for a version already present,
even when its archive bytes are identical.
It also rejects a new version without a matching tarball attachment.
Consequently,
blindly replaying the same request is not a successful resume strategy.
For each existing target version,
compare the served bytes and declared integrity with the source;
skip only a verified match and stop on disagreement.
Submit only missing version attachments.

The publish path retains resolution fields from the first stored version.
Complete all input checks before the first publish,
then verify the result through the temporary registry's reads.
Transfer the original distribution tags and supplied publication times as well as the version manifests.
Avoid inventing a new `latest` tag from traversal order.

Replay preserves archive bytes but not every administrative metadata field.
[Anonymous publishing removes `_npmUser`][resume-check],
and [manifest merging changes `time.modified`][replay-time].
Original creation and version times can be supplied and retained.
These changes must be distinguished from archive or dependency changes in verification.

### Authentication files and shutdown

The old export's `--include-auth` option copies configured existing files to
`<out>/htpasswd` and `<out>/tokens.db`.
It does not export SQLite journal sidecars.
The [configuration builder][old-auth-paths] defaults the token database to a sibling of the htpasswd file.
Explicit auth paths are not relocated by the `--storage` override;
the export configuration must point at the source mount.

The [auth writer][old-auth-mode] creates a new Unix destination with mode `0600`.
Its owner is the exporting process,
not an ownership copy of the source file.
The migration output and final auth paths must be writable by the final pnpr process.
The old and new images both declare the `pnpr` user,
but numeric UID and mount ownership must be checked from the actual images and fixture mounts.

The [local auth schema remains version `2`][auth-schema].
The source comparison has no changes to `token_store.rs`,
`token_store/sqlite.rs`,
or `htpasswd.rs` between the pinned alpha.16 fork and alpha.17.
This supports copying those auth files after the temporary replay server exits,
then loading them when the production configuration starts.
Do not load the production auth files into the anonymous replay server.
Account and token preservation still need a fixture check.

Alpha.17 [handles SIGINT and Unix SIGTERM][shutdown]
and passes that shutdown signal into Axum's graceful shutdown path.
The Node parent must await child-process termination before copying auth files
or writing a migration-complete marker.
Only a completed replay,
verification,
auth copy,
and child cleanup should allow production startup.

[oidc-fields]: https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/pnpr/crates/auth/src/oidc/provider_metadata.rs#L28-L42
[oidc-discovery]: https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/pnpr/crates/auth/src/oidc/provider_metadata.rs#L77-L102
[oidc-claims]: https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/pnpr/crates/config/src/oidc.rs#L62-L76
[image-doc]: https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/pnpr/docs/installation.md#L27-L49
[image-workflow]: https://raw.githubusercontent.com/pnpm/pnpm/6b5b8ac53a6120a644c3693ecac8aaed8af88174/.github/workflows/release.yml#:~:text=docker-pnpr%3A,sbom%3A%20true
[layout]: https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/pnpr/crates/storage/src/layout.rs#L24-L68
[layout-marker]: https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/pnpr/crates/storage/src/layout_state.rs#L49-L56
[commands]: https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/pnpr/crates/pnpr/src/cli_commands.rs#L1-L110
[attachments]: https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/pnpr/crates/pnpr/src/server/publishing/attachments.rs#L326-L350
[htpasswd]: https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/pnpr/crates/config/src/config_file.rs#L428-L439
[npm-routes]: https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/pnpr/crates/pnpr/src/server/routing.rs#L265-L295
[metrics]: https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/pnpr/crates/pnpr/src/server/routing.rs#L123-L151
[cargo-auth]: https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/pnpr/crates/pnpr/src/server/oidc.rs#L268-L323
[old-main]: https://github.com/Aquaticat/pnpm/blob/01dc1384276b91f2b5a3664b6bf8532617e4e6f9/pnpr/crates/pnpr/src/main.rs#L50-L102
[old-export]: https://github.com/Aquaticat/pnpm/blob/01dc1384276b91f2b5a3664b6bf8532617e4e6f9/pnpr/crates/pnpr/src/export.rs#L18-L78
[old-store]: https://github.com/Aquaticat/pnpm/blob/01dc1384276b91f2b5a3664b6bf8532617e4e6f9/pnpr/crates/storage/src/local_store.rs#L28-L45
[old-export-blobs]: https://github.com/Aquaticat/pnpm/blob/01dc1384276b91f2b5a3664b6bf8532617e4e6f9/pnpr/crates/pnpr/src/export.rs#L146-L182
[resume-check]: https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/pnpr/crates/pnpr/src/server/publishing/manifest.rs#L18-L118
[replay-time]: https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/pnpr/crates/storage/src/publish.rs#L384-L403
[old-auth-paths]: https://github.com/Aquaticat/pnpm/blob/01dc1384276b91f2b5a3664b6bf8532617e4e6f9/pnpr/crates/config/src/backend_config.rs#L268-L281
[old-auth-mode]: https://github.com/Aquaticat/pnpm/blob/01dc1384276b91f2b5a3664b6bf8532617e4e6f9/pnpr/crates/auth/src/htpasswd.rs#L113-L147
[auth-schema]: https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/pnpr/crates/auth/src/schema.rs#L1-L74
[shutdown]: https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/pnpr/crates/pnpr/src/server/health.rs#L9-L39
