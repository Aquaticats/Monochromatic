# Encrypted env store loading without mise

## Status

Accepted 2026-10-18.
Owner decision recorded in
[`doc/planning/env-local-json-secrets-loading.md`](../planning/env-local-json-secrets-loading.md),
which keeps the decision ids and the interview question ids.
Implementation started 2026-10-18.

This single record consolidates the four areas the interview settled:
the `@monochromatic-dev/env-local-json-core` package,
the search-fetch integration,
the one-time credential migration and host wiring,
and the tests and CI that keep them honest.
The planning document is no longer a proposal under review.

## Context

Local developer secrets live in a gitignored,
sops/age-encrypted
`.env.local.json` at the repository root
(`doc/philosophy/secrets.md`).
mise loads it through its generated `[env]` `_.file` directive,
so every mise-run command sees the values in `process.env`.

Two harnesses launch search-fetch outside mise:
the Pi extension and the DSH MCP server.
The DSH subprocess scrub drops every environment name matching
`KEY|PASSWORD|SECRET|TOKEN` plus names beginning with `DSH_`,
so provider credentials cannot arrive as environment variables there.
Before this decision,
the shared search-fetch core read its provider keys from
plaintext `exaApiKey` and `linkupApiKey` fields in
`~/.pi/agent/extensions/pi-search-fetch.json`.
That file was a second secret home with a second lifecycle.

The goal is one credential home,
read with or without mise,
without putting a secret in an environment variable that a child process inherits.

## Decisions

### A generic store module owns reading and decryption (D1, D10)

The mechanism ships as `@monochromatic-dev/env-local-json-core`
under `package/env-local-json/core`.
search-fetch is its first consumer,
and the module stays generic:
it knows the store file name and the sops/age identity chain,
not search-fetch.
The file is `.env.local.json` and the explicit-path variable is
`ENV_LOCAL_JSON_PATH`.

### Environment wins, the store fills gaps (D2)

Precedence is `process.env` > `.env.local.json` > the old Pi credential fields.
The old Pi fields are no longer a runtime source (D15),
so the effective order
is the environment,
then the store.
A requested key that is already non-blank in the supplied environment is never
read from the store and never returned.
Callers merge with `{ ...env, ...values }`,
which preserves environment
precedence without mutating `process.env`.

### Discovery: explicit path, repository anchor, bounded walk (D3, D4)

An explicit path is authoritative.
A caller passes `path` directly,
and the host exports
`ENV_LOCAL_JSON_PATH` to the same location so the working directory does not
decide which store is read.
The `path` option wins when both are set.
A missing,
non-file,
or unreadable explicit path raises
`EnvLocalJsonPathError` and never falls through to discovery.

Without an explicit path,
discovery tries the nearest repository root first.
A repository root is a directory carrying a Git repository marker or a mise
`[monorepo]` marker.
The reader checks for `.env.local.json` at the nearest repository root,
then at
each outer repository root,
using the existing
`@monochromatic-dev/fs-path-core` root finder.
If no repository root holds a store,
the reader walks upward from
`realpath(cwd)` to the nearest of three bounds:
the home directory when `cwd` sits under home,
the containing repository root when one exists,
otherwise the filesystem root.
A store found outside a repository root logs a warning.
Finding no store is not an error;
the reader returns `{ values: {}, loaded: false }`.

Lazy strictness is deliberate (D4).
A missing store or a missing key means absent,
so keyless operations such as an
anonymous `web_fetch` keep working.
A store that exists but cannot be read,
validated,
decrypted,
or shaped raises a
named error.
search-fetch catches that expected failure class and degrades (D11).

### sops and identity (D5, D6)

Decryption shells out to the `sops` executable found on the curated `PATH`.
The reader never scans a toolchain's private install layout.
A missing executable raises `EnvLocalJsonSopsMissingError` with remediation.

Identity resolution mirrors mise,
and the `secrets:edit` CLI shares it:
`MISE_SOPS_AGE_KEY`,
then `MISE_SOPS_AGE_KEY_FILE` when that file exists,
then `SOPS_AGE_KEY_FILE` when that file exists,
then `SOPS_AGE_KEY`,
then `age.txt` in the mise configuration directory
(`${MISE_CONFIG_DIR:-${XDG_CONFIG_HOME:-$HOME/.config}/mise}/age.txt`).
A missing identity raises `EnvLocalJsonMissingIdentityError`,
which lists every
candidate and its state,
never key material.

