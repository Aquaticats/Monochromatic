/**
 Tests that one editor's edit nested too deeply to read costs the repair chunk
 that edit and nothing else.

 THE SHAPE UNDER TEST. An editor's edit is text from one voice. The apply gate
 checked what the edit kept and what it quoted but never whether any grammar
 reads the text it produces, so an edit of hundreds of nested list markers was
 applied, judged and checked, and then the parse that settles the chunk raised
 the plain grammar's refusal out of the whole chunk. Each case here has two
 editors write the same region, one of them the deep edit.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RepairModels, } from '@monochromatic-dev/translation-repair-assembly';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  repairChunk,
  repairPreparedDocument,
} from '../dist/final/node/index.mjs';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

//region Deep edit tests

/**
 Original of the chunk: a nap and a bowl, two paragraphs.
 */
const SOURCE_TEXT = '小猫在窗台上打盹。\n\n碗是满的。\n';

/**
 Archive English of the chunk, the second paragraph mistranslating the bowl.
 */
const TARGET_TEXT = 'The kitten dozes on the windowsill.\n\nThe bowl is empty.\n';

/**
 The editor who writes the deep edit.
 */
const DEEP_EDITOR = SEAT_HYPER_OPENROUTER_VISION_EDITOR;

/**
 The editor who writes an ordinary edit of the same region.
 */
const ORDINARY_EDITOR = SEAT_SYNTHETIC_VISION_WITHHELD;

/**
 What the ordinary editor writes for the bowl.
 */
const ORDINARY_EDIT = 'The bowl is full.';

/**
 Roster of two editors, three critics and panelists, and three checkers apart
 from every editor.
 */
const MODELS: RepairModels = {
  criticModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD,],
  panelModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD,],
  editorModelIds: [DEEP_EDITOR, ORDINARY_EDITOR,],
  judgeModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD,],
  checkerModelIds: [
    SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
    SEAT_SYNTHETIC_TEXT_EVERYWHERE,
    SEAT_HYPER_OPENROUTER_UNMEASURED,
  ],
};

/**
 What a prober reports for a region it finds no introduced defect in.
 */
const CLEAN_DEFECT_CHECK = {
  verdict: 'no-introduced-defect-found',
  category: '',
  severity: '',
  evidence: '',
  omittedText: '',
  verdictReason: '',
} as const;

/**
 Claim every critic files: the bowl is full, not empty.
 */
const BOWL_CLAIM = {
  category: 'accuracy/mistranslation',
  severity: 'major',
  summary: 'A full bowl is rendered as empty.',
  sourceQuote: '碗是满的。',
  targetQuote: 'The bowl is empty.',
} as const;

/**
 Original of the two-slice document: a nap under one heading, a bowl under
 another.
 */
const DOCUMENT_SOURCE_TEXT = '## 小睡\n\n小猫在窗台上打盹。\n\n## 碗\n\n碗是满的。\n';

/**
 Archive English of the two-slice document, the bowl mistranslated.
 */
const DOCUMENT_TARGET_TEXT = '## Nap\n\nThe kitten dozes on the windowsill.\n\n## Bowl\n\nThe bowl is empty.\n';

/**
 Original of the one-slice document the refinement phase reads: a bowl under
 a heading, in one paragraph long enough to be eligible for it.
 */
const LONG_SOURCE_TEXT = '## 碗\n\n碗是满的。小猫在窗台上看了很久，一动也不动，因为她希望有人很快再把它装满，好让她吃一顿饭。\n';

/**
 Archive English of that document, the bowl mistranslated.
 */
const LONG_TARGET_TEXT = '## Bowl\n\nThe bowl is empty. The kitten watches it from the windowsill for a very long time '
  + 'without moving at all, because she hopes someone will fill it again soon so that she can eat a meal.\n';

/**
 What the ordinary editor writes for the first sentence of the long paragraph.
 */
const LONG_ORDINARY_EDIT = 'The bowl is full.';

/**
 Claim every critic files on the nap: dozing is not napping.
 */
const NAP_CLAIM = {
  category: 'accuracy/mistranslation',
  severity: 'major',
  summary: 'Napping is rendered as dozing.',
  sourceQuote: '小猫在窗台上打盹。',
  targetQuote: 'The kitten dozes on the windowsill.',
} as const;

/**
 An edit chosen by the region it is asked about, so two slices of one document
 are edited differently.
 */
