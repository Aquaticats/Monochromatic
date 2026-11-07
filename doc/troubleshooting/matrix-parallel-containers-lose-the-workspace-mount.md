# test-harness-core-matrix: parallel container combinations lose `/workspace` on an SELinux host, but pass alone

## Symptom

On 2026-11-01,
`mise run //package/env-local-json/core:test:container` (podman 5.8.9,
Fedora host,
SELinux enforcing,
`getenforce` reports `Enforcing`) failed in both Ubuntu combinations,
in two runs out of two.
The container-side stderr named a different failure in each combination,
and the pair swapped between the runs:

```text
sh: 1: cd: can't cd to /workspace
```

```text
Error: Cannot find module '/workspace/package/env-local-json/core/src/env-local-json.unit.matrix.test.ts'
```

In both,
the earlier steps of the same script had worked
(`apt-get`,
the locked Node download,
which does `cd /workspace` and reads `toolchain.lock.json`),
so the mount existed and was readable at first.
The test file was present on the host.

Each combination passed when run alone,
in two runs out of two
(root:
36 s,
non-root:
54 s,
both `[PASS]` with the test's two cases).

## What was measured

The harness starts one `podman run --rm --memory=2g --cpus=2 -v <repository root>:/workspace:Z` per combination
and runs them concurrently (`concurrency limit: 4` in its log).
The mount is the whole worktree,
including its `node_modules`.

A probe with two containers sharing one two-file `:Z` mount,
the second started while the first slept,
did not reproduce the loss:
the first container still listed the directory afterwards.
So the mechanism is not shown.
The working hypothesis is that `:Z` relabels the shared tree at each container start,
that relabeling a tree this large takes long enough to overlap the other container's reads,
and that SELinux denies the first container the files in the meantime.
Nothing here measured a label change or an SELinux denial.

The CI log of `env-local-json` run 41272422291 shows the same two combinations
reaching the test body on the Ubuntu runner and failing there only on `mise`,
so this host-specific loss was not what failed CI.

## Verified workaround

Run one combination at a time through the harness API from a throwaway script outside the repository:

```ts
// ${HOME}/temp/agent/matrix-one.ts: node matrix-one.ts <repository root> <root|user>
import { join } from 'node:path';
const [root, user] = process.argv.slice(2);
const { matrix } = await import(join(root, 'package/test-harness/core-matrix/src/index.ts'));
await matrix({
  files: [join(root, 'package/env-local-json/core/src/env-local-json.unit.matrix.test.ts')],
  os: ['container:ubuntu'],
  user: [user],
  runtime: ['node'],
});
```

Both combinations passed this way on 2026-11-01.

## What does not work

- Re-running `test:container` unchanged.
   Two of two parallel runs failed.
- Reading the apt output.
   It ends in `debconf` frontend notices that are not errors;
   the failing command is in the `stderr` of the `SubprocessError`.

## Upstream filing decision

Nothing is filed.
The harness is repository-owned (`package/test-harness/core-matrix`),
and with the mechanism unproven there is no podman defect to report.
Proving or refuting the relabel hypothesis needs a denial record
(`ausearch -m avc` or the journal) taken during a parallel run.
