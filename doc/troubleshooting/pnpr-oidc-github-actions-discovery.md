# pnpr 0.1.0-alpha.11 rejects every GitHub Actions workload publish with HTTP 500 "OIDC provider unavailable"

## Symptom

A GitHub Actions job publishes to pnpr with a `pnpr_workload_`-prefixed ID token,
 as `pnpr/crates/pnpr/OIDC.md` documents,
 and npm prints:

```text
npm error 500 Internal Server Error - PUT https://pnpr.c.aquati.cat/~monochromatic-dev/@monochromatic-dev%2fconfig-typescript
```

pnpr logs:

```json
{"level":"ERROR","fields":{"message":"request failed","err":"Internal error: OIDC provider unavailable","error_kind":"internal","status":"500 Internal Server Error"},"target":"pnpr_error"}
```

Every token whose `iss` is `https://token.actions.githubusercontent.com` gets this response,
 including tokens with a forged signature,
 so the failure happens before pnpr looks at the signature or the claims.
It showed up in `pnpr-publish` run 34913557042 for every package that reached its publish step.

## Root cause

### pnpr parses GitHub's discovery document as full provider metadata

A request whose bearer token starts with `pnpr_workload_` goes to OIDC workload verification
 (`pnpr/crates/pnpr/src/server/authentication.rs:181`):

```rust
if let Some(jwt) = raw_token.strip_prefix("pnpr_workload_") {
    let workload = state.inner.identity.oidc
        .workload(jwt)
        .await?
```

Verification first loads the issuer's metadata
 (`pnpr/crates/auth/src/oidc/workload_verification.rs:37`):

```rust
pub(super) async fn verify_workload_token(
    &self,
    provider: &Provider,
    raw: &str,
) -> Result<bool> {
    let metadata = self.metadata(provider, false).await?;
```

`metadata` runs openidconnect discovery,
 and any discovery error becomes `unavailable()`
 (`pnpr/crates/auth/src/oidc.rs:352` to `354`):

```rust
let issuer = IssuerUrl::new(provider.config.issuer.clone()).map_err(|_| rejected())?;
let metadata =
    CoreProviderMetadata::discover_async(issuer, self).await.map_err(|_| unavailable())?;
```

`unavailable()` is an internal error,
 which pnpr answers with HTTP 500
 (`pnpr/crates/auth/src/oidc.rs:421`):

```rust
fn unavailable() -> RegistryError {
    RegistryError::Internal { reason: "OIDC provider unavailable".to_string() }
}
```

### openidconnect requires `authorization_endpoint`; GitHub Actions does not publish one

`CoreProviderMetadata` is openidconnect's `ProviderMetadata`,
 whose `authorization_endpoint` field is not optional
 (openidconnect-rs 4.0.1,
 `src/discovery/mod.rs:60` to `64`):

```rust
issuer: IssuerUrl,
authorization_endpoint: AuthUrl,
token_endpoint: Option<TokenUrl>,
userinfo_endpoint: Option<UserInfoUrl>,
jwks_uri: JsonWebKeySetUrl,
```

GitHub Actions' issuer only mints ID tokens for workflow jobs,
 so `https://token.actions.githubusercontent.com/.well-known/openid-configuration`
 lists `issuer`,
 `jwks_uri`,
 `subject_types_supported`,
 `response_types_supported` (`["id_token"]`),
 `claims_supported`,
 `id_token_signing_alg_values_supported`,
 and `scopes_supported`,
 with no `authorization_endpoint` (fetched 2026-09-15).
Deserialization therefore fails,
 discovery fails,
 and every GitHub workload token gets HTTP 500.

pnpr's own tests do not catch it:
 `verifies_rs256_workload_tokens` uses the GitHub issuer string
 but builds metadata from the `metadata` test helper,
 which always includes `authorization_endpoint`
 (`pnpr/crates/auth/src/oidc/tests.rs:67`):

```rust
fn metadata(issuer: &str) -> Value {
    json!({"issuer": issuer, "authorization_endpoint": format!("{issuer}/authorize"),
```

### Readings ruled out

- The Hetzner egress firewall (`package/config/tofu/hetzner.tf`) was the first suspect.
   It is not the cause:
   a local container with unrestricted egress returns the same 500,
   and `140.82.112.0/20`,
    which `token.actions.githubusercontent.com` resolved into (`140.82.114.22`),
    is in GitHub's `api.github.com/meta` ranges that the firewall allows.
