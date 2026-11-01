import { AUDIT_ARMS, } from './audit-sensitivity-input.ts';
import { runAuditSensitivity, } from './audit-sensitivity-run.ts';
import {
  type CommandLineOf,
  createProbeRunClient,
  reportingRefusals,
  resolveRunsDir,
} from '@monochromatic-dev/translation-repair-run/ts';
import { digestPipeline, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { nowAsIso, } from './probe-run-clock.ts';
import { readRunnerClosure, } from '@monochromatic-dev/translation-repair-insertion/ts';
import { readStageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Audit sensitivity
// Asks whether the rendering audit can detect a planted defect at all, and
// whether it invents one where there is none.
//
// THIS IS THE INSTRUMENT'S OWN GUARD TEST. Everything else about the audit is
// checked against scripted replies, which prove the code does what it says and
// prove nothing about what real auditors do with the prompt. Two arms are the
// least that can say anything: a rendering with one dropped negator, where a
// working instrument corroborates a defect, and a faithful rendering, where a
// working instrument corroborates none.
//
// THREE MEASUREMENTS PER ARM, KEPT APART, because a single count cannot say
// which part failed:
//
// -   ORACLE HITS, per voice: did this auditor point anywhere near the planted
//     defect, whatever it called the defect and whether or not anyone agreed.
//     This is about the AUDITORS.
// -   DROPPED CLAIMS, per voice: did an auditor point at it and fail to anchor.
//     This is about the PROMPT and the SCREEN.
// -   CORROBORATED, AGREED and NEAR MISSES: did the matcher bring two voices
//     together. This is about the MATCHER.
//
// A run that scores three oracle hits, zero drops and zero agreement of either
// tier is a matcher problem. One that scores zero oracle hits is not.
//
// `agreed` IS A SUPERSET OF `corroborated`, since exactly equal focus intervals
// trivially overlap. So `corroborated=1 agreed=1` is ONE defect reported at two
// strengths, not two findings, and `corroborated=0 agreed=1` is the loose tier
// catching what the strict one missed.
//
// Inputs live in `audit-sensitivity-input.ts` and are cat-themed invention. NO
// corpus text takes part.
//
// IT KEEPS ITS ANSWERS, which it did not always. Both arms ran three times on
// 2026-08-17 and the results existed only in the terminal that ran them, so
// nothing can now re-read them, and the instrument's stability across those
// runs cannot be checked by anyone. Every invocation now lands under
// `audit-sensitivity/` in the runs directory, carrying the roster and the
// pipeline digest that produced it. Standard output is unchanged.
//
// IT ONLY RUNS WHEN INVOKED, which it also did not always. Both arms used to
// execute at module scope, so anything that imported this file bought a full
// roster of calls by importing it. `coverage-probe.ts` has carried the guard
// against exactly that from the start; this file did not, and nothing but the
// bundler's entry list stood between an ordinary import and the spend.

/**
 Reads what only a real process has and hands it to the arms.

 @param line - the probe's command line, read whole by `reportingRefusals`, which
 refuses any argument, since this probe reads none; it names the script run

 @example
 ```ts
 await main({ line, },);
 ```
 */
async function main({ line, }: { readonly line: CommandLineOf<'audit-sensitivity'>; },): Promise<void> {
  /**
   When this invocation began, read before any call so the record dates the
   run rather than the moment it happened to finish.
   */
  const startedAt = nowAsIso();

  /**
   Digest over built output, which is the only identity that moves when the
   code moves but the commit does not.

   READ AT THE START, not at the end. A long run gives a developer plenty of
   time to rebuild, and `rendering-audit-settled` was caught doing exactly
   that: `dist` was rebuilt while a run was in flight, so the digest it was
   about to stamp described a build that had never probed anything. Node loads
   the code once, at startup; the identity that answers for a run is the one
   present THEN.
   */
  const { digest: pipelineDigest, } = await digestPipeline({ dir: import.meta.dirname, },);

  /**
   Chunks this entry imports, read from the executing file at run START for
   the same reason the digest is: a rebuild mid-run would otherwise stamp a
   build that never ran.
   */
  const runnerClosure = await readRunnerClosure({ entryPath: line.script, },);

  return runAuditSensitivity({
    arms: AUDIT_ARMS,
    newClient: createProbeRunClient,
    runsDir: await resolveRunsDir(),
    startedAt,
    now: nowAsIso,
    pipelineDigest,
    runnerClosure,
    stageSettings: readStageSettings({ env: process.env, },),
  },);
}

// Guarded so this runs only when INVOKED, never as an import side effect: for a
// probe that spends a full roster per arm, loading the library would otherwise
// buy the calls.
if (import.meta.main)
  await reportingRefusals({
    what: 'audit-sensitivity',
    argv: process.argv,
    env: process.env,
    run: main,
  },);

//endregion Audit sensitivity
