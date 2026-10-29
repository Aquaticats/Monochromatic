/**
 Tests for the settled audit's drive over a real throwaway archive and corpus
 with a scripted client, reader, clock and runs directory: what it refuses
 before it prints, what it prints, what it asks, and what it keeps.

 No model is called and no real archive or clone is read: the client is the
 audit's scripted one, every auditor of which finds nothing, so every row
 holds an empty report that still names who was asked.

 CAPTURING `console.log` IS PROCESS-WIDE, which is why this file runs at
 `concurrency: 1`.

 Fixtures are cat-themed invention on a throwaway git repository. No corpus
 content appears here.

 @module
 */

import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { RUN_MODELS, } from '@monochromatic-dev/translation-repair-run';
import type { RunnerClosure, } from '@monochromatic-dev/translation-repair-insertion';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  runSettledAudit,
  type SettledArtifactReading,
  settledArtifactRecord,
  settledBuyingLine,
} from '../../dist/final/node/index.mjs';
import {
  auditScriptedClient,
  QUIET_AUDIT_REPORT,
} from '@monochromatic-dev/translation-repair-provider/ts/audit-scripted-client.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';
import {
  ENTRY_ID,
  makeCorpus,
  SOURCE_PAGE,
  TARGET_PAGE,
  writeArtifact,
} from '@monochromatic-dev/translation-repair-artifact/ts/corpus-run/settled-archive.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Digest every run claims, ending in the eight characters a kept file's name carries.
 */
const DIGEST = `sha256-tree-v1:${'c'.repeat(56,)}cafef00d`;

/**
 Closure every run records.
 */
const CLOSURE: RunnerClosure = {
  kind: 'read',
  entry: 'rendering-audit-settled.mjs',
  chunks: ['run-config-abc.mjs',],
};

/**
 Roster every run records, whatever the scripted client answers for.
 */
const ROSTER = [
  'hf:cat/Tabby-1',
  'hf:cat/Mouser-1',
];

/**
 Instant the first clock reading gives.
 */
const STARTED_AT = '2026-10-06T06:00:00.000Z';

/**
 Instant the second clock reading gives.
 */
const FINISHED_AT = '2026-10-06T06:05:00.000Z';

/**
 The line the population report prints for the one artifact every archive of these cases holds.
 */
const POPULATION_LINE = 'first-run/mittens.json  subjects=3 retained=2 replaced=1 displaced=0 undecided=3 '
  + 'verification=verified';

/**
 Name the kept run lands under, from the first clock reading and the digest's tail.
 */
const KEPT_NAME = '2026-10-06T06-00-00.000Z-cafef00d.json';

/**
 What one drive did, for the case to read.
 */
type DriveSeen = {
  /**
   Lines the drive printed, one per `console.log` call.
   */
  readonly printed: readonly string[];

  /**
   Model ids the scripted client was asked, in order.
   */
  readonly asked: readonly string[];

  /**
   Times the client was built.
   */
  readonly clients: number;

  /**
   Times the page reader was called.
   */
  readonly reads: number;

  /**
   Times the runs directory was asked for.
   */
  readonly runsAsked: number;

  /**
   What the drive threw, empty when it completed.
   */
  readonly failures: readonly unknown[];

  /**
   Runs directory the kept run lands in.
   */
  readonly runsDir: string;

  /**
   Corpus clone the artifact was read against.
   */
  readonly cloneDir: string;

  /**
   Archive the artifact was written to.
   */
  readonly archiveDir: string;

  /**
   Commit the artifact claims its pair was read at.
   */
  readonly commitSha: string;
};

/**
 Runs the drive once over an archive of one settled artifact, with every part scripted.

 @param sinon - case's own sandbox, so the stub on `console.log` is restored when the case ends

 @param typed - flags after the archive and the clone

 @param empty - whether the archive is left holding nothing

 @param resolve - how the runs directory is named, the scratch directory unless a case brings its own

 @param check - what the case reads while the throwaway directories still exist

 @returns Nothing; the check holds the assertions

 @example
 ```ts
 await driveOver({ sinon: ctx.sinon, typed: ['--cap', '0',], check: async function read({ printed, },) { expect(printed.length,).toBe(3,); }, },);
 ```
 */
