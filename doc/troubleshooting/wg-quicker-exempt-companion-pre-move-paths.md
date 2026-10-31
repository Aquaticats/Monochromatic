# `wg-quicker` from `113633b259` to `db02fa4c92`: `wg-quicker up` with `ExemptMark` throws `BypassRouteError: wg-quicker-exempt is unavailable` although the companion is built

## Symptom

After `mise run //package/wg/cli-quicker-exempt:build` succeeds,
`wg-quicker up` from the repository checkout dies before any network mutation:

```console
$ wg-quicker up fr-par-fr1
[warn] [2026-10-31T02:45:18.387Z] [networks] ASN AS4167 contributed no networks; skipping ASN
BypassRouteError: wg-quicker-exempt is unavailable. Build it with `mise run //package/wg/cli-quicker-exempt:build` or set WG_QUICKER_EXEMPT_COMMAND.
    at resolveApplicationExemptionCommand (file:///var/home/user/Monochromatic/package/wg/cli-quicker/dist/final/node/index.mjs:4:8623)
    at async main (file:///var/home/user/Monochromatic/package/wg/cli-quicker/dist/final/node/index.mjs:27:7643)
PrivilegeError: Privileged wg-quicker process failed with exit code 1.
```

Surface patterns that produce it:

- `wg-quicker up <interface>` when the parsed `[Interface]` section carries `ExemptMark` and neither
  `WG_QUICKER_EXEMPT_COMMAND` nor a searched `PATH` holds `wg-quicker-exempt`.
  A config without `ExemptMark` skips resolution entirely and only warns.
- `mise run //package/wg/cli-quicker:run -- up <interface>`,
  which exercises the source-module layout branch of the same probe chain.
- `node src/application-exemption-command.unit.test.ts`,
  red on `finds repository sibling release outside secure path` with the same error text.

The remediation text is misleading.
Building the companion never helps,
because the probe never looks where the build lands.

## Root cause

### Step 1: `up` resolves the companion before the first network mutation

`package/wg/cli-quicker/src/index.ts:126` calls the resolver as soon as the config carries `ExemptMark`:

```ts
// package/wg/cli-quicker/src/index.ts:120-126
    } else {
      /**
       Exact companion path resolved before first network mutation.
       */
      const command = await resolveApplicationExemptionCommand();
```

### Step 2: the resolver probes four sources in order

`package/wg/cli-quicker/src/application-exemption-command.ts:272` prefers the explicit
`WG_QUICKER_EXEMPT_COMMAND`,
then the paired repository build,
then a root-owned installation on the privileged `PATH`,
then a caller installation on the captured caller `PATH`.
The workspace probe is `findWorkspaceCompanion`
(`package/wg/cli-quicker/src/application-exemption-command.ts:227`).

### Step 3: the package-root check still demanded the pre-move directory names

`currentPackageRoot`
(`package/wg/cli-quicker/src/application-exemption-command.ts:192`)
walks up from the module directory and validates the landing with `packageRootCandidate`
(`package/wg/cli-quicker/src/application-exemption-command.ts:172`).
Before fix `34df734099` the two layout constants named the tree the moves replaced:

```ts
// package/wg/cli-quicker/src/application-exemption-command.ts:31,36 at 34df734099^
const WG_QUICKER_DIRECTORY = 'wg-quicker';
const CLI_DIRECTORY = 'cli';
```

```ts
// package/wg/cli-quicker/src/application-exemption-command.ts:170-173 at 34df734099^
  if (basename(candidate,) !== WG_QUICKER_DIRECTORY)
    return COMMAND_NOT_FOUND;
  if (basename(dirname(candidate,),) !== CLI_DIRECTORY)
    return COMMAND_NOT_FOUND;
```

Commit `113633b259` moved `cli/wg-quicker` to `wg/cli-quicker` and `cli/wg-quicker-exempt` to
`wg/cli-quicker-exempt` while keeping crate and binary names unchanged,
so the real package root is basename `cli-quicker` under parent `wg`.
Both walk-up candidates therefore returned the absence sentinel on every run.

### Step 4: the sibling probe named the pre-move companion directory

Even a matching package root would have missed the build,
because `findWorkspaceCompanion` joined `wg-quicker-exempt`:

```ts
// package/wg/cli-quicker/src/application-exemption-command.ts:234-239 at 34df734099^
    join(
      dirname(packageRoot,),
      'wg-quicker-exempt',
      'target',
      'release',
      DEFAULT_COMMAND,
    ),
```

The moved directory is `cli-quicker-exempt`,
and the binary its build produces is `target/release/wg-quicker-exempt`
(crate `wg-quicker-exempt`).

### Step 5: the fallbacks cannot see `target/release`

With every workspace probe missing,
the two `PATH` lookups find nothing because the companion only exists inside the repository
`target` trees,
and the final diagnostic throws
(`package/wg/cli-quicker/src/application-exemption-command.ts:336`)
before interface creation.

### Step 6: why it stayed broken

`113633b259` updated this file's TSDoc
("Directory expected to be `package/wg/cli-quicker`")
and the unit test expectation to the moved layout,
but not the constants or the sibling join.
The same commit's message records
"Known breakage at this commit:
... no lint,
type,
or test task has run.
A follow-up commit regenerates and verifies",
and that follow-up never ran `//package/wg/cli-quicker:buildAndTest`.
`finds repository sibling release outside secure path` stayed red from 2026-10-29 until `34df734099`.

## Verification

Version under test:
repository `main` at `db02fa4c92`
(resolver fix `34df734099`,
discovery tests `03ca61953c`),
companion `package/wg/cli-quicker-exempt/target/release/wg-quicker-exempt` from
`mise run //package/wg/cli-quicker-exempt:build`,
CLI bundle from `mise run //package/wg/cli-quicker:build`.

Runnable harness,
red before the fix and green after:

```console
$ cd package/wg/cli-quicker
$ node src/application-exemption-command.unit.test.ts
[info] [resolveApplicationExemptionCommand] [PASS] uses exact configured executable, rejects unavailable configured executable, resolves configured bare name through captured caller path, finds repository sibling release outside secure path, finds repository sibling release from source module layout, prefers repository sibling release build over debug build, uses repository sibling debug build when release build is absent, falls back to installed companion on privileged path, falls back to caller installation in captured caller path, reports build guidance when every companion candidate is missing
```

Guard-removal experiment proving these tests can fail:
reverting the three layout literals to their pre-move values,
rebuilding,
and re-running turns exactly the four workspace-discovery cases red with
`BypassRouteError: wg-quicker-exempt is unavailable...`,
while the six non-workspace cases stay green.
Restoring the literals returns all ten to `PASS`.

Real-CLI check on the rebuilt bundle:

```console
$ mise run //package/wg/cli-quicker:test:integration:bypass
wg-quicker bypass integration passed
```

That runs the built `up` and `down` lifecycle inside a disposable root network namespace
with the companion stubbed as `WG_QUICKER_EXEMPT_COMMAND=/usr/bin/true`.

### Patterns that fail

- `wg-quicker up <interface>` with `ExemptMark` and no override,
  from the repository checkout,
  at any commit in `113633b259` to `db02fa4c92`.
- The four workspace-discovery unit cases at those commits
  (bundle layout,
  source layout,
  release preferred over debug,
  debug-only workspace).

### Patterns that work cleanly

- Pre-fix and post-fix:
  `WG_QUICKER_EXEMPT_COMMAND=/absolute/path/to/wg-quicker-exempt`.
- Pre-fix and post-fix:
  a bare `WG_QUICKER_EXEMPT_COMMAND` name found through `WG_QUICKER_CALLER_PATH`.
- Pre-fix and post-fix:
  `wg-quicker-exempt` installed on the privileged or captured caller `PATH`.
- Post-fix only:
  repository sibling discovery,
  release preferred over debug,
  from both the bundle and the source module layouts.

## Verified workarounds

1.  Pin the built companion by absolute path:

    ```console
    $ WG_QUICKER_EXEMPT_COMMAND="$PWD/package/wg/cli-quicker-exempt/target/release/wg-quicker-exempt" wg-quicker up fr-par-fr1
    ```

    Tradeoff:
    the path is fixed to one build location,
    so a `target/debug` build or a checkout move leaves it stale,
    and the variable must be set for every invocation or exported.
2.  Put `package/wg/cli-quicker-exempt/target/release` on the caller `PATH`.
    Tradeoff:
    the directory pollutes `PATH` for every program the user runs,
    and resolution then depends on `PATH` order.
    The privileged `PATH` is irrelevant here because the caller `PATH` is captured before the sudo relaunch.
3.  Update past `34df734099` and rebuild the CLI with `mise run //package/wg/cli-quicker:build`.
    Tradeoff:
    `dist` is gitignored,
    so a checkout that skips the rebuild keeps shipping the stale literals.
    This is the fix,
    listed last only because the error message suggests none of it.

## What does not work

- `mise run //package/wg/cli-quicker-exempt:build` alone,
  the remediation the error names.
  The binary is present and executable;
  the probe never looks in `target/release`.
- Rebuilding `wg-cli-quicker` from pre-fix source.
  The stale literals compile into the fresh bundle and the same probe misses.
- `sudo wg-quicker ...` typed directly.
  Sudo's `secure_path` excludes the workspace `node_modules/.bin` directory;
  see `doc/troubleshooting/sudo-1-9-secure-path-workspace-cli.md`.
- Re-running `wg-quicker up` unchanged.
  The failing probe reads the filesystem layout only,
  so nothing about retrying moves the package directories.

## Upstream filing decision

Nothing to file.
The failing code is first-party
(`package/wg/cli-quicker`),
so there is no external tracker to file against.

Walking the six constraints anyway:

1.  **Is it really upstream's fault?**
    No;
    there is no upstream and the stale literals are ours,
    introduced by `113633b259`.
2.  **Can upstream fix it?**
    Not applicable;
    the fix landed here as `34df734099`.
3.  **Are they supporting this use case?**
    Not applicable.
4.  **Would the repo welcome our contribution?**
    Not applicable;
    the work is already committed in this repository.
5.  **Will they likely fix it?**
    Not applicable.
6.  **Have we prototyped a minimal fix compatible with their architecture?**
    `34df734099` corrects the three layout literals and
    `03ca61953c` locks every discovery branch,
    with the guard-removal experiment recorded in
    [Verification](#verification).
