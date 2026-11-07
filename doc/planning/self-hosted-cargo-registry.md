# Host our Rust crates through pnpr

## Proposal at 2026-11-04T19:15:49-05:00

Extend the existing pnpr deployment with a named hosted Cargo registry.
The owner confirmed that it should host our crates with anonymous downloads,
while third-party dependencies continue to come directly from crates.io.
Reuse the current HTTPS origin,
Coolify deployment,
and Caddy routing.
The proposed registry name is `monochromatic-crates`,
with Cargo index URL `sparse+https://pnpr.c.aquati.cat/cargo/~monochromatic-crates/index/`.
The name and URL are proposed configuration,
not a deployed endpoint.

pnpr already documents Cargo support as introduced in `v0.1.0-alpha.11`.
The `0.1.0-alpha.17` source documents the named-registry sparse index and Cargo API routes.
The architectural next step therefore extends the incumbent's existing Cargo surface.
Source:
[pnpr ecosystem support at the alpha.17 revision][pnpr-cargo].

The owner separately requested the pnpr `0.1.0-alpha.17` upgrade.
That npm upgrade has completed;
the Cargo registry and its authentication extension remain this proposal.
The [alpha.17 upgrade assessment](pnpr-alpha17-upgrade.md)
records its separate storage,
configuration,
and npm workload-publishing checks,
with a link to the completed production verification.

## The owner's answers at 2026-11-07T02:03:01-05:00

- "My thinking is to deploy all releases on pnpr,
  but only publish rc and stable releases on crates.io".
  So the published crate set is every crate,
  every release;
  the crates.io side is in `doc/planning/crates-io-footprint.md`.
- Cargo login from GitHub Actions:
  "Patch pnpr in your fork".
  Add Cargo to pnpr's tokenless (workload) publishing in `Aquaticat/pnpm`,
  run that image,
  and offer the patch upstream,
  as the 2026-09-15 npm decision did (`doc/decision/private-npm-registry.md`, lines 30 to 44).
  Checked at `pnpr@0.1.0-alpha.18` (released 2026-11-05T01:03:59Z):
  `pnpr/crates/pnpr/src/server/oidc.rs` still refuses it,
  line 274 "OIDC workloads require a hosted npm registry and explicit packages"
  and `check_ecosystem` at line 381 accepting only npm.
- pnpr version:
  "Together with Cargo".
  The Cargo patch is written against alpha.18 or newer,
  and the deployment moves to it in the same step.
  alpha.18 keeps package metadata in SQLite and reads no earlier store,
  so all npm packages are published again then (release notes of `pnpr@0.1.0-alpha.18`).
- Crate archive retention:
  "No copies, like npm".
  The no-backup rule of the npm registry covers crates too;
  a lost store breaks outside lock files that pin older crate versions,
  and that is accepted.
- Timing:
  this work waits behind the layout and lint work the owner put first
  ("aside from layout + linting fixes, pause everything else").
- Checked live at 2026-11-07T06:43:38Z:
  `https://pnpr.c.aquati.cat/-/ready` answers `{"status":"ok","storage":"ok","auth":"ok"}`,
  and `https://pnpr.c.aquati.cat/cargo/~monochromatic-crates/index/config.json` answers `404`
  (no Cargo registry is configured yet).

## Authentication is the missing integration

The inspected alpha.17 revision is `6b5b8ac53a6120a644c3693ecac8aaed8af88174`.
Its `validate_single_workload` accepts only hosted npm registries with explicit package names.
The exact diagnostic is `OIDC workloads require a hosted npm registry and explicit packages`.
Its `check_workload_request` then matches an npm `PUT` path containing an allowed package name.
Source:
[pnpr workload validation and request authorization][pnpr-workload].

Cargo publishing uses a shared upload endpoint;
the package name is carried inside the upload metadata.
Source:
[Cargo publishing protocol][cargo-publish].
Consequently,
widening the accepted URL path would not preserve package-specific authorization.
Tokenless Cargo publishing needs a real pnpr authentication extension that verifies the token
and authorizes the parsed registry and crate identity before accepting a publish.

The extension should preserve the existing signed-token checks,
issuer and audience restrictions,
immutable repository and owner IDs,
and workflow claims.
The current identity values are generated into `package/config/pnpr/config.yaml`,
lines 18 to 27.
Cargo metadata parsing must have explicit bounds,
and the authorized crate identity must agree with the uploaded archive and stored index identity.
No implementation of this extension has been selected or adopted.
The existing [npm registry decision](../decision/private-npm-registry.md)
rejected stored publishing tokens;
this proposal preserves workload publishing rather than introducing one implicitly.

## Work to authorize and verify

- [ ] Extend pnpr's Cargo workload authentication and add the named hosted registry through
  `package/config/pnpr`'s existing configuration generator.
  Verify anonymous index and archive reads,
  successful authorized Cargo publication,
  and refusal of anonymous,
  wrong-workflow,
  wrong-registry,
  and wrong-crate uploads.
  Configure no crates.io upstream.
- [ ] Define the published crate set and integrate selection,
  registry-specific packaging,
  and dependency-first publishing with the existing manifest and release owners.
  Inspect normalized archive manifests to prove internal dependencies resolve from the intended registry
  and public dependencies retain their crates.io identities.
  Crates.io releases cannot depend on a crate available only from this registry.
  The published set is not decided by this proposal.
- [ ] Decide Cargo archive retention before deploying storage.
  The npm no-backup decision does not automatically cover Cargo:
  rebuilding a lost version can produce different archive bytes from those recorded in existing lockfiles.
  Retaining exact published archives and accepting loss of historical locked versions are different recovery policies.
- [ ] If this repository will consume its own packages through the registry,
  extend the existing Rust catalog and cargo-shadow owners to carry registry identity through
  dependency declarations,
  policy lookups,
  comparisons,
  and caches.
  The current catalog rejects `registry` dependency keys;
  cargo-shadow's alternate-registry refusal was reproduced.
  This work is conditional:
  publishing from local path dependencies for external consumers does not itself require switching local consumption.
- [ ] Verify the complete publishing and consumption path with real Cargo in disposable fixtures.
  Publish a dependency and a dependent crate,
  then build a fresh external consumer using both a hosted crate and a crates.io dependency.
  Inspect the consumer lockfile sources,
  exercise an unauthorized publish,
  and verify retained archive checksums across a registry restart.
  A registry health response alone does not establish these properties.

The [Cargo protocol and repository integration evidence](self-hosted-cargo-registry-protocol.md)
contains the native Cargo constraints,
source findings for the existing owners,
and the reproduced cargo-shadow boundary.

## Evidence limits

The pnpr alpha.17 code and documentation selections were fetched at the stated commit
and matched in the browser-rendered pages.
Scoped Markdown lint and Pandoc GFM rendering passed;
the proposed URL,
internal links,
and task checkboxes were inspected in renderer output.
The npm upgrade has its own verification work.
This proposal does not claim a successful Cargo workload publish,
a deployed Cargo endpoint,
or a completed external-consumer test.

[pnpr-cargo]: https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/pnpr/docs/ecosystems.md#L132-L153
[pnpr-workload]: https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/pnpr/crates/pnpr/src/server/oidc.rs#L268-L324
[cargo-publish]: https://doc.rust-lang.org/cargo/reference/registry-web-api.html#:~:text=Metadata%20of%20the%20package%20as%20a%20JSON%20object.