async function driveOver(
  {
    sinon,
    typed,
    empty = false,
    resolve,
    check,
  }: {
    readonly sinon: Parameters<typeof divertingConsoleLog>[0]['sinon'];
    readonly typed: readonly string[];
    readonly empty?: boolean;
    readonly resolve?: () => Promise<string>;
    readonly check: (seen: DriveSeen,) => Promise<void>;
  },
): Promise<void> {
  await using corpus = await makeCorpus();
  await using archive = await scratchDir({ prefix: 'settled-drive-archive-', },);
  await using runs = await scratchDir({ prefix: 'settled-drive-runs-', },);
  using printed = divertingConsoleLog({ sinon, },);
  if (!empty) {
    await writeArtifact({
      archiveDir: archive.path,
      runSet: 'first-run',
      prepared: prepareDocumentPair({
        sourceText: SOURCE_PAGE,
        targetText: TARGET_PAGE,
      },),
      corpusSha: corpus.commitSha,
      entryId: ENTRY_ID,
    },);
  }
  /**
   Model ids asked.
   */
  const asked: string[] = [];
  /**
   What the drive threw.
   */
  const failures: unknown[] = [];
  /**
   Times the client was built, the reader called and the directory asked for.
   */
  const counts = {
    clients: 0,
    reads: 0,
    runsAsked: 0,
  };
  /**
   Clock readings still to give.
   */
  const instants = [
    STARTED_AT,
    FINISHED_AT,
  ];
  try {
    await runSettledAudit({
      line: lineOf({
        command: 'rendering-audit-settled',
        typed: [
          '--archive',
          archive.path,
          '--clone',
          corpus.cloneDir,
          ...typed,
        ],
      },),
      newClient: function buildClient(): ModelClient {
        counts.clients += 1;
        return auditScriptedClient({
          reportFor: function quiet() {
            return QUIET_AUDIT_REPORT;
          },
          asked,
        },);
      },
      reader: function readNothing(): Promise<string> {
        counts.reads += 1;
        return Promise.resolve('',);
      },
      runCloneDir: corpus.cloneDir,
      roster: ROSTER,
      resolveRuns: function resolveScratch(): Promise<string> {
        counts.runsAsked += 1;
        return (resolve === undefined) ? Promise.resolve(runs.path,) : resolve();
      },
      now: function nextInstant(): string {
        return nonNullishOrThrow(instants.shift(),);
      },
      pipelineDigest: DIGEST,
      runnerClosure: CLOSURE,
      stageSettings: SCRIPTED_STAGE_SETTINGS,
    },);
  }
  catch (error) {
    failures.push(error,);
  }
  await check({
    printed: printed.lines,
    asked,
    ...counts,
    failures,
    runsDir: runs.path,
    cloneDir: corpus.cloneDir,
    archiveDir: archive.path,
    commitSha: corpus.commitSha,
  },);
}

/**
 Orders model ids by code unit, the one order the cases put voices in.

 @param left - first model id

 @param right - second model id

 @returns Negative, zero or positive as `left` sorts before, with or after `right`

 @example
 ```ts
 const ordered = ['xiaomi/a', 'hf:b',].toSorted(byModelId,);
 ```
 */
function byModelId(left: string, right: string,): number {
  if (left === right)
    return 0;
  return (left < right) ? -1 : 1;
}

/**
 One audited slice as the drive keeps it when every auditor found nothing, its text identity reduced
 to its kind and its voices put in model order, since the order they answered in is not the claim.

 @param row - row as written to the kept run

 @returns Row in a form that does not depend on the order the voices answered

 @example
 ```ts
 const settled = settledRow({ row, },);
 ```
 */
function settledRow({ row, }: { readonly row: Record<string, unknown>; },): Record<string, unknown> {
  /**
   The report the row carries.
   */
  const report = row.report as { readonly rows: readonly { readonly modelId: string; }[]; };
  return {
    ...row,
    textIdentity: (row.textIdentity as { readonly kind: string; }).kind,
    report: {
      ...report,
      rows: report.rows.toSorted(function byModel(left, right,): number {
        return byModelId(
          left.modelId,
          right.modelId,
        );
      },),
    },
  };
}