- A claim,
   audience,
   or subject mismatch in `package/config/pnpr/config.yaml` would not return 500.
   Once metadata loads,
    a failed signature check refetches metadata at most once per 30 seconds
    and then returns `Ok(false)`,
    which ends in `rejected()`,
    an HTTP 401 (`pnpr/crates/auth/src/oidc/workload_verification.rs:43` to `48`).

## Verification

Versions under test:
 `@pnpm/pnpr@0.1.0-alpha.11` (npm dist-tag `next` on 2026-09-15,
 released from pnpm/pnpm `19eb39448649c926bc63b0e9fa16f0e340701460`),
 source read at pnpm/pnpm `8f20a3fd69748b1f6eb5c6a7c54d5917456a2013`,
 whose `pnpr/crates/auth/src` matches `f38f11ec1054255497148354ff97ef6ed2434c78`;
 openidconnect 4.0.1 (`b639b5d39eac6903238867aeb2b29326502e6b26`).
The discovery code in `oidc.rs` at the release commit differs from `8f20a3f` only in formatting and module layout.

### Registry harness

A GitHub-issuer JWT with a forged signature,
 sent as a workload publish
 (`probe-oidc-discovery.ts`,
 run with `node probe-oidc-discovery.ts <origin>`):

```ts
// probe-oidc-discovery.ts
const origin = process.argv[2] ?? 'https://pnpr.c.aquati.cat';
const encode = (value: object): string => Buffer.from(JSON.stringify(value)).toString('base64url');
const now = Math.floor(Date.now() / 1000);
const jwt = [
  encode({ alg: 'RS256', typ: 'JWT', kid: 'probe' }),
  encode({
    iss: 'https://token.actions.githubusercontent.com',
    aud: 'https://pnpr.c.aquati.cat',
    sub: 'repo:Aquaticat/Monochromatic:ref:refs/heads/main',
    iat: now,
    exp: now + 300,
  }),
  Buffer.from('not-a-signature').toString('base64url'),
].join('.');
const response = await fetch(`${origin}/~monochromatic-dev/@monochromatic-dev%2fprobe-nonexistent`, {
  method: 'PUT',
  headers: { authorization: `Bearer pnpr_workload_${jwt}`, 'content-type': 'application/json' },
  body: '{}',
});
console.log(response.status, await response.text());
```

Outcomes:

- `https://pnpr.c.aquati.cat` (Coolify,
   Hetzner egress firewall):
   `500 Internal Server Error`.
- `http://127.0.0.1:17677` (`mise run //package/config/pnpr:start:image`,
   unrestricted egress):
   `500 Internal Server Error`,
   logged as `Internal error: OIDC provider unavailable` after 550 ms,
   the time the discovery fetch took.

### Metadata parse harness

```toml
# Cargo.toml
[dependencies]
openidconnect = { version = "=4.0.1", default-features = false }
serde_json = "1"
```

```rust
// src/main.rs
use openidconnect::core::CoreProviderMetadata;

fn main() {
    let raw = std::fs::read_to_string("github-actions-openid-configuration.json").unwrap();
    match serde_json::from_str::<CoreProviderMetadata>(&raw) {
        Ok(_) => println!("as served: parsed"),
        Err(error) => println!("as served: {error}"),
    }
    let mut patched: serde_json::Value = serde_json::from_str(&raw).unwrap();
    patched["authorization_endpoint"] =
        "https://token.actions.githubusercontent.com/authorize".into();
    match serde_json::from_value::<CoreProviderMetadata>(patched) {
        Ok(_) => println!("with authorization_endpoint added: parsed"),
        Err(error) => println!("with authorization_endpoint added: {error}"),
    }
}
```

```bash
# harness directory, with GitHub's document saved beside Cargo.toml
curl --silent --output github-actions-openid-configuration.json https://token.actions.githubusercontent.com/.well-known/openid-configuration
podman run --rm --memory=2g --cpus=2 --volume "$PWD:/work:Z" --workdir /work docker.io/library/rust:1.97-bookworm cargo run --quiet
```

```text
as served: missing field `authorization_endpoint` at line 55 column 1
with authorization_endpoint added: parsed
```

### Documents that parse and documents that fail

Parse:

