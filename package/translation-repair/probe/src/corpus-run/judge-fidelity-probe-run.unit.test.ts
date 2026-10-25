/**
 Tests for the fidelity probe's run over scripted references, judges and clock:
 what it refuses before anything is read, what a zero cap says, what a bounded
 and a complete run write to disk and print, and in which order.

 No model is called and no corpus clone is read: the reference reader and the
 client builder are handed in, so every ballot is a scripted reply. The real
 reviewed manifest supplies the request's specifications, and the invented
 cat texts of the fixture module stand in for what the pinned clone would hold.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { existsSync, } from 'node:fs';
import {
  readdir,
  readFile,
  stat,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';

import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  type DamageAttempt,
  deleteOneSentence,
} from '@monochromatic-dev/translation-repair-repair';
import {
  FidelityReferenceError,
  type FidelityReferenceSpec,
  REVIEWED_FIDELITY_REFERENCES,
  type ReviewedFidelityReference,
  runFidelityProbe,
} from '../../dist/final/node/index.mjs';
import { capturingLoggerPair, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  expectRefusal,
  refusalMessage,
  REVIEW_REFERENCE,
  REVIEW_SOURCE,
} from '../fidelity-reference.test-fixture.ts';
import { sortedByModelId, } from '@monochromatic-dev/translation-repair-markup/ts/model-order.test-fixture.ts';
import {
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { replyWith, } from '@monochromatic-dev/translation-repair-provider/ts/scripted-reply-outcome.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Both judges the cases run alone, written the way the flag takes them.
 */
const TWO_JUDGES = `${SEAT_HYPER_OPENROUTER_UNMEASURED},${SEAT_BEDROCK_ONLY_TEXT}`;

/**
 Digest every run claims, ending in the eight characters a kept file's name carries.
 */
const DIGEST = `sha256-tree-v1:${'c'.repeat(56,)}cafef00d`;

/**
 Closure every run records.
 */
const CLOSURE = {
  kind: 'read',
  entry: 'judge-fidelity-probe.mjs',
  chunks: ['run-config-abc.mjs',],
} as const;

/**
 Reviewed specification the bounded cases select, the first the manifest holds.
 */
const SPEC: FidelityReferenceSpec = nonNullishOrThrow(REVIEWED_FIDELITY_REFERENCES[0],);

/**
 Instant the first clock reading gives.
 */
const STARTED_AT = '2026-10-06T06:00:00.000Z';

/**
 Instant the second clock reading gives.
 */
const FINISHED_AT = '2026-10-06T06:05:00.000Z';

/**
 The one deletion every invented reference carries, which the rows' counts come from.
 */
const DELETION: Extract<DamageAttempt, { readonly kind: 'damaged'; }> = (function builtDeletion() {
  /**
   What the builder made of the invented reference.
   */
  const built = deleteOneSentence({ cleanText: REVIEW_REFERENCE, },);
  if (built.kind !== 'damaged')
    throw new Error('the invented reference must admit a deletion',);
  return built;
})();

/**
 What one run did, for the case to read.
 */
type ProbeRunSeen = {
  /**
   Lines the probe logged itself, one per entry; the judges' own progress lines carry timings and are left out.
   */
  readonly lines: readonly string[];

  /**
   Everything written to the output sink.
   */
  readonly written: string;

  /**
   Model ids the scripted client was asked, in order.
   */
  readonly asked: readonly string[];

  /**
   Specifications the reference reader was handed, one entry per call.
   */
  readonly readFor: readonly (readonly FidelityReferenceSpec[])[];

  /**
   What each client build saw: the payload directory it was given and whether the plan was on disk by then.
   */
  readonly clientBuilds: readonly {
    readonly promptPayloadDir: string;
    readonly planWasOnDisk: boolean;
  }[];

  /**
   Runs directory requests, one entry per call.
   */
  readonly runsAsked: readonly string[];

  /**
   What the run threw, empty when it completed.
   */
  readonly failures: readonly unknown[];

  /**
   For each clock reading, in order, how many reference reads the invented
   reader had been handed by then.
   */
  readonly readsBeforeClock: readonly number[];
};

