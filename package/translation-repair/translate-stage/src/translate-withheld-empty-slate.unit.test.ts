/**
 Guards ledger B39: a slice the archive translates, whose wording the
 deterministic floor refuses, reaches the translate stage as an absent
 incumbent. Where every translator was heard and none proposed wording the
 floor accepts, the slate was empty in both rounds, the judge raised
 `no-candidate`, and the slice attempt rethrew it for a content slice, so the
 entry stopped. The owner's rule is to keep the archive at such a slice and
 ship (2026-09-27, "No valid wording"; "Preference + polish"), so the
 follow-up round keeps the archive's wording and names why. A slate where
 nobody was heard still interrupts the entry: that is the hour, not the
 passage.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  runTranslateStage,
  TranslationRepairInterruptedError,
} from '../dist/final/node/index.mjs';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the stage under test.
 */
const l = tagged({ tag: 'translate-withheld-empty-slate-test', },);

/**
 Original wish addressing the cat directly.
 */
const SOURCE = '明天早上，你还要陪我去院子里追那只黄蝴蝶哦！';

/**
 Archive wording that turned the address into narration, which the address
 floor refuses.
 */
const ARCHIVE = 'Tomorrow morning, she still has to come chase that yellow butterfly in the yard with me!';

/**
 What every translator proposes: the same narration in other words, refused
 by the same floor.
 */
const PROPOSAL = 'Tomorrow morning, she must chase that yellow butterfly in the yard with me again!';

/**
 Translators, disjoint from the judges.
 */
const TRANSLATORS: readonly RosterModelId[] = [
  'hf:cat/Cat-A',
  'hf:cat/Cat-B',
].map(function toId(id,) {
  return id as unknown as RosterModelId;
},);

/**
 Judges, three so selection could reach its two ballots with one to spare.
 */
const JUDGES: readonly RosterModelId[] = [
  'hf:cat/Cat-C',
  'hf:cat/Cat-D',
  'hf:cat/Cat-E',
].map(function toId(id,) {
  return id as unknown as RosterModelId;
},);

/**
 Client whose translators propose the refused narration and defend it when
 asked, or fail every call; no judge is ever asked, since no slate forms.

 @param heard - whether the translators answer at all

 @returns Client answering both translator turns

 @example
 ```ts
 const client = translatorsClient({ heard: true, },);
 ```
 */
function translatorsClient({ heard, }: { readonly heard: boolean; },): ModelClient {
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
       Which sheet this is.
       */
      const schema = request.responseFormat
        ?.json_schema
        .name;
      if (!heard)
        throw new Error('fixture translator unreachable',);
      /**
       Reply this sheet gets.
       */
      const value: unknown = (schema === 'translation_report')
        ? { translation: PROPOSAL, }
        : (schema === 'translation_repair_report')
          ? {
            resolution: 'as-intended',
            translation: '',
            translatorNote: 'the narration is deliberate',
          }
          : undefined;
      if ((value === undefined) || (!request.validate(value,)))
        throw new Error(`no fixture reply for ${String(schema,)}`,);
      return {
        kind: 'ok',
        value: value as ValueT,
        rawText: JSON.stringify(value,),
      };
    },
  };
}

/**
 Runs the stage over the slice with the archive's refused wording.

 @param heard - whether the translators answer at all

 @returns What the stage settled on

 @example
 ```ts
 const result = await stageOver({ heard: true, },);
 ```
 */
async function stageOver({ heard, }: { readonly heard: boolean; },): Promise<unknown> {
  return await runTranslateStage({
    client: translatorsClient({ heard, },),
    translatorModelIds: TRANSLATORS,
    judgeModelIds: JUDGES,
    sourceText: SOURCE,
    incumbentText: ARCHIVE,
    incumbentKind: 'present',
    lineStructured: false,
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

await describe({
  name: 'a slice whose archive wording fails the floor and no translator proposes any (ledger B39)',
  children: [
    it({
      name: 'KEEPS THE ARCHIVE AND SHIPS where every translator was heard and none proposed wording the floor '
        + 'accepts, naming the empty slate, where the entry used to stop',
      fn: async () => {
        /**
         What the address floor says of the translators' narration.
         */
        const dropped = 'Your translation drops the address in the second person the ORIGINAL carries: where the '
          + 'ORIGINAL writes 你 or 您 once, your translation carries no "you" and more third-person pronouns than '
          + 'the ORIGINAL writes there ("she": 1 against 0), so a pronoun stands where the address stood. A pronoun '
          + 'the ORIGINAL writes is rendered as written where it stands: address the person the ORIGINAL addresses.';
        /**
         What each of the two rounds records: every translator's refused
         narration and defence of it, and the empty slate.
         */
        const round = [
          `translate-invalid (${String(TRANSLATORS[1],)}): ${dropped}`,
          `translate-repair-as-intended (${String(TRANSLATORS[1],)}): the narration is deliberate`,
          `translate-invalid (${String(TRANSLATORS[0],)}): ${dropped}`,
          `translate-repair-as-intended (${String(TRANSLATORS[0],)}): the narration is deliberate`,
          `translate-candidate-refused (${String(TRANSLATORS[1],)}): ${dropped}`,
          `translate-candidate-refused (${String(TRANSLATORS[0],)}): ${dropped}`,
          'translate incumbent excluded by deterministic source floor',
          'translate-candidates (2/2 heard, 0 distinct, 0 collapsed)',
          'translate-no-candidate',
        ];
        /**
         What the stage settled on.
         */
        const result = await stageOver({ heard: true, },) as { readonly findings: readonly string[]; };
        // The translators' lines come in the order their answers arrived, which
        // says nothing, so the findings are compared as a sorted list; both
        // rounds are on it, the follow-up named between them (ledger B41).
        expect({
          ...result,
          findings: result.findings.toSorted(),
        },).toEqual({
          text: ARCHIVE,
          origin: 'incumbent',
          producer: {
            kind: 'incumbent',
            matched: [],
          },
          voteWeight: 0,
          tally: {
            judgesAvailable: 0,
            ballots: 0,
            abstentions: 0,
            selfVotes: 0,
          },
          ballots: [],
          heardTranslators: 2,
          candidateCount: 0,
          slate: [],
          selectedIndex: 0,
          shippedIndex: 0,
          perCandidate: [],
          decision: 'no-candidate',
          findings: [
            ...round,
            'translate-followup-round (after no-candidate, 0 rejected candidates)',
            ...round,
          ].toSorted(),
        },);
      },
    },),
    it({
      name: 'STILL INTERRUPTS where no translator was heard, which says nothing about the passage',
      fn: async () => {
        /**
         What the stage raised.
         */
        const raised = await (async function attempt(): Promise<unknown> {
          try {
            return await stageOver({ heard: false, },);
          }
          catch (error) {
            return error;
          }
        })();
        expect(raised,).toBeInstanceOf(TranslationRepairInterruptedError,);
        expect((raised as { readonly reason?: unknown; }).reason,).toBe('provider-unavailable',);
      },
    },),
  ],
},);
