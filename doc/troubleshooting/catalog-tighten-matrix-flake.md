# catalog-tighten matrix: a one-time, non-reproducing install-layer flake

One scenario in the `catalog-tighten.matrix` e2e suite failed once under
container concurrency,
 then did not recur across roughly 495 further scenario
runs.
 The tool's own resolution logic is deterministic given the filesystem,
 so
the transient lives in the install layer (`pnpm install` under corepack in a
podman container),
 not in `catalog-tighten`.
 This is a non-reproduction record,
not a diagnosis:
 it exists so a future session that catches the failing output
does not restart from zero.
 A separate,
 real defect surfaced during the
investigation (the fixture pinned an old pnpm) and was fixed;
 see the fixture
pnpm drift section.

## Symptom

Running the full matrix
(`mise run //package/dependency-catalog/cli-tighten.matrix:test:matrix`),
 which runs
the scenarios in containers at concurrency 2,
 the `stale orphan` scenario failed
once:

```txt
[catalog-tighten install-layout matrix] [stale orphan] FAIL
Error: [stale orphan] expected "picomatch: >=4.0.0 -> >=4.0.2", got:
```

The captured tool output after `got:` was truncated in the terminal and never
recorded,
 and the failure has not recurred,
 so the actual classification the
tool emitted that run is unknown.
 The scenario expects a `tighten` (`picomatch`
resolves to the installed `4.0.2` and the catalog floor is raised);
 the failure
means the output did not contain that tighten line.
 It was concurrent with a
second scenario failing for an unrelated,
 already fixed reason (the `store-only`
scenario premise,
 corrected in commit `77f8efe7a`).

Frequency:
 one occurrence.
 Re-running `stale orphan` in isolation passed
immediately,
 and the next full-matrix run was green.

## Status: not root-caused (non-reproducing)

The failing output was not captured and the failure did not reproduce,
 so there
is no source-level root cause here.
 What follows is what the investigation could
establish:
 the failure is not in `catalog-tighten`'s logic,
 plus the
reproduction attempts that came up empty.

## What was ruled out: the tool's own logic

For the matrix fixture,
 `catalog-tighten` is deterministic given the on-disk
layout,
 so a correct install can only yield `tighten`.
 The fixture's
`pnpm-workspace.yaml` declares a single catalog entry (`picomatch`),
 so there is
no cross-entry concurrency:
 `catalogEntries.map` in
`package/dependency-catalog/cli-tighten/src/index.ts:297` folds over exactly one
entry.

Resolution reads a fixed importer list in a fixed order and returns the first
hit,
 with no shared mutable state,
 in
`package/dependency-catalog/cli-tighten/src/version-resolve.ts:242`:

```ts
const candidateDirs = [
  monorepoRoot,
  ...workspaceRoots,
];
const versions = await Promise.all(candidateDirs.map(async function readCandidate(
  dir,
): Promise<string | typeof NO_MANIFEST_VERSION> {
  return await readVersionFromPackageJson(join(
    dir,
    modulesDir,
    npmName,
    'package.json',
  ),);
},),);
const found = versions.find(function hasVersion(
  version,
): version is string {
  return version !== NO_MANIFEST_VERSION;
},);
```

Given a complete install,
 one consumer's `node_modules/picomatch/package.json`
reads `4.0.2` and the tool tightens.
 The `stale-orphan` mutation seeds a higher
`picomatch@4.0.4` into `node_modules/.pnpm` with no symlink and runs after
install;
 it never touches the consumer symlinks,
 and the resolver above never
reads `.pnpm` on the tighten path.
 So the only way `stale orphan` reaches the
miss branch (`package/dependency-catalog/cli-tighten/src/index.ts:347`) is if a
consumer's `picomatch` manifest was unreadable at read time,
 which requires the
install layout to have been incomplete despite `pnpm install` exiting 0.

## Verification: the non-reproduction harness

Environment at the time of the failure:

- pnpm `11.9.0`,
   the value the fixture then hardcoded (see the drift section);
  provisioned by corepack.
- Base image `docker.io/library/node:24-slim`.
- podman `5.8.3`,
   containers capped at `--memory 2g --cpus 2`.
- Host:
   16 cores,
   tens of GiB free,
   so not resource-constrained.

Two bounded reproduction harnesses were run (both under `scratchpad/`,
disposable):

- Six install-heavy scenarios concurrently per iteration,
   30 iterations:
   180
  scenario runs,
   0 failures.
- The full matrix via the real harness,
   15 runs:
   roughly 315 more scenario runs,
  0 failures.

Roughly 495 scenario runs,
 zero reproductions,
 against an original base rate of
one occurrence.
 Concurrency pressure heavier than the original 2-way run
