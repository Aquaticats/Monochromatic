import {
  listCorpusPeople,
  readCorpusFile,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  type CommandLineOf,
  createProbeRunClient,
  readRunCorpusPinSetting,
  reportingRefusals,
  resolveRunsDir,
  RUN_MODELS,
} from '@monochromatic-dev/translation-repair-run/ts';
import { digestPipeline, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { outsideReadsFrom, } from '@monochromatic-dev/translation-repair-corpus-pass/ts';
import { nowAsIso, } from './probe-run-clock.ts';
import { runSettledAudit, } from './rendering-audit-settled-drive.ts';
import { readRunnerClosure, } from '@monochromatic-dev/translation-repair-insertion/ts';
import { readStageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Settled rendering audit
// Runs the rendering audit over every decided slice of every settled version 2
// artifact in an archive, and keeps what it heard.
//
// THIS IS TELEMETRY AND NOTHING ELSE. The producing path does not change, and
// nothing this reports may gate what ships. The instrument's own production
// error rate is unmeasured: its false-negative half is open, and a measurement
// found one of the three checkers raises claims at a tenth the rate of
// the others. An instrument in that state can be believed about itself and not
// about the corpus.
//
// WHAT IT IS POINTED AT, measured rather than estimated and written up in
// `doc/audit/rendering-audit-settled-population.md`: 40 subjects across four
// artifacts, two entries settled twice. EVERY delivery row is `decided`.
//
// SIXTEEN OF THE FORTY AUDIT THE ARCHIVE'S OWN ENGLISH, because the judges
// preferred the incumbent there. The instrument was built for output with no
// BEFORE text, which is the other twenty-four. Both are worth auditing, since
// both are what the document carries, but reading them in ONE denominator would
// blur the first real measurement this produces. So every row says which it was
// and the two are reported apart.
//
// NOT A CENSUS, whatever "every artifact" suggests. Two entries cannot settle
// anything about a particular entry. The name of this probe says `settled`
// rather than `census` on purpose: a probe directory outlives the paragraph
// that qualifies it.
//
// READ `corroborated` AND `agreed` APART. The strict tier asks whether voices
// picked the same characters and the loose one asks whether they were talking
// about the same thing; four runs of `audit-sensitivity` now show the strict
// tier reporting a unanimous defect as nothing.
//
// THE RELOCATION RULE WAS FIXED BEFORE THE RUN. Per-slice judging
// cannot tell a relocation from a fabrication, so a passage the archive carried
// across a slice boundary reads as an omission on one slice and an unsupported
// addition on its neighbour. Paired omission and addition findings on ADJACENT
// slices of one entry count as ONE relocation. Written down here because
// deciding it after seeing the tally is a goalpost move.

/**
 Hands the audit what only a real process has.

 @param line - the audit's command line, read whole by `reportingRefusals`

 @example
 ```ts
 await main({ line, },);
 ```
 */
async function main({ line, }: { readonly line: CommandLineOf<'rendering-audit-settled'>; },): Promise<void> {
  /**
   Corpus clone and commit of the run, after any environment override, read
   inside the refusal boundary (ledger B302): its clone is the one read when
   `--clone` names none, and the commit each artifact carries is its own.
   */
  const { pin, } = readRunCorpusPinSetting({ env: process.env, },);

  /**
   What the run reads from outside the pipeline: the cited pages through the
   environment's key and caches, which the audit asks of each original.
   */
  const outsideReads = outsideReadsFrom({
    env: process.env,
    fetchFn: fetch,
    pin,
    listPeople: listCorpusPeople,
    readFile: readCorpusFile,
  },);

  /**
   Digest over built output, which is the only identity that moves when the
   code moves but the commit does not.

   READ AT THE START, not at the end, and that ordering is the whole point.
   A long run gives a developer plenty of time to rebuild, and this probe was
   caught doing exactly that: `dist` was rebuilt while a 40-subject run was in
   flight, so the digest the run was about to stamp described a build that had
   never audited anything. Node loads the code once, at startup; the identity
   that answers for a run is the one present THEN.
   */
  const { digest: pipelineDigest, } = await digestPipeline({ dir: import.meta.dirname, },);

  return runSettledAudit({
    line,
    newClient: createProbeRunClient,
    reader: outsideReads.references,
    runCloneDir: pin.cloneDir,
    roster: RUN_MODELS.checkerModelIds,
    resolveRuns: resolveRunsDir,
    now: nowAsIso,
    pipelineDigest,
    // Chunks this entry imports, read from the executing file at run START
    // for the same reason the digest is: a rebuild mid-run would otherwise
    // stamp a build that never ran.
    runnerClosure: await readRunnerClosure({ entryPath: line.script, },),
    stageSettings: readStageSettings({ env: process.env, },),
  },);
}

if (import.meta.main)
  await reportingRefusals({
    what: 'rendering-audit-settled',
    argv: process.argv,
    env: process.env,
    run: main,
  },);

//endregion Settled rendering audit
