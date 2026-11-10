# `mise run prepare` taking 50+ seconds on WSL because shell commands invoke `/mnt/c/` binaries and `pnpm exec` each

## Symptom

`mise run prepare` runs for ~50 seconds on WSL.
 Most of the
time is taken by external binary invocations and `pnpm exec`
calls;
 actual filesystem work (existence checks,
 config
validation) completes in milliseconds when isolated.

## Root cause

Three compounding factors:

1.  WSL filesystem overhead when executing binaries from
    `/mnt/c/`.
     Each invocation crosses the WSL-Windows interop
    layer,
     adding hundreds of milliseconds per call.
2.  `pnpm exec` commands take ~27 seconds each in WSL because
    pnpm scans the entire workspace to construct the runner
    environment before the underlying command starts.
3.  Unnecessary command executions when simple file checks
    would suffice (e.g. "is X installed?
    " runs the binary
    instead of checking for its presence).

In WSL environments,
 the cost of process creation plus
filesystem translation turns millisecond operations into
tens-of-seconds waits when repeated across a prepare script.

## Verification

Versions under test:

- WSL 2 on Windows 11
- pnpm 10.
  x
- Bun 1.3.
  x
- Workspace at HEAD as of 2025-06-16

Reproduce:
 run `time mise run prepare` on a cold WSL session.
Measure each subtask via `mise run prepare -v` to identify
which commands dominate;
 the slow ones invoke `pnpm exec` or
`/mnt/c/` binaries.

## Verified workaround

Replace shell commands with TypeScript scripts that use
filesystem checks where possible:

- File-system checks instead of running binary commands for
  "is X present?
  " or "is X up to date?
  ".
- Auto-decline pnpm reinstall prompt programmatically.
- Check whether packages exist before syncing.
- Cross-platform installation scripts in TS rather than per-OS
  shell scripts.
- Use native OS commands (`which` on Linux/macOS,
   `where.exe`
  on Windows) for existence checks rather than running the
  target binary just to read its version.
- Add PATH updates to `~/.profile` so snap binaries resolve in
  the prepare context without a full shell session.

Result (workspace prepare):

- Before:
   50+ seconds.
- After:
   1.54 seconds (~97% improvement).
- Each Bun TypeScript script consistently takes ~80-100 ms;
  the actual work (file checks) takes <10 ms.

Tradeoff:
 more scripts to maintain in TypeScript instead of
one-liner shell invocations.
 The TS files are short and live
in `package/module/es/src/`;
 the maintainability win comes
with the speed win because the scripts are typed.

## What does not work

- Setting `mise` to a faster runtime (Bun instead of Node):
  helps marginally;
   the dominant cost is the WSL boundary
  crossing,
   not the runtime startup.
- Caching `pnpm exec` results:
   pnpm does not expose a stable
  cache for the resolution step;
   the slowness is in the
  resolution itself.
- Running prepare from a Windows-side shell:
   bypasses the WSL
  filesystem overhead but moves the dev environment off
  Linux;
   not viable.

## Why we do not file this upstream

The slowness is a consequence of WSL's filesystem-interop
costs and pnpm's resolution model,
 both intentional.
 Walking
the 5 constraints:

1.  **Is it really upstream's fault?
    ** Not exactly.
     WSL
    crossings are inherent to the platform;
     pnpm's resolution
    is necessarily slow to be correct.
2.  **Can upstream fix it?
    ** WSL improvements have shipped
    over the years but the boundary cost will remain
    significant.
     pnpm could expose finer caching but has not.
3.  **Are they supporting this use case?
    ** Yes (WSL is
    supported;
     pnpm runs in WSL).
4.  **Will they likely fix it?
    ** No targeted fix expected.
5.  **Have we prototyped a minimal fix?
    ** N/A.

Decision:
 no upstream report.
 The TS-script refactor solves
the workspace-side problem entirely.

## Key takeaway

In WSL environments,
 avoid executing binaries when filesystem
checks suffice.
 The overhead of process creation plus
filesystem translation turns millisecond operations into
30-second waits.
