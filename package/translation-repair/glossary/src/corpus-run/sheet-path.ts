import { stat, } from 'node:fs/promises';
import { join, } from 'node:path';

import { isAsciiAlphanumeric, } from '@monochromatic-dev/translation-repair-markup/ts';
import { rethrowUnlessMissingPath, } from '@monochromatic-dev/translation-repair-typography/ts';
import { StatedRefusalError, } from '../stated-refusal.ts';

//region Grading sheet path
// Decides where a grading sheet is written, and refuses to write over one that
// may already carry human grades.
//
// The hazard this exists to remove is concrete rather than theoretical. Round
// one's sheet was written to a fixed `grading-sheet.md`, the user graded it IN
// PLACE, and 24 of its 50 items came back carrying free-text rationale that no
// other file reproduces (the gate verdict records the Y/N tally, not the
// reasoning). Drawing round two through the same fixed name would have silently
// destroyed all of it, and a draw is exactly the routine command someone runs
// without thinking twice.
//
// Two independent defenses, because either alone still loses data. Naming the
// sheet after its draw seed means two rounds cannot target one path at all;
// refusing to overwrite means that even a repeated draw within ONE round, or a
// seed someone reuses by accident, cannot clobber grading already done.
// Preliminary sheets are exempt from the refusal: they are ungraded scratch
// that is meant to be redrawn as the pool grows.

/**
 Characters a draw seed may contribute to a file name besides ASCII letters
 and digits.

 The seed reaches a path, so it crosses into filesystem grammar where `/` and
 `..` mean traversal rather than text. Seeds are developer-set constants today,
 which is an argument for the check being cheap, not for omitting it.
 */
const SEED_MARKS: ReadonlySet<string> = new Set([
  '.',
  '_',
  '-',
],);

/**
 Composes the refuse-to-clobber message, joined from lines rather than
 concatenated inline so the message is one expression at the `super` call.

 @param path - sheet that already exists

 @returns Multi-line failure message

 @example
 ```ts
 const message = existingSheetMessage({ path: '/runs/sheet.md', },);
 ```
 */
function existingSheetMessage({ path, }: { readonly path: string; },): string {
  return [
    `Refusing to overwrite an existing final grading sheet: ${path}`,
    'It may already carry human grades, which nothing else reproduces.',
    'Move or rename it first, or draw with a different seed.',
  ].join('\n',);
}

/**
 Composes the rejected-seed message.

 @param seed - seed that cannot become a file name

 @returns Multi-line failure message

 @example
 ```ts
 const message = unsafeSeedMessage({ seed: '../escape', },);
 ```
 */
function unsafeSeedMessage({ seed, }: { readonly seed: string; },): string {
  return [
    `Draw seed is not usable as a file name: ${JSON.stringify(seed,)}`,
    'Allowed: letters, digits, dot, underscore, hyphen.',
  ].join('\n',);
}

/**
 Which sheet of the grading pair a path is for.

 Two sheets rather than two questions on one, because showing a grader the
 correction makes the alleged defect look more real and would move the
 detection grades. They are separate files so the second cannot be read early.

 @example
 ```ts
 const kind: SheetKind = 'repair';
 ```

 @internal
 */
export type SheetKind = 'detection' | 'repair' | 'manifest';

/**
 File-name stem and extension for each draw output.

 The detection sheet keeps its original name because two graded rounds already
 live under it, and renaming would make one continuous series look like two
 different measurements.

 The manifest is not a sheet and is never shown to a grader; it rides here
 because it must be seed-named and overwrite-protected exactly as the sheets
 are. Losing it silently would cost more than losing a sheet, since a sheet
 can be redrawn and the mapping it records cannot.
 */
const DRAW_OUTPUTS: Readonly<Record<SheetKind, {
  readonly stem: string;
  readonly extension: string;
}>> = {
  detection: {
    stem: 'grading-sheet',
    extension: 'md',
  },
  repair: {
    stem: 'repair-sheet',
    extension: 'md',
  },
  manifest: {
    stem: 'sample-manifest',
    extension: 'json',
  },
};

/**
 Raised when a final grading sheet already exists at the target path, which
 may mean it already carries human grades.

 @example
 ```ts
 throw new GradedSheetExistsError({ path: '/runs/grading-sheet-seed.md', },);
 ```

 @internal
 */
export class GradedSheetExistsError extends StatedRefusalError {
  /**
   Declares this message safe to forward: it names the sheet path and nothing inside it.
   */
  override readonly messageNamesOnly: true = true;