/**
 Reference reader answering with an invented reference for every specification it is handed.

 @param readFor - array each call's specifications are appended to

 @returns The reader the run takes

 @example
 ```ts
 const readReferences = inventedReader({ readFor: [], },);
 ```
 */
function inventedReader(
  { readFor, }: { readonly readFor: (readonly FidelityReferenceSpec[])[]; },
): Parameters<typeof runFidelityProbe>[0]['readReferences'] {
  return function readInvented({ specs, },): Promise<readonly ReviewedFidelityReference[]> {
    readFor.push(specs,);
    return Promise.resolve(specs.map(function referenceOf(spec,): ReviewedFidelityReference {
      return {
        spec,
        sourceText: REVIEW_SOURCE,
        referenceText: REVIEW_REFERENCE,
        damages: [DELETION,],
        pages: {
          sourceFile: REVIEW_SOURCE,
          archiveFile: REVIEW_REFERENCE,
        },
      };
    },),);
  };
}

/**
 Client whose every judge backs the first ballot position.

 @param asked - array each call's model id is appended to

 @returns The client the run asks through

 @example
 ```ts
 const client = firstPositionClient({ asked: [], },);
 ```
 */
function firstPositionClient({ asked, }: { readonly asked: string[]; },): ModelClient {
  return {
    chatText: () => Promise.reject(new Error('chatText unused by selection',),),
    quotas: () => Promise.reject(new Error('quotas unused by selection',),),
    chatJson: (request,) => {
      asked.push(request.modelId,);
      return Promise.resolve(replyWith({
        report: {
          best: 1,
          assessment: 'purrs',
        },
        request,
      },),);
    },
  };
}

/**
 Runs the probe once over scripted parts, keeping its files in the directory given.

 @param runsDir - runs directory the run keeps its files in

 @param typed - what the operator wrote after the script path

 @param read - reference reader, the invented one unless a case brings its own

 @returns What the run did, once it settles, including what it threw

 @example
 ```ts
 const seen = await probeIn({ runsDir: scratch.path, typed: ['--cap', '0',], },);
 ```
 */
async function probeIn(
  {
    runsDir,
    typed,
    read,
  }: {
    readonly runsDir: string;
    readonly typed: readonly string[];
    readonly read?: Parameters<typeof runFidelityProbe>[0]['readReferences'];
  },
): Promise<ProbeRunSeen> {
  /**
   Lines the run logged.
   */
  const logged = capturingLoggerPair();
  /**
   Model ids asked.
   */
  const asked: string[] = [];
  /**
   Specifications the reader was handed.
   */
  const readFor: (readonly FidelityReferenceSpec[])[] = [];
  /**
   Client builds seen.
   */
  const clientBuilds: {
    readonly promptPayloadDir: string;
    readonly planWasOnDisk: boolean;
  }[] = [];
  /**
   Text written to the sink.
   */
  const chunks: string[] = [];
  /**
   Runs directory requests.
   */
  const runsAsked: string[] = [];
  /**
   What the run threw.
   */
  const failures: unknown[] = [];
  /**
   Reference reads seen at each clock reading.
   */
  const readsBeforeClock: number[] = [];
  /**
   Clock readings still to give.
   */
  const instants = [
    STARTED_AT,
    FINISHED_AT,
  ];
  try {
    await runFidelityProbe({
      line: lineOf({
        command: 'judge-fidelity-probe',
        typed,
      },),
      pin: {
        cloneDir: join(
          runsDir,
          'no-clone-is-read',
        ),
        commitSha: SPEC.corpusSha,
      },
      readReferences: read ?? inventedReader({ readFor, },),
      resolveRuns: function resolveScratch(): Promise<string> {
        runsAsked.push(runsDir,);
        return Promise.resolve(runsDir,);
      },
      newClient: function buildClient({ promptPayloadDir, },): ModelClient {
        clientBuilds.push({
          promptPayloadDir,
          planWasOnDisk: existsSync(join(
            dirname(promptPayloadDir,),
            'plan.json',
          ),),
        },);
        return firstPositionClient({ asked, },);
      },
      now: function nextInstant(): string {
        readsBeforeClock.push(readFor.length,);
        return nonNullishOrThrow(instants.shift(),);
      },
      pipelineDigest: DIGEST,
      runnerClosure: CLOSURE,
      perCallTimeoutMs: HANG_STOP_MS,
      log: logged.logger,
      out: {
        write: function keep(text: string,): void {
          chunks.push(text,);
        },
      },
      stageSettings: SCRIPTED_STAGE_SETTINGS,
    },);
  }
  catch (error) {
    failures.push(error,);
  }
  return {
    lines: logged.lines.filter(function isOwn(line,): boolean {
      return !line.startsWith('[runFidelityTrial]',);
    },),
    written: chunks.join('',),
    asked,
    readFor,
    clientBuilds,
    runsAsked,
    failures,
    readsBeforeClock,
  };
}