- Any document carrying `issuer`,
   `authorization_endpoint`,
   `jwks_uri`,
   `response_types_supported`,
   `subject_types_supported`,
   and `id_token_signing_alg_values_supported`,
   such as pnpr's `metadata` test helper
   or GitHub's document with `authorization_endpoint` added.

Fail with `missing field authorization_endpoint`:

- GitHub Actions' document as served.
- Any workload-only issuer that omits the login endpoints.

## Verified workarounds

### Patch pnpr: workload-only providers read only `issuer` and `jwks_uri`

[`pnpr-oidc-github-actions-discovery.patch`](pnpr-oidc-github-actions-discovery.patch)
 applies to pnpm/pnpm `8f20a3f`.
A provider with no `login` block now fetches the discovery document,
 reads only `issuer` and `jwks_uri`,
 checks that `issuer` matches the configured one,
 and fetches the key set;
 providers with `login` keep full openidconnect discovery.
The metadata cache holds the key set plus,
 for login providers,
 the full metadata,
 so refresh and rate-limit behavior is unchanged.
It adds `workload_discovery_accepts_issuers_without_login_endpoints`,
 whose mock issuer serves GitHub's shape.

Tradeoffs:

- A workload-only discovery skips openidconnect's content-type check on the discovery response;
   the body must still parse as JSON with the two fields,
   and the key-set fetch keeps openidconnect's checks.
- A provider configured with both `login` and `workloads`
   still needs an issuer that publishes `authorization_endpoint`.
- It is not released:
   using it means building pnpr from source instead of installing `@pnpm/pnpr@next`.

Verification (bounded container,
 3 GiB memory,
 2 CPUs,
 one build job,
 pnpm-managed cargo sources removed from `.cargo/config.toml` so cargo fetches from crates.io):

```bash
# disposable pnpm/pnpm clone at 8f20a3f
podman run --rm --memory=3g --cpus=2 --volume "$PWD:/work:Z" --workdir /work --env CARGO_BUILD_JOBS=1 docker.io/library/rust:1.97-bookworm cargo test --package pnpr-auth
```

Unpatched source,
 new test only:

```text
thread 'oidc::tests::workload_discovery_accepts_issuers_without_login_endpoints' (6571) panicked at pnpr/crates/auth/src/oidc/tests.rs:260:14:
called `Result::unwrap()` on an `Err` value: Internal { reason: "OIDC provider unavailable" }
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 44 filtered out; finished in 0.04s
```

Patched source,
 whole `pnpr-auth` package:

```text
test oidc::tests::workload_discovery_accepts_issuers_without_login_endpoints ... ok
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.14s
```

Patched `pnpr` binary (`cargo build --package pnpr --bin pnpr`,
 same bounds),
 run with this repository's generated `package/config/pnpr/config.yaml`
 and probed with the registry harness against live GitHub:

```text
401 Authentication required for OIDC credentials
```

pnpr logged the request as `"status":401,"latency_ms":825`:
 it fetched GitHub's discovery document and key set,
 then rejected the forged signature,
 where the unpatched release answers the same token with 500.
A genuine Actions token was not exercised,
 because GitHub issues one only inside a workflow run.

## What does not work

- Pointing pnpr at a local discovery shim.
   `issuer` is both the expected token `iss` and the discovery base URL
   (`OidcProvider` in `pnpr/crates/config/src/oidc.rs` has no separate discovery or key-set URL),
   so the shim would have to answer for `token.actions.githubusercontent.com`.
   Redirecting that name to a sidecar fails too:
    pnpr's OIDC client resolves through `PublicResolver`,
    which rejects any private address
    (`pnpr/crates/auth/src/oidc/network.rs:12` to `22`),
    and the sidecar would need a certificate for GitHub's hostname.
- Allowing more egress in `package/config/tofu/hetzner.tf`:
   discovery reaches GitHub and fails while parsing the response.
- Changing claims,
   subject,
   or audience in `package/config/pnpr/config.yaml`:
   the failure comes before any of them are compared.

## Upstream filing decision

`.out-of-scope/` has no pnpm or pnpr entry (checked 2026-09-15).

Duplicate search on pnpm/pnpm,
 issues and pull requests,
 open and closed:
 `pnpr oidc`,
 `pnpr workload`,
 `OIDC provider unavailable`,
 `authorization_endpoint`,
 and `token.actions.githubusercontent.com`.
