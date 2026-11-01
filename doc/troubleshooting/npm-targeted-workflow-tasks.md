# Targeted npm workflow tasks disappear after configuration regeneration

## Symptom and separate publishing failure

The logger `0.4.2` version bump reaches `origin/main` in commit `3befa69c14`.
Automatic release run `39104844467` builds and packs it,
then pnpm emits `ERR_PNPM_FAILED_TO_PUBLISH` with HTTP 404.
That response does not establish the authentication cause.
The existing logger release record documents the same automatic-workflow response for `0.4.1`
and a successful targeted npm workflow using trusted publishing.

The targeted recovery run `39105453035` reaches its pack step,
where Mise emits `no task //:npm:module:pack found`.
That failure occurs before packing or an npm upload.
It is independent of the automatic workflow's HTTP 404.

## Owned configuration cause

Commit `61d38bd704` added `npm:module:pack` and `npm:module:publish` to `mise.toml`.
The authoritative `mise.no-env.toml` never received those definitions.
`file-enforcer.config.ts` owns `mise.toml` through `generateMiseToml`,
which reads the authoritative source and appends the generated environment section.
Its normal regeneration therefore removes tasks present only in the generated output.

The targeted workflow at `.github/workflows/npm-publish.yml` still calls those task names.
The missing definitions belong to the configuration source,
so changing npm authentication or retrying the unchanged workflow cannot repair this task-discovery failure.

## Correction and verification

The established pack and publish definitions are restored in `mise.no-env.toml`.
The original `generateMiseToml` owner then regenerates `mise.toml`.
The output adds only the task definitions;
the generated environment remains unchanged.

The corrective invocation scopes execution to that existing generator because another worker has concurrent
changes in `package/config/pnpr/config.yaml`.
A private import hook retains the original owner implementation and replaces only the root parallel dispatch
with `generateMiseToml()`.
The unrelated configuration owners do not run.
This changes neither the managed-output format nor the project generator's source.

A real `npm:module:pack` invocation succeeds with `NPM_MODULE=logger` and a private `NPM_PACK_DIR`.
It produces `monochromatic-dev-module-logger-0.4.2.tgz`.
The regenerated publishing tasks reach the remote in commit `d6d1d67b06`.
Targeted run `39106517976` uses that revision.
It succeeds, including the npm upload.
The registry serves `0.4.2` and identifies GitHub Actions as its trusted publisher.
The registry's SLSA statement names `.github/workflows/npm-publish.yml`,
source commit `d6d1d67b069de6dd6cff8dc7543f06bd6fd94caa`,
and that workflow invocation.

A fresh installation in a disposable consumer confirms version and repository metadata,
public root and Node exports,
and the absence of the workspace-only `/ts` export.
It writes five tagged, large, reentrant and independently owned-file records as complete JSONL.
The release record is [the logger publication plan](../planning/module-logger-release.md).

The source and generated configuration now agree,
so a subsequent normal regeneration retains the targeted release tasks.
The automatic workflow's HTTP 404 remains a distinct unresolved publishing path;
the targeted workflow is the recorded recovery path.
