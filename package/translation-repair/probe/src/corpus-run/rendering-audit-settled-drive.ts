import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import { readCorpusFile, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { listArtifactFiles, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import type { PassReferenceReader, } from '@monochromatic-dev/translation-repair-corpus-pass/ts';
import { persistProbeRun, } from './probe-store.ts';
import type { RunnerClosure, } from '@monochromatic-dev/translation-repair-insertion/ts';
import { readAuditArguments, } from './rendering-audit-settled-args.ts';
import {
  auditOne,
  capped,
  eligibleSubjects,
  printPopulation,
  withCitedReferences,
} from './rendering-audit-settled-buy.ts';
import type { CommandLineOf, } from '@monochromatic-dev/translation-repair-run/ts';
import {
  readArchiveSubjects,
  type SettledArtifactReading,
} from './rendering-audit-settled-input.ts';
import { printSettledRow, } from './rendering-audit-settled-progress.ts';
import {
  type SettledAuditRow,
  SETTLED_AUDIT_PROBE,
} from './rendering-audit-settled-row.ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Settled audit drive
// Reads the archive, audits what was asked for, and keeps the answers, once
// the process has handed over what only a real process has: the command line,
// a way to build the client, the reader of the pages an original cites, the
// roster, the runs directory, the clock and the build's identity.

/**
 What a persisted run says about one artifact it read, so a kept run names
 which files it read without needing the archive to still exist.

 @param reading - one artifact as the archive reader returned it

 @returns The artifact's place, identity, subject count and verification kind

 @example
 ```ts
 const artifact = settledArtifactRecord({ reading, },);
 ```
 */
export function settledArtifactRecord(
  { reading, }: { readonly reading: SettledArtifactReading; },
): Record<string, unknown> {
  /**
   What this artifact was and what it offered.
   */
  const {
    runSet,
    artifactFile,
    entryId,
    artifactDigest,
    subjects,
    verification,
  } = reading;

  return {
    runSet,
    artifactFile,
    entryId,
    artifactDigest,
    subjects: subjects.length,
    verification: verification.kind,
  };
}

/**
 Line saying how many of the selectable subjects a run buys, with a blank
 line either side so it stands apart from the population and the progress.

 @param buying - subjects the run will buy

 @param eligible - subjects the entry filter left, which the buy is a fraction of

 @returns The line, with its surrounding newlines

 @example
 ```ts
 console.log(settledBuyingLine({ buying: 2, eligible: 3, },),);
 ```
 */
export function settledBuyingLine(
  {
    buying,
    eligible,
  }: {
    readonly buying: number;
    readonly eligible: number;
  },
): string {
  return `\nBUYING ${String(buying,)} of ${String(eligible,)} selectable ${
    wordForCount({
      count: eligible,
      one: 'subject',
      many: 'subjects',
    },)
  }\n`;
}

/**
 Reads the archive, audits what was asked for, and keeps the answers.

 @param line - the audit's command line, read whole by `reportingRefusals`

 @param newClient - builds the one client the whole run asks through, called
 only once something is bought, so a zero cap still needs no key

 @param reader - reads what the pages an original cites say

 @param runCloneDir - clone of the run's corpus pin, read when `--clone`
 names none

 @param roster - models the run records as its roster

 @param resolveRuns - names the runs directory the kept run lands in

 @param now - the clock, read for the start and for the finish

 @param pipelineDigest - digest of the built output, read when the process started

 @param runnerClosure - chunks the executing entry imports, read when the process started

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @throws {@link StatedRefusalError} when the archive holds nothing to audit,
 which means the run was pointed somewhere wrong rather than that everything
 is clean

 @example
 ```ts
 await runSettledAudit({ line, newClient, reader, runCloneDir, roster, resolveRuns, now, pipelineDigest, runnerClosure, stageSettings, },);
 ```
 */
export async function runSettledAudit(
  {
    line,
    newClient,
    reader,
    runCloneDir,
    roster,
    resolveRuns,
    now,
    pipelineDigest,
    runnerClosure,
    stageSettings,
  }: {
    readonly line: CommandLineOf<'rendering-audit-settled'>;
    readonly newClient: () => ModelClient;
    readonly reader: PassReferenceReader;
    readonly runCloneDir: string;
    readonly roster: readonly string[];
    readonly resolveRuns: () => Promise<string>;
    readonly now: () => string;
    readonly pipelineDigest: string;
    readonly runnerClosure: RunnerClosure;
    readonly stageSettings: StageSettings;
  },
): Promise<void> {
  /**
   When this invocation began, read before any call so the record dates the
   run rather than the moment it happened to finish.
   */
  const startedAt = now();

  /**
   What the command line asked for.
   */
  const asked = readAuditArguments({
    line,
    runCloneDir,
  },);

  /**
   Every artifact the archive holds, parsed, re-prepared and verified. Free.
   */
  const readings = await readArchiveSubjects({
    archiveDir: asked.archiveDir,
    cloneDir: asked.cloneDir,
    readFile: readCorpusFile,
    listFiles: listArtifactFiles,
  },);

  // BEFORE anything is printed. An archive with nothing in it means the run was
  // pointed somewhere wrong, and a population report followed by `BUYING 0 of 0`
  // reads like a clean archive right up until the throw.
  if (readings.length === 0)
    throw new StatedRefusalError({ says: `no artifacts under ${asked.archiveDir}`, },);

  printPopulation({ readings, },);

  /**
   Subjects the entry filter left, which is what a capped buy is a fraction of.
   */
  const eligible = eligibleSubjects({
    readings,
    onlyIds: asked.onlyIds,
  },);

  /**
   Subjects this run will buy.
   */
  const buying = capped({
    eligible,
    cap: asked.cap,
  },);
  console.log(settledBuyingLine({
    buying: buying.length,
    eligible: eligible.length,
  },),);

  /**
   Where this run will be kept, named before anything is bought: a directory
   that cannot be named should cost no roster call, where it used to be asked
   for after the last subject was audited.
   */
  const runsDir = await resolveRuns();

  /**
   What the roster said about each, in order.

   SEQUENTIAL rather than concurrent: these share one roster, and interleaved
   progress lines would make the stream unreadable, which is the only thing a
   long run offers a watcher.
   */
  const rows: SettledAuditRow[] = [];
  if (buying.length > 0) {
    /**
     One client for the whole run, built here rather than per subject: a
     client carries the provider seats, and the seat report reads one
     run-wide tally, so one client is what a run is. Built only once
     something is bought, so `--cap 0`, the wiring check that reads the
     archive and asks nobody, still needs no key.
     */
    const client = newClient();

    /**
     Every bought subject beside what its page cites, read once per page
     before any roster call, through the reader the producing run used, so
     the auditors see what its critics and panels saw (ledger B29).
     */
    const cited = await withCitedReferences({
      subjects: buying,
      reader,
    },);
    for (const {
      subject,
      references,
    } of cited) {
      /* oxlint-disable no-await-in-loop -- sequential by design: every subject shares one roster, and concurrent asks would interleave the progress stream a long run exists to be watched through */
      /**
       What the roster said about this one.
       */
      const row = await auditOne({
        subject,
        references,
        client,
        stageSettings,
      },);
      /* oxlint-enable no-await-in-loop */
      rows.push(row,);
      printSettledRow({ row, },);
    }
  }

  /**
   Where this run was kept, said out loud so the answers are findable.
   */
  const keptAt = await persistProbeRun({
    runsDir,
    probeName: SETTLED_AUDIT_PROBE,
    run: {
      startedAt,
      finishedAt: now(),
      pipelineDigest,
      runnerClosure,
      roster,
      subject: {
        archiveDir: asked.archiveDir,
        cloneDir: asked.cloneDir,
        cap: asked.cap,
        onlyIds: asked.onlyIds,
        artifacts: readings.map(function recorded(reading,): Record<string, unknown> {
          return settledArtifactRecord({ reading, },);
        },),
      },
      rows,
    },
  },);
  console.log(`\nkept at ${keptAt}`,);
}

//endregion Settled audit drive
