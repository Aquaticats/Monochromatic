import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  carryChoiceOf,
  type CarryChoiceSubject,
  type CarryOffer,
  carryRunOffFinalistsOf,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import type {
  ArtifactConsistencyCandidate,
  ArtifactConsistencyCarry,
  ArtifactConsistencyRepeat,
} from '@monochromatic-dev/translation-repair-artifact/ts';
import { askCarryChoice, } from './pass-consistency-carry-ask.ts';
import {
  editsOf,
  partlyCarriedFindingOf,
} from './pass-consistency-carry-edits.ts';
import { shownOf, } from './pass-consistency-carry-offers.ts';
import type {
  CarryContext,
  CarryUnit,
  LocatedPlace,
  UnitCarried,
} from './pass-consistency-types.ts';
import { RUN_PER_CALL_TIMEOUT_MS, } from '@monochromatic-dev/translation-repair-run/ts';

//region Pass consistency carry choose
// THE BENCH'S CHOICE AMONG A BLOCK'S CANDIDATES, and what the choice comes to:
// the carry's record for each receiving place of the block and the replacements
// the winner makes. A block with no alternative to its own English is not put to
// the bench. A splice wins for a block that is carried in part: it replaces the
// words of the receiving places code can move the chosen words into, and the places
// it cannot keep their words, which `settleCarried` leaves `kept-current` because no
// replacement names them. The bench's record is the block's, one under each place,
// so the splice stands among the candidates of every place of the block, and a
// finding says which places were carried and which kept their words.

/**
 The record of a carry that ended without a choice among candidates, once for
 each receiving place of its unit.

 @param labels - labels of the receiving places

 @param decision - how it ended

 @param candidates - texts on offer, if any

 @returns Record under each label

 @example
 ```ts
 const carries = carryWithoutChoice({ labels: ['OCCURRENCE 2',], decision: { kind: 'failed', detail: 'Error', }, candidates: [], },);
 ```
 */
export function carryWithoutChoice(
  {
    labels,
    decision,
    candidates,
  }: {
    readonly labels: readonly string[];
    readonly decision: ArtifactConsistencyCarry['decision'];
    readonly candidates: readonly ArtifactConsistencyCandidate[];
  },
): readonly ArtifactConsistencyCarry[] {
  return labels.map(function carryOf(label,): ArtifactConsistencyCarry {
    return {
      label,
      candidates,
      ballots: [],
      usable: 0,
      quorum: 0,
      decision,
    };
  },);
}

/**
 Has the bench choose among a block's candidates and turns the choice into the
 records and the replacements.

 @param context - what the carry reads

 @param repeat - repeated wording or frame

 @param chosen - chosen place and its located rendering

 @param unit - unit

 @param offers - distinct candidates, current block among them

 @param findings - what the candidates' making reported

 @returns Carry's record for each receiving place, replacements it makes and what it reported

 @example
 ```ts
 const carried = await choose({ context, repeat, chosen, unit, offers, findings: [], },);
 ```
 */