/**
 Judges in the order the flag names them, which is the roster's order.
 */
const JUDGES = [
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_BEDROCK_ONLY_TEXT,
];

/**
 Flags every run case that asks something starts from: one reviewed entry, one defect family, two judges alone.
 */
const ASKING_TWO_JUDGES = [
  '--only',
  SPEC.entryId,
  '--damage',
  'deletion',
  '--candidates',
  TWO_JUDGES,
  '--candidates-alone',
];

/**
 What a row holds once both scripted judges back the first ballot position.

 @param direction - which side holds the reference

 @param cleanFirst - whether the reference is listed first

 @param clean - whether the first position held the reference

 @returns The row the run keeps and prints

 @example
 ```ts
 const row = rowOf({ direction: 'preserve', cleanFirst: true, clean: true, },);
 ```
 */
function rowOf(
  {
    direction,
    cleanFirst,
    clean,
  }: {
    readonly direction: 'preserve' | 'replace';
    readonly cleanFirst: boolean;
    readonly clean: boolean;
  },
): Record<string, unknown> {
  return {
    referenceId: SPEC.id,
    entryId: SPEC.entryId,
    sourceRange: SPEC.source,
    archiveRange: SPEC.archive,
    referenceHash: SPEC.referenceHash,
    changedChars: DELETION.changedChars,
    damageDetail: DELETION.damageDetail,
    trialId: `${SPEC.id}/deletion`,
    direction,
    damageKind: 'deletion',
    cleanFirst,
    verdict: clean ? 'clean' : 'damaged',
    correct: clean,
    ballots: [
      SEAT_BEDROCK_ONLY_TEXT,
      SEAT_HYPER_OPENROUTER_UNMEASURED,
    ].map(function ballotOf(modelId,): Record<string, unknown> {
      return {
        modelId,
        picked: clean ? 'clean' : 'damaged',
        reason: 'purrs',
        weight: 1,
      };
    },),
    declineReason: '',
  };
}

/**
 The four rows of one reference and one defect when every judge backs the first position, in matrix order.
 */
const FOUR_ROWS = [
  rowOf({
    direction: 'preserve',
    cleanFirst: true,
    clean: true,
  },),
  rowOf({
    direction: 'preserve',
    cleanFirst: false,
    clean: false,
  },),
  rowOf({
    direction: 'replace',
    cleanFirst: true,
    clean: true,
  },),
  rowOf({
    direction: 'replace',
    cleanFirst: false,
    clean: false,
  },),
];

/**
 A row, or a document of rows, read back from JSON: only the parts the ballots sort touches are typed.
 */
type SortableDocument = {
  readonly ballots?: readonly { readonly modelId: string; }[];
  readonly rows?: readonly SortableDocument[];
};

