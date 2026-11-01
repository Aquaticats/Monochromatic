# File-enforcer staleness recording is incomplete and quietly unsound

Two defects share one cause: the staleness manifest's dependency set comes from
a cooperative in-process read tracker rather than from the filesystem, so it is
simultaneously too small and unsound on the lazy path and aborted on the eager
path.

Defect 1, the manifest is never written, makes incremental skip do nothing.
Defect 2, a skipped builder can miss a real input, makes it wrong when it does
something. Defect 2 is the more serious: it silently produces output that a
full build would not, with exit code 0.

## Defect 2: a skip can be unsound

### Symptom

A lazy builder whose real input is read with plain `node:fs` is skipped after
that input changes. The destination keeps stale content and the process exits 0.

### Reproduction

In a throwaway worktree:

```ts
// Lazy builder: the only path that consults the manifest.
import { overwrite } from '.../src/index.ts';
import { readFile } from 'node:fs/promises';

await overwrite({
  dest: './demo/UNSOUND.out',
  content: async function derivedFromUntrackedRead(): Promise<string> {
    console.log('   >>> builder EXECUTED');
    return await readFile('./demo/REAL-DEP.txt', 'utf8');
  },
});
```

Run once with `REAL-DEP.txt` holding `ORIGINAL`. The builder executes and
`UNSOUND.out` holds `ORIGINAL`.

Change `REAL-DEP.txt` to `MUTATED` and run again. The builder does not execute,
and `UNSOUND.out` still holds `ORIGINAL` while its input holds `MUTATED`.

The positive control confirms this is a skip and not a no-op: deleting
`UNSOUND.out` and re-running executes the builder and writes `MUTATED`. A full
build and an incremental build therefore disagree, which is the definition of
an unsound incremental builder.

### Cause

`rememberFreshStalenessEntry` records `trackedReads`, which comes from the
global `_reads` set populated by `trackRead`
(`package/file-enforcer/core/src/tracker.ts`). Only `cat()`,
`globResults()`, and `addWatchedPaths()` call `trackRead`. A builder that reads
through `node:fs/promises` records nothing, so its entry lists only the config
file, that entry stays fresh, and the next run skips the builder and returns
early before its content callback ever runs.

This is not an edge case in this repo. The root config reads real inputs outside
the tracked API at six sites: `glob` at lines 489, 681, and 2067, `lstat` at
lines 581 and 738, and `readFile` at line 1065, plus subprocesses through
`nanoSpawn`. Any of those reads feeding a lazy builder would produce the skip
demonstrated above.

## Defect 1: the manifest is never recorded for the root config

### Symptom

The root config regenerates every destination on every run, and the persisted
manifest is never rewritten.

Measured on 2026-09-16:

- `/var/home/user/Monochromatic/node_modules/.cache/file-enforcer/staleness-manifest.json`
  had mtime `Jul 15 01:30`, two months stale against a config last changed `Sep 14`.
- `package/file-enforcer/core/node_modules/.cache/file-enforcer/staleness-manifest.json`
  had mtime `Sep 9 15:11`.
- All 117 entries in the root manifest point at `/tmp/file-enforcer-integ-*`,
  the temporary directories of the package's own container and integration
  tests. None corresponds to a production destination.

Three runs of the unmodified root config in a throwaway worktree took 0.929s,
0.924s, and 0.926s. A content edit to `.browserslistrc` produced the same wall
time (0.920s) and still wrote `.browserslistrc.resolved.local.json`.

### No lazy builder exists in the root config

The staleness gate runs only inside the lazy builders. `writeLazyIfChanged` and
`writeLazyEach` call `freshStalenessEntryExists` and return early when the entry
is fresh (`package/file-enforcer/core/src/io/write-lazy.ts`). Eager writes
go straight to `writeIfChanged` with no read of prior entries
(`package/file-enforcer/core/src/io/write.ts`).

