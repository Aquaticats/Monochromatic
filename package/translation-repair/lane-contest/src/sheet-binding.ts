import type { SheetIdentity, } from '@monochromatic-dev/translation-repair-adjudication/ts';
import type { SampleManifest, } from './sample-manifest.ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Sheet binding
// The one check both scorers run before joining a graded sheet to a manifest.
//
// Kept in one place because the two scorers ask the same question and had two
// copies of half the answer: `score-probe` compared seed and corpus pin, and
// `score-agreement` compared nothing at all, so it would happily score any
// sheet against whatever pre-grades sat under the default seed.
//
// The join is BY POSITION. Nothing in a sheet names an issue: the sheets print
// no issue ids, deliberately, because a 64-hex string is noise a human has to
// read past. So the sheet's header is the only evidence available that these
// grades are about these items, and everything this module can check lives
// there.

/**
 Which half of the binding disagrees, a closed list so a message is built from
 these and never from a value read off a sheet or a manifest.
 */
type SheetMismatchKind =
  | 'corpus-pin'
    | 'digest'
    | 'manifest-only-digest'
    | 'seed'
    | 'sheet-only-digest';

/**
 Which sheet a refusal speaks of, a literal each scorer writes.
 */
type SheetLabel = 'detection sheet' | 'repair sheet';

/**
 What each mismatch says about the pair and what the operator can do, with no
 value from either file in it.
 */
const MISMATCH_REASONS: Readonly<Record<SheetMismatchKind, string>> = {
  'seed': 'belong to different draws: their seeds differ. Item counts can match across unrelated draws of the '
    + 'same size, so position is not evidence they describe the same items. Name a sheet and a manifest that one '
    + 'draw wrote.',
  'corpus-pin': 'were produced against different corpus commits. The same entry can carry different text at two '
    + 'commits, so the grades and the artifacts would be about different documents. Name a sheet and a manifest '
    + 'produced against one corpus commit.',
  'digest': 'carry different draw digests while agreeing on seed and corpus pin. The draw is deterministic in its '
    + 'seed but not in its pool, so the same seed drawn after another entry settled names a different set of items '
    + 'at the same positions. Name the manifest this sheet was drawn with.',
  'sheet-only-digest': 'disagree about whether this draw is bound: the sheet carries a draw digest and the manifest '
    + 'carries none. One draw writes both in the same instant, so this pair was assembled from two different draws, '
    + 'or the manifest lost its digest. Name a sheet and a manifest that one draw wrote.',
  'manifest-only-digest': 'disagree about whether this draw is bound: the manifest carries a draw digest and the '
    + 'sheet carries none. One draw writes both in the same instant, so this pair was assembled from two different '
    + 'draws, or the sheet lost its digest. Name a sheet and a manifest that one draw wrote.',
};

/**
 What a sheet that declares no draw seed says, with no value from the file in it.
 */
const UNPLACED_REASON = 'declares no draw seed, so nothing can say which draw it came from. Every sheet the '
  + 'formatters write carries a "Draw seed: " header; a file without one cannot be paired with a manifest or with '
  + 'pre-grades except by guessing. Name a sheet a draw wrote.';

/**
 Raised when a graded sheet and the manifest it is scored against cannot describe one draw.

 A stated refusal, because the operator named the files, or left the defaults
 that name them, and can mend it by naming a pair one draw wrote. The message
 is built from a closed kind and the two paths, and quotes nothing read from
 either file.

 @example
 ```ts
 throw new SheetBindingError({
   kind: 'digest',
   sheetLabel: 'repair sheet',
   sheetPath: 'runs/grading-sheet-cat.md',
   manifestPath: 'runs/sample-manifest-cat.json',
 },);
 ```
 */
export class SheetBindingError extends StatedRefusalError {
  /**
   Declares this message safe to forward: it holds two file paths, a sheet
   label and sentences written here, never a value read from either file.
   */
  override readonly messageNamesOnly: true = true;

  /**
   Builds the refusal from which half disagrees and which files were read.

   @param fault - closed kind of the mismatch with the label and the paths of
   the files compared; a sheet with no seed names no manifest, which was not
   looked for yet

   @example
   ```ts
   throw new SheetBindingError({ kind: 'no-seed', sheetLabel: 'detection sheet', sheetPath, },);
   ```
   */
  public constructor(
    fault:
      | {
        readonly kind: SheetMismatchKind;
        readonly sheetLabel: SheetLabel;
        readonly sheetPath: string;
        readonly manifestPath: string;
      }
        | {
          readonly kind: 'no-seed';
          readonly sheetLabel: SheetLabel;
          readonly sheetPath: string;
        },
  ) {
    super({
      says: (fault.kind === 'no-seed')
        ? `${fault.sheetLabel} ${fault.sheetPath} ${UNPLACED_REASON}`
        : `${fault.sheetLabel} ${fault.sheetPath} and manifest ${fault.manifestPath} ${
          MISMATCH_REASONS[fault.kind]
        }`,
    },);
    this.name = 'SheetBindingError';
  }
}

/**
 How firmly a sheet was tied to the manifest it is being scored against.

 @example
 ```ts
 const strength: SheetBindingStrength = 'digest';
 ```
 */
