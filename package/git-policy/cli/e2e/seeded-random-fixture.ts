/**
 Seeded pseudo-random source for replayable end-to-end workloads.

 Every workload decision draws from a generator derived from the run seed and a stable label,
 so filtering scenarios or Git versions never shifts another scenario's decisions,
 and one printed seed replays the same workload.

 @module
 */

//region Types

/**
 Deterministic random source.
 */
export type SeededRandom = Readonly<{
  /**
   Seed this source started from.
   */
  seed: number;
  /**
   Returns the next float in `[0, 1)`.
   */
  next: () => number;
  /**
   Returns an integer in the inclusive range.
   */
  integer: (range: Readonly<{
    min: number;
    max: number
  }>,) => number;
  /**
   Returns one element of a non-empty array.
   */
  pick: <const Element,>(items: readonly Element[],) => Element;
  /**
   Returns a shuffled copy.
   */
  shuffle: <const Element,>(items: readonly Element[],) => readonly Element[];
  /**
   Returns an independent source derived from this seed and a label.
   */
  fork: (label: string,) => SeededRandom;
}>;

//endregion Types

//region Errors

/**
 Seed text or a random request is invalid.
 */
export class SeededRandomError extends Error {
  /**
   Creates a seeded-random error.

   @param message - reason

   @example
   ```ts
   throw new SeededRandomError('empty pick');
   ```
   */
  constructor(message: string,) {
    super(message,);
    this.name = 'SeededRandomError';
  }
}

//endregion Errors

//region Hashing and generation

/**
 Largest seed value plus one; seeds are unsigned 32-bit integers.
 */
export const SEED_MODULUS = 4_294_967_296;

/**
 The only characters a seed's text may hold.
 */
const DECIMAL_DIGITS = '0123456789';

/**
 FNV-1a 32-bit offset basis.
 */
const FNV_OFFSET = 0x81_1C_9D_C5;

/**
 FNV-1a 32-bit prime.
 */
const FNV_PRIME = 0x01_00_01_93;

/**
 Mulberry32 increment constant.
 */
const MULBERRY_INCREMENT = 0x6D_2B_79_F5;

/**
 Mulberry32 first shift.
 */
const SHIFT_A = 15;

/**
 Mulberry32 second shift.
 */
const SHIFT_B = 7;

/**
 Mulberry32 final shift.
 */
const SHIFT_C = 14;

/**
 Mulberry32 mixing multiplier.
 */
const MIX_MULTIPLIER = 61;

/**
 Derives a child seed from a seed and a label with FNV-1a over UTF-16 code units.

 @param seed - parent seed

 @param label - stable label naming what the child decides

 @returns unsigned 32-bit child seed

 @example
 ```ts
 deriveSeed({ seed: 42, label: 'scenario:baseline' });
 ```
 */
export function deriveSeed({
  seed,
  label,
}: Readonly<{
  seed: number;
  label: string;
}>,): number {
  /**
   Text hashed so equal seeds and labels always collide.
   */
  const text = `${String(seed,)}:${label}`;
  return Array.from(
    { length: text.length, },
    function codeUnit(
      _unused,
      index,
    ) {
    // Labels are ASCII, so code points and code units coincide.
      return text.codePointAt(index,) ?? 0;
    },
  )
    .reduce(
    function fnvStep(
      hash,
      unit,
    ) {
      return Math.imul(
        hash ^ unit,
        FNV_PRIME,
      ) >>> 0;
    },
    FNV_OFFSET,
    );
}

/**
 Creates a Mulberry32 generator.

 @param seed - unsigned 32-bit seed

 @returns deterministic source

 @throws {@link SeededRandomError} when the seed is not an unsigned 32-bit integer

 @example
 ```ts
 createSeededRandom(42).integer({ min: 1, max: 6 });
 ```
 */
