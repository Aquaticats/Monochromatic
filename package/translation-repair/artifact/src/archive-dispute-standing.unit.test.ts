/**
 Guards the owner's answer of 2026-09-27 ("No eligible standing", sixteenth
 addendum of `doc/decision/translation-repair-ineligible-standing.md`): on a
 disputed slice the repair lane's text stands in for the archive only where
 the checkers confirmed every disputing issue resolved in it and the lane did
 not withdraw the slice. Everywhere else the slice has no eligible standing:
 the archive's own wording and the repair lane's text are refused like any
 text the deterministic rule refuses, so neither is a candidate, a fallback,
 a standing or a lane offer, and a translator copying the archive is
 withheld. Measured on six runs, 86 of 87 disputed slices had no disputing
 issue resolved.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { validateTranslatedSlice, } from '@monochromatic-dev/translation-repair-translate-floor';
import {
  translateSliceInput,
  translateSliceKey,
} from '@monochromatic-dev/translation-repair-translate-lane';
import { runTranslateStage, } from '@monochromatic-dev/translation-repair-translate-stage';
import { readStandingVerdict, } from '@monochromatic-dev/translation-repair-consolidate';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { archiveDisputesOf, } from '@monochromatic-dev/translation-repair-archive';
import type {
  AdjudicatedIssue,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  archiveStandInFor,
  laneTextsForSlate,
} from '../dist/final/node/index.mjs';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Picture marker the archive carries on its own line, which lets a trailing
 transcript be held out of writing and judging as a target-only run.
 */
const MARKER = `<PhotoScroll photos={['${['$', '{path}',].join('',)}/photos/letter.webp']} />`;

/**
 Logger the stages write through, whose output is not under test.
 */
const l = tagged({ tag: 'archive-dispute-standing-test', },);

/**
 Invented original: the cat slept by the window.
 */
const SOURCE = '猫在窗边睡着了。';

/**
 Archive rendering, which adds a purr the original never states.
 */
const ARCHIVE = 'The cat fell asleep by the window, purring loudly.';

/**
 Repair lane's text where its patch lost and a refinement kept the reading.
 */
const REFINED = 'By the window the cat fell asleep, purring loudly.';

/**
 A faithful rendering.
 */
const FRESH = 'The cat fell asleep by the window.';

/**
 Accepted addition issue against the archive.
 */
const ADDITION: AdjudicatedIssue = {
  issueId: 'adjudicated/purr',
  status: 'accepted',
  severity: 'major',
  claims: [{
    claimId: 'issue/purr',
    claim: {
      category: 'accuracy/addition',
      severity: 'major',
      summary: 'The translation adds that the cat purred, which the original never states.',
      spans: [],
    },
  },],
  tallies: {},
};

/**
 Disputes read off one chunk under the given resolution and withdrawal.

 @param resolvedIssueIds - issues the checkers confirmed fixed

 @param withdrawn - whether the lane withdrew the slice

 @returns Dispute over slice 3, if any

 @example
 ```ts
 const dispute = disputeOf({ resolvedIssueIds: [], withdrawn: false, },);
 ```
 */
function disputeOf(
  {
    resolvedIssueIds,
    withdrawn,
  }: {
    readonly resolvedIssueIds: readonly string[];
    readonly withdrawn: boolean;
  },
): ReturnType<typeof archiveDisputesOf> {
  return archiveDisputesOf({
    chunks: [{
      sliceIndex: 3,
      repairedText: REFINED,
      changed: true,
      issues: [ADDITION,],
      resolvedIssueIds,
    },],
    withdrawnSliceIndices: withdrawn ? [3,] : [],
  },);
}

/**
 The unresolved dispute every consolidation case reads.
 */
const UNRESOLVED = disputeOf({
  resolvedIssueIds: [],
  withdrawn: false,
},);

/**
 Translators and judges for the stage case.
 */
const TRANSLATORS: readonly RosterModelId[] = [
  'hf:cat/Cat-A',
  'hf:cat/Cat-B',
].map(function toId(id,) {
  return id as unknown as RosterModelId;
},);

/**
 Judges who rendered nothing.
 */
const JUDGES: readonly RosterModelId[] = [
  'hf:cat/Cat-C',
  'hf:cat/Cat-F',
  'hf:cat/Cat-G',
].map(function toId(id,) {
  return id as unknown as RosterModelId;
},);

