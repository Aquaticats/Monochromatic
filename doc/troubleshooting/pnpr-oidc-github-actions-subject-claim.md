# pnpr 0.1.0-alpha.11 workload publishes fail with HTTP 401 `Authentication required for OIDC credentials` after GitHub's `use_immutable_subject` OIDC customization rewrites the `sub` claim

## Symptom

Every `pnpr-publish` run fails each `npm publish` with:

```text
npm error 401 Unauthorized - PUT https://pnpr.c.aquati.cat/~monochromatic-dev/@monochromatic-dev%2f<name>
```

Last success:
run `38774367340` at 2026-10-15T17:05:08Z.
First failure:
run `38838410358` at
2026-10-16T02:03:09Z.
Every run after that fails the same way.
npm prints no response body for a 401,
so the run log names no cause.

Surface patterns and the error variant each produces,
measured on the same JWT:

- Any request with the `pnpr_workload_<jwt>` credential `package/config/pnpr/src/publish-steps.ts`
  sends,
  on any surface:
  `401 Authentication required for OIDC credentials`.
- The same JWT without the `pnpr_workload_` prefix:
  `401 Authentication required for package "<name>"`.
- The `/npm/...` path on the alpha.11 base,
  which routes npm at the root only:
  `405` with no body.

## Root cause

### GitHub mints an ID-anchored `sub` claim under `use_immutable_subject`

The repository's OIDC customization (measured 2026-10-21) returns:

```sh
# gh api /repos/Aquaticats/Monochromatic/actions/oidc/customization/sub
{"use_default":true,"use_immutable_subject":true,"sub_claim_prefix":"repo:Aquaticats@343218767/Monochromatic@710240121"}
```

With that setting on,
the issuer mints `sub` as `<sub_claim_prefix>:ref:<ref>`.
Captured from
workflows runs `39657429832` (push event) and `39659040999` (`workflow_dispatch` event),
both 2026-10-21:

```text
"sub": "repo:Aquaticats@343218767/Monochromatic@710240121:ref:refs/heads/main"
```

instead of the name-only form `repo:Aquaticats/Monochromatic:ref:refs/heads/main`.
Both event kinds
mint the same string,
so one trusted subject covers the push and dispatch triggers.

GitHub documents this as the "system-generated GUIDs" OIDC customization.
Other consumers hit the
same rewrite;
a related report [pins the behavior to repositories created after 2026-07-15][created],
and [an npm trusted-publishing report][npm-cli] and [a Terraform provider feature request][terraform]
track the same claim shape.

[created]: https://github.com/RDI-Foundation/agentbeats-leaderboard-template/issues/29
[npm-cli]: https://github.com/npm/cli/issues/9969
[terraform]: https://github.com/integrations/terraform-provider-github/issues/3548

The setting's activation date for this repository is not exposed by the API.
The first failing run is
2026-10-16T02:03Z,
hours after the repository transferred from `Aquaticat` to `Aquaticats`;
the trust
regen commit `15e8a365d1` updated the names and IDs but kept the name-only `sub` shape.

### The trust generator pinned the name-only form

`file-enforcer.config.ts` generated the trust `subject` as
`repo:${PNPR_GITHUB_IDENTITY.repository}:ref:${PNPR_GITHUB_IDENTITY.ref}`,
which wrote
`package/config/pnpr/config.yaml` as:

```yaml
            subject: 'repo:Aquaticats/Monochromatic:ref:refs/heads/main'
```

### pnpr compares `sub` by exact string equality

`pnpr/crates/auth/src/oidc/workload.rs:131`:

```rust
fn binding_matches(binding: &OidcWorkloadIdentity, payload: &Value) -> bool {
    if payload.get("sub").and_then(Value::as_str) != Some(binding.subject.as_str()) {
        return false;
    }
    ...
}
```

Walked from the request:

1. `pnpr/crates/pnpr/src/server/authentication.rs:260` strips the `pnpr_workload_` prefix and calls
   `OidcState::workload`.
2. `pnpr/crates/auth/src/oidc/workload_verification.rs:24` verifies the JWT signature,
   issuer,
   audience,
   and times;
   the captured claims satisfy all of them (`iss`,
   `aud`,
   `iat`,
   `exp`,
   `nbf`).
3. `match_workload_binding` compares `sub` to `identity.subject` by exact equality
   (`workload.rs:131`),
   the ID-anchored claim matches no binding,
   and `matched` stays `None`.
4. `pnpr/crates/auth/src/oidc.rs:243` turns that into one opaque rejection:

```rust
pub(super) fn rejected() -> RegistryError {
    RegistryError::Unauthenticated { resource: "OIDC credentials".to_string() }
}
```

### Earlier readings that were wrong

