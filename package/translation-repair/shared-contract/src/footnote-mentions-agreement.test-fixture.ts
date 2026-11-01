/**
 The texts and readers behind the agreement test of the footnote mention scan
 and the footnote graph: the two probe tables the lead measured by hand, and a
 seeded generator of URL-shaped texts with marker shapes in and beside them.
 Nothing here is a fixed answer: each text is read by BOTH readers on the
 current build, so a change in either parser moves the comparison and never
 leaves a stored expectation behind. The texts are cat-themed invention.

 @module
 */

import { footnoteMentions, } from '@monochromatic-dev/translation-repair-document-structure';
import { parseDocument, } from '../dist/final/node/index.mjs';

/**
 Characters that may stand before a bare literal, each a different class to
 the parse: nothing, whitespace, a Han character, a digit, a full stop, a
 full-width colon, the opening punctuation GFM names, a bracket, a closing
 bracket, a letter of each case.
 */
const PRECEDERS: readonly string[] = [
  '',
  ' ',
  '猫',
  '7',
  '.',
  '：',
  '(',
  '*',
  '_',
  '[',
  ']',
  '~',
  't',
  'T',
  '<',
  '>',
  '`',
  '\\',
  '"',
  '）',
  '!',
];

/**
 Beginnings of a URL the generator writes, each with a marker shape inside.
 */
const URL_BODIES: readonly string[] = [
  'https://cat.example/[^9]x',
  'HTTP://cat.example/[^9]x',
  'www.cat.example/[^9]x',
  'WWW.cat.example/[^9]x',
  'https://[^9]x',
  'www.[^9]x',
  'https://c[^9]',
  'https://cat.example[^9]',
  'xhttps://cat.example/[^9]',
  'ab:[^9]',
];

/**
 Pieces the generator joins: words, marker shapes, URL openings, closers and
 openers of a link, an image, an angle autolink and a code span, a break, a
 fence, an indent, a block quote and list opener, a comment, a raw HTML tag,
 and the punctuation that ends or splits a URL.
 */
const PIECES: readonly string[] = [
  'cat',
  '猫',
  ' ',
  ' ',
  '\n',
  '.',
  ')',
  '(',
  '）',
  '[^9]',
  '[^8]',
  '[^3]',
  'https://',
  'http://',
  'www.',
  'cat.example/',
  '[a](',
  '](',
  '[a](u)',
  '![a](u)',
  '![a [^9]](u)',
  '<',
  '>',
  '"',
  '`',
  '\\',
  ']',
  '[',
  '[a]: https://c.example/[^9]',
  '```',
  '~~~',
  '\n    ',
  '\n> ',
  '\n- ',
  '<!--',
  '-->',
  '<span title="',
  '</span>',
];

/**
 Characters in a scheme one more than the longest an angle autolink takes.
 */
const BEYOND_SCHEME_LENGTH = 33;

/**
 The edge texts of the mention scan's case about a URL's edges, and the
 tail and validity shapes of a bare literal, as the lead probed them.
 */
export const EDGE_TEXTS: readonly string[] = [
  '[a](https://c.example/(x)[^9]y) [^3]',
  String.raw`[a](https://c.example/\)[^9]) [^3]`,
  'A HTTPS://C.EXAMPLE/[^9] [^3]',
  'A <ab:[^9]> [^3]',
  'A <a:[^9]> [^3]',
  `A <${'a'.repeat(BEYOND_SCHEME_LENGTH,)}:[^9]> [^3]`,
  'A xhttps://c.example/[^9] [^3]',
  'A [a]( [^9]) [^3]',
  'A [a](<cat [^9] [^3]',
  'A [a](<cat\n[^9]> [^3]',
  'A <https://c.example/ [^9]> [^3]',
  'A <https://c.example/<[^9]> [^3]',
  'A <https://c.example/[^9]',
  'A https://[^9]x [^3]',
  'A https://c.example/x[^9] [^3]',
  'A https://c.example/x[^9]. [^3]',
  'A https://c.example[^9] [^3]',
  'A https://c[^9] [^3]',
  'A www.[^9]x [^3]',
  'A www.c[^9] [^3]',
  'A www.c.example[^9] [^3]',
  'A https://c.example/x<[^9] [^3]',
  'A https://c.example/x>[^9] [^3]',
  'A https://c.example/x"[^9] [^3]',
  'A https://c.example/x）[^9] [^3]',
  'A `https://c.example/[^9]` [^3]',
  'A [see https://c.example/[^9] here](u) [^3]',
  'A **https://c.example/[^9]** [^3]',
];

/**
 Texts that put a marker shape in a URL after each preceder, from the lead's
 table of literal beginnings.

 @returns One text per preceder and URL body

 @example
 ```ts
 const texts = precederTexts();
 ```
 */
export function precederTexts(): readonly string[] {
  return PRECEDERS.flatMap(function afterPreceder(before,): string[] {
    return URL_BODIES.map(function withBody(body,): string {
      return `${before}${body} naps.`;
    },);
  },);
}

/**
 Multiplier of the linear congruential generator behind the seeded stream
 (Numerical Recipes), which the stream uses for its choices only.
 */
const GENERATOR_MULTIPLIER = 1_664_525;

/**
 Increment of the same generator.
 */
const GENERATOR_INCREMENT = 1_013_904_223;

/**
 Numbers the generator's state can take, which is also the divisor that maps a
 state into `[0, 1)`.
 */
const GENERATOR_RANGE = 4_294_967_296;

/**
 Makes a deterministic stream of numbers in `[0, 1)` from one seed, so a
 generated table is the same on every run.

 @param seed - integer the stream starts from

 @returns A function giving the next number each call

 @example
 ```ts
 const draw = seededStream({ seed: 7, },);
 ```
 */