export async function choose(
  {
    context,
    repeat,
    chosen,
    unit,
    offers,
    findings,
  }: {
    readonly context: CarryContext;
    readonly repeat: ArtifactConsistencyRepeat;
    readonly chosen: LocatedPlace;
    readonly unit: CarryUnit;
    readonly offers: readonly CarryOffer[];
    readonly findings: readonly string[];
  },
): Promise<UnitCarried> {
  /**
   The candidates as the artifact records them.
   */
  const candidates = offers.map(function recorded({
    label,
    kind,
    writers,
    text,
  },): ArtifactConsistencyCandidate {
    return {
      label,
      kind,
      writers,
      text,
    };
  },);
  /**
   Labels of the receiving places, whose records are one.
   */
  const placeLabels = unit.receivers
    .map(function labelOf({ label, },): string {
      return label;
    },);
  if (offers.length < 2) {
    return {
      carries: carryWithoutChoice({
        labels: placeLabels,
        decision: {
          kind: 'none',
          reason: 'no-alternative',
        },
        candidates,
      },),
      edits: [],
      findings,
    };
  }
  /**
   Labels the sheet shows.
   */
  const labels = offers.map(function labelOf({ label, },): string {
    return label;
  },);
  /**
   What both sheets of the carry show of the places.
   */
  const shown = shownOf({
    context,
    repeat,
    chosen,
    unit,
  },);
  /**
   Logger pre-tagged with this function's name.
   */
  const cl = tagged({
    tag: choose.name,
    l: context.l,
  },);
  /**
   The sheet's subject: what both sheets show of the places and every candidate on offer.
   */
  const subject: CarryChoiceSubject = {
    repeat,
    receiving: shown.receiving,
    chosen: shown.chosen,
    keeping: shown.keeping,
    ...((shown.before === undefined) ? {} : { before: shown.before, }),
    ...((shown.after === undefined) ? {} : { after: shown.after, }),
    candidates: offers.map(function shownCandidate({
      label,
      kind,
      text,
    },) {
      return {
        label,
        current: kind === 'current',
        text,
      };
    },),
    ...((context.identityContext === undefined) ? {} : { identityContext: context.identityContext, }),
    ...((context.referenceContext === undefined) ? {} : { referenceContext: context.referenceContext, }),
    ...((context.lifeEnded === undefined) ? {} : { lifeEnded: context.lifeEnded, }),
    ...((context.subjectNames === undefined) ? {} : { subjectNames: context.subjectNames, }),
  };
  /**
   The judges' choices.
   */
  const asked = await askCarryChoice({
    client: context.client,
    modelIds: context.judgeModelIds,
    subject,
    cache: context.cache,
    signal: context.signal,
    exchangeTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
    l: cl,
    stageSettings: context.stageSettings,
  },);
  /**
   What the choices decided.
   */
  const {
    usable: firstUsable,
    choice: firstChoice,
  } = carryChoiceOf({
    voices: asked.voices,
    labels,
    quorum: asked.quorum,
  },);
  /**
   The finalists when the choices reached the quorum and none has a strict majority, whose run-off the owner's rule of
   2026-10-11 asks for among contests (the lead's adoption of 2026-10-15 for this ask, extended to a lone leader, open to the
   owner's veto).
   */
  const finalists = carryRunOffFinalistsOf({
    voices: asked.voices,
    labels,
    quorum: asked.quorum,
  },);
  if (finalists.length > 0) {
    cl.info(
      `the choice among the candidates had no strict majority (${String(firstUsable,)} usable); asking the bench once more among the finalists ${
        finalists.join(' and ',)
      }`,
    );
  }
  /**
   The bench asked once more among the finalists, absent when there are none. Its sheet shows those candidates only,
   under the labels they had, and its answer is kept under a key of its own.
   */
  const runOffAsked = (finalists.length === 0)
    ? undefined
    : await askCarryChoice({
      client: context.client,
      modelIds: context.judgeModelIds,
      subject: {
        ...subject,
        candidates: subject.candidates
          .filter(function isFinalist({ label, },): boolean {
            return finalists.includes(label,);
          },),
      },
      cache: context.cache,
      signal: context.signal,
      exchangeTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
      l: cl,
      stageSettings: context.stageSettings,
      runOff: true,
    },);
  /**
   What the run-off's choices decided, counted against the finalists only with the run-off's own quorum; a second split keeps
   the page as it stands.
   */
  const runOffCounted = (runOffAsked === undefined)
    ? undefined
    : carryChoiceOf({
      voices: runOffAsked.voices,
      labels: finalists,
      quorum: runOffAsked.quorum,
    },);
  /**
   What the carry came to: the run-off's count when there was a run-off, the first count otherwise.
   */
  const choice = runOffCounted?.choice ?? firstChoice;
  if ((runOffAsked !== undefined) && (runOffCounted !== undefined)) {
    /**
     The run-off's choices as heard.
     */
    const { voices: heard, } = runOffAsked;
    /**
     What the run-off's count decided.
     */
    const { choice: ended, } = runOffCounted;
    /**
     How the run-off ended, in words.
     */
    const ending = (ended.kind === 'chosen') ? `${ended.label} won` : `no winner (${ended.reason})`;
    cl.info(
      `the run-off among the candidates counted ${String(runOffCounted.usable,)} usable of ${String(heard.length,)} heard `
        + `(quorum ${String(runOffAsked.quorum,)}): ${ending}`,
    );
  }
  /**
   The candidate the choices named, as a list of zero or one.
   */
  const winner = (choice.kind === 'chosen')
    ? offers.filter(function isNamed({ label, },): boolean {
      return label === choice.label;
    },)
    : [];
  return {
    carries: placeLabels.map(function carryOf(label,): ArtifactConsistencyCarry {
      return {
        label,
        candidates,
        ballots: asked.voices
          .map(function recordedBallot({
            seat,
            reason,
            choice: chose,
          },) {
            return {
              seat,
              reason,
              choice: chose,
            };
          },),
        usable: firstUsable,
        quorum: asked.quorum,
        ...((runOffAsked === undefined) ? {} : {
          runOff: {
            finalists,
            ballots: runOffAsked.voices
              .map(function recordedRunOffBallot({
                seat,
                reason,
                choice: chose,
              },) {
                return {
                  seat,
                  reason,
                  choice: chose,
                };
              },),
            usable: runOffCounted?.usable ?? 0,
            quorum: runOffAsked.quorum,
          },
        }),
        decision: choice,
      };
    },),
    edits: editsOf({
      unit,
      winner,
    },),
    findings: [
      ...findings,
      ...asked.findings,
      ...(runOffAsked?.findings ?? []),
      ...partlyCarriedFindingOf({
        unit,
        winner,
      },),
    ],
  };
}

//endregion Pass consistency carry choose
