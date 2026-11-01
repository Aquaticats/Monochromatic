//region Destination address texts
// TEST SUPPORT, NOT PACKAGE SOURCE. Texts holding a bare web address with a
// path built from the pieces the autolink literal's trail rule tells apart,
// and the sentence around it that ends the address or does not: sentence
// punctuation, the emphasis marks, balanced and unbalanced parentheses, the
// character reference form (`&amp;`) well formed and malformed, and the
// bracket form (`]` before whitespace, before `(` or `[`, and before
// anything else), and whitespace past ASCII after a path and after a trail.
// The texts are enumerated and sampled by a fixed stride, not
// drawn at random, so a failing text is the same one on every run. They hold
// no character the scanner ends a run at by its own choice (a quotation mark,
// an angle bracket, a backtick, full-width punctuation), where it differs
// from the parse by design. Invented addresses on `c.example`.

/**
 Pieces a path is built from, each a different class to the trail rule.
 */
const PATH_PIECES: readonly string[] = [
  'a',
  'Cat',
  '_',
  '-',
  '/',
  '.',
  '?',
  '=',
  '&',
  '&amp;',
  '&lt;',
  '&#35;',
  '&amp',
  '&1;',
  '&;',
  '%20',
  '(',
  ')',
  '(cat)',
  ',',
  '#',
  ':',
  '!',
  ';',
  '*',
  '~',
  '[',
  ']',
  '](',
  '][',
];

/**
 What follows the address in the text: nothing, whitespace and words, the
 punctuation a sentence ends with, a closing parenthesis, the forms of the
 trail rule, a bracket closing the address's own opening one, and whitespace
 past ASCII (a no-break space, an ideographic space, a line separator), which
 the parse ends an address at as it ends one at a space, after a path and
 after each form of the trail.
 */
const SUFFIXES: readonly string[] = [
  '',
  ' then',
  '.',
  ', then',
  ')',
  ').',
  ' (see)',
  '!',
  '&amp;',
  '&amp;.',
  '&amp; then',
  '&amp;x',
  '&x',
  '&',
  ']',
  '].',
  ']]',
  '] then',
  ']x',
  '.]',
  '_]',
  '.&amp;]',
  '\u{00A0}then',
  '\u{3000}then',
  '.\u{2028}then',
  ']\u{3000}then',
  '&amp;\u{00A0}then',
];

/**
 Text before the address: a sentence's words, and an opening parenthesis.
 */
const PREFIXES: readonly string[] = [
  'see ',
  '(see ',
];

/**
 Stride between sampled paths of the longer lengths, coprime with the number
 of such paths so the sample visits every residue.
 */
const SAMPLE_STRIDE = 7_919;

/**
 Stride between the suffixes the sampled paths take, coprime with the number
 of suffixes so every run of that many positions takes every suffix once.
 */
const SUFFIX_STRIDE = 5;

/**
 Longest path the generator writes.
 */
const LONGEST_PATH = 4;

/**
 Longest path written for every suffix and prefix; longer ones are sampled.
 */
const EXHAUSTIVE_PATH = 2;

/**
 Shortest path of the sample, the length past the exhaustive ones.
 */
const FIRST_SAMPLED_PATH = EXHAUSTIVE_PATH + 1;

/**
 Paths of one length in a fixed order, the index read as digits of a number
 in the base of the pool.

 @param length - pieces in the path

 @param index - which of the paths of that length

 @returns The path

 @example
 ```ts
 const path = pathAt({ length: 2, index: 31, },); // PATH_PIECES[1] then PATH_PIECES[1]
 ```
 */
function pathAt(
  {
    length,
    index,
  }: {
    readonly length: number;
    readonly index: number;
  },
): string {
  return Array.from(
    { length, },
    function digit(
      _unused,
      place,
    ): string {
      return PATH_PIECES[
        Math.floor(index / (PATH_PIECES.length ** place),) % PATH_PIECES.length
      ] ?? '';
    },
  )
    .join('',);
}

/**
 Every path of one length, written in full.

 @param length - pieces in each path

 @returns The paths in a fixed order

 @example
 ```ts
 const paths = pathsOfLength({ length: 2, },);
 ```
 */
function pathsOfLength({ length, }: { readonly length: number; },): readonly string[] {
  return Array.from(
    { length: PATH_PIECES.length ** length, },
    function pathOf(
      _slot,
      index,
    ): string {
      return pathAt({
        length,
        index,
      },);
    },
  );
}

/**
 One text of the sampled longer paths.

 @param position - which of the sampled texts

 @returns The text: a prefix, the address with a path of three or four pieces,
 and a suffix, all chosen by the position. The suffix moves fastest, then the
 prefix, then the path's length, each by its own count of positions, so every
 suffix meets every prefix and every length; chosen by the position's
 remainders alone, the three would move in step, and a suffix would only ever
 meet one prefix

 @example
 ```ts
 const text = sampledText({ position: 3, },);
 ```
 */
function sampledText({ position, }: { readonly position: number; },): string {
  /**
   Lengths the sample covers, the ones past the exhaustive paths.
   */
  const lengths = LONGEST_PATH - EXHAUSTIVE_PATH;

  /**
   Pieces in this sampled path, which moves once every suffix has met every
   prefix.
   */
  const length = FIRST_SAMPLED_PATH
    + (Math.floor(position / (SUFFIXES.length * PREFIXES.length),) % lengths);

  /**
   The path, taken at a fixed stride through the paths of its length.
   */
  const path = pathAt({
    length,
    index: (position * SAMPLE_STRIDE) % (PATH_PIECES.length ** length),
  },);

  /**
   Text before the address, which moves once every suffix has been taken.
   */
  const prefix = PREFIXES[Math.floor(position / SUFFIXES.length,) % PREFIXES.length] ?? '';

  /**
   Text after the address.
   */
  const suffix = SUFFIXES[(position * SUFFIX_STRIDE) % SUFFIXES.length] ?? '';
  return `${prefix}https://c.example/${path}${suffix}`;
}

/**
 The texts a bare address is read over: every path of up to two pieces with
 every suffix after the first prefix, and a fixed-stride sample of the paths
 of three and four pieces with a suffix and a prefix chosen by the sample's
 position.

 @param sampled - how many texts to sample of the longer paths

 @returns Texts in a fixed order

 @example
 ```ts
 const texts = addressTexts({ sampled: 100, },);
 ```
 */
export function addressTexts({ sampled, }: { readonly sampled: number; },): readonly string[] {
  /**
   Texts of the paths written in full.
   */
  const exhaustive = Array.from(
    { length: EXHAUSTIVE_PATH, },
    function ofLength(
      _unused,
      offset,
    ): readonly string[] {
      return pathsOfLength({ length: offset + 1, },);
    },
  )
    .flat()
    .flatMap(function withSuffixes(path,): readonly string[] {
      return SUFFIXES.map(function written(suffix,): string {
        return `see https://c.example/${path}${suffix}`;
      },);
    },);
  return [
    ...exhaustive,
    ...Array.from(
      { length: sampled, },
      function sample(
        _unused,
        position,
      ): string {
        return sampledText({ position, },);
      },
    ),
  ];
}

//endregion Destination address texts
