/**
 The lines that say what emptying a run's scratch folder did: how much it removed, how much `keep/` holds, how much room the
 volume gained, and every entry it had to leave where it was.

 The sizes it prints are the sizes of the files removed, and the line says so: on a filesystem that shares data between copies,
 a file's size is not the room its removal gave back, and the room the volume gained is printed beside it as its own figure.

 @module
 */

import type { UnremovedEntry, } from './deleting-what-a-run-left-in-its-scratch-folder.ts';
import type { ScratchFolderEmptiness, } from './emptying-a-scratch-folder.ts';

/**
 How many entries a line names before it counts the rest, so a folder of a thousand unreadable entries still answers with lines a
 reader can read.
 */
export const ENTRIES_NAMED_IN_A_LINE = 10;

/**
 How many bytes one kibibyte is.
 */
const BYTES_IN_A_KIBIBYTE = 1_024;

/**
 How many bytes one mebibyte is.
 */
const BYTES_IN_A_MEBIBYTE: number = BYTES_IN_A_KIBIBYTE * BYTES_IN_A_KIBIBYTE;

/**
 How many bytes one gibibyte is.
 */
const BYTES_IN_A_GIBIBYTE: number = BYTES_IN_A_MEBIBYTE * BYTES_IN_A_KIBIBYTE;

/**
 How many bytes one tebibyte is.
 */
const BYTES_IN_A_TEBIBYTE: number = BYTES_IN_A_GIBIBYTE * BYTES_IN_A_KIBIBYTE;

/**
 One step of the sizes this prints, largest first, so a figure is read in the unit a reader thinks in.
 */
const SIZE_STEPS: readonly {
  readonly bytes: number;
  readonly unit: string
}[] = [
  {
    bytes: BYTES_IN_A_TEBIBYTE,
    unit: 'TiB',
  },
  {
    bytes: BYTES_IN_A_GIBIBYTE,
    unit: 'GiB',
  },
  {
    bytes: BYTES_IN_A_MEBIBYTE,
    unit: 'MiB',
  },
  {
    bytes: BYTES_IN_A_KIBIBYTE,
    unit: 'KiB',
  },
];

/**
 A folder that was emptied, which is the answer every line here is built from.
 */
type EmptiedScratchFolder = Extract<ScratchFolderEmptiness, { readonly emptied: 'done' }>;

/**
 Says one size in the unit a reader thinks in, with one decimal place above the bytes themselves.

 @param bytes - the size in bytes

 @returns the size in words, such as `1.4 GiB`

 @example
 ```ts
 const text = bytesTextOf(1_048_576,);
 ```
 */
export function bytesTextOf(bytes: number,): string {
  /**
   The unit this figure is read in, which is the largest one it reaches.
   */
  const step = SIZE_STEPS.find(function holdsIt(candidate,) {
    return bytes >= candidate.bytes;
  },);
  if (step === undefined) {
    return `${String(bytes,)} B`;
  }
  return `${(bytes / step.bytes).toFixed(1,)} ${step.unit}`;
}

/**
 Says how many of something one count names, in the singular or the plural.

 @param count - how many

 @param one - the word for one of them

 @param many - the word for more than one

 @returns the count and its word

 @example
 ```ts
 const text = countTextOf({ count: 3, one: 'file', many: 'files', },);
 ```
 */
function countTextOf({
  count,
  one,
  many,
}: {
  readonly count: number;
  readonly one: string;
  readonly many: string;
},): string {
  return `${String(count,)} ${(count === 1) ? one : many}`;
}

/**
 Says how much room the volume holding the folder gained, which is the figure a full disk is freed by.

 @param runName - the run whose folder it was

 @param emptying - the folder that was emptied

 @returns the line, which names both readings so a reader sees what was measured

 @example
 ```ts
 const line = roomGainedLineOf({ runName: 'a', emptying, },);
 ```
 */
