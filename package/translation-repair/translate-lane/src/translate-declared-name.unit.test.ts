/**
 Tests for the refusal that stops a translate replacement dropping a name the
 archive itself declared.

 WHAT THIS EXISTS TO CATCH. Asked whether a rendering that removed a
 front-matter alias had left anything out, every judge on the roster said it
 had not, and rewording their criteria moved half of them. A rule half a panel
 disagrees with is not a rule, so {@link findDroppedDeclaredNames} decides this
 one without asking anybody. That makes the wiring the whole question: a guard
 that is computed and not consulted looks identical to a guard that passed.

 A GUARD PROVES NOTHING UNTIL SHOWN TO FAIL, so the cases come in threes. One
 settlement accepts a replacement, one refuses the same shape of replacement
 for dropping a declared name, and one accepts that same dropping replacement
 once nothing is declared. The outer two are what say the refusal came from
 the declared list rather than from a run that fell over before deciding
 anything.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { isAsciiDigit, } from '@monochromatic-dev/translation-repair-markup';
import {
  settleTranslateSlice,
  type TranslateSliceRecord,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger the lane writes its progress to.
 */
const l = tagged({ tag: 'translate-declared-name-test', },);

/**
 Schema name the translate stage asks translators for, which is how a request
 is told apart from a judge's ballot without reading its prose.
 */
const TRANSLATE_SCHEMA = 'translation_report';

/**
 Heading the selection sheet puts before each candidate, followed by its
 number.
 */
const CANDIDATE_MARKER = 'CANDIDATE ';

/**
 Ballot answer meaning no candidate is acceptable.
 */
const DECLINE = 0;

/**
 Original this slice renders.
 */
const SOURCE_TEXT = '喵喵是家里最小的猫，大家都叫她团子，她整天睡在窗台上。';

/**
 Name the archive's own front matter declares.
 */
const DECLARED_NAME = 'Meowmeow';

/**
 Alias the same front matter declares, which is the form real judges removed.
 */
const DECLARED_ALIAS = 'Dumpling';

/**
 Translation already in the archive, carrying both declared forms.
 */
const INCUMBENT_TEXT =
  'Meowmeow is the smallest cat in the house, and everyone calls her Dumpling; she sleeps on the windowsill all day.';

/**
 Wording common to both fresh renderings and to neither the original nor the
 incumbent, so a judge can find which numbered candidate is the fresh one
 without being told.
 */
const FRESH_SENTINEL = 'youngest cat here';

/**
 Fresh rendering that reads better and keeps every declared form.
 */
const KEEPS_EVERY_NAME =
  'Meowmeow is the youngest cat here, known to all as Dumpling, and she naps on the windowsill from dawn to dusk.';

/**
 Fresh rendering that reads better and drops one declared form, which is the
 trade real judges accepted.
 */
const DROPS_THE_ALIAS =
  'Meowmeow is the youngest cat here, and she naps on the windowsill from dawn to dusk.';

/**
 Translation in the archive that names the person by the display name alone,
 where the original names her by her alias (XingZ60 slice 80: the original
 writes the alias and the archive writes the display name).
 */
const NAME_ONLY_INCUMBENT_TEXT = 'Meowmeow is the smallest cat in the house, and she sleeps on the windowsill all day.';

/**
 Fresh rendering that names her by the alias the original writes, and so
 carries a declared form the archive did not and loses the one it did.
 */
const SWAPS_THE_NAME_FOR_THE_ALIAS =
  'Dumpling is the youngest cat here, and she naps on the windowsill from dawn to dusk.';

/**
 Fresh rendering that fuses an undeclared word onto the alias, the shape of
 XingZ60 slice 80's `Xing Lili`, a name nobody declared.
 */
const FUSES_A_WORD_ONTO_THE_ALIAS =
  'Wooly Dumpling is the youngest cat here, and she naps on the windowsill from dawn to dusk.';

/**
 Forms the archive declares for this person.
 */
const DECLARED_NAMES: readonly string[] = [
  DECLARED_NAME,
  DECLARED_ALIAS,
];

/**
 Models that render the slice.
 */
const TRANSLATORS: readonly RosterModelId[] = [
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_HYPER_VISION,
];

/**
 Whole roster the judges are drawn from, translators included.
 */
const JUDGES: readonly RosterModelId[] = [
  ...TRANSLATORS,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
];

/**
 Builds the slice pair the lane settles over one archive wording.

 @param incumbentText - wording the archive carries for the slice

 @returns Slice pair over the fixture's original

 @example
 ```ts
 const slice = sliceCarrying({ incumbentText: INCUMBENT_TEXT, },);
 ```
 */
function sliceCarrying({ incumbentText, }: { readonly incumbentText: string; },): ChunkPair {
  return {
    source: {
      sliceIndex: 1,
      nodes: [],
      startOffset: 0,
      endOffset: SOURCE_TEXT.length,
      text: SOURCE_TEXT,
    },
    target: {
      sliceIndex: 1,
      nodes: [],
      startOffset: 0,
      endOffset: incumbentText.length,
      text: incumbentText,
    },
  };
}