Only pnpm/pnpm#14666 (the feature pull request)
 and pnpm/pnpm#14802 (the release) matched,
 and neither reports this failure.

1.   **Is it really upstream's fault?**
      Yes.
      OpenID Connect Discovery 1.0 lists `authorization_endpoint` as required,
       so GitHub's document is incomplete by the letter of the spec,
       but pnpr documents GitHub Actions as its workload issuer,
       and workload verification never uses the authorization endpoint.
2.   **Can upstream fix it?**
      Yes;
       the prototype confines the change to `pnpr/crates/auth/src/oidc`.
3.   **Are they supporting this use case?**
      Yes.
      `pnpr/crates/pnpr/OIDC.md` configures
       `issuer: https://token.actions.githubusercontent.com` for keyless publishing,
       and `verifies_rs256_workload_tokens` names the same issuer.
4.   **Would the repo welcome our contribution?**
      Yes,
       with disclosure.
      `CONTRIBUTING.md` "AI-assisted contributions" welcomes agent-assisted work,
       asks contributors to understand and test it,
       and requires a footer naming the agent and model.
5.   **Will they likely fix it?**
      Likely.
      OIDC publishing landed in pnpm/pnpm#14666 and shipped in `0.1.0-alpha.11`;
       no maintainer statement declines GitHub Actions support.
6.   **Have we prototyped a minimal fix compatible with their architecture?**
      Yes.
      [`pnpr-oidc-github-actions-discovery.patch`](pnpr-oidc-github-actions-discovery.patch)
       keeps the existing cache,
       refresh interval,
       and error mapping,
       adds a regression test that fails on `8f20a3f` with the production error,
       and passes the whole `pnpr-auth` suite (45 tests).
      Formatting with pnpm's rustfmt fork,
       clippy,
       and dylint were not run.

Decision:
 fileable,
 but the owner answered "Not now" on 2026-09-15,
 so nothing is posted;
 agents never post upstream themselves.