All fifteen write call sites in the root config pass an already-computed
`content` value: four `await cat(...)` calls, the rest template literals, and
one `manifestText` variable. None passes a function or an `async` builder, and
the single `overwriteEach` passes a plain `skills` array rather than a lazy
`files` builder. So even a correctly populated manifest could not skip any
root-config work.

### An absent watched path aborts every recording

Recording stamps every path in the global read set. `readFileStamps` returns the
`ABSENT_FILE_STAMPS` sentinel if *any* path is absent
(`src/io/staleness-stamps.ts`), and `rememberFreshStalenessEntry` early-returns
on that sentinel (`src/io/staleness.ts`).

`assertForbiddenRootContextAbsent()` calls `addWatchedPaths(['./CONTEXT.md'])`
(`file-enforcer.config.ts` line 579), and `addWatchedPaths` delegates to
`trackRead` (`src/tracker.ts`), so `./CONTEXT.md` joins the set that
`rememberEagerWrite` copies into `trackedReads`
(`src/io/write-staleness.ts` line 64).

`CONTEXT.md` is absent by design, and `AGENTS.md` rule DPL forbids it, so every
recording hits the absent sentinel and returns without writing an entry.

### Reproduction

Absent watched path, then an eager overwrite, recorded no entries. The same
call without `addWatchedPaths` recorded one entry. A lazy builder recorded even
in isolation, confirming the gate itself works and that eager writes are the
un-gated path.

## Third symptom: the manifest is too large as well as too small

The recorded set is the union of every API read performed so far in the run, not
the transitive inputs of one destination. In a two-stage chain declaring
`b.mid` before `c.out`, the entry for `c.out` listed `a.src` as well as
`b.mid`, because `a.src` had been read earlier by an unrelated stage.

So the dependency set is over-approximate on the eager path and
under-approximate on the lazy path. Neither is the per-destination input set an
incremental builder needs.

## Fourth symptom: test pollution shares the cause

The 117 `/tmp/file-enforcer-integ-*` entries in the root manifest come from the
same two structural properties: a global mutable read and destination tracker
shared across configs, and a default manifest path resolved by walking up to
the nearest `node_modules`. Test runs that pass no `manifestPath` write into
whatever manifest their temporary directory resolves to, and a `node_modules`
symlink pointed at the live tree redirects them into the real manifest.

## Consequences

`package/file-enforcer/core/TODO.md` claims "the current in-memory cache
makes full re-runs fast (~2ms warm)". That figure is unreachable for this
config, which has no lazy builder to consult a manifest.

`doc/audit/file-enforcer.md` scores file-enforcer as passing axis A5 (persisted
manifest that skips unchanged work across separate runs). A5 is now scoped in
that document: it pays nothing for the root config, and on the lazy path where
it does fire it can skip unsoundly.

## Fix directions

Defect 2 is the load-bearing one, because a skip decision that can miss an input
is worse than no skip decision. The capture boundary has three known shapes, and
each costs something file-enforcer currently is:

- constrain generation to a checked read API, which is the descriptor pattern
  `AGENTS.md` rule AD3 and the package README both reject;
- trace reads at the operating-system level, as LaForge and Rattle do, which is
  a new platform layer and a research-solved but nontrivial problem (Rattle's
  soundness needed formal correction in the perfect-dependencies work);
- require declared input edges, which is the `make` and `mise` model.

Independent of that choice, the cheaper repairs:

- Distinguish "watched path may be absent" from "read path must be present" in
  the tracker, since `assertForbiddenRootContextAbsent` needs the former.
- Record per-destination inputs captured around the builder rather than the
  union of all reads so far in the run.
- Stamp source content, not only size and mtime, so a touch on an intermediate
  does not force a downstream rerun.
- Stop resolving the default manifest path by directory walk, and keep test
  fixtures on explicit `manifestPath` values so they cannot reach a real
  manifest through a symlink.