type SliceEdit = {
  /**
   Text the editor's sheet holds when it is asked about the slice.
   */
  readonly onSheetsWith: string;

  /**
   What the editor writes for that slice.
   */
  readonly newText: string;
};

/**
 The rewriter the refinement phase asks, apart from the editors and checkers.
 */
const REWRITER = SEAT_HYPER_VISION;

/**
 Text of every message one exchange carried, joined.

 @param request - exchange as the stage sent it

 @returns Sheet the stage was sent

 @example
 ```ts
 const sheet = sheetOf({ request, },);
 ```
 */
function sheetOf({ request, }: { readonly request: ChatJsonRequest<unknown>; },): string {
  return request.messages
    .map(function toText(message,): string {
      return messageText({ message, },);
    },)
    .join('\n',);
}

/**
 Times a marker begins an item of the sheet one exchange carried.

 @param request - exchange as the stage sent it

 @param marker - text each item of the sheet begins with

 @returns How many items the sheet carries

 @example
 ```ts
 const claims = itemsOnSheet({ request, marker: '\nCLAIM ', },);
 ```
 */
function itemsOnSheet(
  {
    request,
    marker,
  }: {
    readonly request: ChatJsonRequest<unknown>;
    readonly marker: string;
  },
): number {
  return sheetOf({ request, },)
    .split(marker,)
    .length - 1;
}

/**
 Claims every critic files on the sheet it was sent: the bowl claim on a sheet
 that holds the bowl sentence, none on a sheet that does not.

 @param sheet - sheet the critic was sent

 @returns Claims on the sentence the sheet holds

 @example
 ```ts
 const issues = claimsOn({ sheet: '碗是满的。', },);
 ```
 */
function claimsOn({ sheet, }: { readonly sheet: string; },): readonly object[] {
  return sheet.includes('碗是满的。',) ? [BOWL_CLAIM,] : [];
}

/**
 One-based numbers of the items of the sheet one exchange carried, which a
 reply answers in order.

 @param request - exchange as the stage sent it

 @param marker - text each item of the sheet begins with

 @returns Numbers 1 through the count of items

 @example
 ```ts
 const claims = sheetNumbers({ request, marker: '\nCLAIM ', },);
 ```
 */
function sheetNumbers(
  {
    request,
    marker,
  }: {
    readonly request: ChatJsonRequest<unknown>;
    readonly marker: string;
  },
): readonly number[] {
  return Array.from(
    { length: itemsOnSheet({ request, marker, },), },
    function toNumber(_unused, index,): number {
      return index + 1;
    },
  );
}

/**
 Client answering every stage of one chunk, the editors from a table.

 @param edits - what each editor writes for the bowl, by editor

 @param ordinary - what an editor not named in `edits` writes

 @param rewrites - what each rewriter writes for the first paragraph of its
 sheet, by rewriter, none for a rewriter not named

 @param claims - claims every critic files, those on the sheet's own text when
 absent

 @param sliceEdits - edits chosen by the slice the editor is asked about, which
 take the place of the editor's own

 @returns Client honoring the script

 @example
 ```ts
 const client = chunkClient({ edits: { [DEEP_EDITOR]: '- cat', }, },);
 ```
 */