- An earlier probe sent the raw JWT without the `pnpr_workload_` prefix and read
  `401 Authentication required for package "@monochromatic-dev/oidc-claims-probe"` as the workload
  rejection.
  Wrong:
  that string is the anonymous path.
  The prefixed probe in run `39658559544`
  returns `401 Authentication required for OIDC credentials`,
  the `rejected()` body quoted in the
  call chain.
- Client-side suspects were probed and cleared before the claims were read:
  npm `11.20.0` on node
  `v26.12.0` (the workflow toolchain) sends `authorization: Bearer <token>` for the exact
  `//<host>/~monochromatic-dev/:_authToken` npmrc key the tooling writes,
  confirmed with an
  echo-server probe,
  and the repository and owner IDs in the config matched `gh api`.
- `repository_id` `710240121`,
  `repository_owner_id` `343218767`,
  and `workflow_ref`
  `Aquaticats/Monochromatic/.github/workflows/pnpr-publish.yml@refs/heads/main` are minted exactly as
  the trust config lists them,
  all as JSON strings,
  so the claims block was never the failing step.

## Verification

Versions under test:
production image
`ghcr.io/aquaticat/pnpr:0.1.0-alpha.11-oidc-workload-discovery.e75bec8f75dd@sha256:aea64907740a1301b0a39e3bdc6dd120f102e44668f65928b5654d19e68fc9bb`,
trust config `package/config/pnpr/config.yaml` generated at commit `f6d16035c1`,
GitHub's issuer
`https://token.actions.githubusercontent.com`.

Runnable harnesses:

- The customization read:
  `gh api /repos/Aquaticats/Monochromatic/actions/oidc/customization/sub`.
- The token and response harness:
  the `[DEBUG-401]` step of `.github/workflows/pnpr-publish.yml`,
  run by dispatching `gh workflow run pnpr-publish.yml`.
  It requests the workflow's ID token,
  prints
  only the decoded claims (never the token),
  and sends publish-shaped `PUT` requests for
  `@monochromatic-dev/oidc-claims-probe`,
  a package outside the trust list,
  so nothing can be published.

Patterns that fail:

```text
# run 39658559544, trust still name-only
probe /~monochromatic-dev/@monochromatic-dev%2foidc-claims-probe with workload credential -> 401 Authentication required for OIDC credentials
probe /npm/~monochromatic-dev/@monochromatic-dev%2foidc-claims-probe with workload credential -> 401 Authentication required for OIDC credentials
```

Patterns that work cleanly:

```text
# run 39658559544, same JWT, control
probe /~monochromatic-dev/@monochromatic-dev%2foidc-claims-probe with raw JWT (control) -> 401 Authentication required for package "@monochromatic-dev/oidc-claims-probe"
probe /npm/~monochromatic-dev/@monochromatic-dev%2foidc-claims-probe with raw JWT (control) -> 405
```

The raw JWT control shows the probe distinguishes the anonymous branch from the workload branch,
so
the workload-credential line is the reproduction of the publish failure body.

Fix measurement:
the regenerated `package/config/pnpr/config.yaml` subject was compared to the minted
claim with node;
the two strings are identical.

Fix measurement,
green:
run `39690652845` at 2026-10-21T23:02Z,
after the trust fix deployed as commit `fd2890ba0d`:

```text
# run 39690652845, trust pinning the minted subject
probe /~monochromatic-dev/@monochromatic-dev%2foidc-claims-probe with workload credential -> 403 "pnpr-publish-ci" is not allowed to use workload credentials outside the configured npm publish targets
probe /~monochromatic-dev/@monochromatic-dev%2foidc-claims-probe with raw JWT (control) -> 401 Authentication required for package "@monochromatic-dev/oidc-claims-probe"
```

The same run's `Publish missing versions to pnpr` step published 42 backlog versions,
so the incident is closed on the live `0.1.0-alpha.11` base.
An earlier prediction of the refusal wording as
`user "pnpr-publish-ci" cannot use workload credentials outside the configured npm publish targets`
was wrong;
the quoted line is the measured body.

## Verified workarounds

### Pin the minted `sub` form in the generated trust

`file-enforcer.config.ts` now composes `PNPR_GITHUB_WORKLOAD_SUBJECT` from the identity fields it
already pinned (`repositoryOwner`,
`repositoryOwnerId`,
`repositoryName`,
`repositoryId`,
`ref`):

