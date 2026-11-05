# pnpr alpha.17 upgrade

## Source review at 2026-11-04T19:11:37-05:00

The owner asked, "Bump pnpr to 0.1.0-alpha.17 too?"
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

[oidc-fields]: https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/pnpr/crates/auth/src/oidc/provider_metadata.rs#L28-L42
[oidc-discovery]: https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/pnpr/crates/auth/src/oidc/provider_metadata.rs#L77-L102
[oidc-claims]: https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/pnpr/crates/config/src/oidc.rs#L62-L76
[image-doc]: https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/pnpr/docs/installation.md#L27-L49
[image-workflow]: https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/.github/workflows/release.yml#L1713-L1805
[layout]: https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/pnpr/crates/storage/src/layout.rs#L24-L68
[layout-marker]: https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/pnpr/crates/storage/src/layout_state.rs#L49-L56
[commands]: https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/pnpr/crates/pnpr/src/cli_commands.rs#L1-L110
[attachments]: https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/pnpr/crates/pnpr/src/server/publishing/attachments.rs#L326-L350
[htpasswd]: https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/pnpr/crates/config/src/config_file.rs#L428-L439
[npm-routes]: https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/pnpr/crates/pnpr/src/server/routing.rs#L265-L295
[metrics]: https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/pnpr/crates/pnpr/src/server/routing.rs#L123-L151
[cargo-auth]: https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/pnpr/crates/pnpr/src/server/oidc.rs#L268-L323
