import {
  ROSTER_MODEL_IDS,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  idListFlag,
  type ReadsFlag,
  type ReadsSwitch,
  RUN_ROSTER,
} from '@monochromatic-dev/translation-repair-run/ts';

//region Probe candidates
// WHICH SEATABLE MODELS A PROBE MEASURES BESIDE THE SEATED ONES, read off the
// command line.
//
// THE RUN ROSTER IS THE MEASURED ROSTER. Since 2026-09-01 a model takes a role
// on the numbers `judge-fidelity-probe` and `producer-calibrate` report, and
// since 2026-09-07 the roster names two sizes nobody has measured, the Gemma 4
// sizes only Amazon Bedrock serves. Those stay out of `RUN_ROSTER` until the
// probes have read them, and the probes ran the run roster, so nothing could
// ever measure them: a seat that needs a number to be taken could not be asked
// for one.
//
// `--candidates` BREAKS THAT CIRCLE. It names seatable ids, comma separated,
// that join the probe's roster for that run only: they judge beside the seated
// judges in the fidelity probe, and write and judge beside the seated roster in
// the calibration. Nothing outside the probes reads it. A name the roster does
// not know is refused with the names it does, because a probe that quietly ran
// without the model it was started for would print a clean standing over the
// wrong roster, which is the failure `asked-count.ts` exists to stop.
//
// `--candidates-alone` RUNS THE CANDIDATES WITHOUT THE SEATED ROSTER. The
// fidelity probe reads each ballot by the judge that cast it, so two judges
// measured on their own answer the same per-judge question as eleven, and they
// answer it at the candidates' provider only: a probe run beside a production
// pass then spends nothing the pass is spending. The calibration ranks
// producers by disinterested ballots, which two seats cannot supply, so it runs
// the flag too and is refused by `assertJudgeableProducerRoster` on its own
// terms.

/**
 Flag the candidate ids are written after.
 */
const CANDIDATES_FLAG = '--candidates';

/**
 Flag that leaves the seated roster out and runs the candidates by themselves.
 */
const ALONE_FLAG = '--candidates-alone';

/**
 Reads seatable ids named after `--candidates`.

 @param line - the probe's command line, read whole by `reportingRefusals`
 and passed in so this is testable without a subprocess

 @returns Candidate ids in the order written, none when the flag is absent

 @throws StatedRefusalError when the flag carries no id, or names one the
 roster does not know

 @example
 ```ts
 const candidates = readCandidateIds({ line, },);
 ```
 */
export function readCandidateIds(
  { line, }: { readonly line: ReadsFlag<'candidates'>; },
): readonly RosterModelId[] {
  /**
   Ids as written after the flag, split on commas, space and blanks dropped.
   */
  const written = idListFlag({
    asked: line.flag('candidates',),
    naming: `seatable id of ${ROSTER_MODEL_IDS.join(',',)}`,
  },);

  return written.map(function seatable(id,): RosterModelId {
    /**
     Roster spelling this id matches, absent when the roster knows no such id.
     */
    const known = ROSTER_MODEL_IDS.find(function is(rosterId,): boolean {
      return rosterId === id;
    },);
    if (known === undefined) {
      throw new StatedRefusalError({
        says: `${CANDIDATES_FLAG} names ${
          JSON.stringify(id,)
        }, which is not seatable; the roster knows ${ROSTER_MODEL_IDS.join(',',)}`,
      },);
    }
    return known;
  },);
}

/**
 Reads whether `--candidates-alone` was written.

 @param line - the probe's command line, read whole by `reportingRefusals`
 and passed in so this is testable without a subprocess

 @returns Whether the candidates run without the seated roster

 @example
 ```ts
 const alone = readCandidatesAlone({ line, },);
 ```
 */
export function readCandidatesAlone(
  { line, }: { readonly line: ReadsSwitch<'candidates-alone'>; },
): boolean {
  return line.switched('candidates-alone',);
}

/**
 Roster a probe runs: the seated roster, then every candidate it does not
 already seat, in the order the candidates were named; or the candidates by
 themselves when asked to run alone.

 @param candidates - ids read by {@link readCandidateIds}

 @param alone - whether the seated roster stays out,
 read by {@link readCandidatesAlone}

 @returns Seated roster followed by the candidates new to it, or the
 candidates each once

 @throws StatedRefusalError when asked to run the candidates alone and none
 was named, since a probe over nobody measures nothing

 @example
 ```ts
 const roster = probeRosterWith({
   candidates: readCandidateIds({ line, },),
   alone: readCandidatesAlone({ line, },),
 },);
 ```
 */
export function probeRosterWith(
  {
    candidates,
    alone,
  }: {
    readonly candidates: readonly RosterModelId[];
    readonly alone: boolean;
  },
): readonly RosterModelId[] {
  /**
   Candidates each once, in the order named.
   */
  const once = candidates.filter(function isFirst(
    candidate,
    at,
  ): boolean {
    return candidates.indexOf(candidate,) === at;
  },);
  if (alone) {
    if (once.length === 0) {
      throw new StatedRefusalError({
        says: `${ALONE_FLAG} runs the candidates without the seated roster, and ${CANDIDATES_FLAG} named none`,
      },);
    }
    return once;
  }

  /**
   Candidates the seated roster does not already carry.
   */
  const joining = once.filter(function isNew(candidate,): boolean {
    return !RUN_ROSTER.includes(candidate,);
  },);
  return [
    ...RUN_ROSTER,
    ...joining,
  ];
}

//endregion Probe candidates
