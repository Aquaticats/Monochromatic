/**
 Tests pre-polish baseline contributor authority floor.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { NaturalnessRepairInterruptedError, } from '@monochromatic-dev/translation-repair-adjudication';
import { applyFinalPolish, } from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Invented-size roster for every polish role.
 */
const ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
] as const;

/**
 Dispute note the consolidate gate read over this slice.
 */
const DISPUTE_NOTE = 'ARCHIVE RENDERING DISPUTED: the adjudicators accepted 1 claim(s) about the tabby\'s nap.';

/**
 What the repair panel accepted about the slice, which no dispute covers.
 */
const REPAIR_CLAIMS_NOTE = 'REPAIR PANEL CLAIMS ACCEPTED: the repair lane\'s adjudicators accepted 1 claim(s); the claims: '
  + '(1) accuracy/mistranslation minor: dozed softens the tabby\'s nap.';

/**
 Client proposing one rewrite, recording each polish gate sheet.

 @param gateSheets - where each polish gate sheet is recorded

 @param sheetsBySchema - where every sheet is recorded under its reply
 schema's name, for the cases reading what each role was shown

 @param rewrite - refiner's proposal, faithful one unless a case needs
 another

 @returns Scripted client
 */
function recordingClient(
  {
    gateSheets,
    sheetsBySchema,
    rewrite = 'The tabby napped in the warm window all afternoon.',
  }: {
    readonly gateSheets: string[];
    readonly sheetsBySchema?: Map<string, readonly string[]>;
    readonly rewrite?: string;
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by structured polish stages',);
    },
    chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Stage the request belongs to, by its reply schema.
       */
      const schema = request.responseFormat?.json_schema.name ?? '';
      if (schema === 'consolidation_polish_gate')
        gateSheets.push(JSON.stringify(request.messages,),);
      sheetsBySchema?.set(schema, [
        ...(sheetsBySchema.get(schema,) ?? []),
        JSON.stringify(request.messages,),
      ],);
      /**
       Reply for the stage.
       */
      const value: unknown = (schema === 'refine_report')
        ? { rewrites: [{ paragraph: 1, newText: rewrite, },], }
        : (schema === 'candidate_ballot')
          ? { best: 1, assessment: 'more idiomatic', }
          : (schema === 'consolidation_polish_gate')
            ? { verdict: 'polished', unsupported: [], dropped: [], assessment: 'equally faithful', }
            : (schema === 'absolute_naturalness_review')
              ? { isAcceptable: true, findings: [], assessment: 'publication-ready', }
              : {};
      if (!request.validate(value,))
        throw new Error(`synthetic ${schema} reply failed validation`,);
      return {
        kind: 'ok',
        value,
        rawText: JSON.stringify(value,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused by polish stages',);
    },
  };
}