Only a curated child environment reaches sops:
`PATH`,
`HOME`,
`XDG_CONFIG_HOME`,
`TMPDIR`,
the locale names,
and the resolved
identity as `SOPS_AGE_KEY` or `SOPS_AGE_KEY_FILE`.
The `edit` subcommand adds `EDITOR` and `VISUAL`.
One invocation carries a 15 second timeout and a 4 MiB captured output bound.

### The reader returns fill-ins, not a secret record (D7, D8)

The public call is
`readEnvLocalJson({ keys, path?, cwd?, env?, home?, signal? })`,
and it returns
`{ values, path?, loaded }`.
`values` holds only requested keys that the environment did not supply.
`path` is the canonical store path,
present exactly when `loaded` is `true`.
`loaded` is `true` only when a store was discovered and decrypted.
There is no cache,
no reload hook,
and no hydrate helper.

Decrypted top-level values are strict strings.
A non-string,
a blank-after-trim string,
or a value still starting with `ENC[`
raises `EnvLocalJsonShapeError`.
There is no scalar coercion and no flattening of nested objects or arrays.

### Encryption is checked at the ciphertext (D9, D14)

The reader parses ciphertext before any identity lookup or sops call and
requires a top-level `sops` metadata object with a version,
a MAC,
and at least
one supported key group (`age`,
`pgp`,
`kms`,
`gcp_kms`,
`azure_kv`,
or
`hc_vault`).
Plaintext JSON and malformed ciphertext both raise `EnvLocalJsonPlaintextError`.

Validation happens at the ciphertext,
not after decryption.
The decrypted document is checked as a map of strict strings,
so a metadata key
can never become a secret name.
sops stdout and stderr are never forwarded into a log,
an error,
or an error
cause.
Diagnostics name paths,
key names,
failure classes,
and numeric exit codes only
(D16).

### search-fetch reads the store inside loadLinkupConfig (D11, D15)

`loadLinkupConfig` requests exactly the provider keys that `process.env` did not
supply,
then resolves each key with environment precedence and a per-key origin
of `env` or `store`.
Source metadata records `storePath`,
`storeLoaded`,
and a value-free
`storeError`.

When the store is unusable,
startup degrades instead of failing.
The blocklist still loads,
GitHub-routed `web_fetch` still works through `gh`,
and calls that need a missing provider key fail with the existing message that
names the environment variable and the store,
plus the store diagnostic.

The Pi credential fields are removed from runtime support.
A config shape that still carries `exaApiKey` or `linkupApiKey`,
or a legacy
`pi-linkup.json` file,
raises `SearchFetchMigrationRequiredError` naming
`mise run secrets:migrate`.
The config schema accepts only `blocklist`.

The dsh bundle and the Claude Code registration stay unchanged unless the
verification in the migration fails (D19).
The dsh `cordis.patch.yml` comment is corrected to name the encrypted store as
the credential source,
to record that the scrub drops `SOPS_AGE_KEY_FILE` while
`ENV_LOCAL_JSON_PATH` survives,
and to keep any environment value out of the
bundle.

### One-time migration (D12)

`mise run secrets:migrate` runs the migration in
`package/search-fetch/core/src/migrate.ts`.
It reads the raw Pi config and legacy file,
writes both provider values into the
existing encrypted store through `sops set --value-stdin`,
and verifies them by
decrypting the store in memory and comparing values.
Only after verification does it delete the credential fields,
the legacy file,
and the shell exports.
It also ensures `.bashrc` exports `ENV_LOCAL_JSON_PATH` to the canonical store.

Backups of every existing source file are copied to
`~/temp/agent/env-local-json-migration-<timestamp>` with their original modes.
A failure restores them.
The backups are kept until the end-to-end verification passes and one more day,
then removed by the operator (D21).

### Host wiring keeps one home, and mise keeps its loader (D18, D20)

`.bashrc` exports
`ENV_LOCAL_JSON_PATH=/var/home/user/Monochromatic/.env.local.json`.
The `EXA_API_KEY` and `LINKUP_API_KEY` exports are removed,
so no shadowing copy
silently wins.
The age identity stays at `~/.config/mise/age.txt`.