export type SheetBindingStrength =
  | 'digest'
    | 'header-only';

/**
 Refuses a sheet and manifest that do not describe one draw.

 Seed and corpus pin are checked first and always, because a disagreement
 there is unambiguous. They are not sufficient on their own: the draw is
 deterministic in its seed but not in its POOL, and the pool grows with every
 entry that settles, so one seed at one corpus commit names different item
 sets at different times. Equal item counts do not help either, since two
 unrelated draws of the same size match on count and mislabel every verdict.

 @param identity - what the sheet's header declares

 @param manifest - manifest the grades would be joined against

 @param sheetLabel - which sheet this is, for the failure message

 @param sheetPath - file the sheet was read from, which the refusal names so the operator can find it

 @param manifestPath - file the manifest was read from, which the refusal names for the same reason

 @returns Which check actually held, so a caller can report a weak binding
 rather than implying a strong one

 @throws {@link SheetBindingError} when the two describe different draws, since joining
 them by position would mislabel every verdict rather than fail

 @example
 ```ts
 const strength = assertSheetMatchesManifest({
   identity,
   manifest,
   sheetLabel: 'repair sheet',
   sheetPath,
   manifestPath,
 },);
 ```
 */
export function assertSheetMatchesManifest(
  {
    identity,
    manifest,
    sheetLabel,
    sheetPath,
    manifestPath,
  }: {
    readonly identity: SheetIdentity;
    readonly manifest: SampleManifest;
    readonly sheetLabel: SheetLabel;
    readonly sheetPath: string;
    readonly manifestPath: string;
  },
): SheetBindingStrength {
  if (identity.seed !== manifest.seed)
    throw new SheetBindingError({
      kind: 'seed',
      sheetLabel,
      sheetPath,
      manifestPath,
    },);

  if (identity.corpusSha !== manifest.corpusSha)
    throw new SheetBindingError({
      kind: 'corpus-pin',
      sheetLabel,
      sheetPath,
      manifestPath,
    },);

  /**
   Whether the sheet declares a digest.
   */
  const sheetBound = identity.drawDigest !== '';

  /**
   Whether the manifest declares one.
   */
  const manifestBound = manifest.drawDigest !== undefined;

  // Absence has to be SYMMETRIC to mean anything. One draw writes all three
  // files in one breath and now always computes a digest, so a pair where
  // exactly one side carries one did not come from a single draw: it is a
  // legacy sheet paired with a new manifest, or a sheet whose header line was
  // lost. Both are the mislabelling this check exists to prevent, and treating
  // them as legacy would let the newer file's presence be ignored by the older
  // file's absence.
  if (sheetBound !== manifestBound)
    throw new SheetBindingError({
      kind: sheetBound ? 'sheet-only-digest' : 'manifest-only-digest',
      sheetLabel,
      sheetPath,
      manifestPath,
    },);

  // Absent on BOTH sides means the draw predates the binding, which is true of
  // every sheet drawn before it existed and is not a fault. Refusing those
  // would strand graded sheets that nothing can redraw, since a final draw
  // refuses to overwrite itself precisely because it may already carry hours of
  // grading.
  if (!sheetBound)
    return 'header-only';

  if (identity.drawDigest !== manifest.drawDigest)
    throw new SheetBindingError({
      kind: 'digest',
      sheetLabel,
      sheetPath,
      manifestPath,
    },);

  return 'digest';
}

/**
 Reads the draw a sheet belongs to, refusing a sheet that names none.

 Every sheet the formatters have ever written declares its seed, so a sheet
 without one is not an older sheet: it is a file nothing can place. Falling
 back to the current default seed would resolve that file's pre-grades and
 manifest under whatever round is being worked on now, which is exactly the
 mispairing the binding exists to stop, arriving through the back door.

 @param identity - what the sheet's header declares

 @param sheetLabel - which sheet this is, for the failure message

 @param sheetPath - file the sheet was read from, which the refusal names so the operator can find it

 @returns Seed the sheet declares

 @throws {@link SheetBindingError} when the sheet declares no seed

 @example
 ```ts
 const seed = requireSheetSeed({ identity, sheetLabel: 'detection sheet', sheetPath, },);
 ```
 */
export function requireSheetSeed(
  {
    identity,
    sheetLabel,
    sheetPath,
  }: {
    readonly identity: SheetIdentity;
    readonly sheetLabel: SheetLabel;
    readonly sheetPath: string;
  },
): string {
  if (identity.seed === '')
    throw new SheetBindingError({
      kind: 'no-seed',
      sheetLabel,
      sheetPath,
    },);
  return identity.seed;
}

/**
 Sentence explaining what a weak binding did and did not establish.

 Printed rather than silent, so a run scored under the older check never reads
 as one the digest confirmed.
 */
export const HEADER_ONLY_BINDING_NOTE: string =
  'NOTE sheet and manifest agree on seed and corpus pin, but one of them '
    + 'carries no draw digest, so this join rests on the file names and the '
    + 'header rather than on the items. Draws taken before the digest existed '
    + 'read this way, and are scoreable; a NEW draw reading this way means the '
    + 'digest was dropped somewhere and the pairing is unproven.';

//endregion Sheet binding