await describe({
  name: applyFinalPolish.name,
  children: [
    it({
      name: 'PAUSES INVALID BASE before no-polish fallback can retain contributor respelling',
      fn: async () => {
        const client: ModelClient = {
          chatText: async () => {
            throw new Error('model calls must not run before baseline floor',);
          },
          chatJson: async () => {
            throw new Error('model calls must not run before baseline floor',);
          },
          quotas: async () => {
            throw new Error('meter unused by baseline floor fixture',);
          },
        };
        let caught: unknown;
        try {
          await applyFinalPolish({
            client,
            settlement: {
              terminal: 'consolidated',
              text: 'Contributors for this entry: Snowflake',
              floor: { kind: 'proposals', validModelIds: [], },
              verdicts: [],
              rewrapped: false,
              demoted: false,
              findings: [],
            },
            subject: {
              sourceText: '本条目贡献者：雪猫',
              incumbentText: 'Contributors for this entry: [Snow](https://example.test/snow)',
            },
            lineStructured: false,
            sliceIndex: 1,
            eligible: true,
            signal: AbortSignal.timeout(HANG_STOP_MS,),
            perCallTimeoutMs: HANG_STOP_MS,
            l: tagged({ tag: 'contributor-baseline-floor-test', },),
            stageSettings: SCRIPTED_STAGE_SETTINGS,
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(NaturalnessRepairInterruptedError,);
      },
    },),
    it({
      name: 'CARRIES the slice\'s dispute note to every polish gate sheet (ledger S12: the consolidate gate read '
        + 'it before approving the base, and the polish gate deciding a rewrite of that base never did)',
      fn: async () => {
        /**
         Every polish gate sheet the settlement sent.
         */
        const gateSheets: string[] = [];
        await applyFinalPolish({
          client: recordingClient({ gateSheets, },),
          settlement: {
            terminal: 'consolidated',
            text: 'The tabby spent the whole afternoon napping in the warm window.',
            floor: { kind: 'proposals', validModelIds: [], },
            verdicts: [],
            rewrapped: false,
            demoted: false,
            findings: [],
          },
          subject: {
            sourceText: '虎斑猫整个下午都在温暖的窗边打盹。',
            incumbentText: 'The tabby dozed by the warm window all afternoon long.',
            archiveDisputeNote: DISPUTE_NOTE,
          },
          lineStructured: false,
          sliceIndex: 2,
          polishConfig: {
            refinerModelIds: [ROSTER[0],],
            judgeModelIds: ROSTER,
            gateModelIds: ROSTER,
            declaredNames: [],
            definitions: '',
          },
          eligible: true,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l: tagged({ tag: 'polish-dispute-note-test', },),
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(gateSheets.length,).toBeGreaterThan(0,);
        expect(gateSheets.every(function carriesNote(sheet,): boolean {
          return sheet.includes(DISPUTE_NOTE,);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'CARRIES the claims the repair panel accepted for the slice to every polish gate sheet, a slice no '
        + 'dispute covers included (CuspariaKLSY21 slice 0)',
      fn: async () => {
        /**
         Every polish gate sheet the settlement sent.
         */
        const gateSheets: string[] = [];
        await applyFinalPolish({
          client: recordingClient({ gateSheets, },),
          settlement: {
            terminal: 'consolidated',
            text: 'The tabby spent the whole afternoon napping in the warm window.',
            floor: { kind: 'proposals', validModelIds: [], },
            verdicts: [],
            rewrapped: false,
            demoted: false,
            findings: [],
          },
          subject: {
            sourceText: '虎斑猫整个下午都在温暖的窗边打盹。',
            incumbentText: 'The tabby dozed by the warm window all afternoon long.',
            acceptedRepairClaimsNote: REPAIR_CLAIMS_NOTE,
          },
          lineStructured: false,
          sliceIndex: 2,
          polishConfig: {
            refinerModelIds: [ROSTER[0],],
            judgeModelIds: ROSTER,
            gateModelIds: ROSTER,
            declaredNames: [],
            definitions: '',
          },
          eligible: true,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l: tagged({ tag: 'polish-repair-claims-test', },),
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(gateSheets.length,).toBeGreaterThan(0,);
        expect(gateSheets.every(function carriesNote(sheet,): boolean {
          return sheet.includes(REPAIR_CLAIMS_NOTE,);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'CARRIES the claims the repair panel accepted to the polish WRITERS as well as the gate (ledger B805: '
        + 'the writers rewrote the base without the evidence the gate then judged them against), and a slice with '
        + 'none shows its writers no such block',
      fn: async () => {
        /**
         Polishes one slice and returns the writers' sheets.

         @param note - claims the panel accepted, none left out
         @returns Every sheet the refiner was sent
         */
        async function refinerSheetsWith(
          { note, }: { readonly note?: string; },
        ): Promise<readonly string[]> {
          /** Every sheet the polish sent, by reply schema. */
          const sheetsBySchema = new Map<string, readonly string[]>();
          await applyFinalPolish({
            client: recordingClient({ gateSheets: [], sheetsBySchema, },),
            settlement: {
              terminal: 'consolidated',
              text: 'The tabby spent the whole afternoon napping in the warm window.',
              floor: { kind: 'proposals', validModelIds: [], },
              verdicts: [],
              rewrapped: false,
              demoted: false,
              findings: [],
            },
            subject: {
              sourceText: '虎斑猫整个下午都在温暖的窗边打盹。',
              incumbentText: 'The tabby dozed by the warm window all afternoon long.',
              ...((note === undefined) ? {} : { acceptedRepairClaimsNote: note, }),
            },
            lineStructured: false,
            sliceIndex: 2,
            polishConfig: {
              refinerModelIds: [ROSTER[0],],
              judgeModelIds: ROSTER,
              gateModelIds: ROSTER,
              declaredNames: [],
              definitions: '',
            },
            eligible: true,
            signal: AbortSignal.timeout(HANG_STOP_MS,),
            perCallTimeoutMs: HANG_STOP_MS,
            l: tagged({ tag: 'polish-writer-claims-test', },),
            stageSettings: SCRIPTED_STAGE_SETTINGS,
          },);
          return sheetsBySchema.get('refine_report',) ?? [];
        }
        const withClaims = await refinerSheetsWith({ note: REPAIR_CLAIMS_NOTE, },);
        const without = await refinerSheetsWith({},);
        expect(withClaims.length,).toBeGreaterThan(0,);
        expect(withClaims.every(function carriesNote(sheet,): boolean {
          return sheet.includes(REPAIR_CLAIMS_NOTE,);
        },),).toBe(true,);
        // THE CONTROL REACHES THE WRITER TOO, or "no such block" would hold of no sheet at all.
        expect(without.length,).toBeGreaterThan(0,);
        expect(without.some(function mentions(sheet,): boolean {
          return sheet.includes('REPAIR PANEL CLAIMS ACCEPTED',);
        },),).toBe(false,);
      },
    },),
    it({
      name: 'CARRIES the dispute note and the accepted claims to the polish WRITERS and to the JUDGES of their '
        + 'rewrites as well as the gate (ledger B805: a judge of a text sees what its writer saw), and a slice with '
        + 'neither shows them no such block',
      fn: async () => {
        /**
         Polishes one slice and returns the sheets of the writers and of the rewrite judges.

         @param notes - which notes the slice carries, none for a slice with neither
         @returns Every refiner sheet and every rewrite ballot sheet the polish sent
         */
        async function sheetsWith(
          { notes = false, }: { readonly notes?: boolean; },
        ): Promise<{ readonly writers: readonly string[]; readonly judges: readonly string[]; }> {
          /** Every sheet the polish sent, by reply schema. */
          const sheetsBySchema = new Map<string, readonly string[]>();
          await applyFinalPolish({
            client: recordingClient({ gateSheets: [], sheetsBySchema, },),
            settlement: {
              terminal: 'consolidated',
              text: 'The tabby spent the whole afternoon napping in the warm window.',
              floor: { kind: 'proposals', validModelIds: [], },
              verdicts: [],
              rewrapped: false,
              demoted: false,
              findings: [],
            },
            subject: {
              sourceText: '虎斑猫整个下午都在温暖的窗边打盹。',
              incumbentText: 'The tabby dozed by the warm window all afternoon long.',
              ...(notes
                ? {
                  archiveDisputeNote: DISPUTE_NOTE,
                  acceptedRepairClaimsNote: REPAIR_CLAIMS_NOTE,
                }
                : {}),
            },
            lineStructured: false,
            sliceIndex: 2,
            polishConfig: {
              refinerModelIds: [ROSTER[0],],
              judgeModelIds: ROSTER,
              gateModelIds: ROSTER,
              declaredNames: [],
              definitions: '',
            },
            eligible: true,
            signal: AbortSignal.timeout(HANG_STOP_MS,),
            perCallTimeoutMs: HANG_STOP_MS,
            l: tagged({ tag: 'polish-judge-notes-test', },),
            stageSettings: SCRIPTED_STAGE_SETTINGS,
          },);
          return {
            writers: sheetsBySchema.get('refine_report',) ?? [],
            judges: sheetsBySchema.get('candidate_ballot',) ?? [],
          };
        }
        const withNotes = await sheetsWith({ notes: true, },);
        const without = await sheetsWith({},);
        for (const role of ['writers', 'judges',] as const) {
          expect(withNotes[role].length,).toBeGreaterThan(0,);
          for (const note of [DISPUTE_NOTE, REPAIR_CLAIMS_NOTE,]) {
            expect(withNotes[role].every(function carriesNote(sheet,): boolean {
              return sheet.includes(note,);
            },),).toBe(true,);
          }
          // THE CONTROL REACHES THE ROLE TOO, or "no such block" would hold of no sheet at all.
          expect(without[role].length,).toBeGreaterThan(0,);
          for (const heading of ['REPAIR PANEL CLAIMS ACCEPTED', 'ARCHIVE RENDERING DISPUTED',]) {
            expect(without[role].some(function mentions(sheet,): boolean {
              return sheet.includes(heading,);
            },),).toBe(false,);
          }
        }
      },
    },),
    it({
      name: 'REFUSES a polish that lands on a disputed wording (ledger B29): the consolidation refuses the '
        + 'archive\'s disputed reading as standing, lane offer and proposal, and the polish ran after it '
        + 'without the list',
      fn: async () => {
        /** Text the settlement approved before the polish. */
        const base = 'The tabby spent the whole afternoon napping in the warm window.';
        /** The final settlement over a slice whose disputed wording is exactly what the refiner proposes. */
        const final = await applyFinalPolish({
          client: recordingClient({ gateSheets: [], },),
          settlement: {
            terminal: 'consolidated',
            text: base,
            floor: { kind: 'proposals', validModelIds: [], },
            verdicts: [],
            rewrapped: false,
            demoted: false,
            findings: [],
          },
          subject: {
            sourceText: '虎斑猫整个下午都在温暖的窗边打盹。',
            incumbentText: 'The tabby dozed by the warm window all afternoon long.',
            archiveDisputeNote: DISPUTE_NOTE,
            disputedWordings: [{
              text: 'The tabby napped in the warm window all afternoon.',
              reason: 'the archive rendering the adjudicators disputed',
            },],
          },
          lineStructured: false,
          sliceIndex: 2,
          polishConfig: {
            refinerModelIds: [ROSTER[0],],
            judgeModelIds: ROSTER,
            gateModelIds: ROSTER,
            declaredNames: [],
            definitions: '',
          },
          eligible: true,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l: tagged({ tag: 'polish-disputed-wording-test', },),
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(final.text,).toBe(base,);
      },
    },),
    it({
      name: 'CARRIES the slice\'s cited references to the refiner, the rewrite judges and the polish gate (class '
        + 'forty-one), and to no naturalness review, whose sheet judges wording and leaves fidelity to the gate',
      fn: async () => {
        /** What the cited page says, marked so a sheet carrying it is plain. */
        const references = 'CITED PAGE SAYS: the tabby naps in the window every afternoon.';
        /** Every sheet the polish sent, by reply schema. */
        const sheetsBySchema = new Map<string, readonly string[]>();
        await applyFinalPolish({
          client: recordingClient({ gateSheets: [], sheetsBySchema, },),
          settlement: {
            terminal: 'consolidated',
            text: 'The tabby spent the whole afternoon napping in the warm window.',
            floor: { kind: 'proposals', validModelIds: [], },
            verdicts: [],
            rewrapped: false,
            demoted: false,
            findings: [],
          },
          subject: {
            sourceText: '虎斑猫整个下午都在温暖的窗边打盹。',
            incumbentText: 'The tabby dozed by the warm window all afternoon long.',
            referenceContext: references,
          },
          lineStructured: false,
          sliceIndex: 2,
          polishConfig: {
            refinerModelIds: [ROSTER[0],],
            judgeModelIds: ROSTER,
            gateModelIds: ROSTER,
            declaredNames: [],
            definitions: '',
          },
          eligible: true,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l: tagged({ tag: 'polish-references-test', },),
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        /**
         For each role, how many sheets it was sent and how many carried the
         references.
         */
        const carried = [
          'refine_report',
          'candidate_ballot',
          'consolidation_polish_gate',
          'absolute_naturalness_review',
        ].map(function countFor(schema,): readonly [number, number,] {
          /** Sheets this role was sent. */
          const sheets = sheetsBySchema.get(schema,) ?? [];
          return [
            sheets.length,
            sheets.filter(function carries(sheet,): boolean {
              return sheet.includes(references,);
            },).length,
          ];
        },);
        expect(carried.map(function isAsExpected([sent, carrying,],): string {
          return (sent === 0) ? 'never asked' : ((carrying === sent) ? 'all' : ((carrying === 0) ? 'none' : 'some'));
        },),).toEqual([ 'all', 'all', 'all', 'none', ],);
      },
    },),
    it({
      name: 'REFUSES a polish that drops the declared form from a linked title when the configuration carries the '
        + 'page\'s declared names (class one hundred fourteen), before any gate; the same polish reaches the gate '
        + 'when the configuration carries none',
      fn: async () => {
        /** The approved text, whose linked title carries the declared form. */
        const base = 'The cat said: [Good morning. Eat all your breakfast today, Mittens.]'
          + '(https://example.invalid/breakfast-mimi.html)';
        /**
         Polish of one configuration, with the gate sheets it sent.

         @param declaredNamePairs - page's declared names, none left out

         @returns Final text beside the gate sheets
         */
        async function polishWith(
          { declaredNamePairs, }: {
            readonly declaredNamePairs?: readonly { readonly source: string; readonly rendering: string; }[];
          },
        ): Promise<{ readonly text: string; readonly gateSheets: readonly string[]; }> {
          /** Every polish gate sheet this run sent. */
          const gateSheets: string[] = [];
          /** Settlement after the polish. */
          const final = await applyFinalPolish({
            client: recordingClient({
              gateSheets,
              rewrite: 'The cat said: [Morning! Eat up your breakfast today, Whiskers.]'
                + '(https://example.invalid/breakfast-mimi.html)',
            },),
            settlement: {
              terminal: 'consolidated',
              text: base,
              floor: { kind: 'proposals', validModelIds: [], },
              verdicts: [],
              rewrapped: false,
              demoted: false,
              findings: [],
            },
            subject: {
              sourceText: '猫咪说：[早安。今天也要乖乖吃饭哦，咪咪。](https://example.invalid/breakfast-mimi.html)',
              incumbentText: base,
            },
            lineStructured: false,
            sliceIndex: 3,
            polishConfig: {
              refinerModelIds: [ROSTER[0],],
              judgeModelIds: ROSTER,
              gateModelIds: ROSTER,
              declaredNames: [],
              definitions: '',
              ...((declaredNamePairs === undefined) ? {} : { declaredNamePairs, }),
            },
            eligible: true,
            signal: AbortSignal.timeout(HANG_STOP_MS,),
            perCallTimeoutMs: HANG_STOP_MS,
            l: tagged({ tag: 'polish-declared-pairs-test', },),
            stageSettings: SCRIPTED_STAGE_SETTINGS,
          },);
          return {
            text: final.text,
            gateSheets,
          };
        }
        /** Without the page's declared names. */
        const undeclared = await polishWith({},);
        /** With them. */
        const declared = await polishWith({ declaredNamePairs: [{ source: '咪咪', rendering: 'Mittens', },], },);
        expect([
          [undeclared.text === base, undeclared.gateSheets.length > 0,],
          [declared.text === base, declared.gateSheets.length > 0,],
        ],).toEqual([
          [false, true,],
          [true, false,],
        ],);
      },
    },),
  ],
},);