/**
 Finds where a run of digits ends.

 @param text - sheet being read

 @param from - offset the digits start at

 @returns Offset of first character that is not a digit

 @example
 ```ts
 const end = firstNonDigit({ text: sheet, from: 12, },);
 ```
 */
function firstNonDigit(
  { text, from, }: { readonly text: string; readonly from: number; },
): number {
  for (let cursor = from; cursor < text.length; cursor += 1) {
    /**
     Character under the cursor.
     */
    const character = text.charAt(cursor,);
    if (!isAsciiDigit({ character, },))
      return cursor;
  }
  return text.length;
}

/**
 Reads which numbered candidate on a sheet carries some wording.

 JUDGES VOTE BY POSITION AND THE SLATE IS ROTATED PER SLICE, so a fixture that
 always answered 1 would be voting for whichever candidate the rotation
 happened to put there. This finds the candidate by its text instead, which is
 what a judge does.

 @param sheet - whole sheet one judge received

 @param sentinel - wording only one candidate carries

 @returns Candidate number, or {@link DECLINE} when no candidate carries it

 @throws When a candidate heading is not followed by its number, which means
 the sheet no longer numbers candidates the way this fixture reads them

 @example
 ```ts
 const best = candidateNumberCarrying({ sheet, sentinel: FRESH_SENTINEL, },);
 ```
 */
function candidateNumberCarrying(
  { sheet, sentinel, }: { readonly sheet: string; readonly sentinel: string; },
): number {
  /**
   Where the wording appears.
   */
  const at = sheet.indexOf(sentinel,);
  if (at === (-1))
    return DECLINE;

  /**
   Heading opening the candidate that carries it.
   */
  const marker = sheet.lastIndexOf(CANDIDATE_MARKER, at,);
  if (marker === (-1))
    return DECLINE;

  /**
   Where its number starts.
   */
  const start = marker + CANDIDATE_MARKER.length;

  /**
   Number as the sheet spells it.
   */
  const digits = sheet.slice(start, firstNonDigit({ text: sheet, from: start, },),);
  if (digits === '')
    throw new Error(`candidate heading at ${String(marker,)} carries no number`,);
  return Number(digits,);
}

/**
 Builds a client whose translators all return one rendering and whose judges
 all choose it.

 EVERY TRANSLATOR RETURNS THE SAME TEXT on purpose. The slate then holds
 exactly two candidates, the archive's and the fresh one, which is the
 comparison the guard sits in front of.

 @param rendering - translation every translator proposes

 @returns Client answering both roles

 @example
 ```ts
 const client = judgingClient({ rendering: DROPS_THE_ALIAS, },);
 ```
 */
function judgingClient({ rendering, }: { readonly rendering: string; },): ModelClient {
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
       Whole sheet this role received, in call order.
       */
      const content = request.messages
        .map(function toContent(message,) {
          return messageText({ message, },);
        },)
        .join('\n',);

      /**
       Schema the caller asked for, which names the role.
       */
      const schema = request.responseFormat
        ?.json_schema
        .name;

      /**
       Answer this role gives.
       */
      const reply: unknown = (schema === TRANSLATE_SCHEMA)
        ? { translation: rendering, }
        : {
          best: candidateNumberCarrying({ sheet: content, sentinel: FRESH_SENTINEL, },),
          assessment: 'fixture',
        };
      if (!request.validate(reply,)) {
        return {
          kind: 'schema-mismatch',
          rawText: JSON.stringify(reply,),
          detail: 'reply failed the wire guard',
        };
      }
      return {
        kind: 'ok',
        value: reply,
        rawText: JSON.stringify(reply,),
      };
    },
  };
}

/**
 Settles the slice once with one rendering and one declared-name list.

 @param rendering - translation every translator proposes and every judge
 chooses

 @param declaredNames - forms preparation found in the front matter

 @param incumbentText - wording the archive carries for the slice, the
 archive's own wording when absent

 @param declaredSubjectForms - forms among them that name the subject,
 absent when preparation named none

 @returns Record the lane settled on

 @example
 ```ts
 const record = await settleWith({ rendering: DROPS_THE_ALIAS, declaredNames: DECLARED_NAMES, },);
 ```
 */