/**
 Client whose first translator copies the archive every time it is asked and
 whose second renders faithfully, and whose judges want the purr wherever the
 sheet offers it, else the first candidate.

 @returns Client over the translate stage

 @example
 ```ts
 const client = copyingClient();
 ```
 */
function copyingClient(): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by the translate lane',);
    },
    quotas: async () => {
      throw new Error('quotas unused by the translate lane',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Stage the request belongs to, by its reply schema.
       */
      const schema = request.responseFormat?.json_schema.name ?? '';
      /**
       Sheet as the voice reads it.
       */
      const sheet = request.messages
        .map(function textOf(message,): string {
          return messageText({ message, },);
        },)
        .join('\n',);
      /**
       Rendering a translator call gets: the archive copied by the first
       translator, the faithful one by the second.
       */
      const translation = (request.modelId === TRANSLATORS[0]) ? ARCHIVE : FRESH;
      /**
       Where the purr stands on the judge sheet, minus one when nowhere.
       */
      const purring = sheet.split('CANDIDATE ',)
        .slice(1,)
        .findIndex(function carriesPurr(block,): boolean {
          return block.includes('purring',);
        },);
      /**
       Reply for the stage.
       */
      const value: unknown = (schema === 'translation_report')
        ? { translation, }
        : {
          best: (purring === (-1)) ? 1 : (purring + 1),
          assessment: 'keeps the purr',
        };
      if (!request.validate(value,))
        throw new Error(`fixture ${schema} reply failed validation`,);
      return {
        kind: 'ok',
        value: value as ValueT,
        rawText: JSON.stringify(value,),
      };
    },
  };
}

