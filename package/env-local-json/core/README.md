# `@monochromatic-dev/env-local-json-core`

Read requested keys from the repository's sops/age-encrypted `.env.local.json`
without loading the whole secret set and without mutating `process.env`.

The module never logs or throws a secret value.
It returns only the keys the
caller asked for,
treats the store as encrypted-only,
and resolves the age
identity the same way mise does.

## Install

The package is workspace-private for now.
Workspace consumers declare:

```json
{
  "dependencies": {
    "@monochromatic-dev/env-local-json-core": "workspace:*"
  }
}
```

`sops` must be on `PATH` at call time.
`age-keygen` is needed only by tests
and fixture tooling.

## Use

```ts
import { readEnvLocalJson, } from '@monochromatic-dev/env-local-json-core';

const { values, loaded, path, } = await readEnvLocalJson({
  keys: ['EXA_API_KEY', 'LINKUP_API_KEY'],
  env: process.env,
});

if (loaded)
  console.log(`read fill-ins from ${path}`);
```

Merge with the environment while preserving its precedence:

```ts
const resolvedEnvironment = { ...env, ...values, };
```

`values` contains only requested keys that were absent or blank in the passed
environment and present in the store.
Callers therefore never need to decide
which source wins.

## Semantics

- Only requested keys can appear in the result.
- `process.env` is never mutated.
  Pass `env` to read a different environment.
- When every requested key already has a non-blank environment value,
  the
  store is not discovered,
  read,
  or decrypted;
  the result is
  `{ values: {}, loaded: false }`.
- No store means the same absent result,
  with no error.
- A blank value in the environment counts as absent.
- `loaded: true` means the store was decrypted and validated;
  `path` is its
  canonical real path.
- The module caches nothing,
  hydrates nothing,
  and reloads nothing.

## Discovery

An explicit `path` option is authoritative,
and when it is absent a non-blank
`env.ENV_LOCAL_JSON_PATH` supplies the same authoritative path,
resolved
against `cwd`.
Neither falls through to discovery,
and a missing,
unreadable,
or non-file explicit path is a hard `EnvLocalJsonPathError`.

Without `path`,
discovery runs in this order:

1. The nearest repository root,
   either a Git repository or a mise monorepo,
   that contains `.env.local.json`.
   Outer repository roots are checked when
   the nearest one has no store.
2. A bounded upward walk from `realpath(cwd)` that stops at the nearest of:
   the repository root when `cwd` is inside one,
   `realpath(home)` when `cwd`
   is under home,
   or the filesystem root.

A store found by the bounded walk outside any repository root logs a warning
naming its path.
A store in a repository subdirectory is still accepted.

## Identity resolution

The decryption identity mirrors mise,
in this order:

1. `MISE_SOPS_AGE_KEY`,
   inline key material.
2. `MISE_SOPS_AGE_KEY_FILE`,
   when the named file exists.
3. `SOPS_AGE_KEY_FILE`,
   when the named file exists.
4. `SOPS_AGE_KEY`,
   inline key material.
5. `${MISE_CONFIG_DIR:-${XDG_CONFIG_HOME:-$HOME/.config}/mise}/age.txt`.

A missing identity raises `EnvLocalJsonMissingIdentityError`,
whose
`candidates` list names every option consulted and why it was skipped.
Key
material never appears in that message.

Only a curated environment reaches the sops child process:
`PATH`,
`HOME`,
`XDG_CONFIG_HOME`,
`TMPDIR`,
locale names,
the mise tool locations and
trusted-config path when set,
and the resolved identity variable.
Unrelated
credentials in the parent environment are not forwarded.

## Encryption and validation

The reader rejects a store before resolving an identity or running sops when
the file is not valid JSON,
has no top-level `sops` metadata object,
lacks a
version or MAC,
or has no supported key group.

After decryption it requires a JSON object whose every top-level value is:

- a string,
- non-blank after trimming,
- not still prefixed with `ENC[`.

Those failures are `EnvLocalJsonShapeError`.
Decrypted content and sops output
are never included in an error or log.

## CLI

The package ships a small editing entry point:

```bash
env-local-json edit .env.local.json
```

It resolves the identity with the same precedence as the reader,
then runs
`sops edit` with inherited stdio and the curated child environment,
keeping
the user's `EDITOR` and `VISUAL`.
The sops exit status is passed through.
Malformed usage exits with status 2.

This replaces hardcoding `SOPS_AGE_KEY_FILE` in a `secrets:edit` task.

## Internal capabilities

`src/identity.ts`,
`src/write.ts`,
and the CLI are internal modules.
They are
reachable through `@monochromatic-dev/env-local-json-core/testing` for
tests and migration tooling,
not from the public index:

- `resolveSopsIdentity({ env, home, })` returns the resolved identity.
- `buildSopsChildEnv({ env, home, identity, interactive, })` returns the
  curated child environment.
- `writeEnvLocalJsonValues({ path, values, env, home, })` requires an
  existing encrypted store,
  copies it to a same-directory temporary file at
  mode 0600,
  applies each value with `sops set --value-stdin`,
  and renames the
  temporary file over the store atomically.
  A failure removes the temporary
  file and leaves the store untouched.

## Errors

- `EnvLocalJsonPathError` - explicit path missing,
  unreadable,
  or not a file;
  also an unusable discovery directory.
- `EnvLocalJsonReadError` - a discovered store existed but could not be read.
- `EnvLocalJsonPlaintextError` - the store is not a sops envelope.
- `EnvLocalJsonMissingIdentityError` - no identity candidate is usable.
- `EnvLocalJsonSopsMissingError` - `sops` is not on the curated `PATH`.
- `EnvLocalJsonDecryptError` - sops failed,
  timed out,
  was aborted,
  or
  exceeded the output bound;
  `kind` and `exitCode` describe the failure
  without sops output.
- `EnvLocalJsonShapeError` - decrypted content failed validation.

## Testing

Unit tests generate real age identities and sops ciphertext in `mktemp`
directories and cannot pass without `sops` and `age-keygen`;
a missing tool
fails with the `mise install sops age` remediation.
The repository's own
store is never read or written.
A container matrix test exercises the built
artifact under Ubuntu as root and as a non-root user.

Run,
from the repository root:

```bash
mise run //package/env-local-json/core:build
mise run //package/env-local-json/core:lint:types
mise run //package/env-local-json/core:lint:oxlint
mise run //package/env-local-json/core:test:unit
mise run //package/env-local-json/core:test:container
```