async function settleWith(
  {
    rendering,
    declaredNames,
    incumbentText = INCUMBENT_TEXT,
    declaredSubjectForms,
  }: {
    readonly rendering: string;
    readonly declaredNames: readonly string[];
    readonly incumbentText?: string;
    readonly declaredSubjectForms?: readonly string[];
  },
): Promise<TranslateSliceRecord> {
  /**
   Slice pair over the archive wording under test.
   */
  const slice = sliceCarrying({ incumbentText, },);

  /**
   Preparation the slice belongs to.
   */
  const prepared: PreparedDocumentPair = {
    sourceText: SOURCE_TEXT,
    targetText: incumbentText,
    slices: [slice,],
    lineStructuredSliceIndices: new Set<number>(),
    declaredNames,
    ...((declaredSubjectForms === undefined) ? {} : { declaredSubjectForms, }),
    alignmentFindings: [],
    unclaimedTargetBlocks: [],
    alignmentPairCount: 1,
  };

  return await settleTranslateSlice({
    client: judgingClient({ rendering, },),
    slice,
    prepared,
    models: {
      translatorModelIds: TRANSLATORS,
      judgeModelIds: JUDGES,
    },
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

await describe({
  name: 'translate declared-name survival',
  children: [
    it({
      name: 'POSITIVE CONTROL: ACCEPTS a replacement that keeps every declared form, so a later '
        + 'assertion that some settlement kept the archive text is reading a guard rather than a '
        + 'harness whose judges never chose anything',
      fn: async () => {
        const record = await settleWith({
          rendering: KEEPS_EVERY_NAME,
          declaredNames: DECLARED_NAMES,
        },);
        expect(record.disposition,).toBe('stage-result',);
        expect(record.outputText,).toBe(KEEPS_EVERY_NAME,);
        expect(record.changed,).toBe(true,);
      },
    },),
    it({
      name: 'REFUSES a replacement that drops a declared form, keeping the archive text and naming '
        + 'what was lost, which is the decision no judge on the roster made unaided',
      fn: async () => {
        const record = await settleWith({
          rendering: DROPS_THE_ALIAS,
          declaredNames: DECLARED_NAMES,
        },);
        expect(record.disposition,).toBe('refused-declared-name',);
        expect(record.outputText,).toBe(INCUMBENT_TEXT,);
        expect(record.changed,).toBe(false,);
        expect((record.disposition === 'refused-declared-name') ? record.droppedDeclaredNames : undefined,).toEqual([
          DECLARED_ALIAS,
        ],);
      },
    },),
    it({
      name: 'ACCEPTS that same dropping replacement when the archive declares nothing, so the '
        + 'refusal is attributable to the declared list and not to anything else about that wording',
      fn: async () => {
        const record = await settleWith({
          rendering: DROPS_THE_ALIAS,
          declaredNames: [],
        },);
        expect(record.disposition,).toBe('stage-result',);
        expect(record.outputText,).toBe(DROPS_THE_ALIAS,);
      },
    },),
    it({
      name: 'ACCEPTS a replacement that names the person by another declared form of hers, the shape of '
        + 'XingZ60 slice 80: the archive writes the display name where the original writes the alias, '
        + 'and the replacement writes the alias, so the person is named and the archive text is not kept',
      fn: async () => {
        const record = await settleWith({
          rendering: SWAPS_THE_NAME_FOR_THE_ALIAS,
          declaredNames: DECLARED_NAMES,
          incumbentText: NAME_ONLY_INCUMBENT_TEXT,
          declaredSubjectForms: DECLARED_NAMES,
        },);
        expect(record.disposition,).toBe('stage-result',);
        expect(record.outputText,).toBe(SWAPS_THE_NAME_FOR_THE_ALIAS,);
        expect(record.changed,).toBe(true,);
      },
    },),
    it({
      name: 'REFUSES A SWAP THAT FUSES AN UNDECLARED WORD ONTO THE ALIAS, since `Wooly Dumpling` is a renamed '
        + 'person and not another declared form of hers',
      fn: async () => {
        const record = await settleWith({
          rendering: FUSES_A_WORD_ONTO_THE_ALIAS,
          declaredNames: DECLARED_NAMES,
          incumbentText: NAME_ONLY_INCUMBENT_TEXT,
          declaredSubjectForms: DECLARED_NAMES,
        },);
        expect(record.disposition,).toBe('refused-declared-name',);
        expect(record.outputText,).toBe(NAME_ONLY_INCUMBENT_TEXT,);
        expect((record.disposition === 'refused-declared-name') ? record.droppedDeclaredNames : undefined,).toEqual([
          DECLARED_NAME,
        ],);
      },
    },),
    it({
      name: 'REFUSES that same swap when preparation named no subject identity, so the acceptance of that swap '
        + 'is attributable to the identity and to nothing else about that wording',
      fn: async () => {
        const record = await settleWith({
          rendering: SWAPS_THE_NAME_FOR_THE_ALIAS,
          declaredNames: DECLARED_NAMES,
          incumbentText: NAME_ONLY_INCUMBENT_TEXT,
        },);
        expect(record.disposition,).toBe('refused-declared-name',);
        expect(record.outputText,).toBe(NAME_ONLY_INCUMBENT_TEXT,);
        expect((record.disposition === 'refused-declared-name') ? record.droppedDeclaredNames : undefined,).toEqual([
          DECLARED_NAME,
        ],);
      },
    },),
  ],
},);
