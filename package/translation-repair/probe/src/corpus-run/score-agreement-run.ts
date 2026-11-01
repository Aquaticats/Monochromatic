import { join, } from 'node:path';

import {
  parseGradedSheet,
  parsePreGrades,
  scoreGradeAgreement,
  scoreGradedPrecision,
} from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { readSheetIdentity, } from '@monochromatic-dev/translation-repair-adjudication/ts';
import { parseRunJson, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { DEFAULT_SAMPLE_SEED, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import {
  assertSheetMatchesManifest,
  HEADER_ONLY_BINDING_NOTE,
  parseSampleManifest,
  requireSheetSeed,
} from '@monochromatic-dev/translation-repair-lane-contest/ts';
import {
  type CommandLineOf,
  writtenOr,
} from '@monochromatic-dev/translation-repair-run/ts';
import {
  preGradeName,
  readNamedOrBeside,
} from './score-agreement-read.ts';
import { readSheetText, } from './score-sheet-text.ts';
import {
  printAgreement,
  printPrecision,
} from './score-agreement-print.ts';

//region Score agreement run
// Reads a graded detection sheet and, when one exists, the blind pre-grades
// recorded beside it, then prints precision and pre-grade agreement.
//
// Prints COUNTS AND POSITIONS ONLY. The sheet quotes UNLICENSED corpus text and
// this output is meant to be pasteable into a verdict or a message, so nothing
// it emits carries a quote, a claim, or a grader's rationale.
//
// Precision is reported whether or not pre-grades exist, because that is the
// milestone gate number and it must not depend on a calibration artifact being
// present.

/**
 Prints precision and, when pre-grades exist, agreement against them.

 @param runsDir - durable, gitignored output root the default files sit in

 @param line - the report's command line, read whole by `reportingRefusals`

 @throws {@link StatedRefusalError} when the sheet cannot be read, or a manifest or pre-grades file the
 operator named is not there

 @example
 ```ts
 await printGradeReport({ runsDir, line, },);
 ```
 */
export async function printGradeReport(
  {
    runsDir,
    line,
  }: {
    readonly runsDir: string;
    readonly line: CommandLineOf<'score-agreement'>;
  },
): Promise<void> {
  /**
   Graded sheet path, defaulting to this seed's final sheet.
   */
  const sheetPath = writtenOr({
    asked: line.flag('sheet',),
    unwritten: join(
      runsDir,
      `grading-sheet-${DEFAULT_SAMPLE_SEED}.md`,
    ),
  },);

  /**
   Sheet contents, read once and used for both identity and verdicts.
   */
  const sheetText = await readSheetText({
    path: sheetPath,
    label: 'graded sheet',
    remedy: 'name the sheet that exists with --sheet, or put the default one in the runs directory',
  },);

  /**
   Draw this sheet declares, which decides which pre-grades may be joined to
   it.

   Read off the sheet rather than assumed from {@link DEFAULT_SAMPLE_SEED}.
   `--sheet` can point anywhere, and a fixed default seed meant an earlier
   round's graded sheet could be scored against THIS round's pre-grades, by
   position, reporting a confident agreement rate between two unrelated
   draws.
   */
  const identity = readSheetIdentity({ text: sheetText, },);

  /**
   Seed the pre-grades and manifest are looked up under.
   */
  const seed = requireSheetSeed({
    identity,
    sheetLabel: 'detection sheet',
    sheetPath,
  },);

  // Validated BEFORE anything is reported, and not beside the code that needs
  // it. Placing this check next to the pre-grade join put it after the early
  // return taken when no pre-grades exist, so the run that most looks like a
  // plain precision reading was exactly the one that checked nothing.
  /**
   What --manifest carried, or that nobody wrote it.
   */
  const manifestAsked = line.flag('manifest',);

  /**
   Path the manifest of the draw this sheet came from is looked for at.
   */
  const manifestPath = writtenOr({
    asked: manifestAsked,
    unwritten: join(
      runsDir,
      `sample-manifest-${seed}.json`,
    ),
  },);

  /**
   Manifest of the draw this sheet came from, when one sits beside it.
   */
  const manifest = await readNamedOrBeside({
    path: manifestPath,
    label: 'sample manifest',
    flag: 'manifest',
    named: manifestAsked.kind === 'written',
  },);
  if (manifest.found) {
    /**
     How firmly the sheet is tied to that manifest; refuses if it is not.
     */
    const binding = assertSheetMatchesManifest({
      identity,
      manifest: parseSampleManifest({
        value: parseRunJson({
          text: manifest.text,
          from: 'sample manifest',
        },),
      },),
      sheetLabel: 'detection sheet',
      sheetPath,
      manifestPath,
    },);
    if (binding === 'header-only')
      console.log(HEADER_ONLY_BINDING_NOTE,);
  }
  else
    console.log(
      'NOTE no manifest found beside this sheet, so nothing proves the '
        + 'pre-grades this report reads describe the same draw. Both files are joined by '
        + 'POSITION and neither prints an issue id.',
    );

  /**
   Human's grades read off the sheet.
   */
  const human = parseGradedSheet({ text: sheetText, },);

  printPrecision({
    items: human.length,
    precision: scoreGradedPrecision({ human, },),
  },);

  /**
   What --pre-grades carried, or that nobody wrote it.
   */
  const preGradesAsked = line.flag('pre-grades',);

  /**
   Blind pre-grades, when calibration recorded any for this draw.
   */
  const preGrades = await readNamedOrBeside({
    path: writtenOr({
      asked: preGradesAsked,
      unwritten: join(
        runsDir,
        preGradeName({ seed, },),
      ),
    },),
    label: 'pre-grades file',
    flag: 'pre-grades',
    named: preGradesAsked.kind === 'written',
  },);
  if (!preGrades.found) {
    console.log('AGREEMENT none: no pre-grades recorded for this draw',);
    return;
  }

  printAgreement({
    agreement: scoreGradeAgreement({
      agent: parsePreGrades({ text: preGrades.text, },),
      human,
    },),
  },);
}

//endregion Score agreement run