export function createSeededRandom(seed: number,): SeededRandom {
  if ((!Number.isSafeInteger(seed,)) || (seed < 0)
    || (seed >= SEED_MODULUS))
    throw new SeededRandomError(`seed must be an unsigned 32-bit integer, got ${String(seed,)}`,);
  /**
   Generator state advanced by every draw.
   */
  const state = { value: seed, };
  /**
   Draws the next float.

   @returns float in `[0, 1)`

   @example
   ```ts
   next();
   ```
   */
  function next(): number {
    state.value = (state.value + MULBERRY_INCREMENT) >>> 0;
    /**
     First mixing round.
     */
    const first = Math.imul(
      state.value ^ (state.value >>> SHIFT_A),
      state.value | 1,
    );
    /**
     Second mixing round.
     */
    const second = first ^ (first + Math.imul(
      first ^ (first >>> SHIFT_B),
      first | MIX_MULTIPLIER,
    ));
    return ((second ^ (second >>> SHIFT_C)) >>> 0) / SEED_MODULUS;
  }
  /**
   Draws an inclusive integer.

   @param min - smallest result

   @param max - largest result

   @returns integer within bounds

   @example
   ```ts
   integer({ min: 0, max: 3 });
   ```
   */
  function integer({
    min,
    max,
  }: Readonly<{
    min: number;
    max: number
  }>,): number {
    if ((!Number.isSafeInteger(min,)) || (!Number.isSafeInteger(max,))
      || (max < min))
      throw new SeededRandomError(`invalid integer range ${String(min,)}..${String(max,)}`,);
    return min + Math.floor(next() * ((max - min) + 1),);
  }
  return {
    seed,
    next,
    integer,
    pick<const Element,>(items: readonly Element[],): Element {
      // The empty case is refused before any draw, so the caller sees the argument at fault and the sequence is untouched.
      if (items.length === 0)
        throw new SeededRandomError('cannot pick from an empty array',);
      /**
       Chosen element.
       */
      const chosen = items[integer({
        min: 0,
        max: items.length - 1,
      },)];
      if (chosen === undefined)
        throw new SeededRandomError('cannot pick from an empty array',);
      return chosen;
    },
    shuffle<const Element,>(items: readonly Element[],): readonly Element[] {
      return items
        .map(function keyed(item,): Readonly<{
          key: number;
          item: Element
        }> {
          return {
            key: next(),
            item,
          };
        },)
        .toSorted(function byKey(
          left,
          right,
        ) {
          return left.key - right.key;
        },)
        .map(function unkeyed(entry,) {
          return entry.item;
        },);
    },
    fork(label: string,): SeededRandom {
      return createSeededRandom(deriveSeed({
        seed,
        label,
      },),);
    },
  };
}

//endregion Hashing and generation

//region Seed text

/**
 Parses a seed from task input,
 generating a fresh one when absent.

 @param text - decimal seed text, or empty for a fresh seed

 @param fresh - source of a fresh seed, injectable for tests

 @returns seed and whether it was generated

 @throws {@link SeededRandomError} on non-decimal or out-of-range text

 @example
 ```ts
 parseSeed({ text: '42', fresh: () => 7 }); // => { seed: 42, generated: false }
 ```
 */
export function parseSeed({
  text,
  fresh,
}: Readonly<{
  text: string;
  fresh: () => number;
}>,): Readonly<{
  seed: number;
  generated: boolean;
}> {
  if (text.trim() === '')
    return {
      seed: fresh(),
      generated: true,
    };
  /**
   Seed text without surrounding whitespace.
   */
  const trimmed = text.trim();
  // `Number` also reads hexadecimal, binary, octal, exponent and signed spellings, so the digits are checked first.
  /**
   Parsed decimal seed.
   */
  const seed = trimmed.split('',)
    .every(function isDecimalDigit(character,) {
      return DECIMAL_DIGITS.includes(character,);
    },) ? Number(trimmed,) : Number.NaN;
  if ((!Number.isSafeInteger(seed,)) || (seed < 0)
    || (seed >= SEED_MODULUS))
    throw new SeededRandomError(`seed must be a decimal unsigned 32-bit integer, got ${text}`,);
  return {
    seed,
    generated: false,
  };
}

//endregion Seed text