mise keeps its `[env]` `_.file` loader for mise-run commands.
The module is the no-mise path,
not a replacement loader.
Both readers point at the same store,
and environment precedence means the two
cannot disagree about which value wins.
No reader mutates `process.env`.

### Package surfaces (D23)

The reader is exported from the package index.
An internal `./testing` subpath exports the identity resolver,
the atomic writer,
and the shape validators for built-artifact tests.
The package also ships the `env-local-json edit <file>` bin,
which runs
`sops edit` with the same identity resolver and preserves the user's editor
selection.

### Tests and CI (D13, D17, D22)

Tests create disposable fixtures under a temporary directory:
`age-keygen` writes an identity,
`sops --encrypt` writes a store,
and
`SOPS_AGE_KEY_FILE` points at the identity.
No test reads the real store or commits key material.
The plaintext rejection has a guard test that is shown to fail before it is
trusted.
The container matrix runs the reader in Ubuntu as root and as a non-root user
through `@monochromatic-dev/test-harness-core-matrix`,
which installs the repository's Node
version from the mounted config.

`.github/workflows/env-local-json.yml` is path-filtered to the affected
packages,
workflow,
and lockfiles.
The host job installs node,
pnpm,
sops,
and age,
then runs the matrix runner,
the reader build and unit tests,
and the search-fetch,
MCP,
and Pi verification
tasks.
The container job runs the reader container matrix through podman.

## Rejected alternatives

- Hydrating `process.env` inside search-fetch:
  spawned subprocesses would inherit the provider keys,
  against the dsh scrub
  intent.
- A whole-record reader plus a hydrate helper and an in-memory cache:
  it returns secrets the caller did not ask for and adds invalidation state.
  The keyed,
  fill-ins-only reader is smaller and least-privilege.
- Accepting plaintext JSON:
  contradicts "The loader is encrypted-only" in `doc/philosophy/secrets.md`.
- Pure-JS sops and age decryption:
  an unreviewed crypto surface to maintain for one file format.
- Scanning mise's install layout for the `sops` binary:
  hardcodes one toolchain's private paths into a generic module.
- A checked-in encrypted fixture with a throwaway identity:
  commits key-shaped material to git history.
- Keeping the Pi credential fields as a lowest-precedence fallback:
  leaves duplicate plaintext secrets in place and keeps two homes alive.
- Flattening or coercing nested JSON values:
  an invented convention with collisions and no round trip.
- A fixed path relative to the installed package:
  breaks the moment the package runs outside this monorepo.
- Throwing on any unreadable file:
  breaks keyless operations such as an anonymous `web_fetch`.
- Four separate decision records:
  the four areas share one precedence rule,
  one identity chain,
  and one store,
  so they are accepted together in this file.

## Risks

### The deletion gate is decrypt-only (D21)

The migration proves that the encrypted store decrypts to the migrated values.
It does not prove that every old plaintext copy is unrecoverable.
The backup directory holds the old Pi config,
the legacy file,
the store
ciphertext,
and `.bashrc` until the operator removes it.
Filesystem snapshots and the backup tool's own history may retain earlier
copies.
The migration also does not revoke provider keys at the provider.

The owner accepted this gate because the old copies are local,
mode-restricted,
and short-lived relative to the verification window.
The residual risk is that an old key value survives in a backup or snapshot
after the migration.
Rotate the provider keys if exposure is suspected,
and remove the migration backup directory only after verification plus one day.

### Degraded startup hides the store failure from models

A degraded server still accepts tool calls and fails only the calls that need a
missing key.
That is intentional,
because the blocklist and GitHub routing remain useful.
The store diagnostic travels in error text,
so the failure is visible in the
tool result rather than in a startup crash.

### The export is only as good as the launching environment

`ENV_LOCAL_JSON_PATH` reaches a child only when the harness process inherited
it.
A harness started outside a shell that sourced `.bashrc` falls back to
repository-root discovery,
which succeeds when its working directory is inside
the checkout.

## Consequences

The change surface,
exact file citations,
and verification plan live in
[`doc/planning/env-local-json-secrets-loading.md`](../planning/env-local-json-secrets-loading.md).
The policy in [`doc/philosophy/secrets.md`](../philosophy/secrets.md) gains the
no-mise reader and the two provider key names.
The coverage ledger in
[`doc/planning/mise-removal-coverage.md`](../planning/mise-removal-coverage.md)
records the module as the owner of store reading and decryption.