function seededStream({ seed, }: { readonly seed: number; },): () => number {
  /**
   State the stream advances, held in a property so the closure holds no `let`.
   */
  const stream = { state: seed % GENERATOR_RANGE, };
  return function nextNumber(): number {
    /**
     Next state before it is brought into `[0, GENERATOR_RANGE)`, which the
     signed product may leave below zero.
     */
    const advanced = (Math.imul(
      stream.state,
      GENERATOR_MULTIPLIER,
    ) + GENERATOR_INCREMENT) % GENERATOR_RANGE;
    stream.state = (advanced + GENERATOR_RANGE) % GENERATOR_RANGE;
    return stream.state / GENERATOR_RANGE;
  };
}

/**
 Picks one element by a number of the stream.

 @param pool - elements to pick from

 @param roll - number in `[0, 1)`

 @returns The picked element

 @throws {@link Error} when the pool is empty

 @example
 ```ts
 const piece = pick({ pool: ['a', 'b',], roll: 0.7, },);
 ```
 */
function pick(
  {
    pool,
    roll,
  }: {
    readonly pool: readonly string[];
    readonly roll: number;
  },
): string {
  /**
   Element at the rolled index.
   */
  const picked = pool[Math.floor(roll * pool.length,)];
  if (picked === undefined)
    throw new Error('unreachable: a roll in [0, 1) indexes a non-empty pool',);
  return picked;
}

/**
 Generates URL-shaped texts with marker shapes in and beside them: a
 preceder, a URL body, then a few pieces of the alphabet, every text ending
 in a marker the parse always reads.

 @param seed - seed of the stream

 @param count - texts to generate

 @param maxPieces - most pieces after the URL body

 @returns The texts, the same on every run for one seed

 @example
 ```ts
 const texts = generatedTexts({ seed: 1, count: 3_000, maxPieces: 6, },);
 ```
 */
export function generatedTexts(
  {
    seed,
    count,
    maxPieces,
  }: {
    readonly seed: number;
    readonly count: number;
    readonly maxPieces: number;
  },
): readonly string[] {
  /**
   The stream every choice draws from.
   */
  const draw = seededStream({ seed, },);
  return Array.from(
    { length: count, },
    function generated(): string {
      /**
       Pieces after the URL body.
       */
      const tail = Array.from(
        { length: Math.floor(draw() * (maxPieces + 1),), },
        function piece(): string {
          return pick({
            pool: PIECES,
            roll: draw(),
          },);
        },
      );
      return [
        pick({
          pool: PRECEDERS,
          roll: draw(),
        },),
        pick({
          pool: URL_BODIES,
          roll: draw(),
        },),
        ...tail,
        ' [^3]',
      ].join('',);
    },
  );
}

/**
 What both readers say about one text.
 */
export type Reading = {
  /**
   The text read.
   */
  readonly text: string;

  /**
   GFM references the footnote graph reads off the parse, identifiers sorted.
   */
  readonly graph: readonly string[];

  /**
   GFM identifiers the mention scan counts as references, sorted.
   */
  readonly scan: readonly string[];
};

/**
 Reads one text with both readers.

 @param text - text to read

 @returns Both readers' GFM identifiers

 @example
 ```ts
 const { graph, scan, } = readBoth({ text: 'A cat [^3].', },);
 ```
 */
export function readBoth({ text, }: { readonly text: string; },): Reading {
  return {
    text,
    graph: parseDocument({ text, },)
      .footnoteGraph
      .references
      .filter(function isGfm(reference,): boolean {
        return reference.convention === 'gfm';
      },)
      .map(function identifierOf(reference,): string {
        return reference.identifier;
      },)
      .toSorted(),
    scan: footnoteMentions({ text, },)
      .filter(function isGfmReference(mention,): boolean {
        return (mention.convention === 'gfm') && (mention.role === 'reference');
      },)
      .map(function identifierOf(mention,): string {
        return mention.identifier;
      },)
      .toSorted(),
  };
}

/**
 How the scan stands to the graph on one text.
 */
type Standing = 'agree' | 'counting' | 'dropping';

/**
 Whether the first sorted list is a sub-multiset of the second.

 @param part - sorted identifiers

 @param whole - sorted identifiers

 @returns Whether every identifier of the part is matched in the whole

 @example
 ```ts
 const within = isSubMultiset({ part: ['3',], whole: ['3', '9',], },);
 ```
 */
function isSubMultiset(
  {
    part,
    whole,
  }: {
    readonly part: readonly string[];
    readonly whole: readonly string[];
  },
): boolean {
  /**
   Whole's identifiers not yet matched.
   */
  const remaining = [...whole,];
  return part.every(function matched(identifier,): boolean {
    /**
     Where this identifier stands among those left.
     */
    const at = remaining.indexOf(identifier,);
    if (at === (-1))
      return false;
    remaining.splice(
      at,
      1,
    );
    return true;
  },);
}

/**
 Places a reading: the scan counts what the graph reads (`agree`), counts
 more (`counting`, the direction the guards can afford), or lacks a reference
 the graph reads (`dropping`, the direction they cannot).

 @param reading - both readers' answers on one text

 @returns The scan's standing

 @example
 ```ts
 const standing = standingOf({ reading: readBoth({ text, },), },);
 ```
 */
export function standingOf({ reading, }: { readonly reading: Reading; },): Standing {
  /**
   The graph's list as one string.
   */
  const graphKey = reading.graph
    .join('\n',);
  /**
   The scan's list as one string.
   */
  const scanKey = reading.scan
    .join('\n',);
  if (graphKey === scanKey)
    return 'agree';
  return isSubMultiset({
    part: reading.graph,
    whole: reading.scan,
  },)
    ? 'counting'
    : 'dropping';
}
