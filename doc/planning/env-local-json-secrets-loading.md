# Plan: load search-fetch secrets from `.env.local.json` without mise

## Status

Accepted 2026-10-18.
The accepted decision record is
[`doc/decision/env-local-json-store-loading.md`](../decision/env-local-json-store-loading.md).
This document keeps the decision ids and the interview question ids,
records the evidence and change surface,
and tracks implementation.
Implementation started 2026-10-18.
It is no longer a proposal under review.

## Problem

Local secrets live in an age-encrypted `.env.local.json` at the repo root
(`doc/philosophy/secrets.md`,
section
"Approach:
mise-native encryption,
not a dedicated secrets tool").
mise loads it through the generated `[env]` `_.file` directive
(`file-enforcer.config.ts:652-668`).
When mise is not in the launch path,
nothing in that path reads the store.

search-fetch resolves its two provider keys,
`EXA_API_KEY` and `LINKUP_API_KEY`
(`package/search-fetch/core/src/config-constants.ts:12,17`),
from `process.env` and,
before this change,
from plaintext `exaApiKey`/`linkupApiKey` fields in
`~/.pi/agent/extensions/pi-search-fetch.json`
(filename constant `package/search-fetch/core/src/config-constants.ts:31`).
Those fields are now rejected (D15).

The dsh launch path shows why env injection cannot be the only channel:
the harness scrub drops env names matching `KEY|PASSWORD|SECRET|TOKEN` plus
`DSH_*` from the spawned MCP server
(`package/search-fetch/dsh-plugin/cordis.patch.yml:10-19`),
so provider keys cannot arrive as environment variables there.
The scrub keeps `HOME`,
`PATH`,
and `ENV_LOCAL_JSON_PATH`,
and drops `SOPS_AGE_KEY_FILE`,
which is why the store path is exported from the host and the identity stays at
the mise default (D18).

The win is one secrets home instead of a second plaintext key file,
plus loading that works with no mise in the loop.

## Requirements

- R1:
  search-fetch resolves both provider keys from the repo's `.env.local.json` with mise absent.
- R2:
  the loader is encrypted-only,
  matching `doc/philosophy/secrets.md` ("The loader is encrypted-only").
- R3:
  no hydrated key enters `process.env` inside search-fetch,
  so spawned subprocesses (`gh`) inherit nothing new.
- R4:
  the loading mechanism is a generic module other packages can adopt.
- R5:
  the two key values migrate out of `pi-search-fetch.json`;
  its key fields are rejected at runtime.
- R6:
  tests create disposable fixtures and commit no key material.

## Evidence

- The `_.file` loader:
  `file-enforcer.config.ts:652-668`,
  generated into `mise.toml`.
- The secrets tasks,
  in the source file that generates `mise.toml`:
  `mise.no-env.toml:1212-1233` (`secrets:edit` and `secrets:migrate`).
- sops and age are mise-provisioned tools:
  `mise.no-env.toml:79-80`.
- Env beats the store in key resolution:
  `package/search-fetch/core/src/config-schema.ts:131-166` (`resolveApiKey`).
- Key resolution call sites,
  per-key origin,
  and the store diagnostic:
  `package/search-fetch/core/src/config.ts:145-225`.
- The remaining plaintext config store and its paths:
  `package/search-fetch/core/src/config-file.ts` (`readOptionalConfigJson`),
  paths in `package/search-fetch/core/src/config-paths.ts`.
- The accepted config keys and the rejected credential fields:
  `package/search-fetch/core/src/config-constants.ts:51-61`,
  `package/search-fetch/core/src/config-schema.ts:251-272`.
- The migration and its backup directory:
  `package/search-fetch/core/src/migrate.ts`.
- The dsh env scrub and credential comment:
  `package/search-fetch/dsh-plugin/cordis.patch.yml:10-19`.
- The reader,
  discovery,
  identity chain,
  and write seam:
  `package/env-local-json/core/src/read-env-local-json.ts`,
  `src/discovery.ts`,
  `src/identity.ts`,
  `src/write.ts`,
  `src/store-shape.ts`.
- Tests:
  `package/env-local-json/core/src/env-local-json.unit.matrix.test.ts`,
  `package/search-fetch/core/src/config-store.unit.test.ts`,
  `src/migrate.unit.test.ts`.
- CI:
  `.github/workflows/env-local-json.yml`.
- Ciphertext hazard for naive readers:
  `doc/troubleshooting/pi-provider-credential-loading.md`.
- mise loader strictness and container exclusions:
  `doc/troubleshooting/mise-sops-container.md`.

## Settled decisions

- D1 (Q1):
  the mechanism ships as generic `package/env-local-json/core`,
  published as `@monochromatic-dev/env-local-json-core`;
  search-fetch is its first consumer.
- D2 (Q2 and Q10):
  precedence is `process.env` > `.env.local.json`;
  the store fills only absent-or-blank env values.
- D3 (Q3):
  discovery is an explicit path first,
  with the `path` option winning over `ENV_LOCAL_JSON_PATH` and no fallthrough
  when that path is
  missing or unreadable (hard `EnvLocalJsonPathError`);
  then the nearest repository root (`.git` or `[monorepo]` marker) holding
  `.env.local.json`,
  checked outward through outer repository roots;
  then a bounded walk from `realpath(cwd)` up to the nearest of the home directory
  (when under home),
  the repository root (when under one),
  or the filesystem root,
  warning when the store is found outside a repository root.
- D4 (Q4):
  lazy strictness.
  Missing file or key means absent;
  a present-but-unreadable or undecryptable store throws,
  and search-fetch catches that
  expected failure and degrades.
- D5 (Q5 and Q12):
  decryption shells out to `sops` found on `PATH` only;
  absence is a named error with remediation.
- D6 (Q11):
  identity chain mirrors mise and is shared with `secrets:edit`:
  `MISE_SOPS_AGE_KEY`,
  `MISE_SOPS_AGE_KEY_FILE` (existing file),
  `SOPS_AGE_KEY_FILE` (existing file),
  `SOPS_AGE_KEY`,
  then `age.txt` under `${MISE_CONFIG_DIR:-${XDG_CONFIG_HOME:-$HOME/.config}/mise}`.
  No key moves.
- D7 (Q6 and Q8):
  the reader is keyed and returns fill-ins only:
  `readEnvLocalJson({ keys, path?, cwd?, env?, home?, signal? })` returns
  `{ values, path?, loaded }`.
  No whole-secret-record API,
  no cache,
  no reload,
  no hydrate helper.
- D8 (Q9):
  decrypted values are strict strings;
  non-string,
  blank-after-trim,
  and `ENC[` values are rejected with a named error.
  No coercion and no flattening.
- D9 (Q13):
  encrypted-only.
  Ciphertext must carry a top-level `sops` object with a version,
  a MAC,
  and a
  supported key group,
  checked before any identity lookup or sops call;
  otherwise `EnvLocalJsonPlaintextError`.
- D10 (Q14):
  naming is `package/env-local-json/core`,
  with the explicit-path override variable `ENV_LOCAL_JSON_PATH`.
- D11 (Q15):
  search-fetch reads the store inside `loadLinkupConfig` and never mutates
  `process.env`.
  Each key records an origin of `env` or `store`;
  `source` carries `storePath`,
  `storeLoaded`,
  and a value-free `storeError`.
  Degraded startup keeps the blocklist and GitHub-routed `web_fetch`,
  and fails calls
  that need a missing key with the diagnostic.
- D12 (Q16):
  migration is automated with verify-then-delete:
  read raw old files,
  write both keys through `sops set --value-stdin` (values never on argv),
  decrypt and compare,
  then delete the Pi credential fields,
  the legacy file,
  and the shell exports.
  Backups stay under `~/temp/agent` until verification passes plus one day.
- D13 (Q17):
  tests generate fixtures at runtime in a temp dir
  (`age-keygen` identity,
  `sops --encrypt` store,
  `SOPS_AGE_KEY_FILE` pointed at
  the identity),
  and a container matrix runs them in Ubuntu as root and non-root.
- D14:
  the sops metadata check happens on ciphertext,
  before decryption;
  decrypted content is validated as strict strings,
  so no metadata key can become a
  secret name.
- D15 (revised from Q10):
  the Pi credential fields are removed from runtime support.
  A config still carrying them,
  or a legacy `pi-linkup.json`,
  raises
  `SearchFetchMigrationRequiredError` naming `mise run secrets:migrate`.
- D16:
  values never appear in logs or errors,
  only key names and paths;
  sops stdout and stderr are never forwarded or attached as an error cause.
- D17:
  the plaintext-rejection guard test is proven to fail before it is trusted
  (guard removed,
  rebuilt,
  run,
  restored),
  and the doc updates listed under
  "Change surface" ship with the code.
- D18:
  host wiring is an `export ENV_LOCAL_JSON_PATH=/var/home/user/Monochromatic/.env.local.json`
  line in `~/.bashrc`,
  with the `EXA_API_KEY` and `LINKUP_API_KEY` exports removed.
  The age identity stays at `~/.config/mise/age.txt`.
- D19:
  the dsh bundle and the Claude Code registration stay unchanged unless
  verification fails.
  The dsh `cordis.patch.yml` comment stops naming the Pi file as the credential
  source,
  records that the scrub drops `SOPS_AGE_KEY_FILE` while
  `ENV_LOCAL_JSON_PATH` survives,
  and pins no env value in the bundle.
- D20:
  mise keeps its `[env]` `_.file` loader.
  The module is the no-mise path,
  not a replacement loader;
  environment precedence means both readers agree on the effective value.
- D21:
  the deletion gate is decrypt-only by owner decision.
  The migration proves the store decrypts to the migrated values;
  it does not prove old plaintext copies are unrecoverable and it does not revoke
  provider keys at the provider.
  The residual risk is accepted and recorded in the decision record.
- D22:
  CI is a path-filtered workflow `.github/workflows/env-local-json.yml` that
  installs node,
  pnpm,
  sops,
  and age on the host,
  runs the reader and search-fetch
  tests,
  and runs the reader container matrix through podman.
- D23:
  the package exposes the reader from its index,
  the identity resolver,
  atomic
  writer,
  and shape validators through a `./testing` subpath for built-artifact
  tests,
  and the `env-local-json edit <file>` bin for interactive editing.
- D24:
  reading and writing are separate surfaces.
  The reader never writes;
  the atomic writer requires an existing sops store and is used by the migration and
  the testing seam,
  while interactive editing goes through `sops edit`.

## Rejected alternatives

- Hydrating `process.env` inside search-fetch (Q15):
  spawned subprocesses would inherit keys,
  against the dsh scrub intent.
- A whole-record reader with a hydrate helper and an in-memory cache (Q6):
  it returns secrets the caller did not request and adds invalidation state.
- Accepting plaintext JSON (Q7 and Q13):
  contradicts "The loader is encrypted-only" in `doc/philosophy/secrets.md`.
- Pure-JS sops and age decryption (Q5):
  an unreviewed crypto surface to maintain for one file format.
- Scanning mise's install layout for the `sops` binary (Q12):
  hardcodes one toolchain's private paths into a generic module.
- Checked-in encrypted fixture with a throwaway identity (Q17):
  commits key-shaped material to git history.
- Keeping the Pi credential fields as an additive fallback (Q10):
  leaves duplicate plaintext secrets in the pi config until an unscheduled manual step.
  The final design rejects the fields outright (D15).
- Flattening or coercing nested JSON values (Q9):
  an invented separator convention with collisions and no round trip.
- A path fixed relative to the installed package (Q3):
  breaks the moment the package runs outside this monorepo.
- Throwing on any unreadable file (Q4):
  breaks keyless operations such as plain `web_fetch` on a file nothing needed.
- Four separate decision records:
  the four areas share one precedence rule,
  one identity chain,
  and one store.

## Change surface

Created:

- `package/env-local-json/core/` with `src/`,
  `src/bin/env-local-json.ts`,
  unit and container tests,
  `README.md`,
  and `mise.toml` mirroring sibling modules.
- `package/search-fetch/core/src/migrate.ts`.
- `.github/workflows/env-local-json.yml`.
- `doc/decision/env-local-json-store-loading.md`.

Touched:

- `package/search-fetch/core/src/config.ts`,
  `config-types.ts`,
  `config-schema.ts`,
  `config-constants.ts`,
  `client-http.ts`,
  `exa-http.ts`,
  `search-fetch-client.ts`,
  the build config,
  and the unit tests.
- `package/search-fetch/mcp/src/index.ts` and
  `package/search-fetch/pi-plugin/src/index.ts` for the degraded-startup diagnostic,
  and `package/search-fetch/pi-plugin/src/mise.verify-extension.ts`.
- `package/test-harness/core-matrix/` for the `node` runtime that runs the container matrix.
- `mise.no-env.toml` for the `secrets:edit` and `secrets:migrate` tasks.
- User credential files outside the repo:
  `~/.pi/agent/extensions/pi-search-fetch.json`,
  the legacy `pi-linkup.json`,
  `~/.bashrc`,
  and `.env.local.json`,
  mutated only per D12.
- Generated files:
  `mise.toml` regenerated from `mise.no-env.toml`,
  `package/config/pnpr/config.yaml` for the new package,
  `pnpm-lock.yaml` for the new workspace importer,
  and the generated package license texts.
- Docs:
  `doc/decision/env-local-json-store-loading.md`,
  `doc/philosophy/secrets.md`,
  `doc/planning/mise-removal-coverage.md`,
  `doc/runbook/search-fetch-credential-single-home.local.md`,
  the READMEs of the pi,
  mcp,
  and dsh-plugin search-fetch packages,
  and the comment in `package/search-fetch/dsh-plugin/cordis.patch.yml`.

## Verification plan

- Task commands:
  `mise run //package/env-local-json/core:build`,
  `:lint:types`,
  `:lint:oxlint`,
  `:test:unit`,
  `:test:container`,
  and the same core checks for `package/search-fetch/core`,
  `package/search-fetch/mcp`,
  and `package/search-fetch/pi-plugin`,
  plus `//package/search-fetch/pi-plugin:verify:extension`
  and mcp's `buildAndTest`.
- User boundary:
  launch the built MCP server the way dsh does
  (secret-looking env names scrubbed,
  `ENV_LOCAL_JSON_PATH` kept,
  `HOME` kept),
  run a real `web_search`,
  and confirm it authenticates from `.env.local.json`.
  Run a GitHub `web_fetch` with no provider key configured and confirm the `gh`
  route still answers while a credential call fails with the store diagnostic.
- After migration:
  `pi-search-fetch.json` holds only `blocklist`,
  `.bashrc` carries the `ENV_LOCAL_JSON_PATH` export and no key exports,
  and the store decrypts to the two key names.
  Keep the backup directory until that verification passes plus one day,
  then remove it.

## Open questions

- Whether other packages adopt the module later (explicitly out of scope here).
- The exact removal date of the migration backup directory is an operator step,
  not an automated deletion (D21).

## Next action

Implementation completed 2026-10-18.
The module,
search-fetch integration,
matrix runtime,
workflow,
migration,
and documentation are in place.
Verification passed:
module build,
types,
oxlint,
unit tests,
and the Ubuntu root and non-root
container matrix;
search-fetch,
mcp,
and pi build and test suites;
the migration against a disposable fixture and against the live credential
files;
and a real Exa search through a scrubbed dsh-shaped environment that loaded
both keys from `.env.local.json` (`exa` provider,
store origins).
`mise.toml` was regenerated from `mise.no-env.toml`,
the pnpr config gained the package,
and the local runbook records the live migration and its backup directory.
The workflow itself has not run on GitHub yet.
