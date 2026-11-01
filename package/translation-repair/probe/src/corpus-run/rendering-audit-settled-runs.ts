import { textsInCodePointOrder, } from '@monochromatic-dev/translation-repair-markup/ts';
import { join, } from 'node:path';

import {
  requireRecord,
  requireString,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { readRunJson, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  namesOfKind,
  StatedRefusalError,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import { repeatBandOf, } from './rendering-audit-settled-band.ts';
import { printBand, } from './rendering-audit-settled-print.ts';
import { auditRepeatsAcross, } from './rendering-audit-settled-repeat.ts';
import {
  type SettledAuditRow,
  SETTLED_AUDIT_PROBE,
} from './rendering-audit-settled-row.ts';

//region Settled audit runs
// Reading persisted runs of the settled audit back: which run is meant, what
// it carries, and the one across-run comparison.
//
// SPLIT OUT OF THE REPORT ENTRY for the reason the buying half is split out of
// the audit entry, recorded in `rendering-audit-settled-buy.ts`: a module the
// barrel imports stops being an entry, and the command stops running.

/**
 Reads the persisted rows of one run.

 VALIDATES ONLY WHAT THE READINGS TOUCH, and says so: a full parser for a
 shape this module also writes would be two copies of one contract, and the
 questions here are answered from a handful of fields.

 Exported through the barrel for the built bundle's tests; `main` and
 `printAcross` are its callers.

 @internal

 @param path - persisted run file

 @returns Rows as the probe wrote them, the archive that run named, and the
 roster it asked, empty for a run written before the roster was kept

 @throws {@link ArtifactParseError} when the file is not an object

 @throws {@link StatedRefusalError} when it carries no rows array, which
 means it is not a run of this probe rather than that the run was quiet

 @example
 ```ts
 const { rows, archiveDir, roster, } = await readRunRows({ path, },);
 ```
 */
export async function readRunRows(
  { path, }: { readonly path: string; },
): Promise<{
  readonly rows: readonly SettledAuditRow[];
  readonly archiveDir: string;
  readonly roster: readonly string[];
}> {
  /**
   Run as written.
   */
  const run = requireRecord({
    value: await readRunJson({ path, },),
    path,
  },);

  /**
   Rows the run carries, still untyped.
   */
  const rows: unknown = run.rows;
  if (!Array.isArray(rows,))
    throw new StatedRefusalError({ says: `${path} carries no rows array`, },);

  /**
   Position of the first row that is not an object, which every later reading
   would read fields off, absent when every row is one.
   */
  const notObjectAt = rows.findIndex(function isNotObject(row: unknown,): boolean {
    return !isJsonRecord(row,);
  },);
  if (notObjectAt !== (-1))
    throw new StatedRefusalError({
      says: `${path} carries a row that is not an object, at position ${String(notObjectAt,)}`,
    },);

  /**
   Roster the run asked, read tolerantly: the field was persisted from the
   first run of this probe, but a file that lacks it is still a run whose
   other readings all answer, and the voice rates then say only what the
   rows say.
   */
  const recordedRoster: unknown = run.roster;

  /**
   Model ids the roster names, which is everything in it that is a string.
   */
  const roster = Array.isArray(recordedRoster,)
    ? recordedRoster.filter(function isId(one: unknown,): one is string {
      return (typeof one) === 'string';
    },)
    : [];

  /**
   What that run was pointed at, in its own words.

   FROM THE FILE, never from this invocation's arguments. Reading an old run
   with `--run` would otherwise print the archive THIS command defaulted to
   and attribute the rows to it, which is a confident misstatement of where
   they came from.
   */
  const subject = requireRecord({
    value: run.subject,
    path: `${path}.subject`,
  },);

  return {
    rows: rows as readonly SettledAuditRow[],
    archiveDir: requireString({
      value: subject.archiveDir,
      path: `${path}.subject.archiveDir`,
    },),
    roster,
  };
}

/**
 Finds the newest run of this probe.

 Names sort lexically by the instant they carry, so the last name is the
 newest run without reading a single file.

 Exported through the barrel for the built bundle's tests; `main` is its
 only caller.

 @internal

 @param runsDir - resolved runs directory

 @returns Path of the newest run

 @throws {@link StatedRefusalError} when the probe has never run, since
 reporting nothing would look exactly like reporting a clean run

 @example
 ```ts
 const path = await newestRun({ runsDir, },);
 ```
 */
export async function newestRun({ runsDir, }: { readonly runsDir: string; },): Promise<string> {
  /**
   Where runs of this probe collect.
   */
  const probeDir = join(
    runsDir,
    SETTLED_AUDIT_PROBE,
  );

  /**
   Every run kept, oldest first: files the store finished, so neither a
   directory named like a later run nor a run still being written under its
   temporary name (ledger B65).
   */
  const kept = textsInCodePointOrder({ texts: (await namesOfKind({
    dir: probeDir,
    kind: 'file',
  },))
    .filter(function isRun(name,): boolean {
      return name.endsWith('.json',);
    },), },);

  /**
   Newest, which is the last name.
   */
  const newest = kept.at(-1,);
  if (newest === undefined)
    throw new StatedRefusalError({ says: `${probeDir} holds no run of ${SETTLED_AUDIT_PROBE}`, },);
  return join(
    probeDir,
    newest,
  );
}

/**
 Counts slots in a phrase that reads correctly at one.

 @param count - how many slots

 @returns Phrase to open a sentence with

 @example
 ```ts
 console.log(`${slotsPhrase({ count: 1, },)} that cannot be checked`,);
 ```
 */
function slotsPhrase(
  { count, }: { readonly count: number; },
): string {
  if (count === 1)
    return 'One slot';
  return `${String(count,)} slots`;
}

/**
 Pairs this run against an earlier one and prints the spread.

 Exported through the barrel for the built bundle's tests; `main` is its
 only caller.

 @internal

 @param rows - rows of the run being reported

 @param against - path of the run to pair against

 @example
 ```ts
 await printAcross({ rows, against, },);
 ```
 */
export async function printAcross(
  {
    rows,
    against,
  }: {
    readonly rows: readonly SettledAuditRow[];
    readonly against: string;
  },
): Promise<void> {
  /**
   Rows of the run being compared against.
   */
  const { rows: earlier, } = await readRunRows({ path: against, },);

  /**
   Subjects both runs bought, split three ways.
   */
  const {
    paired,
    textMoved,
    unverifiable,
  } = auditRepeatsAcross({
    first: earlier,
    second: rows,
  },);

  printBand({
    band: repeatBandOf({ pairs: paired, },),
    over: `the same subjects in ${against}`,
  },);

  // Two different sentences, deliberately. One is about the corpus and one is
  // about the probe, and saying the corpus moved when a run simply did not
  // record what it saw would be a confident claim built from missing evidence.
  if (textMoved.length > 0)
    console.log(
      `  ${slotsPhrase({ count: textMoved.length, },)} recorded by BOTH runs and the text`
        + ` DISAGREES, so the archive moved between them and these are left out:`
        + ` ${textMoved.join(', ',)}`,
    );
  if (unverifiable.length > 0)
    console.log(
      `  ${slotsPhrase({ count: unverifiable.length, },)} that cannot be checked, because one of`
        + ` the runs recorded no text identity. That is a fact about the run and says NOTHING`
        + ` about the archive. No band is quotable over them.`,
    );
}

//endregion Settled audit runs