(six-way) did not raise the rate,
 which argues against registry contention
scaling the flake and for a rare,
 externally-timed transient.

Reproduction command (the exact original condition):

```sh
# /var/home/user/Monochromatic
mise run //package/dependency-catalog/cli-tighten.matrix:test:matrix
```

The assertion already embeds the full tool output in its error
(`package/dependency-catalog/cli-tighten.matrix/src/in-container.ts`,
 the
`expected ... got:\n${output}` throws),
 so a recurrence in CI or a captured run
is diagnosable from the logs without extra instrumentation;
 the original loss
was terminal truncation,
 not missing data.

## Fixture pnpm drift (found and fixed)

The fixture hardcoded `PINNED_PNPM = 'pnpm@11.9.0'` with a comment claiming it
matched the monorepo's pnpm.
 The monorepo pins `pnpm = "latest"` via mise
(`mise.toml:36`),
 which had since advanced to `11.11.0` (`pnpm --version`),
 so
the fixture had drifted:
 the e2e suite installed with an older pnpm than the repo
runs.
 Note this is not a claim that `11.9.0` is a buggy release;
`doc/troubleshooting/pnpm-modules-cache.md` names `11.9.0` only because it was
current when that doc was written,
 and its behavior is not version-specific.

Fixed by resolving the monorepo pnpm at run time and passing it into every
container:

- `combos.ts` replaced the hardcoded constant with `FIXTURE_PNPM_ENV` and a
  `buildRootPackageJson(pnpmSpec)` builder.
- `matrix.unit.matrix.test.ts` resolves `pnpm@<version>` via `pnpm --version` and
  passes it through `--env FIXTURE_PNPM=...`.
- `in-container.ts` reads that env var and installs with it.

After the fix the full matrix is green installing with `11.11.0`.
 This removes a
version-mismatch variable from the environment,
 but it is not claimed to fix the
flake:
 the flake did not reproduce on `11.9.0` either,
 so there is nothing to
verify a fix against.

## Mitigation (proposed, not applied)

Add `retries: 2` to the matrix `it` in `matrix.unit.matrix.test.ts`.
 The harness
supports it (`@monochromatic-dev/test-harness-core`,
 `it({ retries })`).
 A retry
re-runs the whole scenario,
 including a fresh install,
 which is exactly the step
the transient lives in;
 a genuine regression still fails all three attempts
deterministically.
 Tradeoff:
 a true failure takes about three times as long to
surface,
 and retries paper over an install-layer bug rather than fixing one if a
real defect exists.
 This mitigation is reasoned,
 not verified:
 a fix for a
non-reproducing flake cannot be verified against the flake.
 It is left unapplied
pending a decision.

## What does not work

- Single-scenario isolated re-run:
   passes every time,
   so it does not surface the
  flake.
- Six-way concurrency stress (180 runs) and full-matrix looping (15 runs):
   did
  not reproduce,
   so brute-force reproduction did not yield the failing output.
- Static reasoning about the tool:
   rules the tool's logic out but cannot recover
  the environmental transient,
   which is not in the tool's code.

## Upstream filing decision

Decision:
 do not file.
 The audit trail for the six-constraint check:

1. Is it really upstream's fault?
    Cannot confirm.
    The failure is localized to the
   install layer,
    but not attributable to pnpm over corepack,
    the registry,
    or
   podman tmpfs timing.
    Fails this constraint.
2. Can upstream fix it?
    Unknown;
    no identified defect to fix.
3. Are they supporting this use case?
    Not applicable;
    no confirmed pnpm behavior.
4. Would the repo welcome our contribution?
    Not evaluated;
    there is nothing to
   contribute without a reproduction.
5. Will they likely fix it?
    Not applicable;
    no diagnosed issue.
6. Have we prototyped a minimal fix?
    No;
    there is no identified cause to fix.

`.out-of-scope/` was checked:
 no pnpm,
 install,
 corepack,
 container,
 or e2e-flake
exemption (`.out-of-scope/bun-install.md` is a different tool).
 The pnpm tracker
(`pnpm/pnpm`) was searched across open and closed state for `install
nondeterministic node_modules`,
 `corepack partial install`,
 and `install exit 0
incomplete`;
 no matching issue was found.

No issue is drafted.
 A non-reproducing "seen once,
 cannot reproduce,
 cause
unconfirmed" report is not actionable for maintainers and would be a publicity
incident under the default do-not-file policy.
 If a future run captures the
failing tool output,
 reopen this:
 the captured classification (a `MISS`/`UNDCL`
line versus a wrong tighten version) points directly at whether a symlink was
missing or a wrong version resolved,
 which is the missing evidence to file
against pnpm.