function roomGainedLineOf({
  runName,
  emptying,
}: {
  readonly runName: string;
  readonly emptying: EmptiedScratchFolder;
},): string {
  /**
   The room before and after, which exist only when both readings worked.
   */
  const {
    freeBytesBefore: before,
    freeBytesAfter: after,
    freeSpaceProblem: problem,
  } = emptying;
  if ((before === undefined) || (after === undefined)) {
    return `scratch: ${runName}: the room the volume gained could not be read (${problem ?? 'no reading was taken'})`;
  }
  /**
   What the volume's room changed by, which other work on the machine can move either way.
   */
  const gained = after - before;
  if (gained <= 0) {
    return `scratch: ${runName}: room on the volume holding it: ${bytesTextOf(before,)} free before and `
      + `${bytesTextOf(after,)} after, so it gained nothing: either another process wrote there while this ran, or what was `
      + 'removed shared its data with a copy that stays, which a filesystem copying data on write does';
  }
  return `scratch: ${runName}: room on the volume holding it: ${bytesTextOf(before,)} free before and `
    + `${bytesTextOf(after,)} after, so it gained ${bytesTextOf(gained,)}`;
}

/**
 Says one entry was left where it is, and why.

 @param runName - the run whose folder it was

 @param entry - the entry and the reason

 @returns the line

 @example
 ```ts
 const line = leftBehindLineOf({ runName: 'a', entry: { path: '/home/a/temp/agent/a-scratch/closed', reason: 'EACCES' }, },);
 ```
 */
function leftBehindLineOf({
  runName,
  entry,
}: {
  readonly runName: string;
  readonly entry: UnremovedEntry;
},): string {
  return `scratch: ${runName}: left ${entry.path} where it is: ${entry.reason}`;
}

/**
 Says what one emptying did with the folder, in the lines a run's end carries.

 @param runName - the run whose folder it was

 @param folder - the run's scratch folder

 @param emptying - what emptying it answered

 @returns the lines, one per fact a reader acts on

 @example
 ```ts
 const lines = scratchEmptyingLinesOf({ runName: 'a', folder: '/home/a/temp/agent/a-scratch', emptying, },);
 ```
 */
export function scratchEmptyingLinesOf({
  runName,
  folder,
  emptying,
}: {
  readonly runName: string;
  readonly folder: string;
  readonly emptying: ScratchFolderEmptiness;
},): readonly string[] {
  if (emptying.emptied === 'no-folder') {
    return [`scratch: ${runName}: no folder to empty at ${folder}`,];
  }
  if (emptying.emptied === 'refused') {
    return [`scratch: ${runName}: refused to empty ${folder}: ${emptying.reason}`,];
  }
  /**
   The entries left where they are, of which the line names the first few.
   */
  const namedEntries = emptying.leftBehind
    .slice(
    0,
    ENTRIES_NAMED_IN_A_LINE,
    );
  /**
   How many entries the line did not name.
   */
  const unnamedEntries = emptying.leftBehind
    .length
    - namedEntries.length;
  /**
   How many files and folders were removed, in words.
   */
  const removedText = `${countTextOf({
    count: emptying.removedFiles,
    one: 'file',
    many: 'files',
  },)} and ${
     countTextOf({
      count: emptying.removedFolders,
      one: 'folder',
      many: 'folders',
    },)}`;
  /**
   What `keep/` holds, in words.
   */
  const heldText = `${bytesTextOf(emptying.keepBytes,)} in ${
     countTextOf({
      count: emptying.keepFiles,
      one: 'file',
      many: 'files',
    },)}`;
  return [
    `scratch: ${runName}: emptied ${folder} except keep/: ${removedText} removed, whose sizes read `
      + `${bytesTextOf(emptying.removedBytes,)} (a sum of the sizes of the files removed, which a filesystem sharing data `
      + `between copies may have kept for a copy that stays); keep/ holds ${heldText}`,
    roomGainedLineOf({
      runName,
      emptying,
    },),
    ...namedEntries.map(function lineOfEntry(entry,) {
      return leftBehindLineOf({
        runName,
        entry,
      },);
    },),
    ...((unnamedEntries === 0)
      ? []
      : [`scratch: ${runName}: and ${String(unnamedEntries,)} more entries were left where they are`,]),
  ];
}
