# pnpr 0.1.0-alpha.17 rejects Cargo registries in OIDC workload bindings

## Symptom

A hosted Cargo registry with an `auth.oidc[].workloads[].registry` binding prevents pnpr from starting.
The official `0.1.0-alpha.17` image exits with status `1` and this diagnostic:

```text
Error:   × Invalid config: OIDC workloads require a hosted npm registry and explicit
  │ packages
```

This is separate from the GitHub discovery failure documented in
[pnpr-oidc-github-actions-discovery.md](pnpr-oidc-github-actions-discovery.md).
Alpha.17 fixes that discovery failure;
its workload authorization still targets npm.

## Root cause

At release commit `6b5b8ac53a6120a644c3693ecac8aaed8af88174`,
[`pnpr/crates/pnpr/src/server/oidc.rs:268`](https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/pnpr/crates/pnpr/src/server/oidc.rs#L268-L298)
validates every configured workload against the npm registry namespace:

```rust
// pnpr/crates/pnpr/src/server/oidc.rs
if !is_hosted_npm_registry(config, &workload.registry) || workload.packages.is_empty() {
    return Err(RegistryError::InvalidConfig {
        reason: "OIDC workloads require a hosted npm registry and explicit packages".to_string(),
    });
}
```

The request guard in
[`pnpr/crates/pnpr/src/server/oidc.rs:304`](https://github.com/pnpm/pnpm/blob/6b5b8ac53a6120a644c3693ecac8aaed8af88174/pnpr/crates/pnpr/src/server/oidc.rs#L304-L324)
also restricts verified workloads to the exact package-bearing npm publication URL:

```rust
// pnpr/crates/pnpr/src/server/oidc.rs
let base = pnpr_registry::Ecosystem::Npm.base_path();
let local = pnpr_registry::Registries::local_name(&workload.registry);
if *method == axum::http::Method::PUT
    && workload.packages
        .iter()
        .any(|package| decoded == format!("{base}/~{local}/{package}"))
```

Cargo publishes through `api/v1/crates/new` and carries the crate identity in its upload metadata.
An extension must authorize that parsed identity as well as the registry and verified workload.
Merely widening the allowed URL would not retain the exact-package restriction.
The protocol and repository integration implications are recorded in
[self-hosted-cargo-registry-protocol.md](../planning/self-hosted-cargo-registry-protocol.md).

## Verification

The source clone was pinned to the deployed fork revision for the alpha.16 comparison,
and the alpha.17 tag was read at the release commit named in "Root cause".
The source selections were checked in the browser.

The failing container used
`ghcr.io/pnpm/pnpr@sha256:884ba0cd30b81428247f46e2d1e818fe3fbccfd54e38a13600e5ffb0a5198f4e`
on Linux x86_64,
with networking disabled,
one CPU,
512 MiB memory,
128 tasks,
and block-I/O weight `10`.
Its disposable YAML was:

```yaml
# config.yaml in a disposable probe directory
secret: disposable-cargo-registry-probe-secret
auth:
  oidc:
    - name: github
      issuer: https://token.actions.githubusercontent.com
      audience: https://pnpr.c.aquati.cat
      workloads:
        - identity:
            subject: 'repo:example/project:ref:refs/heads/main'
            username: ci
          registry: crates
          packages: [probe]
registries:
  crates:
    type: hosted
    ecosystem: cargo
    access: $all
    packages:
      probe:
        publish: ci
defaultRegistry: crates
```

Run the fixture from its directory:

```bash
# Disposable probe directory; no production volume is mounted.
podman run --rm --pull=never --network=none \
  --cpus=1 --memory=512m --pids-limit=128 --blkio-weight=10 \
  --volume "$PWD/config.yaml:/pnpr/config.yaml:ro,z" \
  ghcr.io/pnpm/pnpr@sha256:884ba0cd30b81428247f46e2d1e818fe3fbccfd54e38a13600e5ffb0a5198f4e \
  --config /pnpr/config.yaml --disable-resolver
```

The executed Node harness passed this exact argument vector to Podman.
It asserted the nonzero exit and the diagnostic,
allowing the diagnostic renderer's line wrapping.
No OIDC network request is needed to reach this configuration failure.

Working controls:

- The same official image starts with the repository's npm configuration after `max_users` becomes `maxUsers`.
- GitHub Actions run
  [41941248129](https://github.com/Aquaticats/Monochromatic/actions/runs/41941248129)
  published `@monochromatic-dev/logger-core@0.4.6` into a disposable server with a signed workload credential.
  It downloaded the same 177,253 archive bytes,
  rejected an anonymous write with `401`,
  and rejected an off-target workload write with `403`.

Failing catalog:

- A workload bound to the hosted Cargo registry exits before listening with the quoted invalid-config diagnostic.
- A valid npm workload cannot pass the source request guard on a Cargo publication URL.
  This second case is a source conclusion;
  a signed Cargo HTTP request was not executed.

## Verified workarounds

No keyless Cargo publishing workaround was verified.
The existing npm workload configuration is verified on upstream alpha.17.
The proposed Cargo authorization extension is recorded in
[self-hosted-cargo-registry.md](../planning/self-hosted-cargo-registry.md).

## What does not work

- Bumping pnpr to alpha.17 alone does not enable Cargo workload bindings.
- Binding the same issuer to a Cargo registry fails the startup validator.
- Treating the discovery fix as Cargo publishing support conflates two different boundaries.

## Upstream filing decision

- Upstream owns the limitation,
  but the documented workload feature is npm publishing;
  this was not established as a regression.
- Upstream could extend authorization after parsing bounded Cargo upload metadata.
- The current documentation and validator do not support this combination.
  The supported-use-case condition for an automatic upstream fix prototype is therefore unmet.
- Contribution eligibility was not assessed for an upstream submission because none is proposed.
- No maintainer commitment to this extension was established.
- No source patch was prototyped.
  The present task asked for a Cargo setup recommendation and the separate npm registry upgrade.

The repository's `.out-of-scope/` search found no pnpr-specific filing rule.
The GitHub searches `pnpr cargo oidc` for issues and `pnpr workload cargo` for pull requests,
both scoped to `pnpm/pnpm`,
returned no matches on 2026-11-05.
These searches do not establish that no related discussion exists under different wording.
No external message was sent.
There is no bug filing artifact to add for a documented unsupported combination.