/**
 What one quiet auditor row holds for a slice of the invented archive.

 @param sliceIndex - slice the row audited

 @param archive - whether it audited the archive's own English

 @param commitSha - commit the artifact claims its pair was read at

 @returns Row, voices in model order and the text identity reduced to its kind

 @example
 ```ts
 const row = quietRow({ sliceIndex: 0, archive: false, commitSha, },);
 ```
 */
function quietRow(
  {
    sliceIndex,
    archive,
    commitSha,
  }: {
    readonly sliceIndex: number;
    readonly archive: boolean;
    readonly commitSha: string;
  },
): Record<string, unknown> {
  return {
    runSet: 'first-run',
    entryId: ENTRY_ID,
    sliceIndex,
    deliveryKind: archive ? 'incumbent-retained' : 'replacement-shipped',
    auditsArchiveText: archive,
    pageRelation: { kind: 'undecided', },
    artifactDigest: `sha256-tree-v1:${'c'.repeat(64,)}`,
    corpusSha: commitSha,
    identityKind: 'declared',
    referencesKind: 'none',
    textIdentity: 'digested',
    report: {
      corroborated: [],
      agreed: [],
      near: [],
      rows: RUN_MODELS.checkerModelIds
        .toSorted(byModelId,)
        .map(function silentVoice(modelId,): Record<string, unknown> {
          return {
            modelId,
            verdict: 'no-defect-found',
            findings: [],
            dropped: [],
          };
        },),
      findings: [],
    },
  };
}

/**
 Reads the one run the drive kept, rows settled.

 @param seen - what the drive did

 @returns Kept run as written, its rows settled

 @example
 ```ts
 const kept = await keptRun({ seen, },);
 ```
 */
async function keptRun({ seen, }: { readonly seen: DriveSeen; },): Promise<Record<string, unknown>> {
  /**
   The probe's directory, which holds exactly the one run.
   */
  const probeDir = join(
    seen.runsDir,
    'rendering-audit-settled',
  );
  expect(await readdir(probeDir,),).toEqual([KEPT_NAME,],);
  /**
   The run as written.
   */
  const written = JSON.parse(await readFile(
    join(
      probeDir,
      KEPT_NAME,
    ),
    'utf8',
  ),) as { readonly rows: readonly Record<string, unknown>[]; };
  return {
    ...written,
    rows: written.rows.map(function settled(row,): Record<string, unknown> {
      return settledRow({ row, },);
    },),
  };
}

/**
 What a kept run holds around its rows, for the invented archive.

 @param seen - what the drive did

 @param cap - cap the run was started with

 @param onlyIds - entry filter the run was started with

 @param rows - rows the run bought

 @returns Whole run, as the drive writes it

 @example
 ```ts
 const expected = keptWith({ seen, cap: 0, onlyIds: [], rows: [], },);
 ```
 */
function keptWith(
  {
    seen,
    cap,
    onlyIds,
    rows,
  }: {
    readonly seen: DriveSeen;
    readonly cap: number;
    readonly onlyIds: readonly string[];
    readonly rows: readonly Record<string, unknown>[];
  },
): Record<string, unknown> {
  return {
    startedAt: STARTED_AT,
    finishedAt: FINISHED_AT,
    pipelineDigest: DIGEST,
    runnerClosure: CLOSURE,
    roster: ROSTER,
    subject: {
      archiveDir: seen.archiveDir,
      cloneDir: seen.cloneDir,
      cap,
      onlyIds,
      artifacts: [
        {
          runSet: 'first-run',
          artifactFile: 'mittens.json',
          entryId: ENTRY_ID,
          artifactDigest: `sha256-tree-v1:${'c'.repeat(64,)}`,
          subjects: 3,
          verification: 'verified',
        },
      ],
    },
    rows,
  };
}

/**
 The line the drive prints last, naming where the run was kept.

 @param seen - what the drive did

 @returns Line, with its leading newline

 @example
 ```ts
 expect(seen.printed.at(-1,),).toBe(keptLine({ seen, },),);
 ```
 */