```diff
# file-enforcer.config.ts
 const PNPR_GITHUB_IDENTITY = {
-  repository: 'Aquaticats/Monochromatic',
+  repositoryOwner: 'Aquaticats',
+  repositoryName: 'Monochromatic',
   repositoryId: '710240121',
   repositoryOwnerId: '343218767',
   workflowPath: '.github/workflows/pnpr-publish.yml',
   ref: 'refs/heads/main',
 } as const;
+
+const PNPR_GITHUB_WORKLOAD_SUBJECT = `repo:${PNPR_GITHUB_IDENTITY.repositoryOwner}@${PNPR_GITHUB_IDENTITY.repositoryOwnerId}`
+  + `/${PNPR_GITHUB_IDENTITY.repositoryName}@${PNPR_GITHUB_IDENTITY.repositoryId}:ref:${PNPR_GITHUB_IDENTITY.ref}`;
```

`package/config/pnpr/config.yaml` then carries:

```yaml
            subject: 'repo:Aquaticats@343218767/Monochromatic@710240121:ref:refs/heads/main'
```

Tradeoff:
the trust follows the minted form,
so turning `use_immutable_subject` off fails the
publishes the same way again.
Remedy then:
regenerate with the name-only form
`repo:Aquaticats/Monochromatic:ref:refs/heads/main`,
or emit both subjects as two workload entries.
The setting and its `sub_claim_prefix` are one `gh api` read away,
so the check is cheap.

### Rejected: turn `use_immutable_subject` off

A `PUT` to the same endpoint could restore the name-only `sub` and leave the old config working.
Rejected:
the ID-anchored claim is the shape `PNPR_GITHUB_IDENTITY` exists for ("IDs keep trust from
following a reused name"),
the plain form follows a name someone else could claim,
and the mutation
fights a documented platform default.

## What does not work

- Keeping the name-only subject while the setting is on:
  `binding_matches` has no wildcard and
  `match_workload_binding` rejects an ambiguous match.
- Reading the cause from the npm log:
  npm prints no response body for a 401.
- Inferring the claim shape from old docs or memory:
  the shape changed between 2026-10-15 and
  2026-10-16;
  the customization API and a captured claim are the authority.
- Trusting only `claims:` while dropping `subject`:
  the schema requires `identity.subject`,
  and the
  subject is what carries the `:ref:refs/heads/main` anchor.

## Upstream filing decision

Nothing to add,
and nothing is filed.
GitHub's issuer implements the documented
"use_immutable subject / system-generated GUIDs" customization;
the stale half was our generated
trust config.
The 6-constraint check,
walked explicitly:

1. **Is it really upstream's fault?**
   No. Behavior,
   not bug:
   the customization API and the minted
   claim agree with each other and with GitHub's documented GUID form.
   Our generator pinned the
   previous shape.
2. **Can upstream fix it?**
   Not applicable as a bug fix.
   GitHub could only change the documented
   feature.
3. **Are they supporting this use case?**
   Yes.
   The customization exists exactly so trust policies
   can pin rename-stable subjects;
   the failing half was our pin.
4. **Would the repo welcome our contribution?**
   Not applicable when there is nothing to fix.
   No
   filing is proposed,
   so no contribution policy is exercised.
5. **Will they likely fix it?**
   Not applicable.
   Related trackers
   (`npm/cli#9969`,
   `integrations/terraform-provider-github#3548`) are other consumers adapting to
   the same claim shape,
   not evidence of an upstream defect.
6. **Have we prototyped a minimal fix compatible with their architecture?**
   The minimal fix is on our
   side of the boundary and is recorded in "Verified workarounds":
   the generator change plus the
   regenerated config,
   measured against the minted claim.

Duplicate search:
`gh search issues 'use_immutable_subject'` found the three trackers linked in
"Root cause" and no issue matching our consumer-side pin problem;
there is no upstream thread to
comment on,
so the additive artifact is this "nothing to add" note.

`.out-of-scope/` holds `bun-install.md`,
`cargo-workspace.md`,
`claude-code-upstream-bugs.md`,
`codex-harness.md`,
`jsr.md`,
`lightningcss.md`,
`low-impact-typescript-formatting.md`,
`module-es-monolith.md`,
`pi-gpt55-long-context.md`,
`terminal-title-fork-parity-tests.md`,
and
`typescript-project-references.md`;
none covers GitHub OIDC subject claims,
so no exemption applies.

## Next action

The trust fix landed alone as commit `fd2890ba0d`,
and the green run `39690652845` closed this incident on the live `0.1.0-alpha.11` base,
per the landing decision recorded in `doc/handover/pnpr-fix-forward.md`,
`## Owner decisions (2026-10-21)`.

1. Remove the `[DEBUG-401]` step from `.github/workflows/pnpr-publish.yml`,
   naming this subject-claim hypothesis in the commit body.
2. The base repin to `0.1.0-alpha.16` and its `/npm` URL move stay held until the person runs the
   `pnpr upgrade-storage` conversion from `doc/runbook/deploy-pnpr-registry.md`,
   `## Move to a newer pnpr base`.
