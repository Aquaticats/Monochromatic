import { mkdir, } from 'node:fs/promises';
import { join, } from 'node:path';

import { writeFileAtomic, } from '@monochromatic-dev/translation-repair-glossary/ts';
import type { RunnerClosure, } from '@monochromatic-dev/translation-repair-insertion/ts';

//region Probe store
// Keeps what a quota-spending probe measured, which used to survive only in
// whatever terminal ran it.
//
// WHY THIS EXISTS. `coverage-probe.ts` and `audit-sensitivity.ts` both printed
// their results to standard output and wrote nothing, on the reasoning that a
// caller would redirect them wherever the measurement was being kept. Nobody
// did. Both coverage scales were probed on 2026-08-16 and both audit arms ran
// three times on 2026-08-17, all of it at real quota cost, and the
// numbers survive only in session transcripts. They cannot be re-read, re-scored, or checked by anyone, and
// the block-scale null that question 28 leans on is among them.
//
// WHY A PER-RUN FILENAME rather than the one `rows.json` that `bench-report.ts`
// writes. That entry point is asked one question at a time; these two are run
// repeatedly against the same subject on purpose, to see whether a verdict is
// stable, and a fixed name would let each rerun destroy the run it was bought
// to be compared against. That is the same loss again by a slower route, so
// runs ACCUMULATE and a reader picks between them.
//
// WHY THE IDENTITY TRAVELS WITH THE ROWS. A verdict is worth keeping only if a
// later reader can say what produced it: what it was pointed at, which models,
// which build. The version 2 artifact learned this the expensive way and a
// probe result inherits it here, so a file found on disk months later answers
// for itself instead of needing the transcript that this module exists to stop
// depending on.
//
// WHY WHAT IT WAS POINTED AT LIVES IN `subject` rather than in a top-level
// corpus pin, which is where it started. `audit-sensitivity` reads NO corpus:
// its inputs are invented fixtures, and a required corpus field would have made
// it record a commit that had nothing to do with its verdicts. Recording a
// value that means "not applicable" as though it were a value is the exact
// defect class this whole generation exists to stop, so the field went where it
// is always true: every probe can say what it was pointed at, and only some of
// them are pointed at a corpus.
//
// WHAT THIS DELIBERATELY DOES NOT DO: interpret rows. Every probe owns its own
// row shape, they are all still prototypes, and a store that named their fields
// would turn each field change into a two-file change for no reader's benefit.

/**
 Identity and answers of one probe invocation.

 @example
 ```ts
 const run: ProbeRun = { startedAt, finishedAt, pipelineDigest, roster, subject, rows, };
 ```
 */
export type ProbeRun = {
  /**
   When this invocation began, ISO 8601.
   */
  readonly startedAt: string;

  /**
   When it finished, so a later reader can price a rerun without timing one.
   */
  readonly finishedAt: string;

  /**
   Digest over built output, which moves whenever anything that ran changed.

   Stronger than a commit: a worktree can be clean at a known commit and still
   build something else.
   */
  readonly pipelineDigest: string;

  /**
   Chunks the executing entry imports, which is the identity a COMPARISON
   needs and the tree digest cannot give.

   The digest moves whenever anything in the tree moves, including code this
   run never loaded, so two runs of byte-identical probe code carry different
   digests and nothing tells that apart from a real change. The closure moves
   only when this runner's own code does.
   */
  readonly runnerClosure: RunnerClosure;

  /**
   Models asked, in roster order, since a verdict over six voices is not
   comparable to one over three.
   */
  readonly roster: readonly string[];

  /**
   What this invocation was pointed at, in the probe's own terms: a corpus
   commit and entry ids, fixture names, a cap, whatever bounds what its rows
   can be read to say.

   OPEN RATHER THAN NAMED because probes are pointed at different kinds of
   thing. Forcing a corpus commit here would make a fixture-only probe record
   one it never read.
   */
  readonly subject: Readonly<Record<string, unknown>>;

  /**
   Answers, one per attempt, failures included.

   Failures belong here rather than being dropped: a probe whose roster fell
   over reads exactly like a quiet one once the failures are gone.
   */
  readonly rows: readonly unknown[];
};

/**
 Renders an instant into something safe and sortable as a filename.

 Colons are legal in a POSIX filename and awful to quote, copy and complete,
 so they become hyphens. Nothing else is dropped, which keeps the name
 lexically sortable and losslessly readable back to the instant.

 @param startedAt - ISO 8601 instant

 @returns Filename-safe rendering

 @example
 ```ts
 stampFor({ startedAt: '2026-08-17T12:00:00.000Z', },);
 ```

 Shared with the recall scorecard store, which stamps its files the same way.

 @internal
 */
export function stampFor({ startedAt, }: { readonly startedAt: string; },): string {
  return startedAt
    .split(':',)
    .join('-',);
}

/**
 How many trailing digest characters go in a filename.

 Enough to separate two builds of one afternoon by eye. The whole digest is
 inside the file, so this only has to disambiguate, never to identify.
 */
const DIGEST_IN_NAME = 8;

/**
 Writes one probe run beside the run directory's other artifacts.

 @param runsDir - resolved runs directory, owned by whoever resolved it

 @param probeName - names the subdirectory runs of one probe collect in, so
 two probes cannot interleave their files

 @param run - everything this invocation asked and heard

 @returns Path written, so a caller can say where the answers went instead of
 leaving a reader to search for them

 @example
 ```ts
 const at = await persistProbeRun({ runsDir, probeName: 'coverage-probe', run, },);
 ```
 */
export async function persistProbeRun(
  {
    runsDir,
    probeName,
    run,
  }: {
    readonly runsDir: string;
    readonly probeName: string;
    readonly run: ProbeRun;
  },
): Promise<string> {
  /**
   Where runs of this probe collect.
   */
  const probeDir = join(
    runsDir,
    probeName,
  );
  await mkdir(
    probeDir,
    { recursive: true, },
  );

  /**
   Name carrying both when it ran and which build ran it, so two runs of one
   day and two builds of one minute stay distinguishable on sight.
   */
  const path = join(
    probeDir,
    `${stampFor({ startedAt: run.startedAt, },)}-${
      run.pipelineDigest
        .slice(-DIGEST_IN_NAME,)
    }.json`,
  );
  await writeFileAtomic({
    path,
    text: JSON.stringify(
      run,
      undefined,
      2,
    ),
  },);
  return path;
}

//endregion Probe store