function keptLine({ seen, }: { readonly seen: DriveSeen; },): string {
  return `\nkept at ${
    join(
      seen.runsDir,
      'rendering-audit-settled',
      KEPT_NAME,
    )
  }`;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: `${runSettledAudit.name} refusals`,
      concurrency: 1,
      children: [
        it({
          name: 'REFUSES AN ARCHIVE HOLDING NOTHING before it prints, builds a client or asks for a directory, '
            + 'since the run was pointed somewhere wrong',
          fn: async (ctx) => {
            await driveOver({
              sinon: ctx.sinon,
              typed: [],
              empty: true,
              check: function readRefusal(seen,): Promise<void> {
                expect(seen.failures.length,).toBe(1,);
                expect(seen.failures[0],).toBeInstanceOf(StatedRefusalError,);
                expect(String(seen.failures[0],),).toBe(`StatedRefusalError: no artifacts under ${seen.archiveDir}`,);
                expect(seen.printed,).toEqual([],);
                expect(seen.clients,).toBe(0,);
                expect(seen.runsAsked,).toBe(0,);
                return Promise.resolve();
              },
            },);
          },
        },),
        it({
          name: 'REFUSES AN ENTRY THE ARCHIVE DOES NOT HOLD after the population it read, before any client, read '
            + 'or directory',
          fn: async (ctx) => {
            await driveOver({
              sinon: ctx.sinon,
              typed: [
                '--only',
                'nobody',
              ],
              check: function readRefusal(seen,): Promise<void> {
                expect(seen.failures.length,).toBe(1,);
                expect(seen.failures[0],).toBeInstanceOf(StatedRefusalError,);
                expect(String(seen.failures[0],),).toBe(
                  'StatedRefusalError: --only asks for "nobody", which the settled archive does not hold',
                );
                expect(seen.printed,).toEqual([POPULATION_LINE,],);
                expect(seen.clients,).toBe(0,);
                expect(seen.reads,).toBe(0,);
                expect(seen.runsAsked,).toBe(0,);
                return Promise.resolve();
              },
            },);
          },
        },),
        it({
          name: 'ASKS FOR THE RUNS DIRECTORY BEFORE IT BUYS, so a directory that cannot be named costs no roster '
            + 'call, no client and no page read',
          fn: async (ctx) => {
            await driveOver({
              sinon: ctx.sinon,
              typed: [
                '--cap',
                '1',
              ],
              resolve: function unnameable(): Promise<string> {
                return Promise.reject(new Error('no runs directory can be named here',),);
              },
              check: function readRefusal(seen,): Promise<void> {
                expect(seen.failures.length,).toBe(1,);
                expect(String(seen.failures[0],),).toBe('Error: no runs directory can be named here',);
                expect(seen.printed,).toEqual([
                  POPULATION_LINE,
                  '\nBUYING 1 of 3 selectable subjects\n',
                ],);
                expect(seen.asked,).toEqual([],);
                expect(seen.clients,).toBe(0,);
                expect(seen.reads,).toBe(0,);
                return Promise.resolve();
              },
            },);
          },
        },),
      ],
    },),

    describe({
      name: `${runSettledAudit.name} over one archive`,
      concurrency: 1,
      children: [
        it({
          name: 'KEEPS A RUN THAT BOUGHT NOTHING AT A ZERO CAP: the population, the buying line and where it was '
            + 'kept, no client, no page read and no roster call',
          fn: async (ctx) => {
            await driveOver({
              sinon: ctx.sinon,
              typed: [
                '--cap',
                '0',
              ],
              check: async function readKept(seen,): Promise<void> {
                expect(seen.failures,).toEqual([],);
                expect(seen.printed,).toEqual([
                  POPULATION_LINE,
                  '\nBUYING 0 of 3 selectable subjects\n',
                  keptLine({ seen, },),
                ],);
                expect(seen.clients,).toBe(0,);
                expect(seen.reads,).toBe(0,);
                expect(seen.asked,).toEqual([],);
                expect(await keptRun({ seen, },),).toEqual(keptWith({
                  seen,
                  cap: 0,
                  onlyIds: [],
                  rows: [],
                },),);
              },
            },);
          },
        },),
        it({
          name: 'KEEPS THE ENTRY FILTER IT WAS GIVEN beside the cap, and counts the subjects that filter left',
          fn: async (ctx) => {
            await driveOver({
              sinon: ctx.sinon,
              typed: [
                '--only',
                ENTRY_ID,
                '--cap',
                '0',
              ],
              check: async function readKept(seen,): Promise<void> {
                expect(seen.failures,).toEqual([],);
                expect(seen.printed[1],).toBe('\nBUYING 0 of 3 selectable subjects\n',);
                expect(await keptRun({ seen, },),).toEqual(keptWith({
                  seen,
                  cap: 0,
                  onlyIds: [ENTRY_ID,],
                  rows: [],
                },),);
              },
            },);
          },
        },),
        it({
          name: 'BUYS THE PREFIX THE CAP ALLOWS through one client and one page read, prints a line per audited '
            + 'slice as it lands, and keeps a row for each beside the population',
          fn: async (ctx) => {
            await driveOver({
              sinon: ctx.sinon,
              typed: [
                '--cap',
                '2',
              ],
              check: async function readKept(seen,): Promise<void> {
                expect(seen.failures,).toEqual([],);
                expect(seen.printed,).toEqual([
                  POPULATION_LINE,
                  '\nBUYING 2 of 3 selectable subjects\n',
                  'first-run/mittens#0 FRESH   undecided claimed=0 corroborated=0 agreed=0 near=0',
                  'first-run/mittens#1 ARCHIVE undecided claimed=0 corroborated=0 agreed=0 near=0',
                  keptLine({ seen, },),
                ],);
                expect(seen.clients,).toBe(1,);
                expect(seen.reads,).toBe(1,);
                expect(seen.asked.length,).toBe(2 * RUN_MODELS.checkerModelIds.length,);
                expect(await keptRun({ seen, },),).toEqual(keptWith({
                  seen,
                  cap: 2,
                  onlyIds: [],
                  rows: [
                    quietRow({
                      sliceIndex: 0,
                      archive: false,
                      commitSha: seen.commitSha,
                    },),
                    quietRow({
                      sliceIndex: 1,
                      archive: true,
                      commitSha: seen.commitSha,
                    },),
                  ],
                },),);
              },
            },);
          },
        },),
      ],
    },),

    describe({
      name: settledBuyingLine.name,
      concurrency: 1,
      children: [
        it({
          name: 'SAYS ONE SUBJECT IN THE SINGULAR when the filter left one, and several in the plural',
          fn: async () => {
            expect(settledBuyingLine({
              buying: 1,
              eligible: 1,
            },),).toBe('\nBUYING 1 of 1 selectable subject\n',);
            expect(settledBuyingLine({
              buying: 2,
              eligible: 5,
            },),).toBe('\nBUYING 2 of 5 selectable subjects\n',);
          },
        },),
        it({
          name: 'SAYS NOTHING IS BOUGHT OF NONE SELECTABLE in the plural',
          fn: async () => {
            expect(settledBuyingLine({
              buying: 0,
              eligible: 0,
            },),).toBe('\nBUYING 0 of 0 selectable subjects\n',);
          },
        },),
      ],
    },),

    describe({
      name: settledArtifactRecord.name,
      concurrency: 1,
      children: [
        it({
          name: 'RECORDS WHAT AN ARTIFACT WAS AND OFFERED: its place, its identity, how many subjects, and the '
            + 'kind of its verification without the detail',
          fn: async () => {
            /**
             An artifact whose provenance check refused, with the detail a reader would not keep.
             */
            const reading: SettledArtifactReading = {
              runSet: 'first-run',
              artifactFile: 'mittens.json',
              entryId: 'mittens',
              artifactDigest: 'sha256-tree-v1:cafef00d',
              verification: {
                kind: 'refused',
                detail: 'slice 3 moved',
              },
              subjects: [],
            };
            expect(settledArtifactRecord({ reading, },),).toEqual({
              runSet: 'first-run',
              artifactFile: 'mittens.json',
              entryId: 'mittens',
              artifactDigest: 'sha256-tree-v1:cafef00d',
              subjects: 0,
              verification: 'refused',
            },);
          },
        },),
      ],
    },),
  ],
},);
