/**
 Test-only persisted runs of the settled rendering audit: the rows the
 probe writes, in the shape the probe store keeps them, written under a
 throwaway runs directory. Moved out of
 `rendering-audit-settled-report.unit.test.ts` when the report's driver
 needed the same runs.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { digestAuditedText, } from '../../dist/final/node/index.mjs';

//region Settled run files
// Rows and the runs that hold them.

/**
 Directory the probe store collects this probe's runs in.
 */
export const PROBE_NAME = 'rendering-audit-settled';

/**
 Roster every fixture run records.
 */
export const ROSTER: readonly string[] = [
  'hf:cat/Tabby-1',
  'hf:cat/Mouser-1',
];

/**
 Archive every fixture run says it read, a label only.
 */
export const ARCHIVE = '/nowhere/naptime-archive';

/**
 Characters in a SHA-1 object id.
 */
const OBJECT_ID_LENGTH = 40;

/**
 One pair of texts, used wherever two rows are meant to match.
 */
export const SAME_TEXTS = {
  sourceText: '毛毛跳上窗台。',
  candidateText: 'Mittens jumped onto the windowsill.',
  referenceContext: '',
} as const;

/**
 A different rendering of the same original.
 */
export const OTHER_TEXTS = {
  sourceText: '毛毛跳上窗台。',
  candidateText: 'Mittens hopped up on the sill.',
  referenceContext: '',
} as const;

/**
 Builds one audited slice as the probe persists it.

 @param sliceIndex - slice index

 @param texts - what the audit was shown, omitted to leave it unrecorded

 @returns Row shaped as the probe persists it

 @example
 ```ts
 const row = rowFor({ sliceIndex: 0, texts: SAME_TEXTS, },);
 ```
 */
export function rowFor(
  {
    sliceIndex,
    texts,
  }: {
    readonly sliceIndex: number;
    readonly texts?: {
      readonly sourceText: string;
      readonly candidateText: string;
      readonly referenceContext: string;
    };
  },
): Readonly<Record<string, unknown>> {
  return {
    runSet: 'naptime-20260825',
    entryId: 'mittens',
    sliceIndex,
    deliveryKind: 'replacement-shipped',
    auditsArchiveText: false,
    pageRelation: { kind: 'survives', },
    artifactDigest: 'sha256-tree-v1:cafef00d',
    corpusSha: 'b'.repeat(OBJECT_ID_LENGTH,),
    identityKind: 'none',
    ...((texts === undefined) ? {} : { textIdentity: digestAuditedText(texts,), }),
    report: {
      corroborated: [],
      agreed: [],
      near: [],
      findings: [],
      rows: [{
        modelId: ROSTER[0],
        verdict: 'no-defect-found',
        findings: [],
        dropped: [],
      },],
    },
  };
}

/**
 Writes one run file in the shape the probe store writes, under a runs
 directory of the caller's choosing.

 @param runsDir - throwaway runs directory

 @param stamp - filename-safe instant the run started at

 @param body - top-level fields, which a case may leave incomplete on purpose

 @returns Path written

 @example
 ```ts
 const path = await writeRun({ runsDir, stamp: '2026-08-25T01-00-00.000Z', body: { rows: [], }, },);
 ```
 */
export async function writeRun(
  {
    runsDir,
    stamp,
    body,
  }: {
    readonly runsDir: string;
    readonly stamp: string;
    readonly body: Readonly<Record<string, unknown>>;
  },
): Promise<string> {
  /**
   Where runs of this probe collect.
   */
  const probeDir = join(
    runsDir,
    PROBE_NAME,
  );
  await mkdir(
    probeDir,
    { recursive: true, },
  );

  /**
   Run file, named the way the store names one.
   */
  const path = join(
    probeDir,
    `${stamp}-cafef00d.json`,
  );
  await writeFile(
    path,
    JSON.stringify(
      body,
      undefined,
      2,
    ),
    'utf8',
  );
  return path;
}

/**
 A complete run over the given rows.

 @param rows - rows it bought

 @param roster - roster it recorded, absent to write a run from before the
 field was kept

 @returns Top-level fields

 @example
 ```ts
 const body = runOver({ rows: [rowFor({ sliceIndex: 0, },),], roster: ROSTER, },);
 ```
 */
export function runOver(
  {
    rows,
    roster,
  }: {
    readonly rows: readonly unknown[];
    readonly roster?: readonly string[];
  },
): Readonly<Record<string, unknown>> {
  return {
    startedAt: '2026-08-25T01:00:00.000Z',
    finishedAt: '2026-08-25T01:10:00.000Z',
    pipelineDigest: 'sha256-tree-v1:cafef00d',
    ...((roster === undefined) ? {} : { roster, }),
    subject: { archiveDir: ARCHIVE, },
    rows,
  };
}

//endregion Settled run files
