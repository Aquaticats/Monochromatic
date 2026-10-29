/**
 Runs the page-level consistency pass over the cat page with scripted judges,
 for the cases that read what the pass recorded, asked and showed.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source';
import type { BudgetView, } from '@monochromatic-dev/translation-repair-provider';
import {
  assertPipelineDigest,
  type PipelineDigest,
} from '@monochromatic-dev/translation-repair-glossary';
import type { ArtifactConsistency, } from '@monochromatic-dev/translation-repair-artifact';
import { runPassConsistency, } from '../../dist/final/node/index.mjs';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';
import {
  archiveEnglishOf,
  otherSheetBehaviour,
  preparedCatPage,
  projectedCatPage,
  scriptedJudgeClient,
  type SeatBehaviour,
  type SeatScript,
} from '@monochromatic-dev/translation-repair-run/ts/corpus-run/consistency-page.test-fixture.ts';
import { sheetKindOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/consistency-sheet-reading.test-fixture.ts';

//region Consistency run fixture

/**
 Logger the pass writes to, whose lines are not under test.
 */
const l = tagged({ tag: 'pass-consistency-test', },);

/**
 Hex digits in the digest every case's cache is stamped with.
 */
const DIGEST_HEX_LENGTH = 64;

/**
 Digest the cache is stamped with, before it is checked for shape.
 */
const DIGEST_TEXT = `sha256-tree-v1:${'c'.repeat(DIGEST_HEX_LENGTH,)}`;
assertPipelineDigest(DIGEST_TEXT,);

/**
 Generation stamp the cache is opened under.
 */
export const DIGEST: PipelineDigest = DIGEST_TEXT;

/**
 What every seat does with a sheet that is not a first-round ballot unless a case says otherwise: the writers
 write nothing, a choice among candidates takes the first candidate that is not the paragraph as the page has it
 now (the chosen words moved in by code), and a run-off takes the first finalist.

 @param call - call the seat was handed, of which only the sheet is read

 @returns Behaviour of the seat

 @example
 ```ts
 const behaviour = otherSheets({ seat, before: 0, sheet, },);
 ```
 */
function otherSheets(call: Parameters<SeatScript>[0],): SeatBehaviour {
  return otherSheetBehaviour({ sheet: call.sheet, },);
}

/**
 What a run of the pass came to.
 */
export type CatRun = {
  /**
   The section the pass recorded.
   */
  readonly consistency: ArtifactConsistency;

  /**
   Seat of every structured call, in order.
   */
  readonly asked: readonly RosterModelId[];

  /**
   Every sheet a judge was shown, in order.
   */
  readonly sheets: readonly string[];
};

/**
 Runs the pass over the cat page with scripted judges.

 @param behave - what each seat does when asked

 @param entryCacheDir - directory the cache is opened in

 @param prepared - preparation of the page, the cat page by default

 @param english - what each slice ships, the archive by default

 @param signal - entry's stop, bound that outlasts the work by default

 @param dark - seats no provider serves, none by default

 @param dryness - which providers read dry to the seat reading, every provider wet by default

 @param lines - sink for the level and text of every line the pass logs, which makes the pass log there instead of to the console

 @returns Section, seats asked and sheets shown

 @example
 ```ts
 const run = await runCatPass({ behave: firstPlaceWins, entryCacheDir, },);
 ```
 */
export async function runCatPass(
  {
    behave,
    others = otherSheets,
    entryCacheDir,
    prepared = preparedCatPage(),
    english = archiveEnglishOf({ prepared, },),
    signal = AbortSignal.timeout(HANG_STOP_MS,),
    dark = new Set<RosterModelId>(),
    dryness,
    lines,
  }: {
    readonly lines?: string[];
    readonly behave: SeatScript;
    readonly others?: SeatScript;
    readonly entryCacheDir: string;
    readonly prepared?: PreparedDocumentPair;
    readonly english?: readonly string[];
    readonly signal?: AbortSignal;
    readonly dark?: ReadonlySet<RosterModelId>;
    readonly dryness?: BudgetView;
  },
): Promise<CatRun> {
  /**
   Seat of every structured call.
   */
  const asked: RosterModelId[] = [];
  /**
   Sheet of every structured call.
   */
  const sheets: string[] = [];
  return {
    consistency: await runPassConsistency({
      client: scriptedJudgeClient({
        behave: function byQuestion(call,): SeatBehaviour {
          return (sheetKindOf({ sheet: call.sheet, },) === 'ballot') ? behave(call,) : others(call,);
        },
        asked,
        sheets,
        dark,
        ...((dryness === undefined) ? {} : { dryness, }),
      },),
      prepared,
      projected: projectedCatPage({
        prepared,
        english,
      },),
      contestSlices: [],
      consolidateSlices: [],
      archiveDisputes: new Map(),
      entryCacheDir,
      pipelineDigest: DIGEST,
      signal,
      overlap: 2,
      l: (lines === undefined) ? l : levelCapturingLogger({ lines, },),
      stageSettings: SCRIPTED_STAGE_SETTINGS,
    },),
    asked,
    sheets,
  };
}

/**
 The kinds of the candidates a carry was offered, sorted so the order of the sheet does not matter.

 @param candidates - candidates the carry recorded

 @returns Kinds, in sorted order

 @example
 ```ts
 const kinds = candidateKindsOf({ candidates: [{ kind: 'splice', }, { kind: 'current', },], },); // ['current', 'splice']
 ```
 */
export function candidateKindsOf(
  { candidates, }: { readonly candidates: readonly { readonly kind: string; }[]; },
): readonly string[] {
  return candidates
    .map(function kindOf({ kind, },): string {
      return kind;
    },)
    .toSorted();
}

//endregion Consistency run fixture
