/**
 Deterministic synthetic file contents.

 Text always ends with exactly one LF and never contains CR,
 so cli-git's default `final-newline` normalization never changes captured bytes;
 binary content starts with NUL so Git and `final-newline` both treat it as binary.

 @module
 */

import type { SeededRandom, } from './seeded-random-fixture.ts';

//region Constants

/**
 Word pool for synthetic lines.
 */
const WORDS: readonly string[] = [
  'alpha',
  'bravo',
  'commit',
  'delta',
  'echo',
  'index',
  'lock',
  'merge',
  'oscar',
  'prepare',
  'ref',
  'tree',
];

/**
 Largest synthesized text size in bytes;
 trace sizes above it are capped to bound container memory.
 */
export const MAX_TEXT_BYTES = 32_768;

/**
 Largest synthesized binary size in bytes.
 */
export const MAX_BINARY_BYTES = 8_192;

/**
 Average synthetic line length used to turn byte sizes into line counts.
 */
const AVERAGE_LINE_BYTES = 40;

/**
 Share of synthetic words that are numbers.
 */
const NUMBER_WORD_SHARE = 0.25;

/**
 Largest number word.
 */
const NUMBER_WORD_MAX = 999;

/**
 Largest byte value.
 */
const BYTE_MAX = 255;

//endregion Constants

//region Text

/**
 Synthesizes one line without its terminator.

 @param random - seeded source

 @returns line text

 @example
 ```ts
 synthesizeLine(random); // => 'merge 12 lock tree'
 ```
 */
export function synthesizeLine(random: SeededRandom,): string {
  return Array.from(
    { length: random.integer({
      min: 2,
      max: 7,
    },), },
    function word() {
      return random.next() < NUMBER_WORD_SHARE ? String(random.integer({
        min: 0,
        max: NUMBER_WORD_MAX,
      },),) : random.pick(WORDS,);
    },
  )
    .join(' ',);
}

/**
 Joins lines into canonical text with exactly one final LF.

 @param lines - lines without terminators

 @returns UTF-8 bytes

 @example
 ```ts
 joinLines(['a', 'b']); // => <Buffer 61 0a 62 0a>
 ```
 */
export function joinLines(lines: readonly string[],): Buffer {
  return Buffer.from(`${(lines.length === 0 ? ['empty',] : lines).join('\n',)}\n`,);
}

/**
 Splits canonical text into lines.

 @param bytes - text ending with LF

 @returns lines without terminators

 @example
 ```ts
 splitLines(Buffer.from('a\nb\n')); // => ['a', 'b']
 ```
 */
export function splitLines(bytes: Buffer,): readonly string[] {
  /**
   Decoded text.
   */
  const text = bytes.toString('utf8',);
  return (text.endsWith('\n',) ? text.slice(
    0,
    -1,
  ) : text).split('\n',);
}

/**
 Synthesizes text of roughly a target size.

 @param random - seeded source

 @param size - target size in bytes, capped at {@link MAX_TEXT_BYTES}

 @returns canonical text

 @example
 ```ts
 synthesizeText({ random, size: 400 });
 ```
 */
export function synthesizeText({
  random,
  size,
}: Readonly<{
  random: SeededRandom;
  size: number;
}>,): Buffer {
  /**
   Line count approximating the size.
   */
  const lines = Math.max(
    1,
    Math.ceil(Math.min(
      size,
      MAX_TEXT_BYTES,
    ) / AVERAGE_LINE_BYTES,),
  );
  return joinLines(Array.from(
    { length: lines, },
    function line() {
      return synthesizeLine(random,);
    },
  ),);
}

/**
 Replaces a block of lines,
 keeping canonical termination.

 @param random - seeded source

 @param current - current text

 @param added - lines to insert

 @param deleted - lines to remove

 @returns edited text, always different from `current`

 @example
 ```ts
 editText({ random, current, added: 3, deleted: 1 });
 ```
 */
export function editText({
  random,
  current,
  added,
  deleted,
}: Readonly<{
  random: SeededRandom;
  current: Buffer;
  added: number;
  deleted: number;
}>,): Buffer {
  /**
   Current lines.
   */
  const lines = splitLines(current,);
  /**
   Lines removed, bounded by what exists.
   */
  const removeCount = Math.min(
    deleted,
    lines.length,
  );
  /**
   Edit position.
   */
  const at = random.integer({
    min: 0,
    max: lines.length - removeCount,
  },);
  /**
   Inserted lines; at least one so the edit is never a no-op.
   */
  const inserted = Array.from(
    { length: Math.max(
      1,
      Math.min(
        added,
        MAX_TEXT_BYTES / AVERAGE_LINE_BYTES,
      ),
    ), },
    function line() {
      return `${synthesizeLine(random,)} ${String(random.integer({
      min: 0,
      max: 1_000_000,
    },),)}`;
    },
  );
  return joinLines([
    ...lines.slice(
      0,
      at,
    ),
    ...inserted,
    ...lines.slice(at + removeCount,),
  ],);
}

//endregion Text

//region Binary

/**
 Synthesizes binary bytes that start with NUL.

 @param random - seeded source

 @param size - target size, capped at {@link MAX_BINARY_BYTES}

 @returns binary bytes

 @example
 ```ts
 synthesizeBinary({ random, size: 128 });
 ```
 */
export function synthesizeBinary({
  random,
  size,
}: Readonly<{
  random: SeededRandom;
  size: number;
}>,): Buffer {
  /**
   Byte count, at least two so the NUL marker leaves room for payload.
   */
  const length = Math.max(
    2,
    Math.min(
      size,
      MAX_BINARY_BYTES,
    ),
  );
  return Buffer.from(Array.from(
    { length, },
    function byte(
      _unused,
      index,
    ) {
      return index === 0 ? 0 : random.integer({
        min: 0,
        max: BYTE_MAX,
      },);
    },
  ),);
}

//endregion Binary
