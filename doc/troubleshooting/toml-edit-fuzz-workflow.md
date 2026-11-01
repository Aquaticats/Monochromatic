# TOML edit fuzz workflow bootstrap and coverage failures

## Scope

This document records the two failures exposed while restoring
`.github/workflows/toml-edit-fuzz.yml` on 2026-08-19.
The workflow had not reached its package checks in retained history because its tool bootstrap stopped first.
Correcting that bootstrap exposed a separate stale coverage baseline.

## Symptom: pnpm tries to install itself

GitHub Actions run
[32284667273](https://github.com/Aquaticat/Monochromatic/actions/runs/32284667273)
failed in `Install only the tools this workflow needs`.
The exact mise diagnostic was:

```text
mise WARN pnpm may be required but was not found.
...
mise ERROR Failed to install npm:pnpm@latest: failed to execute command:
pnpm add --global pnpm@11.22.0 ...: No such file or directory (os error 2)
```

The emitting tool was mise `2026.8.8 linux-x64`.
The workflow asked mise to install `npm:pnpm` on a fresh runner.
Repository configuration selected pnpm as the npm-backend package manager at `mise.toml:269`,
so mise selected the absent pnpm binary to install the pnpm package.

Pinned mise source for `v2026.8.8` confirms both parts:

- [`src/backend/npm.rs:898-913`](https://github.com/jdx/mise/blob/c454c6f4bb86db919300e3ea2d0dd084835a8967/src/backend/npm.rs#L898-L913)
  honors an explicit `npm.package_manager` value;
- [`src/backend/npm.rs:452-465`](https://github.com/jdx/mise/blob/c454c6f4bb86db919300e3ea2d0dd084835a8967/src/backend/npm.rs#L452-L465)
  implements that value by invoking `pnpm add --global`.

The repository already declares the core `pnpm = "latest"` tool at `mise.toml:36`.
The workflow was bypassing that supported declaration by naming the alternate npm backend directly.

## Verified bootstrap fix

Commit `d29a9dd263019e294d312e457b28c9598a2a5b3f` changed only the workflow's tool request:

```yaml
# .github/workflows/toml-edit-fuzz.yml
run: mise install pnpm node 'github:toml-lang/toml-test@latest'
```

This is not a package-manager substitution.
It asks mise for its logical pnpm tool through the registry's primary backend,
then the later dependency step continues to invoke the repository-selected pnpm version.

Mise maintainer guidance in
[discussion 7622](https://github.com/jdx/mise/discussions/7622)
recommends the short `pnpm` request,
which uses the registry's primary backend,
instead of explicitly requesting `npm:pnpm`.
The maintainer also explains that mise treats those backend forms as representations of one logical tool.

Run
[32285631499](https://github.com/Aquaticat/Monochromatic/actions/runs/32285631499)
proved the bootstrap repair at the GitHub boundary:
installation,
bundle build,
type checking,
unit tests,
and fuzz smoke all passed before the later coverage failure.

## Symptom: deterministic coverage regression

The same run then failed in `Coverage gate (no per-file src reachability regression)`:

```text
Error: Coverage regressed in 5 file(s):
  src/resolve-block.ts: 99 -> 75 covered lines
  src/toml-insert-comment-after.ts: 147 -> 110 covered lines
  src/toml-insert-comment-before.ts: 161 -> 143 covered lines
  src/toml-set.ts: 179 -> 174 covered lines
  src/types.ts: 160 -> 159 covered lines
```

Local `mise run //package/toml-edit/core.fuzz:fuzz:coverage` reproduced the same seed,
operation counts,
per-file counts,
and failure.
This ruled out GitHub runner variance.

## Coverage root cause

The baseline was last refrozen by commit `5a66b13c` before later source reductions:

- `332f003ad` moved the shared table-section scan from `resolve-block.ts` to `resolve-document.ts`;
- `b7899bed4` replaced duplicate segment predicates in both comment inserters with `path-prefix.ts` helpers;
- `43f419c5b` replaced `toml-set.ts`'s duplicate header comparison with `segmentsEqual`;
- `ece5b7553` reflowed `types.ts` documentation and removed one source line.

The decreases therefore matched lines removed from or moved out of each named file.
Increases in the destination and other files outweighed those reductions:
the baseline's total covered-line count rose from `6355` to `6381`.

The older baseline had been generated with Node `26.4.0`,
while the failed workflow used Node `26.7.0`.
Running the current source under both versions produced the same `6381/7392` result.
The runtime-version hypothesis was therefore rejected.

## Verified coverage fix

Commit `4e2e0e65e87949e46b7475a3802cde3cc3c2b1e2` refroze
`package/toml-edit/core/coverage-baseline.json` with:

```text
mise run //package/toml-edit/core.fuzz:fuzz:coverage --write
```

A subsequent check-mode invocation passed locally.
GitHub Actions run
[32286257673](https://github.com/Aquaticat/Monochromatic/actions/runs/32286257673)
then passed every workflow step:

- core pnpm tool installation and dependency installation;
- bundle build and type checking;
- unit and fuzz-smoke suites;
- TOML 1.0 and 1.1 conformance;
- deterministic per-file coverage at `6381/7392` lines across `45` files.

## Symptom: every valid and encoder case fails with a mise warning on stderr

Observed on 2026-09-06 (runs for `3d6c20c9f`,
 `6e3ffeb76`,
 `abe8b9434`,
 `36e48b299`):
the Conformance step reports `valid tests: 0 passed, 205 failed`
and `encoder tests: 0 passed, 205 failed` for both TOML versions,
while every invalid case passes,
and each failure's `output from parser-cmd (stderr)` is one line:
`mise WARN deprecated [config.experimental_monorepo_root]: experimental_monorepo_root is deprecated. Use monorepo_root instead. This will be removed in mise 2027.12.0.`

Cause:
toml-test runs the adapters as `node <adapter>`,
 and on the runner `node` is a mise shim;
mise-action installs the latest mise (2026.9.1 that day),
which deprecates the root config's `experimental_monorepo_root` key
and prints the warning to stderr on every shim invocation.
toml-test treats any stderr from a decoder or encoder as the case's result,
so every valid and encoder case fails while the invalid cases,
which expect a failure,
still pass.
The development machine's mise (2026.7.0) does not warn,
so the task passed locally.

## Verified conformance fix

Two changes,
each sufficient on its own for this warning,
both kept:

- `mise.no-env.toml` (the source of the generated root `mise.toml`) now sets `monorepo_root = true`,
  the key the current mise documents;
  mise 2026.7.0 treats it identically to the experimental key
  (probed on a throwaway monorepo:
   `mise tasks ls` lists the same `//`-prefixed tasks under either key).
- The conformance task sets `MISE_LOG_LEVEL=error` for the runner and the adapters,
  so no mise warning of any kind (a deprecated key,
  an available update) reaches the adapters' stderr.
  Verified with a positive control on 2026-09-06:
  `mise --version` prints the "mise version 2026.9.1 available" warning by default
  and nothing but the version under `MISE_LOG_LEVEL=error`.

Confirmed in CI on 2026-09-06:
run 34062395210 on `1c28c42a5` passed every step,
conformance and the coverage gate included,
the first green `toml-edit-fuzz` run since 2026-08-19.

## What does not work

### Reordering `npm:pnpm` and Node

The failed run installed Node before attempting `npm:pnpm`.
Node availability did not provide a pnpm binary,
and mise still selected pnpm because of `npm.package_manager = "pnpm"`.

### Switching repository npm installs away from pnpm

Changing the repository-wide package-manager setting would avoid this cycle,
but it would alter every mise npm-backend tool install to repair one incorrect workflow tool request.
The core pnpm request is narrower and follows upstream guidance.

### Treating intended source movement as lost reachability

The gate compares covered-line counts per file.
Moving covered logic to a shared module can lower source-file counts while preserving or increasing total reachability.
Refreeze only after matching each decrease to an intended source change and confirming deterministic current counts.

## Upstream filing decision

No new upstream report was filed.
Mise's documented maintainer recommendation already covers the bootstrap correction:
request the logical `pnpm` tool rather than forcing its alternate npm backend.
The repository workflow now follows that recommendation.
The stale coverage baseline was repository state,
not external-tool behavior.