Upstream state checked the same day:
 `main` was `8f20a3fd69748b1f6eb5c6a7c54d5917456a2013`,
 every commit touching `pnpr/crates/auth/src/oidc.rs`
 or `pnpr/crates/auth/src/oidc/` after pnpm/pnpm#14666
 is a style or size refactor
 (pnpm/pnpm#14730,
  #14760,
  #14792,
  #14804,
  #14872,
  #14875),
 no open pull request mentions OIDC discovery,
 and the regression test fails on that `main`.

~~~md
Title: pnpr: GitHub Actions workload publishes fail with 500 "OIDC provider unavailable"

Labels: pnpr, bug

`OIDC.md` configures `issuer: https://token.actions.githubusercontent.com` for keyless publishing,
but every workload publish with a GitHub Actions ID token gets HTTP 500:

    Internal error: OIDC provider unavailable

This includes tokens with a forged signature,
so the failure happens before signature or claim checks.

## Cause

`OidcState::metadata` (`pnpr/crates/auth/src/oidc.rs`) runs
`CoreProviderMetadata::discover_async` and maps any error to `unavailable()`.
openidconnect's `ProviderMetadata` requires `authorization_endpoint`
(openidconnect 4.0.1 `src/discovery/mod.rs:61`),
and GitHub Actions' discovery document has none:
https://token.actions.githubusercontent.com/.well-known/openid-configuration

Parsing that document with openidconnect 4.0.1 prints:

    missing field `authorization_endpoint` at line 55 column 1

The tests miss it because the `metadata` helper in `oidc/tests.rs` always adds `authorization_endpoint`.

## Reproduction

With `pnpr@0.1.0-alpha.11` and a provider configured as in `OIDC.md`,
send `PUT /~<registry>/<package>` with
`Authorization: Bearer pnpr_workload_<jwt>`,
where the JWT has `iss: https://token.actions.githubusercontent.com`,
current `iat`/`exp`,
and any signature.
Response: `500`, logged as `Internal error: OIDC provider unavailable`.

## Suggested fix

For providers without `login`, discovery needs only `issuer` and `jwks_uri`.
A prototype makes `metadata` return the key set plus optional full metadata,
fetches `issuer` and `jwks_uri` for workload-only providers,
checks the issuer,
and fetches the key set with `CoreJsonWebKeySet::fetch_async`.
It adds a test whose mock issuer serves GitHub's document shape;
that test fails on `main` with `Internal { reason: "OIDC provider unavailable" }` and passes with the change.
Patch: attach `doc/troubleshooting/pnpr-oidc-github-actions-discovery.patch` or open it as a pull request.

Written by an agent (Claude Code, claude-opus-5).
~~~

## Upstream status re-check 2026-10-21

pnpr has not fixed this.
 The npm `next` dist-tag is `@pnpm/pnpr@0.1.0-alpha.16`
 (published 2026-10-21T10:47Z from pnpm/pnpm `91774e72a04d52ce61db06339696a433d383c764`,
 release PR pnpm/pnpm#17187),
 and that release still carries the whole failure chain:

- `pnpr/crates/auth/src/oidc/provider_metadata.rs:42` maps every discovery error to `unavailable()`.
- `pnpr/crates/auth/src/oidc/provider_metadata.rs:43` requires `metadata.authorization_endpoint()`.
- `pnpr/crates/auth/src/oidc/workload_verification.rs:42` sends every workload token through that discovery.
- `pnpr/crates/auth/src/oidc/tests.rs:72` and `73` still synthesize `authorization_endpoint` in test metadata.
- Workspace `Cargo.toml:134` still pins openidconnect 4.0.1.

pnpm/pnpm#16970 (2026-10-16) split `pnpr/crates/auth/src/oidc.rs` into `pnpr/crates/auth/src/oidc/`,
 and pnpm/pnpm#16501 renamed the refresh flag to a `MetadataFreshness` enum;
 both are shape changes with the same discovery behavior.
GitHub's discovery document,
 fetched 2026-10-21,
 still omits `authorization_endpoint`:
 it serves `claims_supported`,
 `id_token_signing_alg_values_supported`,
 `issuer`,
 `jwks_uri`,
 `response_types_supported`,
 `scopes_supported`,
 and `subject_types_supported`.
Issue and pull request search in pnpm/pnpm for `authorization_endpoint` returns nothing (checked 2026-10-21),
 and the 2026-09-15 filing decision stands unchanged.

### Release probe

The registry harness against `@pnpm/pnpr@0.1.0-alpha.16` installed from npm
 (throwaway instance on 127.0.0.1:17678,
 this repository's generated `package/config/pnpr/config.yaml`
 with its storage paths moved to a scratch directory and its `web:` block removed):

```text
control GET / -> 404
500 Internal Server Error
```

pnpr logged the probe request as:

```json
{"level":"ERROR","fields":{"message":"request failed","err":"Internal error: OIDC provider unavailable","error_kind":"internal","status":"500 Internal Server Error"},"target":"pnpr_error::response"}
```

The control GET proves the instance was up and answering other routes,
 so the 500 comes from the workload path.
This matches `0.1.0-alpha.11` exactly:
 upstream releases through `0.1.0-alpha.16` still reject every GitHub Actions workload publish.

### Migration note for leaving the patched image

`0.1.0-alpha.16` also rejects the generated config's `web:` block
 (``unknown field `web`, expected one of ... ui ...``,
 from `pnpr config-check` against the release binary),
 so returning to an upstream build needs a config migration along with a fixed or patched binary.

### Next action

The patched image pinned in `package/config/pnpr/Containerfile` remains required.
Re-run the release probe when a newer `@pnpm/pnpr` lands on `next`,
 or before any patched-image refresh.

## Fork update 2026-10-21

The patched image now tracks current upstream.
 Branch `pnpr-oidc-workload-discovery` of `Aquaticat/pnpm` sits on pnpm/pnpm `cbe5accc7ace6812e1770f6301d2742c91381c85`
 (`main`,
fetched 2026-10-21) as one commit `01dc1384276b91f2b5a3664b6bf8532617e4e6f9`,
 `fix(pnpr): verify GitHub Actions workload tokens without login metadata`.
 The fix was ported through pnpm/pnpm#16970's `oidc.rs` split and pnpm/pnpm#16501's `MetadataFreshness` enum.

### Ported code map

- `pnpr/crates/auth/src/oidc/provider_metadata.rs:26` and `59-70` cache `Discovered { jwks, login }`;
  workload-only providers call `workload_keys` instead of `CoreProviderMetadata::discover_async`.
- `pnpr/crates/auth/src/oidc/workload_verification.rs:15` and `66-91` parse only `issuer` and `jwks_uri`,
  enforce issuer equality,
  and fetch the key set with `CoreJsonWebKeySet::fetch_async`.
- `pnpr/crates/auth/src/oidc/workload.rs:8` and `33` verify against `&CoreJsonWebKeySet`.
- `pnpr/crates/auth/src/oidc.rs:124` and `163` map absent login metadata to `rejected()`;
  `verified_expiration` takes the key set (`pnpr/crates/auth/src/oidc.rs:205`).

### Port proof

Both runs in `docker.io/library/rust:1.97.0-bookworm` with `--memory=3g --cpus=2` and `CARGO_BUILD_JOBS=1`:

- Unpatched `cbe5accc` plus only the new test fails,
  as required:
  `workload_discovery_accepts_issuers_without_login_endpoints` panics with
  `Internal { reason: "OIDC provider unavailable" }` (0 passed,
  1 failed,
  114 filtered out).
- With the port,
  `cargo test --package pnpr-auth` passes 115 tests with 0 failures.

### Config schema change

Current pnpr names the web UI block `ui` with `enabled`
 (`pub(super) ui: UiFile` at `pnpr/crates/config/src/config_file.rs:235`,
 fields `enabled` and `dir` at `pnpr/crates/config/src/config_file.rs:324`),
 and its root parser rejects unknown fields.
 Measured in both directions with the registry harness:

- The `0.1.0-alpha.16` release binary rejects the old block:
  ``Invalid config: unknown field `web`, expected one of ... ui ...`` (`pnpr config-check`).
- The `0.1.0-alpha.11` base pinned before this update starts normally on the new block,
  because that base drops unknown top-level fields by design
  (its `ConfigFile` at `8f20a3f` declares no `web` field and documents the drop).
  The `web: enable: false` block added with `e49cf3100a` was inert on that base.

`generatePnprConfig` in `file-enforcer.config.ts` now emits `ui: enabled: false`,
 and the generated `package/config/pnpr/config.yaml` carries it.

### Deployment record

`.github/workflows/pnpr-patched-image.yml` run `39646035147` built the rebased port as

```text
ghcr.io/aquaticat/pnpr:0.1.0-alpha.16-oidc-workload-discovery.01dc1384276b@sha256:09a9818eaebba777848f3d01382c97143e232d95085ea4950408ad436505109c
```

`package/config/pnpr/Containerfile` pinned that `tag@digest`,
 the generated config carried `ui: enabled: false`,
 and the deploy files landed in one lockstep commit `4adc1b864c`
 at 2026-10-21T18:01:46Z.
 Local runs of the new image before the pin answered `GET /-/ping` with `{}`,
 `GET /-/ready` with `{"status":"ok","storage":"ok","auth":"ok"}`,
 and `GET /-/metrics` with `pnpr_build_info{version="0.1.0-alpha.16"}`.

### Outage and root cause

The 404 surface became `502` from about 18:04Z and stayed down about 10 minutes,
 because the new base aborts startup on a store in the earlier layout
 and the deployed store holds 163 published packages there with no `layout.json`.
 The container log held `Storage is temporarily unavailable: the hosted store has no layout.json`,
 and the full diagnosis,
 reproduction,
 and remedy are in
[the base-move topic](pnpr-base-move.md).
 A second breaking change rode the same base move:
 npm URLs now answer only under `/npm`.

### Rollback and remedy

Rollback commit `66cff15dc2` restored the `0.1.0-alpha.11` pin,
 and production answered `GET /-/ping` with `{}`
 while a forged workload publish probe at
`https://pnpr.c.aquati.cat/~monochromatic-dev/@monochromatic-dev%2fprobe-nonexistent`
 returned `401 Authentication required for OIDC credentials`.
 The fix-forward is measured and ready but not deployed:
 stop the container,
 run `pnpr upgrade-storage` against the volume with `PNPR_SECRET` set
 (no pre-conversion snapshot,
 under the standing no-backup decision the owner re-confirmed on 2026-10-21),
 then repin the `0.1.0-alpha.16` digest and move every client URL under `/npm` in the same commit.
 The converted store cannot be served by the `0.1.0-alpha.11` reader,
 so a rollback after that step empties the converted store and republishes current versions.
 The image pin and the config change land in one commit,
 because Coolify bakes the config into the rebuilt image and the new base refuses the old block.