await describe({
  name: 'a disputed reading the repair did not fix leaves no eligible standing (owner, 2026-09-27)',
  children: [
    it({
      name: 'LETS THE STAND-IN STAND only where the checkers confirmed every disputing issue resolved and the '
        + 'lane kept the slice',
      fn: async () => {
        expect(UNRESOLVED.get(3,)?.standInEligible,).toBe(false,);
        expect(disputeOf({ resolvedIssueIds: ['adjudicated/purr',], withdrawn: false, },).get(3,)?.standInEligible,)
          .toBe(true,);
        expect(disputeOf({ resolvedIssueIds: ['adjudicated/purr',], withdrawn: true, },).get(3,)?.standInEligible,)
          .toBe(false,);
      },
    },),
    it({
      name: 'REFUSES a text that is a disputed wording, in any spacing, naming why',
      fn: async () => {
        /**
         The wording refused and why.
         */
        const disputedWordings = [{
          text: ARCHIVE,
          reason: 'the archive rendering the adjudicators disputed',
        },];
        const copied = validateTranslatedSlice({
          sourceText: SOURCE,
          candidateText: `  ${ARCHIVE.replace(' by ', '  by\n',)}\n`,
          pageText: ARCHIVE,
          disputedWordings,
        },);
        expect(copied.kind,).toBe('invalid',);
        expect(JSON.stringify(copied,).includes('the archive rendering the adjudicators disputed',),).toBe(true,);
        expect(validateTranslatedSlice({
          sourceText: SOURCE,
          candidateText: FRESH,
          pageText: ARCHIVE,
          disputedWordings,
        },).kind,).toBe('valid',);
      },
    },),
    it({
      name: 'HANDS THE CONSOLIDATION the archive as its incumbent with both disputed wordings, and never ships '
        + 'the stand-in as the standing',
      fn: async () => {
        const read = archiveStandInFor({
          archiveDisputes: UNRESOLVED,
          sliceIndex: 3,
          incumbentText: ARCHIVE,
          choice: 'neither',
          l,
        },);
        expect(read.incumbentText,).toBe(ARCHIVE,);
        expect(read.standInShips,).toBe(false,);
        expect(read.disputedWordings.map(function textOf(wording,): string {
          return wording.text;
        },),).toEqual([
          ARCHIVE,
          REFINED,
        ],);
      },
    },),
    it({
      name: 'REFUSES a standing that is the repair lane\'s text on an unresolved dispute, with no incumbent '
        + 'standing in for it, and offers neither disputed wording on the slate',
      fn: async () => {
        /**
         The disputed wordings as the consolidation reads them.
         */
        const { disputedWordings, } = archiveStandInFor({
          archiveDisputes: UNRESOLVED,
          sliceIndex: 3,
          incumbentText: ARCHIVE,
          choice: 'repair',
          l,
        },);
        const verdict = readStandingVerdict({
          sourceText: SOURCE,
          standingText: REFINED,
          incumbentText: ARCHIVE,
          lineStructured: false,
          choice: 'repair',
          contestVerdict: {
            kind: 'lane-won',
            lane: 'repair',
          },
          sliceIndex: 3,
          l,
          disputedWordings,
        },);
        expect(verdict.standingValid,).toBe(false,);
        expect(verdict.incumbentStandsIn,).toBe(false,);
        expect(verdict.standingRefusal?.includes('disputed',),).toBe(true,);
        expect(laneTextsForSlate({
          sourceText: SOURCE,
          incumbentText: ARCHIVE,
          repairText: REFINED,
          translateText: ARCHIVE,
          standingText: REFINED,
          standingMayShip: false,
          standingEligible: false,
          disputedWordings,
        },),).toEqual([],);
      },
    },),
    it({
      name: 'KEEPS THE ARCHIVE OFF THE TRANSLATE SLATE on an unresolved dispute, withholding a translator that '
        + 'copies it, so the faithful rendering ships though the judges want the purr',
      fn: async () => {
        const result = await runTranslateStage({
          client: copyingClient(),
          translatorModelIds: TRANSLATORS,
          judgeModelIds: JUDGES,
          sourceText: SOURCE,
          incumbentText: ARCHIVE,
          incumbentKind: 'present',
          lineStructured: false,
          disputedWordings: [{
            text: ARCHIVE,
            reason: 'the archive rendering the adjudicators disputed',
          },],
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(result.text,).toBe(FRESH,);
        expect(result.findings.some(function withheldCopy(finding,): boolean {
          return finding.startsWith('translate-candidate-refused',);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'LEAVES AN ELIGIBLE STAND-IN UNREFUSED where a target-only run is held out of it, refusing only the '
        + 'archive',
      fn: async () => {
        const prepared = prepareDocumentPair({
          sourceText: `猫睡了。\n\n${MARKER}`,
          targetText: `The cat slept, purring loudly.\n\n${MARKER}\n\n> Dear cat, rest well.`,
        },);
        const [slice,] = prepared.slices;
        if (slice === undefined)
          throw new Error('fixture requires a prepared slice',);
        const surface = translateSliceInput({
          slice,
          prepared,
          archiveStandIn: `The cat slept.\n\n${MARKER}\n\n> Dear cat, rest well.`,
          disputedWordings: [{
            text: slice.target.text,
            reason: 'the archive rendering the adjudicators disputed',
          },],
        },);
        expect(surface.protectedText,).toBe('\n\n> Dear cat, rest well.',);
        expect((surface.stageInput.disputedWordings ?? []).some(function refusesIncumbent(wording,): boolean {
          return wording.text === surface.stageInput.incumbentText;
        },),).toBe(false,);
        // THE CONTROL: with no stand-in the archive is the incumbent, and its
        // judged part is refused with it.
        const withheld = translateSliceInput({
          slice,
          prepared,
          disputedWordings: [{
            text: slice.target.text,
            reason: 'the archive rendering the adjudicators disputed',
          },],
        },);
        expect((withheld.stageInput.disputedWordings ?? []).some(function refusesIncumbent(wording,): boolean {
          return wording.text === withheld.stageInput.incumbentText;
        },),).toBe(true,);
      },
    },),
    it({
      name: 'KEYS a disputed slice apart from the same slice undisputed',
      fn: async () => {
        /**
         Inputs both keys share.
         */
        const shared = {
          runShape: 'cat-run',
          sourceText: SOURCE,
          incumbentText: ARCHIVE,
          incumbentKind: 'present',
          lineStructured: false,
        } as const;
        expect(translateSliceKey({
          ...shared,
          archiveDisputeNote: 'ARCHIVE RENDERING DISPUTED: the purr',
        },),).not.toBe(translateSliceKey(shared,),);
      },
    },),
  ],
},);
