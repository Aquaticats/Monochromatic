/**
 Tests that the settlement line a repair run logs says which kind of
 "unchanged" a slice settled on, read through the settlement that writes it
 (ledger L12).

 WHY THROUGH THE SETTLEMENT. `repair-chunk-settlement-line.unit.test.ts` pins
 the sentence for each state, and nothing there reads whether the settlement
 hands the line the verdict it reached: a settlement passing a constant would
 log "the archive won" for every lost, empty or refused patch, and every line
 would still read as an ordinary run.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type AdjudicatedIssue,
  parseDocument,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { settleShippedPatch, } from '../dist/final/node/index.mjs';

/**
 Slice the settlement reports.
 */
const SLICE_INDEX = 5;

/**
 Name and alias the archive declares.
 */
const DECLARED_NAMES: readonly string[] = [
  'Meowmeow',
  'Dumpling',
];

/**
 Archive wording, carrying both declared forms.
 */
const ARCHIVE = 'Meowmeow, who everyone called Dumpling, kept the windowsill warm through that whole winter.';

/**
 Repair keeping both forms.
 */
const KEEPS_EVERY_NAME = 'Meowmeow, who everyone called Dumpling, kept the windowsill warm all that winter.';

/**
 Repair dropping the alias.
 */
const DROPS_THE_ALIAS = 'Meowmeow kept the windowsill warm all that winter.';

/**
 Accepted issue the patch was written against.
 */
const ISSUE: AdjudicatedIssue = {
  issueId: 'issue/winter',
  status: 'accepted',
  severity: 'major',
  claims: [],
  tallies: {},
};

/**
 The settlement line for one shipped patch.

 @param patchedText - wording the shipped patch produced

 @param resolved - whether the checkers confirmed the issue fixed

 @returns Info lines the settlement wrote

 @example
 ```ts
 const lines = settlementLines({ patchedText: ARCHIVE, resolved: true, },);
 ```
 */
function settlementLines(
  {
    patchedText,
    resolved,
  }: {
    readonly patchedText: string;
    readonly resolved: boolean;
  },
): readonly string[] {
  /**
   Lines the settlement wrote.
   */
  const said: string[] = [];
  /**
   Keeps one line.

   @param message - line to keep
   */
  const keep = (message: string,): void => {
    said.push(message,);
  };
  /**
   Logger keeping every line.
   */
  const l: Logger = {
    debug: keep,
    error: keep,
    fatal: keep,
    flush: async () => undefined,
    info: keep,
    trace: keep,
    warn: keep,
  };
  settleShippedPatch({
    sliceIndex: SLICE_INDEX,
    declaredNames: DECLARED_NAMES,
    targetText: ARCHIVE,
    shipped: { patch: { patchedText, applied: [], }, } as unknown as Parameters<
      typeof settleShippedPatch
      >[0]['shipped'],
    appliedEnvelopes: { creditableIssues: [ISSUE,], } as unknown as Parameters<
      typeof settleShippedPatch
      >[0]['appliedEnvelopes'],
    tallies: {
      [ISSUE.issueId]: resolved
        ? { fixed: 2, notFixed: 0, worse: 0, resolved: true, regressed: false, }
        : { fixed: 0, notFixed: 0, worse: 2, resolved: false, regressed: true, },
    },
    envelopes: [],
    targetDocument: parseDocument({ text: ARCHIVE, },),
    acceptedCount: 1,
    unenvelopedCount: 0,
    l,
  },);
  return said;
}

/**
 Whether some line reports this slice in the given state.

 @param lines - lines the settlement wrote

 @param state - state phrase the line should open with

 @returns Whether a settlement line says it

 @example
 ```ts
 const says = reports({ lines, state: 'unchanged, the archive won', },);
 ```
 */
function reports(
  {
    lines,
    state,
  }: {
    readonly lines: readonly string[];
    readonly state: string;
  },
): boolean {
  return lines.some(function isThisLine(line,): boolean {
    return line.startsWith(`chunk ${String(SLICE_INDEX,)}: ${state}, `,);
  },);
}

await describe({
  name: 'the settlement hands its line the verdict it reached (ledger L12)',
  children: [
    it({
      name: 'a patch that WON AND DROPPED A DECLARED NAME is logged as refused',
      fn: async () => {
        expect(reports({
          lines: settlementLines({ patchedText: DROPS_THE_ALIAS, resolved: true, },),
          state: 'unchanged, the patch won and was refused for dropping a declared name',
        },),).toBe(true,);
      },
    },),
    it({
      name: 'a patch that WON AND WROTE NOTHING is logged as such',
      fn: async () => {
        expect(reports({
          lines: settlementLines({ patchedText: ARCHIVE, resolved: true, },),
          state: 'unchanged, the patch won and wrote nothing',
        },),).toBe(true,);
      },
    },),
    it({
      name: 'a patch that LOST is logged as the archive winning',
      fn: async () => {
        expect(reports({
          lines: settlementLines({ patchedText: KEEPS_EVERY_NAME, resolved: false, },),
          state: 'unchanged, the archive won (the patch lost on regressions: it made an accepted issue worse)',
        },),).toBe(true,);
      },
    },),
    it({
      name: 'a patch that WON AND CHANGED THE TEXT is logged as repaired',
      fn: async () => {
        expect(reports({
          lines: settlementLines({ patchedText: KEEPS_EVERY_NAME, resolved: true, },),
          state: 'repaired',
        },),).toBe(true,);
      },
    },),
  ],
},);
