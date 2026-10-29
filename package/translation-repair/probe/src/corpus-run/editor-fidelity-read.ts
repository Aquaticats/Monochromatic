import { readFile, } from 'node:fs/promises';
import {
  basename,
  dirname,
  extname,
  join,
} from 'node:path';
import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  type CommandLineOf,
  reportingRefusals,
} from '@monochromatic-dev/translation-repair-run/ts';
import { isWholeNumberText, } from '@monochromatic-dev/translation-repair-markup/ts';
import { rethrowUnlessMissingPath, } from '@monochromatic-dev/translation-repair-typography/ts';
import { checkEditorBench, } from './editor-fidelity-arm-bench.ts';
import { fixRateLinesOf, } from './editor-fidelity-arm-fix-rate.ts';
import { repeatLinesOf, } from './editor-fidelity-arm-repeats.ts';
import { benchReportLines, } from './editor-fidelity-arm-report.ts';
import type {
  ArmLog,
  ExitRecord,
} from './editor-fidelity-arm-facts.ts';

//region Editor fidelity read
// The read step of an editor fidelity bench: it checks the log of every arm before any figure of them is read, prints one
// PASS or FAIL line for each check, and refuses when any is a FAIL. It reads files and prints; the checks are
// `editor-fidelity-arm-check.ts`.
//
// THE EXIT STATUS IS NOT IN A LOG, so the runbook writes it beside the log: `<arm>.exit` holds the status the arm's launch
// returned, for the log `<arm>.log`. An arm with no such file has no recorded status, and its exit check fails.

/**
 Reads a file that may not be there.

 @param path - file to read

 @returns File's text, and whether it was there

 @example
 ```ts
 const { text, present, } = await readIfPresent({ path: 'low.exit', },);
 ```
 */
async function readIfPresent({ path, }: { readonly path: string; },): Promise<{
  readonly text: string;
  readonly present: boolean
}> {
  try {
    return {
      text: await readFile(
        path,
        'utf8',
      ),
      present: true,
    };
  }
  catch (error) {
    rethrowUnlessMissingPath({ error, },);
    return {
      text: '',
      present: false,
    };
  }
}

/**
 Reads the exit status recorded beside a log.

 @param path - log's path

 @returns Status, or that none was recorded or the file holds no whole number

 @example
 ```ts
 const exit = await exitRecordBeside({ path: 'low.log', },);
 ```
 */
async function exitRecordBeside({ path, }: { readonly path: string; },): Promise<ExitRecord> {
  /**
   The file beside the log.
   */
  const {
    text,
    present,
  } = await readIfPresent({
    path: join(
      dirname(path,),
      `${basename(
        path,
        extname(path,),
      )}.exit`,
    ),
  },);
  /**
   What the file says, trimmed.
   */
  const said = text.trim();
  if ((!present) || (!isWholeNumberText({ text: said, },))) {
    return { recorded: false, };
  }
  return {
    recorded: true,
    code: Number(said,),
  };
}

/**
 Reads one arm's log and its exit status.

 @param path - log's path

 @returns Arm

 @throws StatedRefusalError when the log is not there

 @example
 ```ts
 const arm = await readArm({ path: 'low.log', },);
 ```
 */
async function readArm({ path, }: { readonly path: string; },): Promise<ArmLog> {
  /**
   The log.
   */
  const {
    text,
    present,
  } = await readIfPresent({ path, },);
  if (!present) {
    throw new StatedRefusalError({
      says: `the arm log ${path} is not there; name the log each arm wrote, in the order the arms ran`,
    },);
  }
  return {
    arm: basename(
      path,
      extname(path,),
    ),
    lines: text.split('\n',),
    exit: await exitRecordBeside({ path, },),
  };
}

/**
 Reads the logs the command line names, checks them and prints the checks.

 @param line - command line, read whole by `reportingRefusals`

 @throws StatedRefusalError when a log is not there, or any check fails

 @example
 ```ts
 await main({ line, },);
 ```
 */
async function main({ line, }: { readonly line: CommandLineOf<'editor-fidelity-read'>; },): Promise<void> {
  /**
   The arms, read in the order named.
   */
  const arms = await allInInputOrder({
    members: line.positionals
      .map(function armAt(path,): Promise<ArmLog> {
        return readArm({ path, },);
      },),
  },);
  /**
   What the checks said.
   */
  const {
    lines,
    failed,
  } = checkEditorBench({ arms, },);
  /**
   Checks that held; a WARN line is neither a pass nor a failure.
   */
  const passed = lines.filter(function isPass(said,): boolean {
    return said.startsWith('PASS ',);
  },);
  // THE REPEAT AGREEMENT OF EACH ARM IS PRINTED WITH THE CHECKS, even when one fails: the runbook reads the high arm's alone,
  // before the other arms exist, to choose how many passes the later arms take. It informs that choice and the reading of
  // the verdict; it is not a figure of the verdict. The warning of a high fix rate of the default effort follows it, since
  // it says where the repeat warning stops seeing.
  /**
   The check lines, then the repeat lines, then the high fix rate warning.
   */
  const printed = [
    ...lines,
    ...repeatLinesOf({ arms, },),
    ...fixRateLinesOf({ arms, },),
  ];
  process.stdout
    .write(`${printed.join('\n',)}\nCHECKS ${String(passed.length,)} pass, ${String(failed,)} fail\n`,);
  if (failed > 0) {
    throw new StatedRefusalError({
      says: `${String(failed,)} of the checks failed, so the logs of these arms are not evidence of the bench; read the FAIL lines printed before this one`,
    },);
  }
  // THE FIGURES AND THE VERDICT ARE PRINTED ONLY FOR A BENCH WHOSE CHECKS PASSED: an arm with a failed check is not
  // evidence, and a count read off it would be a count of something else.
  process.stdout
    .write(`${benchReportLines({ arms, },)
      .join('\n',)}\n`,);
}

if (import.meta.main)
  await reportingRefusals({
    what: 'editor-fidelity-read',
    argv: process.argv,
    env: process.env,
    run: main,
  },);

//endregion Editor fidelity read