/**
 A row with its ballots sorted by model id.

 @param row - the row, or any object that may hold ballots

 @returns The row with its ballots sorted, the row itself where it holds none

 @example
 ```ts
 const sorted = rowWithSortedBallots({ row: { ballots: [], }, },);
 ```
 */
function rowWithSortedBallots({ row, }: { readonly row: SortableDocument; },): SortableDocument {
  return (row.ballots === undefined)
    ? row
    : {
      ...row,
      ballots: sortedByModelId({ items: row.ballots, },),
    };
}

/**
 JSON text as the writer of the run formats it: the parsed text written again with an indent of two spaces, and a newline after it
 where the writer ends its file with one. Compared with the text on disk, it fails when the writer drops the indent or the final
 newline, which a compare that parses both sides cannot see.

 @param text - JSON text as the run wrote it
 @param newline - whether the writer ends the file with a newline

 @returns The text in the writer's format

 @example
 ```ts
 const formatted = formatterOutputOf({ text: '{"a":1}', newline: true, },); // '{\n  "a": 1\n}\n'
 ```
 */
function formatterOutputOf({ text, newline, }: { readonly text: string; readonly newline: boolean; },): string {
  return `${JSON.stringify(JSON.parse(text,), undefined, 2,)}${newline ? '\n' : ''}`;
}

/**
 A row, or a document of rows, as JSON text with every row's ballots sorted by judge. The ballots are kept in the order the
 prompt-rotated bench answered, which moved when the house tense rule changed the prompt, so both the text the run kept and the
 text the test expects are read in one order.

 @param text - JSON of one row or of an object that holds rows

 @returns The same text with each row's ballots sorted by model id, indented by two spaces

 @example
 ```ts
 const sorted = ballotsSorted({ text: JSON.stringify({ rows: [], }), });
 ```
 */
function ballotsSorted({ text, }: { readonly text: string; },): string {
  /**
   The text read back.
   */
  const parsed = JSON.parse(text,) as SortableDocument;
  return JSON.stringify(
    (parsed.rows === undefined)
      ? rowWithSortedBallots({ row: parsed, },)
      : {
        ...parsed,
        rows: parsed.rows.map(function sortedRow(row,): SortableDocument {
          return rowWithSortedBallots({ row, },);
        },),
      },
    undefined,
    2,
  );
}

/**
 Plan entry of one row, from its position in the matrix.

 @param position - where the row sits

 @returns The entry the plan lists

 @example
 ```ts
 const entry = plannedOf({ position: 0, },);
 ```
 */
function plannedOf({ position, }: { readonly position: number; },): Record<string, unknown> {
  /**
   The row at that position, whose arrangement the entry repeats.
   */
  const row = nonNullishOrThrow(FOUR_ROWS[position],);
  return {
    position,
    trialId: row.trialId,
    direction: row.direction,
    cleanFirst: row.cleanFirst,
    damageKind: 'deletion',
  };
}

/**
 Plan a run of the reviewed entry writes, for as many rows as it attempts.

 @param rows - how many leading rows the run attempts, of four

 @returns The plan document

 @example
 ```ts
 const plan = planOf({ rows: 4, },);
 ```
 */
function planOf({ rows, }: { readonly rows: number; },): Record<string, unknown> {
  return {
    status: 'planned',
    startedAt: STARTED_AT,
    pipelineDigest: DIGEST,
    runnerClosure: CLOSURE,
    referenceManifestDigest: hashContent({ content: JSON.stringify([SPEC,],), },),
    referenceManifest: [SPEC,],
    requestedRoster: JUDGES,
    completeRequestedMatrix: rows === 4,
    fullMatrixRows: 4,
    plannedRows: rows,
    rows: [
      0,
      1,
      2,
      3,
    ].slice(
      0,
      rows,
    )
      .map(function entryAt(position,): Record<string, unknown> {
        return plannedOf({ position, },);
      },),
  };
}

