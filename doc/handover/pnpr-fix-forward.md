# Handover: pnpr fix-forward and the publish 401

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `@monochromatic-dev/module-or-throw` is now `@monochromatic-dev/or-throw-core`.

## Purpose

Record the held `0.1.0-alpha.16` lockstep repin for the pnpr registry,
the owner decisions behind it,
and the separate `pnpr-publish` 401 incident,
so the next turn can finish either without re-deriving
the measurements.

## Owner decisions (2026-10-21)

- The person runs the runbook host steps (stop,
  `pnpr upgrade-storage`);
  the agent pushes the held
  lockstep commit only after the person confirms the conversion.
  Status 2026-10-21T18:55Z:
  unconfirmed.
  The person asked what the conversion is and which runbook names it,
  so the host steps are not started;
  the agent re-explains
  `doc/runbook/deploy-pnpr-registry.md`
  `## Move to a newer pnpr base`
  and re-asks the landing order.
- No pre-conversion snapshot ("Skip it,
  republish on failure").
  Recovery republishes through
  `pnpr-publish`.
  This recovery path is currently impaired by the 401 incident recorded under
  `## Publish 401 incident`;
  see
  `doc/troubleshooting/pnpr-oidc-github-actions-discovery.md` for the outage record.
- Trust pinning (2026-10-21T18:55Z):
  pin the minted subject form only
  (`repo:Aquaticats@343218767/Monochromatic@710240121:ref:refs/heads/main`),
  not both subject forms.
  A later `use_immutable_subject` flip-off would break publishes again
  with a named cause and a one-line remedy.
- Probe step (2026-10-21T18:55Z):
  remove `[DEBUG-401]` when the incident closes,
  as planned.
  No permanent claims preflight.

## Held lockstep commit (uncommitted)

Commit after the person confirms the storage conversion.
Re-verify first with
`mise run //package/config/pnpr:lint:types` and `mise run //package/config/pnpr:test:unit`,
then one
commit with explicit pathspecs and the message before `--` (`fix(config-pnpr): ...`).
Coolify Watch
Paths auto-deploy on `package/config/pnpr/{config.yaml,Containerfile,compose.yaml}`,
so the
Containerfile pin and the `/npm` URL moves must land together.

Files and edits held in the worktree:

1. `package/config/pnpr/Containerfile`:
   comment names the alpha.16 base and the `pnpr upgrade-storage`
   gate;
   `FROM` pinned to
   `ghcr.io/aquaticat/pnpr:0.1.0-alpha.16-oidc-workload-discovery.01dc1384276b@sha256:09a9818eaebba777848f3d01382c97143e232d95085ea4950408ad436505109c`
   (image workflow run `39646035147`).
2. `package/config/pnpr/src/publish-steps.ts` (`registry` constant):
   publish target becomes
   `new URL(\`${origin}/npm/~${target.registryName}/\`)` because the alpha.16 base nests npm at `/npm`.
3. `package/config/pnpr/src/registry-read.ts`:
   the tarball URL template becomes
   `\`${origin}/npm/${name.replace('/',
   '%2f')}\``.
4. `package/config/pnpr/src/publish-plan.ts`:
   comment updated to the `/npm/~<name>/` form and the
   `0.1.0-alpha.11` root-form history.
5. `package/config/pnpr/src/publish-retry.unit.test.ts`:
   `FORBIDDEN_OUTPUT` keeps the verbatim
   root-form URL from run `34922532212` (historical quote) plus a moved-identifier note of the
   post-move URL (WR6).
6. `doc/decision/private-npm-registry.md`:
   moved-identifier note after the root-form `.npmrc` record.
7. `doc/planning/private-npm-registry.md`:
   moved-identifier note after the root-form `.npmrc` record.
8. `package/config/pnpr/README.md`:
   `.npmrc` block flipped to
   `@monochromatic-dev:registry=https://pnpr.c.aquati.cat/npm/~monochromatic-dev/` with a
   moved-identifier note (moved 2026-10-21 from `.../~monochromatic-dev/`,
   the `0.1.0-alpha.11` root
   surface).
9. Landed already,
   not held:
   the publish-401 trust fix as commit `fd2890ba0d`
   (`file-enforcer.config.ts` and the regenerated `package/config/pnpr/config.yaml`),
   verified green in run `39690652845`.

If the worktree loses these edits,
redo them from this list;
the URL shapes,
the pin digest,
and the
gate order are also recorded in `doc/troubleshooting/pnpr-base-move.md` and in the runbook section
`## Move to a newer pnpr base` of `doc/runbook/deploy-pnpr-registry.md`.
After the push,
run the
runbook `### What to check` list (it includes dispatching `pnpr-publish`).

## Publish 401 incident (separate incident, closed 2026-10-21T23:03Z)

Boundary (XIC):
separate from the 2026-10-21 deploy outage.
User-visible boundary is the
`pnpr-publish` workflow run failing `401 Unauthorized` on every `npm publish` PUT;
the outage was a
registry crash loop after the alpha.16 repin.

Closed by commit `fd2890ba0d` on the live `0.1.0-alpha.11` base;
run `39690652845` published 42 backlog versions.

### Evidence

- Last success:
  run `38774367340` at 2026-10-15T17:05:08Z.
  First failure:
  run `38838410358` at
  2026-10-16T02:03:09Z.
  Every run since fails the same way (`gh run list --workflow=pnpr-publish.yml`).
- Only auth-path change in that window:
  `39fc0da926` (generator source) and `15e8a365d1`
  (regenerated `package/config/pnpr/config.yaml`),
  both "trust the transferred repository owner":
  `repo:Aquaticat/Monochromatic` to `repo:Aquaticats/Monochromatic`,
  `repository_owner_id` `66041952`
  to `343218767`,
  `workflow_ref` owner segment likewise.
- Failure text (run `39654896517`):
  `npm error 401 Unauthorized - PUT https://pnpr.c.aquati.cat/~monochromatic-dev/@monochromatic-dev%2ftranslation-repair-markup`.
- Every server-side rejection collapses to one 401 (`rejected()` at
  `pnpr/crates/auth/src/oidc.rs:243` returns `Unauthenticated { resource: "OIDC credentials" }`;
  a failed workload lookup returns `resource: "OIDC workload credentials"` at
  `pnpr/crates/pnpr/src/server/authentication.rs:267`).
  npm prints no response body for these,
  so
  the run log alone cannot distinguish the branch.

### Exonerated (probes run 2026-10-21)

- npmrc auth key shape and token send:
  with npm `11.20.0` on node `v26.12.0` (the workflow's own
  toolchain) a local echo server saw `authorization: Bearer <token>` for the exact key
  `//<host>/~monochromatic-dev/:_authToken` the tooling writes;
  `${NODE_AUTH_TOKEN}` interpolation
  works.
  Probe artifacts:
  `${HOME}/temp/agent/npmrc-probe-2026-10-21/`.
- `publish-steps.ts` passes `NODE_AUTH_TOKEN` in the spawn env and prefixes the minted token with
  `pnpr_workload_`,
  which `authentication.rs:260` strips before verification.
- Trust values match the live repository:
  `gh api /repos/Aquaticats/Monochromatic` reports id
  `710240121`,
  owner id `343218767`,
  full name `Aquaticats/Monochromatic`.
- Production clock:
  response `Date` header matched local UTC within seconds.
  GitHub's discovery
  document still omits `authorization_endpoint` (the patched base tolerates that).

### Root cause (found 2026-10-21T19:00Z)

GitHub now mints `sub` as `repo:Aquaticats@343218767/Monochromatic@710240121:ref:refs/heads/main`
(the repository's OIDC customization reports `use_immutable_subject: true`,
`gh api /repos/Aquaticats/Monochromatic/actions/oidc/customization/sub`),
while the trust config
required `repo:Aquaticats/Monochromatic:ref:refs/heads/main`.
`binding_matches` at
`pnpr/crates/auth/src/oidc/workload.rs:131` compares `sub` by exact string equality,
so every token
fails the binding and `rejected()` at `pnpr/crates/auth/src/oidc.rs:243` returns 401.
`repository_id`,
`repository_owner_id`,
and `workflow_ref` are minted exactly as the trust lists them.
Both push and
`workflow_dispatch` events mint the same `sub` (runs `39657429832`,
`39659040999`).
Full record:
`doc/troubleshooting/pnpr-oidc-github-actions-subject-claim.md`.

The fix (held items 9 and 10 above) makes the generated subject identical to the minted claim,
measured with a node string comparison.
Ranking note:
hypotheses 2 (signature or metadata) and 3
(production serving another config) are refuted by the same claims probe and the rollback deploy
record.

### Probe

A temporary `[DEBUG-401]` step in `.github/workflows/pnpr-publish.yml` (path pinned by the trust
claim `workflow_ref`,
so the token is the real one) prints the decoded claims (never the token) and
sends publish-shaped `PUT` requests for `@monochromatic-dev/oidc-claims-probe`,
which is outside the
trust list:
`check_workload_request` at `pnpr/crates/pnpr/src/server/oidc.rs:304` refuses it with 403
when auth works,
and nothing can be published.

Red case,
run `39658559544` (trust still name-only):
the `pnpr_workload_` credential returns
`401 Authentication required for OIDC credentials`,
the publish failure body;
the raw JWT control
returns `401 Authentication required for package "@monochromatic-dev/oidc-claims-probe"`,
so the
probe distinguishes the two branches.
Green case,
run `39690652845` (trust pinning the minted subject):
the workload credential probe returns
`403 "pnpr-publish-ci" is not allowed to use workload credentials outside the configured npm publish targets`,
the raw JWT control returns the same 401 anonymous body,
and `Publish missing versions to pnpr` published 42 backlog versions.
The `[DEBUG-401]` step is removed when the incident closes,
per the owner decision recorded in `## Owner decisions (2026-10-21)`.

## Next actions (order)

Completed 2026-10-21:
the trust fix landed as `fd2890ba0d`,
run `39690652845` is green with 42 backlog versions published,
and the `[DEBUG-401]` probe was removed as `c0796cbe2e`,
naming the proven subject-claim hypothesis.
The host conversion ran 2026-10-21T23:23Z:
`Moved 182 documents`,
`Moved 444 other files`,
`Removed 183 listing-index and upload files`,
control re-run `The hosted store already has a current layout.json; nothing to do.`
The repin landed as `0f443e98d0`
after `mise run //package/config/pnpr:lint:types` and
`mise run //package/config/pnpr:test:unit` passed.
The runbook `### What to check` list is green:
`/-/ready` `{"status":"ok","storage":"ok","auth":"ok"}`,
`pnpr_build_info{version="0.1.0-alpha.16"}`,
the `/npm` packument serves 200,
the root npm surface is 404,
the anonymous PUT is
`Authentication required for package "@monochromatic-dev/module-or-throw" 401`,
and run `39693597489` reports
`0 of 163 package versions are missing from https://pnpr.c.aquati.cat`.

1. Consumer side (their `.npmrc` files):
   `@monochromatic-dev:registry=https://pnpr.c.aquati.cat/npm/~monochromatic-dev/`,
   and consumer lockfiles re-resolve their tarball URLs
   (documented in `package/config/pnpr/README.md`).
   Tracked as `Aquaticat/labwc-config#2`
   (opened 2026-10-21 at the person's request;
    they handle it in another session).
2. Final report to the person:
   the deploy outage and recovery,
   the `/npm` surface move,
   the publish-401 root cause
   and fix,
   the no-backup decision honored,
   recovery-by-republish unblocked,
   the policy variance
   (a teammate used `git switch --no-enforce-worktree-branch`),
   and container cleanup.

Unknowns (DCK):
the date `use_immutable_subject` turned on is not exposed by the API;
the first
failing run is 2026-10-16T02:03Z,
hours after the repository transfer.
Whether GitHub keeps the
ID-anchored form as the setting's stable output is assumed from the documented GUID customization and
from the two captured runs.
