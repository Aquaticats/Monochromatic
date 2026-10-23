/**
 A cat-themed page whose slices repeat source wording, for the page-level
 consistency pass: a refrain of four lines with one varying slot, rendered four
 ways, and a sentence repeated twice, rendered two ways. The shape follows the
 live case the pass was built for; every word here is invention.

 @module
 */

import type {
  ProviderRecord,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import type { ProjectedLanes, } from '@monochromatic-dev/translation-repair-artifact';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source';
import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import { metersAllWet, } from '@monochromatic-dev/translation-repair-provider/ts/meter-states.test-fixture.ts';
import type {
  BudgetView,
  ChatJsonOutcome,
  ChatJsonRequest,
} from '@monochromatic-dev/translation-repair-provider';
import type {
  ArtifactComparisonRow,
  ArtifactDeliveryRow,
} from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  RUN_LATE_JUDGES,
  type RunClient,
} from '../../dist/final/node/index.mjs';
import {
  REFRAIN_QUOTES,
  SENTENCE_QUOTES,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/consistency-quotes.test-fixture.ts';
import {
  candidateTextsOf,
  currentParagraphOf,
  finalistsOf,
  firstFinalistOf,
  isRefrainSheet,
  sheetKindOf,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/consistency-sheet-reading.test-fixture.ts';

/**
 How many of the late judges answer when the bench splits a run-off in half; the rest throw.
 */
const HEARD_LATE_JUDGES = 4;

//region Consistency page fixture

/**
 The original page: four sections, a refrain clause closing each one and a
 sentence opening the first and standing inside the third.
 */
export const PAGE_SOURCE: string = [
  '## 一',
  '',
  '窗台上的阳光让小猫睡着了。所以这只猫一定是胖胖的小宝贝。',
  '',
  '## 二',
  '',
  '厨房里的鱼香让小猫醒过来了。所以这只猫一定是懒猫的小宝贝。',
  '',
  '## 三',
  '',
  '傍晚的时候，窗台上的阳光让小猫睡着了，小鱼干也笑了。所以这只猫一定是乖仔的小宝贝。',
  '',
  '## 四',
  '',
  '院子里的雨声让小猫跑回家了。所以这只猫一定是暖团的小宝贝。',
  '',
].join('\n',);

/**
 The English of each section as the archive holds it, one entry per section,
 the refrain and the sentence each worded differently from place to place.
 */
const ARCHIVE_SECTIONS: readonly string[] = [
  '## One\n\nThe sunny window put the kitten to sleep. So this cat must be a chubby little treasure.\n',
  '## Two\n\nThe smell of fish in the kitchen woke the kitten. So this cat is surely a lazy little treasure.\n',
  '## Three\n\nIn the evening, the sunlight on the sill lulled the kitten to sleep, and the fish snacks smiled too. '
    + 'So this cat has to be a sweet little darling.\n',
  '## Four\n\nThe rain in the yard sent the kitten running home. So this cat is certainly a warm little dumpling.\n',
];

/**
 The archive page.
 */
const PAGE_ARCHIVE: string = ARCHIVE_SECTIONS.join('\n',);

/**
 Every provider wet: the view the seat reading takes.
 */
const ALL_WET: BudgetView = {
  neuralwatt: false,
  synthetic: false,
  bedrock: false,
  deepseek: false,
  xiaomi: false,
  anthropic: false,
  hyper: false,
  openrouter: false,
};

/**
 The page prepared as the lanes see it.

 @returns Preparation of the original and the archive

 @example
 ```ts
 const prepared = preparedCatPage();
 ```
 */
export function preparedCatPage(): PreparedDocumentPair {
  return prepareDocumentPair({
    sourceText: PAGE_SOURCE,
    targetText: PAGE_ARCHIVE,
  },);
}

/**
 The English each slice carries when the archive stands everywhere.

 @param prepared - preparation of the page

 @returns The archive's text of each slice, in slice order

 @example
 ```ts
 const english = archiveEnglishOf({ prepared: preparedCatPage(), },);
 ```
 */
export function archiveEnglishOf(
  { prepared, }: { readonly prepared: PreparedDocumentPair; },
): readonly string[] {
  return prepared.slices
    .map(function textOf(slice,): string {
      return slice.target
        .text;
    },);
}

/**
 Both lanes' rows over the page, agreeing on the English the caller states for
 each slice, so that is the wording each slice would ship.

 @param prepared - preparation the rows describe

 @param english - what each slice ships, in slice order

 @returns Rows and the comparison

 @example
 ```ts
 const projected = projectedCatPage({ prepared, english: ARCHIVE_SECTIONS, },);
 ```
 */
export function projectedCatPage(
  {
    prepared,
    english,
  }: {
    readonly prepared: PreparedDocumentPair;
    readonly english: readonly string[];
  },
): ProjectedLanes {
  /**
   Comparison row of each slice.
   */
  const comparison = prepared.slices
    .map(function rowOf(
      slice,
      position,
    ): ArtifactComparisonRow {
    /**
     Wording the slice ships.
     */
      const shipped = english[position]
        ?? slice.target
          .text;
    /**
     Whether the archive stands.
     */
      const stands = shipped
        === slice.target
          .text;
      return {
        sliceIndex: slice.target
          .sliceIndex,
        incumbentKind: 'present',
        incumbentText: slice.target
          .text,
        repairText: shipped,
        translateText: shipped,
        laneRelation: stands ? 'archive-stands' : 'both-agree',
        repairOutcome: {
          kind: 'decided',
          acceptedText: shipped,
        },
        translateOutcome: {
          kind: 'decided',
          acceptedText: shipped,
        },
        decisionComparison: {
          kind: 'comparable',
          verdict: 'same',
        },
        repairDelivery: stands ? { kind: 'incumbent-retained', } : { kind: 'replacement-shipped', },
        translateDelivery: stands ? { kind: 'incumbent-retained', } : { kind: 'replacement-shipped', },
      };
    },);
  /**
   Delivery row each lane reports, the same for both.
   */
  const delivery = comparison.map(function deliveryOf(
    row,
    position,
  ): ArtifactDeliveryRow {
    return {
      sliceIndex: row.sliceIndex,
      sourceText: prepared.slices[position]
        ?.source
        .text
        ?? '',
      incumbentKind: 'present',
      incumbentText: row.incumbentText,
      outcome: row.repairOutcome,
      shippedText: row.repairText,
      delivery: row.repairDelivery,
    };
  },);
  return {
    delivery: {
      repair: delivery,
      translate: delivery,
    },
    comparison,
  };
}

/**
 What a scripted seat does when asked.

 @example
 ```ts
 const behaviour: SeatBehaviour = { kind: 'answers', text: '{}', };
 ```
 */
export type SeatBehaviour =
  | {
    /**
     The seat replies with this text.
     */
    readonly kind: 'answers';

    /**
     The reply, as a JSON string.
     */
    readonly text: string;
  }
    | {
    /**
     The seat's call throws.
     */
      readonly kind: 'throws';
    };

/**
 What a scripted client does for each call: the seat asked, how many calls the
 client has had before this one, and the sheet it was shown.

 @example
 ```ts
 const script: SeatScript = function throwsAlways() { return { kind: 'throws', }; };
 ```
 */
export type SeatScript = (
  call: {
    readonly seat: RosterModelId;
    readonly before: number;
    readonly sheet: string;
  },
) => SeatBehaviour;

/**
 Holds of a client none of whose providers is held.

 @returns Zero hold for every provider

 @example
 ```ts
 const holds = noHolds();
 ```
 */
function noHolds(): ProviderRecord<number> {
  return {
    neuralwatt: 0,
    synthetic: 0,
    bedrock: 0,
    deepseek: 0,
    xiaomi: 0,
    anthropic: 0,
    hyper: 0,
    openrouter: 0,
  };
}

/**
 The refusal of a call the scripted client does not serve, as a rejection naming the call.

 @example
 ```ts
 await unservedCall().catch(report);
 ```
 */
function unservedCall(): Promise<never> {
  return Promise.reject(new Error('the scripted client serves only structured calls',),);
}

/**
 The user message of a request, as one text.

 @param request - request the client was handed

 @returns The text of its messages, joined

 @example
 ```ts
 const sheet = sheetOf({ request, },);
 ```
 */
function sheetOf<ValueT,>({ request, }: { readonly request: ChatJsonRequest<ValueT>; },): string {
  return request.messages
    .map(function textOf(message,): string {
      return (typeof message.content) === 'string' ? message.content : '';
    },)
    .join('\n',);
}

/**
 A run client whose every provider is wet and whose judges answer as scripted.

 @param behave - what the seat asked does, handed the seat, how many calls
 the client has had before this one and the sheet it was shown

 @param asked - sink for the seat of every structured call, in order

 @param sheets - sink for the messages of every structured call, in order

 @param dark - seats no provider serves, which the router refuses before any reply

 @param dryness - which providers read dry to the seat reading, every provider wet by default

 @returns Client to drive the consistency pass with

 @example
 ```ts
 const client = scriptedJudgeClient({ behave: () => ({ kind: 'throws', }), asked: [], sheets: [], },);
 ```
 */
export function scriptedJudgeClient(
  {
    behave,
    asked,
    sheets,
    dark = new Set<RosterModelId>(),
    dryness = ALL_WET,
  }: {
    readonly behave: SeatScript;
    readonly asked: RosterModelId[];
    readonly sheets: string[];
    readonly dark?: ReadonlySet<RosterModelId>;
    readonly dryness?: BudgetView;
  },
): RunClient {
  return {
    providerDryness: function viewOfDryness(): Promise<BudgetView> {
      return Promise.resolve(dryness,);
    },
    providerMeterStates: metersAllWet,
    providerHolds: noHolds,
    chatText: unservedCall,
    quotas: unservedCall,
    chatJson: function chatJson<ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> {
      if (dark.has(request.modelId,)) {
        return Promise.reject(new NoProviderForModelError({
          modelId: request.modelId,
          reason: 'every provider serving this model is out of budget',
        },),);
      }
      /**
       Calls the client has had before this one.
       */
      const before = asked.length;
      asked.push(request.modelId,);
      /**
       The sheet this call was shown.
       */
      const sheet = sheetOf({ request, },);
      sheets.push(sheet,);
      /**
       What this seat does.
       */
      const behaviour = behave({
        seat: request.modelId,
        before,
        sheet,
      },);
      if (behaviour.kind === 'throws')
        return Promise.reject(new Error('the scripted seat throws',),);
      /**
       Reply as the sheet parses it.
       */
      const value: unknown = JSON.parse(behaviour.text,);
      return Promise.resolve(request.validate(value,)
        ? {
          kind: 'ok',
          value,
          rawText: behaviour.text,
        }
        : {
          kind: 'schema-mismatch',
          rawText: behaviour.text,
          detail: 'the fixture reply is not a usable ballot',
        },);
    },
  };
}

/**
 One ballot as a judge sends it, reason first.

 @param quotes - the quotes the judge gives

 @param choice - label the judge says every place should follow

 @param keep - labels the judge keeps for their surroundings

 @returns The reply text

 @example
 ```ts
 const text = ballotText({ quotes: SENTENCE_QUOTES, choice: 'OCCURRENCE 1', keep: [], },);
 ```
 */
export function ballotText(
  {
    quotes,
    choice,
    keep,
  }: {
    readonly quotes: readonly Record<string, string>[];
    readonly choice: string;
    readonly keep: readonly string[];
  },
): string {
  return JSON.stringify({
    assessment: 'The first rendering is the most faithful to the original, and the others differ in wording only.',
    candidateQuotes: quotes,
    choice,
    keep,
  },);
}

/**
 A reply that only chooses, with a reason.

 @param choice - label chosen

 @returns Behaviour of a seat that answers with that choice

 @example
 ```ts
 const behaviour = choosing({ choice: 'CANDIDATE 2', },);
 ```
 */
export function choosing({ choice, }: { readonly choice: string; },): SeatBehaviour {
  return {
    kind: 'answers',
    text: JSON.stringify({
      assessment: 'It renders the original best under the house rules.',
      choice,
    },),
  };
}

/**
 What a seat does with a sheet that is not a first-round ballot: a writer
 returns the paragraph as it stands (so it joins the current candidate and
 adds none), a choice among candidates takes the first candidate that is not
 the paragraph as the page has it now (the chosen words moved in by code), and a
 run-off takes the first finalist.

 @param sheet - the sheet the seat was shown

 @returns Behaviour of the seat

 @example
 ```ts
 const behaviour = otherSheetBehaviour({ sheet, },);
 ```
 */
export function otherSheetBehaviour({ sheet, }: { readonly sheet: string; },): SeatBehaviour {
  /**
   The question the sheet asks.
   */
  const kind = sheetKindOf({ sheet, },);
  if (kind === 'writer') {
    return {
      kind: 'answers',
      text: JSON.stringify({ translation: currentParagraphOf({ sheet, },), },),
    };
  }
  if (kind === 'choice') {
    return choosing({ choice: candidateTextsOf({ sheet, },)
      .find(function isAlternative(candidate,): boolean {
        return !candidate.current;
      },)
      ?.label
      ?? 'CANDIDATE 1', },);
  }
  if (kind === 'run-off')
    return choosing({ choice: firstFinalistOf({ sheet, },), },);
  return { kind: 'throws', };
}

/**
 What every seat does with a sheet that is not a first-round ballot when the bench splits a run-off in half: the
 first two late judges choose the first place the run-off shows, the next two the second, and the rest throw, so
 four usable choices tie two and two and the two finalists tie again; the writers write nothing
 and a choice among candidates takes the candidate that is not the paragraph as the page has it now.

 @param seat - the seat asked

 @param sheet - the sheet the seat was shown

 @returns Behaviour of the seat

 @example
 ```ts
 const behaviour = otherSheetsSplittingRunOff({ seat, before: 0, sheet, },);
 ```
 */
export function otherSheetsSplittingRunOff(
  {
    seat,
    sheet,
  }: {
    readonly seat: RosterModelId;
    readonly before: number;
    readonly sheet: string;
  },
): SeatBehaviour {
  if (sheetKindOf({ sheet, },) !== 'run-off')
    return otherSheetBehaviour({ sheet, },);
  /**
   The places the run-off shows.
   */
  const finalists = finalistsOf({ sheet, },);
  /**
   Position of the seat among the late judges.
   */
  const position = RUN_LATE_JUDGES.indexOf(seat,);
  if (position >= HEARD_LATE_JUDGES)
    return { kind: 'throws', };
  return choosing({ choice: (position < 2) ? (finalists[0] ?? '') : (finalists[1] ?? ''), },);
}

/**
 The ballot every seat sends: the first place of each group wins and none is kept.

 @param sheet - the sheet the seat was shown

 @returns Behaviour of a seat that answers it

 @example
 ```ts
 const behaviour = firstPlaceWins({ sheet, },);
 ```
 */
export function firstPlaceWins({ sheet, }: { readonly sheet: string; },): SeatBehaviour {
  if (sheetKindOf({ sheet, },) !== 'ballot')
    return otherSheetBehaviour({ sheet, },);
  return {
    kind: 'answers',
    text: ballotText({
      quotes: isRefrainSheet({ sheet, },) ? REFRAIN_QUOTES : SENTENCE_QUOTES,
      choice: 'OCCURRENCE 1',
      keep: [],
    },),
  };
}

/**
 Seats that answer the refrain's sheet one after another as the list says, in
 the order they are asked, and throw once it runs out; the repeated sentence's
 sheet is answered with the first place winning.

 @param answers - what the k-th seat asked about the refrain does

 @returns Behaviour to hand the scripted client

 @example
 ```ts
 const behave = refrainAnsweredInOrder({ answers: [firstPlaceWins({ sheet: '', },),], },);
 ```
 */
export function refrainAnsweredInOrder(
  { answers, }: { readonly answers: readonly SeatBehaviour[]; },
): SeatScript {
  /**
   Seats asked about the refrain so far.
   */
  const counter = { asked: 0, };
  return function behave({ sheet, },): SeatBehaviour {
    if (!isRefrainSheet({ sheet, },))
      return firstPlaceWins({ sheet, },);
    /**
     What the next seat to be asked does, absent once the list has run out.
     */
    const answer = answers[counter.asked];
    counter.asked += 1;
    return answer ?? { kind: 'throws', };
  };
}

/**
 The refrain's reply of a seat that chooses a place.

 @param choice - label the seat says every place should follow

 @returns Behaviour of a seat that answers with that choice and every quote

 @example
 ```ts
 const behaviour = refrainChoosing({ choice: 'OCCURRENCE 2', },);
 ```
 */
export function refrainChoosing({ choice, }: { readonly choice: string; },): SeatBehaviour {
  return {
    kind: 'answers',
    text: ballotText({
      quotes: REFRAIN_QUOTES,
      choice,
      keep: [],
    },),
  };
}

//endregion Consistency page fixture