/**
 Finds the one run directory the run made in a runs directory.

 @param runsDir - runs directory the run kept its files in

 @returns Its path

 @example
 ```ts
 const runDir = await runDirectoryIn({ runsDir: scratch.path, },);
 ```
 */
async function runDirectoryIn({ runsDir, }: { readonly runsDir: string; },): Promise<string> {
  /**
   Names of the run directories made.
   */
  const names = (await readdir(runsDir,)).filter(function isRun(name,): boolean {
    return name.startsWith('judge-fidelity-',);
  },);
  expect(names.length,).toBe(1,);
  return join(
    runsDir,
    nonNullishOrThrow(names[0],),
  );
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: `${runFidelityProbe.name} refusals`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES UNREVIEWED CONTEXT AS STATED, the reviewed-reference refusal kept as its cause, before it '
            + 'logs, reads, asks for a directory or builds a client',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'judge-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: [
                ...ASKING_TWO_JUDGES,
                '--context',
              ],
            },);
            expect(seen.failures.length,).toBe(1,);
            /**
             What the run threw.
             */
            const [refusal,] = seen.failures;
            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(`StatedRefusalError: ${
              refusalMessage({
                referenceId: 'unreviewed context',
                operation: 'request',
              },)
            }`,);
            expectRefusal({
              refusal: Error.isError(refusal,) ? refusal.cause : refusal,
              referenceId: 'unreviewed context',
              operation: 'request',
            },);
            expect(seen.lines,).toEqual([],);
            expect(seen.readFor,).toEqual([],);
            expect(seen.runsAsked,).toEqual([],);
            expect(seen.clientBuilds,).toEqual([],);
            expect(await readdir(scratch.path,),).toEqual([],);
          },
        },),
        it({
          name: 'REFUSES CANDIDATES RUN ALONE WHEN NONE WAS NAMED, in words, before it logs or reads anything',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'judge-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: ['--candidates-alone',],
            },);
            expect(seen.failures.length,).toBe(1,);
            expect(seen.failures[0],).toBeInstanceOf(StatedRefusalError,);
            expect(String(seen.failures[0],),).toBe(
              'StatedRefusalError: --candidates-alone runs the candidates without the seated roster, '
                + 'and --candidates named none',
            );
            expect(seen.lines,).toEqual([],);
            expect(seen.readFor,).toEqual([],);
          },
        },),
        it({
          name: 'STOPS WHEN A REFERENCE CANNOT BE READ, after naming its judges and before any directory, plan or client',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'judge-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: ASKING_TWO_JUDGES,
              read: function unreadable(): Promise<readonly ReviewedFidelityReference[]> {
                return Promise.reject(new FidelityReferenceError({
                  referenceId: SPEC.id,
                  operation: 'source',
                },),);
              },
            },);
            expect(seen.failures.length,).toBe(1,);
            expectRefusal({
              refusal: seen.failures[0],
              referenceId: SPEC.id,
              operation: 'source',
            },);
            expect(seen.lines,).toEqual([`judges: ${JUDGES.join(', ',)}`,],);
            expect(seen.runsAsked,).toEqual([],);
            expect(seen.clientBuilds,).toEqual([],);
            expect(await readdir(scratch.path,),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: `${runFidelityProbe.name} at a zero cap`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SAYS HOW MANY REFERENCES THE WHOLE MANIFEST SELECTS and asks for nothing: no read, no directory, '
            + 'no client, no file, no output',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'judge-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: [
                '--cap',
                '0',
                '--candidates',
                TWO_JUDGES,
                '--candidates-alone',
              ],
            },);
            expect(seen.failures,).toEqual([],);
            expect(seen.lines,).toEqual([
              `judges: ${JUDGES.join(', ',)}`,
              `preflight only: ${
                String(REVIEWED_FIDELITY_REFERENCES.length,)
              } reviewed reference specifications selected; no corpus or model calls`,
            ],);
            expect(seen.written,).toBe('',);
            expect(seen.readFor,).toEqual([],);
            expect(seen.runsAsked,).toEqual([],);
            expect(seen.clientBuilds,).toEqual([],);
            expect(await readdir(scratch.path,),).toEqual([],);
          },
        },),
        it({
          name: 'SAYS ONE REFERENCE IN THE SINGULAR when the request selects one, and still asks for nothing',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'judge-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: [
                ...ASKING_TWO_JUDGES,
                '--cap',
                '0',
              ],
            },);
            expect(seen.failures,).toEqual([],);
            expect(seen.lines,).toEqual([
              `judges: ${JUDGES.join(', ',)}`,
              'preflight only: 1 reviewed reference specification selected; no corpus or model calls',
            ],);
            expect(seen.readFor,).toEqual([],);
            expect(await readdir(scratch.path,),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: `${runFidelityProbe.name} over one reviewed entry`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS A COMPLETE RUN: the plan before the client, a file per row, the kept run, the progress lines '
            + 'and the rows document',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'judge-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: ASKING_TWO_JUDGES,
            },);
            expect(seen.failures,).toEqual([],);
            /**
             The directory the run made for itself.
             */
            const runDir = await runDirectoryIn({ runsDir: scratch.path, },);
            /**
             Where the kept run lands, named for the start and the build's tail.
             */
            const keptAt = join(
              runDir,
              'judge-fidelity-probe',
              '2026-10-06T06-00-00.000Z-cafef00d.json',
            );
            expect(seen.lines,).toEqual([
              `judges: ${JUDGES.join(', ',)}`,
              'fidelity: 4 reviewed trial rows; use individual ballots for per-model calibration',
              `kept 4 reviewed rows at ${keptAt}`,
            ],);
            expect(seen.readFor,).toEqual([[SPEC,],],);
            expect(seen.runsAsked.length,).toBe(1,);
            expect(seen.clientBuilds,).toEqual([
              {
                promptPayloadDir: join(
                  runDir,
                  'payloads',
                ),
                planWasOnDisk: true,
              },
            ],);
            expect(seen.asked.length,).toBe(8,);
            expect(seen.asked.filter(function isFirst(id,): boolean {
              return id === SEAT_HYPER_OPENROUTER_UNMEASURED;
            },).length,).toBe(4,);
            expect(await readFile(
              join(
                runDir,
                'plan.json',
              ),
              'utf8',
            ),).toBe(JSON.stringify(
              planOf({ rows: 4, },),
              undefined,
              2,
            ),);
            /**
             Each row's file text, one per position, written as the row completed.
             */
            const rowTexts = await Promise.all(FOUR_ROWS.map(async function readRowText(_row, position,): Promise<string> {
              return await readFile(
                join(
                  runDir,
                  `row-${String(position,)}.json`,
                ),
                'utf8',
              );
            },),);
            /**
             The text of the kept run.
             */
            const keptText = await readFile(
              keptAt,
              'utf8',
            );
            expect(rowTexts.map(function sortedRow(text,): string {
              return ballotsSorted({ text, },);
            },),).toEqual(FOUR_ROWS.map(function rowText(row,): string {
              return ballotsSorted({ text: JSON.stringify(row,), },);
            },),);
            // THE FORMAT ON DISK, beside the content compare that reads every ballot in model-id order (n166, Z3): each row file and
            // the kept file are the two-space JSON the writer produces, with no newline after it, and the text the run handed to its
            // writer is that JSON with one newline after it.
            expect(rowTexts.map(function formatted(text,): string {
              return formatterOutputOf({ text, newline: false, },);
            },),).toEqual(rowTexts,);
            expect(formatterOutputOf({ text: keptText, newline: false, },),).toBe(keptText,);
            expect(formatterOutputOf({ text: seen.written, newline: true, },),).toBe(seen.written,);
            expect(ballotsSorted({
              text: keptText,
            },),).toBe(ballotsSorted({
              text: JSON.stringify({
                startedAt: STARTED_AT,
                finishedAt: FINISHED_AT,
                pipelineDigest: DIGEST,
                runnerClosure: CLOSURE,
                roster: JUDGES,
                subject: {
                  status: 'completed',
                  completeRequestedMatrix: true,
                  fullMatrixRows: 4,
                  plannedRows: 4,
                  completedRows: 4,
                  referenceManifestDigest: hashContent({ content: JSON.stringify([SPEC,],), },),
                  promptPayloadDir: join(
                    runDir,
                    'payloads',
                  ),
                  corpusPin: SPEC.corpusSha,
                  referenceManifest: [SPEC,],
                  entriesRequested: [SPEC.entryId,],
                  trialCap: 16,
                  damageKinds: ['deletion',],
                  withContext: false,
                },
                rows: FOUR_ROWS,
              },),
            },),);
            expect(ballotsSorted({ text: seen.written, },),).toBe(ballotsSorted({
              text: JSON.stringify({ rows: FOUR_ROWS, },),
            },),);
          },
        },),
        it({
          name: 'WRITES THE PLAN AND EVERY ROW READABLE BY THE OWNER ALONE, mode 600, since a row can quote the source',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'judge-fidelity-run-', },);
            await probeIn({
              runsDir: scratch.path,
              typed: ASKING_TWO_JUDGES,
            },);
            /**
             The directory the run made for itself.
             */
            const runDir = await runDirectoryIn({ runsDir: scratch.path, },);
            expect((await stat(join(
              runDir,
              'plan.json',
            ),)).mode & 0o777,).toBe(0o600,);
            expect((await stat(join(
              runDir,
              'row-3.json',
            ),)).mode & 0o777,).toBe(0o600,);
          },
        },),
        it({
          name: 'READS THE START OF THE RUN BEFORE IT READS ANY REFERENCE and the finish after the last row, so the '
            + 'plan and the kept run date the whole run and not the end of the corpus read',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'judge-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: [
                ...ASKING_TWO_JUDGES,
                '--cap',
                '1',
              ],
            },);
            expect({
              failures: seen.failures,
              readFor: seen.readFor,
              readsBeforeClock: seen.readsBeforeClock,
            },).toEqual({
              failures: [],
              readFor: [[SPEC,],],
              readsBeforeClock: [0, 1,],
            },);
          },
        },),
        it({
          name: 'WARNS THAT A BOUNDED RUN IS NOT THE COMPLETE MATRIX, plans one row and says one row in the singular',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'judge-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: [
                ...ASKING_TWO_JUDGES,
                '--cap',
                '1',
              ],
            },);
            expect(seen.failures,).toEqual([],);
            /**
             The directory the run made for itself.
             */
            const runDir = await runDirectoryIn({ runsDir: scratch.path, },);
            /**
             Where the kept run lands.
             */
            const keptAt = join(
              runDir,
              'judge-fidelity-probe',
              '2026-10-06T06-00-00.000Z-cafef00d.json',
            );
            expect(seen.lines,).toEqual([
              `judges: ${JUDGES.join(', ',)}`,
              'partial reviewed matrix: 1 of 4 rows; not complete admission evidence',
              'fidelity: 1 reviewed trial row; use individual ballots for per-model calibration',
              `kept 1 reviewed row at ${keptAt}`,
            ],);
            expect(await readdir(runDir,),).toEqual([
              'judge-fidelity-probe',
              'plan.json',
              'row-0.json',
            ],);
            expect(await readFile(
              join(
                runDir,
                'plan.json',
              ),
              'utf8',
            ),).toBe(JSON.stringify(
              planOf({ rows: 1, },),
              undefined,
              2,
            ),);
            expect(ballotsSorted({ text: seen.written, },),).toBe(ballotsSorted({
              text: JSON.stringify({ rows: [FOUR_ROWS[0],], },),
            },),);
            expect(formatterOutputOf({ text: seen.written, newline: true, },),).toBe(seen.written,);
          },
        },),
      ],
    },),
  ],
},);