  /**
   Builds the refuse-to-clobber failure.

   @param path - sheet that already exists
   */
  constructor({ path, }: { readonly path: string; },) {
    super({ says: existingSheetMessage({ path, },), },);
    this.name = 'GradedSheetExistsError';
  }
}

/**
 Raised when a draw seed cannot safely become part of a file name.

 @example
 ```ts
 throw new UnsafeSeedError({ seed: '../escape', },);
 ```

 @internal
 */
export class UnsafeSeedError extends StatedRefusalError {
  /**
   Declares this message safe to forward: it names the seed the operator asked for.
   */
  override readonly messageNamesOnly: true = true;

  /**
   Builds the rejected-seed failure.

   @param seed - seed that cannot become a file name
   */
  constructor({ seed, }: { readonly seed: string; },) {
    super({ says: unsafeSeedMessage({ seed, },), },);
    this.name = 'UnsafeSeedError';
  }
}

/**
 Whether every character of a seed is safe to place in a file name.

 Scans by index rather than iterating code points, which is both what the
 lint rule asks for and what this check actually wants: any multi-byte or
 combining character fails membership anyway, so grapheme correctness is
 irrelevant to a question whose whole answer set is ASCII.

 @param seed - draw seed to validate

 @returns True when the seed is non-empty and entirely allowed characters

 @example
 ```ts
 const safe = isSeedSafe({ seed: 'round-two', },);
 ```
 */
function isSeedSafe({ seed, }: { readonly seed: string; },): boolean {
  if (seed === '')
    return false;
  for (let index = 0; index < seed.length; index += 1)
    if ((!isAsciiAlphanumeric({ character: seed.charAt(index,), },)) && (!SEED_MARKS.has(seed.charAt(index,),)))
      return false;
  return true;
}

/**
 Whether something exists at a path, distinguishing absence from a real fault.

 @param path - filesystem path to probe

 @returns True when the path resolves to an existing entry

 @throws Whatever `stat` raised when the failure was not a plain absence,
 because a permissions or IO fault must not read as "safe to overwrite"

 @example
 ```ts
 const present = await pathExists({ path: '/runs/sheet.md', },);
 ```
 */
async function pathExists({ path, }: { readonly path: string; },): Promise<boolean> {
  try {
    await stat(path,);
    return true;
  }
  catch (error) {
    // ENOENT is the expected "not there" answer. Anything else (EACCES, EIO)
    // means the question went unanswered, and treating that as absence is how
    // a guard talks itself into deleting data.
    rethrowUnlessMissingPath({ error, },);
    return false;
  }
}

/**
 Resolves where this draw's grading sheet belongs, refusing a final path that
 already exists.

 @param runsDir - durable, gitignored run output root

 @param seed - draw seed, which names the sheet so rounds cannot collide

 @param isFinal - whether this is the gate sheet rather than scratch

 @param kind - which sheet of the pair this is; repair grading lives on its
 own sheet so the detection sheet stays the instrument earlier rounds were
 measured with, and defaults to the detection sheet

 @returns Absolute path the sheet may be written to

 @throws {@link UnsafeSeedError} when the seed cannot become a file name

 @throws {@link GradedSheetExistsError} when a final sheet is already there

 @example
 ```ts
 const outPath = await resolveSheetPath({ runsDir, seed, isFinal: true, },);
 ```

 @internal
 */
export async function resolveSheetPath(
  {
    runsDir,
    seed,
    isFinal,
    kind = 'detection',
  }: {
    readonly runsDir: string;
    readonly seed: string;
    readonly isFinal: boolean;
    readonly kind?: SheetKind;
  },
): Promise<string> {
  if (!isSeedSafe({ seed, },))
    throw new UnsafeSeedError({ seed, },);

  /**
   Name parts for this draw output.
   */
  const output = DRAW_OUTPUTS[kind];

  /**
   Path for this seed and draw kind; the seed in the name is what keeps one
   round from targeting another round's file.
   */
  const path = join(
    runsDir,
    isFinal
      ? `${output.stem}-${seed}.${output.extension}`
      : `${output.stem}-${seed}-preliminary.${output.extension}`,
  );

  // Preliminary sheets are redrawn on purpose as the pool grows, so only the
  // final sheet is protected.
  if (isFinal && (await pathExists({ path, },)))
    throw new GradedSheetExistsError({ path, },);

  return path;
}

//endregion Grading sheet path
