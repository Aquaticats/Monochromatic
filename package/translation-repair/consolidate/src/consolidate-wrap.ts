import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import type {
  ConsolidateGateOutcome,
  GateShipped,
} from './consolidate-gate-stage.ts';
import {
  wordForCount,
  wrapReplacementText,
} from '@monochromatic-dev/translation-repair-typography/ts';
import type { HeardVoice, } from '@monochromatic-dev/translation-repair-stage/ts';
import {
  sameWording,
  type TranslateReportWire,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';

//region Consolidation wrap
// APPLIES THE SEMANTIC WRAP TO A CONSOLIDATION THAT SHIPS, which is the one
// producing path with no assembly step to do it.
//
// The repair lane wraps in `repair-assemble.ts` and the translate lane in
// `translate-assemble.ts`, each at the single point its consumers read from.
// The consolidation stage has neither, so nothing re-applied the rule and
// whether a shipped consolidation carried semantic line breaks depended on
// what the producing model happened to emit.
//
// MEASURED BEFORE THIS EXISTED, across both runs of the band pair: seven of
// eleven shipped consolidations in one run and seven of twelve in the other
// lost the wrapping both lanes had, and the two runs disagreed about the same
// slice on identical inputs. `doc/planning/the-third-rendering.md` records it.
//
// DEMOTION IS RE-DERIVED RATHER THAN CARRIED, for the reason
// `wrapTranslateRecords` gives: a consolidation differing from the standing
// text only in its wrapping IS the standing text once wrapped, and shipping it
// as a consolidation would report a change nobody decided on.

/**
 What ships for one gated slice once the wrap has been applied.

 @example
 ```ts
 const shipped: WrappedConsolidation = { ships: 'standing', text: 'It naps.', rewrapped: false, demoted: false, };
 ```
 */
export type WrappedConsolidation = {
  /**
   Rendering that ships, after a wrap that erased the difference sends the
   slice back to its standing text.
   */
  readonly ships: GateShipped;

  /**
   Wording that ships, wrapped when a consolidation won and untouched when
   the standing text did.
   */
  readonly text: string;

  /**
   Whether the wrap altered what the producer emitted, which separates a
   producer that already wrapped from one this rule had to correct.
   */
  readonly rewrapped: boolean;

  /**
   Whether wrapping left nothing between the consolidation and the standing
   text, so what the gate called a change was only a re-wrapping.
   */
  readonly demoted: boolean;
};

/**
 Wraps a winning consolidation, re-deriving whether it still changes anything.

 NEVER APPLIED TO THE STANDING TEXT. A slice the gate settled on its standing
 text keeps that wording byte for byte: wrapping it would turn a decision to
 change nothing into a change, which is what `wrapReplacementText` refuses by
 contract and what the delivery coherence check refuses by measurement.

 NEVER APPLIED TO A LINE-STRUCTURED SLICE EITHER, at this site or at the two
 lane sites. The pipeline hands a governed producer `TRANSLATE_LINE_STRUCTURE_RULE`,
 one output line per original line, and then broke that work afterwards: over
 the 211 line-structured slices of the pinned corpus the wrap changed 189 and
 broke 470 of 1091 lines, after every decider had approved them.

 @param outcome - what the gate settled, whose `ships` decides whether there
 is anything to wrap

 @param consolidatedText - wording the consolidation produced, as emitted

 @param standingText - wording already in place, which a wrapped
 consolidation may turn out to equal

 @param lineStructured - whether the line-structure rule governs this slice,
 which forbids the wrap outright rather than narrowing it

 @param l - stage logger

 @returns What ships, with the wrap applied and demotion re-derived

 @example
 ```ts
 const shipped = wrapConsolidation({ outcome, consolidatedText, standingText, lineStructured, l, },);
 ```
 */
export function wrapConsolidation(
  {
    outcome,
    consolidatedText,
    standingText,
    lineStructured,
    l,
  }: {
    readonly outcome: ConsolidateGateOutcome;
    readonly consolidatedText: string;
    readonly standingText: string;
    readonly lineStructured: boolean;
    readonly l: Logger;
  },
): WrappedConsolidation {
  if (outcome.ships !== 'consolidated')
    return {
      ships: 'standing',
      text: standingText,
      rewrapped: false,
      demoted: false,
    };

  if (lineStructured) {
    /**
     Whether the producer proposed what already stands.

     BOTH TEXTS UNWRAPPED, because neither has been through the wrap on this
     path, and read as the line-structure rule reads them (`sameWording`):
     trailing whitespace and prose quote style are one wording, and a line
     break is the producer's work. Until 2026-09-29 this compared bytes, so a
     proposal that was the standing text with a trailing newline shipped as a
     change (ledger B26).

     THAT CASE STILL DEMOTES. A replacement identical to its incumbent
     survives the footnote guard and lands in the shipped set beside a
     document nobody changed, which is the fault the demote exists for and
     which skipping the wrap does not make go away.
     */
    const unchanged = sameWording({
      proposal: consolidatedText,
      standing: standingText,
      lineStructured: true,
    },);

    if (unchanged) {
      l.info(
        'semantic wrap: skipped on a line-structured slice, and the consolidation is the standing '
          + 'text\'s wording, so the slice keeps what it had',
      );
      return {
        ships: 'standing',
        text: standingText,
        rewrapped: false,
        demoted: true,
      };
    }

    /**
     Lines the producer wrote, which is the whole point of the skip and so
     the one number worth printing beside it.
     */
    const producedLines = consolidatedText
      .split('\n',)
      .length;

    l.info(
      `semantic wrap: skipped on a line-structured slice, shipping ${
        String(producedLines,)
      } ${
        wordForCount({
          count: producedLines,
          one: 'line',
          many: 'lines',
        },)
      } as the producer wrote them`,
    );
    return {
      ships: 'consolidated',
      text: consolidatedText,
      rewrapped: false,
      demoted: false,
    };
  }

  /**
   Consolidation as the rule would have it written.
   */
  const wrapped = wrapReplacementText({ text: consolidatedText, },);

  /**
   Whether the producer had already written it that way.
   */
  const rewrapped = wrapped !== consolidatedText;

  /**
   Whether anything but layout still separates it from what stands
   (`sameWording`, ledger B26). The key reads both through the wrap and folds
   a paragraph's soft line breaks, so a pure re-wrapping of an unwrapped
   standing text demotes, and so does one whose soft breaks fall where no wrap
   would put them, which the byte comparison this replaced let ship. The
   standing text is not always lane output: where a lane contest settled on
   the incumbent, what stands is the archive's own wording, which nothing has
   ever wrapped. The `demoted` branch returns `standingText` itself, so
   the retained wording still leaves here byte for byte.
   */
  const demoted = sameWording({
    proposal: wrapped,
    standing: standingText,
    lineStructured: false,
  },);

  if (demoted) {
    l.info(
      'semantic wrap: the consolidation is the standing text in all but layout, so the slice keeps what it had',
    );
    return {
      ships: 'standing',
      text: standingText,
      rewrapped,
      demoted,
    };
  }

  /**
   Lines the producer wrote, which makes a rewrap legible in a log rather
   than a claim the reader has to take on faith.
   */
  const emittedLines = consolidatedText
    .split('\n',)
    .length;

  /**
   Lines the rule would have it written on.
   */
  const writtenLines = wrapped
    .split('\n',)
    .length;

  if (rewrapped)
    l.info(
      `semantic wrap: rewrapped a shipped consolidation, ${String(emittedLines,)} ${
        wordForCount({
          count: emittedLines,
          one: 'line',
          many: 'lines',
        },)
      } as emitted `
        + `against ${String(writtenLines,)} as written`,
    );

  return {
    ships: 'consolidated',
    text: wrapped,
    rewrapped,
    demoted,
  };
}

/**
 Wraps every proposal on a slate, so both deciders judge the bytes that ship.

 THE DEFECT THIS CLOSES, measured 2026-08-22 over the two most recent runs of
 the band pair's six entries: 15 of the 16 consolidations that shipped had
 their bytes changed by {@link wrapConsolidation} AFTER the slate judges had
 chosen them and the gate had approved them, 9 of 9 in one run and 6 of 7 in
 the other. Both deciders were reading text the run does not publish.

 THE SAME WRAPPER, AND NO CONFIGURATION TO DRIFT. `wrapReplacementText` takes
 a text and nothing else, so this cannot fall out of step with what
 {@link wrapConsolidation} applies afterwards. If the two ever did disagree,
 the candidate dedup would collapse proposals against bytes that never ship,
 which inverts the point of wrapping them early at all.

 SAFE TO APPLY TWICE, and it is applied twice: a winner wrapped here reaches
 {@link wrapConsolidation} and is wrapped again. Measured rather than assumed
 over twelve representative passages, seven of which the first application
 moved and none of which a second application moved again, and again on
 2026-09-29 over the 7,462 distinct texts of the stored comparison rows, 1,876
 of which the first application moved and none the second (ledger B26).

 THE INCUMBENT IS NEITHER WRAPPED NOR PASSED HERE. Wrapping text a lane
 decided to keep would report a change nobody decided on, which
 `wrapReplacementText`'s own contract refuses. The slate's key (`wordingKey`,
 ledger B26) reads both sides through the wrap instead, so a proposal that is
 a pure re-wrapping of an UNWRAPPED archive standing text collapses into it
 on the slate, and {@link wrapConsolidation}'s demotion still catches it
 after the gate.

 A GOVERNED SLICE IS LEFT ALONE, on the same evidence that stops the shipping
 wrap touching one: over the 211 line-structured slices of the pinned corpus
 this rule changes 189 and breaks 470 of the 1091 lines they already carry.
 Wrapping their proposals would put text in front of the judges that breaks
 the very rule they were given to enforce.

 @param voices - proposals that passed the validity floor

 @param lineStructured - whether the verse rule governs this slice, which
 forbids the wrap outright rather than narrowing it

 @returns Same voices carrying proposals as they would ship, or that array
 untouched where the verse rule governs

 @example
 ```ts
 const onTheSlate = wrapConsolidationProposals({ voices, lineStructured, },);
 ```
 */
export function wrapConsolidationProposals(
  {
    voices,
    lineStructured,
  }: {
    readonly voices: readonly HeardVoice<TranslateReportWire>[];
    readonly lineStructured: boolean;
  },
): readonly HeardVoice<TranslateReportWire>[] {
  // THE GOVERNED ANSWER IS THE ARRAY ITSELF, returned rather than rebuilt, so a
  // verse slice reaches the slate byte for byte as it did before this existed.
  if (lineStructured)
    return voices;

  return voices.map(function wrapOneProposal(voice,): HeardVoice<TranslateReportWire> {
    return {
      ...voice,
      value: {
        ...voice.value,
        translation: wrapReplacementText({ text: voice.value
          .translation, },),
      },
    };
  },);
}

//endregion Consolidation wrap