function chunkClient(
  {
    edits,
    ordinary = ORDINARY_EDIT,
    rewrites = {},
    claims,
    sliceEdits = [],
  }: {
    readonly edits: Readonly<Record<string, string>>;
    readonly ordinary?: string;
    readonly rewrites?: Readonly<Record<string, string>>;
    readonly claims?: readonly object[];
    readonly sliceEdits?: readonly SliceEdit[];
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by the repair chunk',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Stage the request belongs to.
       */
      const stage = request.responseFormat
        ?.json_schema
        .name;
      /**
       Wire reply this stage was scripted to give.
       */
      const reply: unknown = (stage === 'critic_report')
        ? { issues: claims ?? claimsOn({ sheet: sheetOf({ request, },), },), }
        : (stage === 'panel_ballot')
          ? {
            verdicts: sheetNumbers({ request, marker: '\nCLAIM ', },)
              .map(function toVerdict(claim,) {
                return {
                  claim,
                  reason: 'The quotes show it.',
                  vote: 'supported',
                };
              },),
          }
          : (stage === 'editor_report')
            ? {
              edits: [{
                region: 1,
                newText: sliceEdits.find(function asked(sliceEdit,): boolean {
                  return sheetOf({ request, },).includes(sliceEdit.onSheetsWith,);
                },)
                  ?.newText
                  ?? edits[request.modelId]
                  ?? ordinary,
              },],
            }
            : (stage === 'refine_report')
              ? {
                rewrites: (rewrites[request.modelId] === undefined)
                  ? []
                  : [{ paragraph: 1, newText: rewrites[request.modelId], },],
              }
              : (stage === 'candidate_ballot')
                ? { best: 1, assessment: 'scripted', }
                : (stage === 'resolution_report')
                  ? {
                    checks: sheetNumbers({ request, marker: '\nISSUE ', },)
                      .map(function toCheck(issue,) {
                        return { issue, verdict: 'fixed', };
                      },),
                  }
                  : (stage === 'introduced_defect_report')
                    ? {
                      checks: sheetNumbers({ request, marker: ' REGION ', },)
                        .map(function toCheck(region,) {
                          return {
                            ...CLEAN_DEFECT_CHECK,
                            region,
                          };
                        },),
                    }
                    : undefined;
      if (reply === undefined)
        throw new Error(`this chunk has no script for stage ${String(stage,)}`,);
      if (!request.validate(reply,))
        throw new Error(`the script failed the ${stage} guard`,);
      return {
        kind: 'ok',
        value: reply,
        rawText: JSON.stringify(reply,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused by the repair chunk',);
    },
  };
}

/**
 Runs the fixture chunk with the two editors writing what the table says.

 @param deepEdit - what the deep editor writes for the bowl

 @returns What the chunk settled on and every line it logged

 @example
 ```ts
 const { outcome, } = await runChunkWithDeepEdit({ deepEdit: `${'- '.repeat(300,)}The bowl is full.`, },);
 ```
 */
async function runChunkWithDeepEdit(
  { deepEdit, }: { readonly deepEdit: string; },
) {
  /**
   Lines the chunk logged.
   */
  const messages: string[] = [];
  /**
   What the chunk settled on.
   */
  const outcome = await repairChunk({
    client: chunkClient({ edits: { [DEEP_EDITOR]: deepEdit, }, },),
    sliceIndex: 0,
    sourceText: SOURCE_TEXT,
    targetText: TARGET_TEXT,
    lineStructured: false,
    models: MODELS,
    declaredNames: [],
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l: capturingLogger({ messages, },),
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return {
    outcome,
    messages,
  };
}

/**
 Runs the one-slice document of the long paragraph, with the ordinary editor
 repairing it and the refinement phase asked of one rewriter.

 @param deepRewrite - what the rewriter writes for the paragraph

 @returns What the document settled on

 @example
 ```ts
 const result = await runLongDocumentWithDeepRewrite({ deepRewrite: `${'- '.repeat(300,)}The bowl is full.`, },);
 ```
 */
async function runLongDocumentWithDeepRewrite(
  { deepRewrite, }: { readonly deepRewrite: string; },
) {
  return await repairPreparedDocument({
    client: chunkClient({
      edits: {},
      ordinary: LONG_ORDINARY_EDIT,
      rewrites: { [REWRITER]: deepRewrite, },
    },),
    prepared: prepareDocumentPair({
      sourceText: LONG_SOURCE_TEXT,
      targetText: LONG_TARGET_TEXT,
    },),
    models: {
      ...MODELS,
      refinerModelIds: [REWRITER,],
    },
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

/**
 Runs the two-slice document with the two editors writing what the table says.

 @param deepEdit - what the deep editor writes for the bowl

 @returns What the document settled on

 @example
 ```ts
 const result = await runDocumentWithDeepEdit({ deepEdit: 'The bowl is full.', },);
 ```
 */
async function runDocumentWithDeepEdit(
  { deepEdit, }: { readonly deepEdit: string; },
) {
  return await repairPreparedDocument({
    client: chunkClient({ edits: { [DEEP_EDITOR]: deepEdit, }, },),
    prepared: prepareDocumentPair({
      sourceText: DOCUMENT_SOURCE_TEXT,
      targetText: DOCUMENT_TARGET_TEXT,
    },),
    models: MODELS,
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: repairChunk.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SHIPS the other editor\'s repair when one editor wrote 300 nested list markers, and records '
            + 'that editor\'s refusal',
          fn: async () => {
            const { outcome, } = await runChunkWithDeepEdit({ deepEdit: `${'- '.repeat(300,)}${ORDINARY_EDIT}`, },);

            expect(outcome.changed,).toBe(true,);
            expect(outcome.repairedText,).toBe(`The kitten dozes on the windowsill.\n\n${ORDINARY_EDIT}\n`,);
            expect(outcome.findings
              .filter(function isRefusal(finding,): boolean {
                return finding.includes('unreadable-replacement',);
              },),).toEqual([
                `${DEEP_EDITOR}: unreadable-replacement (nested too deeply to read: its container markers pass the bound of 256 at line 3, column 513)`,
              ],);
          },
        },),
        it({
          name: 'APPLIES an edit nested 256 list markers deep, the last depth the bound allows',
          fn: async () => {
            /**
             The edit, 256 markers then the ordinary sentence.
             */
            const deepEdit = `${'- '.repeat(256,)}${ORDINARY_EDIT}`;
            const { outcome, } = await runChunkWithDeepEdit({ deepEdit, },);

            expect(outcome.changed,).toBe(true,);
            expect(outcome.findings
              .filter(function isRefusal(finding,): boolean {
                return finding.includes('unreadable-replacement',);
              },),).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: repairPreparedDocument.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'FINISHES the document on the other editor\'s repair of its second slice when one editor wrote '
            + '300 nested list markers there, and leaves the first slice as the archive has it',
          fn: async () => {
            const result = await runDocumentWithDeepEdit({ deepEdit: `${'- '.repeat(300,)}${ORDINARY_EDIT}`, },);

            expect(result.repairedText,).toBe(DOCUMENT_TARGET_TEXT.replace(
              'The bowl is empty.',
              ORDINARY_EDIT,
            ),);
            expect(result.findings
              .filter(function isRefusal(finding,): boolean {
                return finding.includes('unreadable-replacement',);
              },),).toEqual([
                `${DEEP_EDITOR}: unreadable-replacement (nested too deeply to read: its container markers pass the bound of 256 at line 3, column 513)`,
              ],);
          },
        },),
        it({
          name: 'WITHDRAWS the first slice\'s repair and keeps the second\'s when one editor opened a fence there '
            + 'and never closed it and another wrote a fence and 300 open brackets after it, which together leave '
            + 'a page no grammar reads',
          fn: async () => {
            const result = await repairPreparedDocument({
              client: chunkClient({
                edits: {},
                claims: [NAP_CLAIM, BOWL_CLAIM,],
                sliceEdits: [
                  { onSheetsWith: 'CURRENT TEXT: The kitten', newText: '```\ncode', },
                  { onSheetsWith: 'CURRENT TEXT: The bowl', newText: `\`\`\`\n${'['.repeat(300,)}${ORDINARY_EDIT}`, },
                ],
              },),
              prepared: prepareDocumentPair({
                sourceText: DOCUMENT_SOURCE_TEXT,
                targetText: DOCUMENT_TARGET_TEXT,
              },),
              models: MODELS,
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            expect(result.repairedText,).toBe(
              `## Nap\n\nThe kitten dozes on the windowsill.\n\n## Bowl\n\n\`\`\`\n${'['.repeat(300,)}${ORDINARY_EDIT}\n`,
            );
            expect(result.findings
              .filter(function isWithdrawal(finding,): boolean {
                return finding.startsWith('assembly-',);
              },),).toEqual([
                'assembly-structure-single-withdrawal slice 0: reverting this replacement leaves no introduced '
                  + 'structural or footnote defect',
                'assembly-structure-observed unreadable-page (round 1)',
              ],);
          },
        },),
        it({
          name: 'FINISHES the refinement phase on the repaired paragraph when the rewriter wrote 300 nested list '
            + 'markers, which its paragraph gate refused, and keeps the editor\'s repair',
          fn: async () => {
            const result = await runLongDocumentWithDeepRewrite({
              deepRewrite: `${'- '.repeat(300,)}${LONG_ORDINARY_EDIT}`,
            },);

            // The repair assembly lays each clause of the repaired paragraph on a
            // line of its own.
            expect(result.repairedText,).toBe(
              '## Bowl\n\nThe bowl is full.\nThe kitten watches it from the windowsill for a very long time without moving '
                + 'at all,\nbecause she hopes someone will fill it again soon so that she can eat a meal.\n',
            );
            expect(result.findings
              .filter(function isRefusal(finding,): boolean {
                return finding.includes('refine-atom-gate-refused',);
              },),).toEqual([
                `${REWRITER}: refine-atom-gate-refused (paragraph 1, candidate rejected)`,
              ],);
          },
        },),
      ],
    },),
  ],
},);

//endregion Deep edit tests
