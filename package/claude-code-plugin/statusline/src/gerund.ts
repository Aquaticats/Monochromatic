/**
 Gerund vocabulary and scanning over lowercased transcript prose.
 
 One linear pass replaces a global `\b[a-z]+-?[a-z]*ing\b` regex.
 A match can only start where a word starts with an ASCII lowercase letter,
 and at such a start the greedy optional hyphen picks a single longest match,
 so attempting a match only at word starts and advancing one character everywhere else reports every
 match the pattern reported,
 in the same order,
 in O(n) time and O(1) stack.
 
 @module
 */

//region Character classes

/**
 Checks whether one character is an ASCII lowercase letter, the only character a gerund body holds.
 
 @param character - one character, or the empty string where an index falls outside the text
 
 @returns whether character sits in `a` through `z`
 
 @example
 ```ts
 isLowerAsciiLetter('c'); // true
 ```
 */
function isLowerAsciiLetter(character: string,): boolean {
  return (character >= 'a') && (character <= 'z');
}

/**
 Checks whether one character is an ASCII uppercase letter.
 
 Scanned text is lowercased first, yet the word class a boundary tests still counts uppercase,
 so the class stays complete rather than assuming the lowercasing removed them.
 
 @param character - one character, or the empty string where an index falls outside the text
 
 @returns whether character sits in `A` through `Z`
 
 @example
 ```ts
 isUpperAsciiLetter('C'); // true
 ```
 */
function isUpperAsciiLetter(character: string,): boolean {
  return (character >= 'A') && (character <= 'Z');
}

/**
 Checks whether one character is an ASCII decimal digit.
 
 @param character - one character, or the empty string where an index falls outside the text
 
 @returns whether character sits in `0` through `9`
 
 @example
 ```ts
 isAsciiDigit('7'); // true
 ```
 */
function isAsciiDigit(character: string,): boolean {
  return (character >= '0') && (character <= '9');
}

/**
 Checks whether one character continues a word, so a gerund can neither start nor end beside it.
 
 The empty string counts as a non-word character, which makes both ends of the text behave as the
 boundary a word edge requires.
 
 @param character - one character, or the empty string where an index falls outside the text
 
 @returns whether character is an ASCII letter, an ASCII digit, or an underscore
 
 @example
 ```ts
 isWordCharacter('_'); // true
 ```
 */
function isWordCharacter(character: string,): boolean {
  return isLowerAsciiLetter(character,)
    || isUpperAsciiLetter(character,)
    || isAsciiDigit(character,)
    || (character === '_');
}

//endregion Character classes

//region Character runs

/**
 Finds where the ASCII lowercase letter run starting at one index ends.
 
 @param text - lowercased text to scan
 
 @param start - index the run begins at, which may already sit past the end
 
 @returns index one past the last lowercase letter of the run, so `start` itself when there is none
 
 @example
 ```ts
 lowerLetterRunEnd({ text: 'testing the parser', start: 0 }); // 7
 ```
 */
function lowerLetterRunEnd({
  text,
  start,
}: Readonly<{
  text: string;
  start: number;
}>,): number {
  /**
   Index the scan has reached.
   */
  let cursor = start;
  while (isLowerAsciiLetter(text.charAt(cursor,),))
    cursor += 1;
  return cursor;
}

//endregion Character runs

//region Gerund matching

/**
 Suffix every gerund carries.
 */
const GERUND_SUFFIX = 'ing';

/**
 Separator inside a hyphenated compound such as `lint-missing`.
 */
const COMPOUND_HYPHEN = '-';

/**
 Shortest letter run that stands alone as a gerund: one stem letter plus the suffix.
 */
const SHORTEST_PLAIN_RUN = GERUND_SUFFIX.length + 1;

/**
 Shortest letter run that follows a hyphen: the suffix alone, since the stem precedes the hyphen.
 */
const SHORTEST_HYPHENATED_RUN = GERUND_SUFFIX.length;

/**
 Checks whether one lowercase letter run is a gerund ending.
 
 The run must be long enough to hold the stem the shape requires, must end with the suffix, and must
 end at a word edge; a run followed by a digit, an underscore, or another letter is inside a longer
 word and is not a gerund.
 
 @param text - lowercased text holding the run
 
 @param runStart - index the run begins at
 
 @param runEnd - index one past the last letter of the run
 
 @param shortestRun - letter count the shape needs before it can carry the suffix
 
 @returns whether the run is a complete gerund
 
 @example
 ```ts
 isGerundRun({ text: 'compiling', runStart: 0, runEnd: 9, shortestRun: SHORTEST_PLAIN_RUN });
 ```
 */
function isGerundRun({
  text,
  runStart,
  runEnd,
  shortestRun,
}: Readonly<{
  text: string;
  runStart: number;
  runEnd: number;
  shortestRun: number;
}>,): boolean {
  if ((runEnd - runStart) < shortestRun)
    return false;
  if (text.slice(
    runEnd - GERUND_SUFFIX.length,
    runEnd,
  ) !== GERUND_SUFFIX)
    return false;
  return !isWordCharacter(text.charAt(runEnd,),);
}

/**
 Measures the gerund starting at one word start.
 
 A hyphen is consumed when the letters after it form a gerund, because the pattern's optional hyphen
 is greedy and the longer compound wins; otherwise the letters before the hyphen stand alone.
 
 @param text - lowercased text to scan
 
 @param start - index of a word start holding a lowercase letter
 
 @returns length of the gerund at `start`, or zero when the text there is not one
 
 @example
 ```ts
 gerundMatchLength({ text: 'lint-missing', start: 0 }); // 12
 ```
 */
function gerundMatchLength({
  text,
  start,
}: Readonly<{
  text: string;
  start: number;
}>,): number {
  /**
   End of the letters before any hyphen.
   */
  const stemEnd = lowerLetterRunEnd({
    text,
    start,
  },);

  if (text.charAt(stemEnd,) === COMPOUND_HYPHEN) {
    /**
     Index the letters after the hyphen begin at.
     */
    const tailStart = stemEnd + 1;
    /**
     End of the letters after the hyphen.
     */
    const tailEnd = lowerLetterRunEnd({
      text,
      start: tailStart,
    },);
    if (isGerundRun({
      text,
      runStart: tailStart,
      runEnd: tailEnd,
      shortestRun: SHORTEST_HYPHENATED_RUN,
    },))
      return tailEnd - start;
  }

  if (isGerundRun({
    text,
    runStart: start,
    runEnd: stemEnd,
    shortestRun: SHORTEST_PLAIN_RUN,
  },))
    return stemEnd - start;
  return 0;
}

/**
 Collects every gerund in lowercased text, in order of appearance.
 
 @param text - lowercased text to scan
 
 @returns gerunds as they appear, including hyphenated compounds and repeats
 
 @example
 ```ts
 scanGerundMatches('compiling, then testing'); // ['compiling', 'testing']
 ```
 */
export function scanGerundMatches(text: string,): readonly string[] {
  /**
   Gerunds found so far, in order of appearance.
   */
  const matches: string[] = [];
  /**
   Index the scan has reached.
   */
  let cursor = 0;
  while (cursor < text.length) {
    /**
     Whether a gerund may start here: a word start holding a lowercase letter.
     */
    const isCandidateStart = isLowerAsciiLetter(text.charAt(cursor,),)
      && (!isWordCharacter(text.charAt(cursor - 1,),));
    if (!isCandidateStart) {
      cursor += 1;
      continue;
    }
    /**
     Length of the gerund at this start, or zero when there is none.
     */
    const length = gerundMatchLength({
      text,
      start: cursor,
    },);

    if (length > 0) {
      matches.push(text.slice(
        cursor,
        cursor + length,
      ),);
      cursor += length;
      continue;
    }
    // Nothing inside a word can start a gerund either, since every index there is preceded by a word
    // character, so the next attempt worth making is the next word start.
    cursor += 1;
  }
  return matches;
}

//endregion Gerund matching

//region Gerund filters

/**
 Words ending in "-ing" that are not meaningful activity verbs.
 */
const NOISE_GERUNDS = new Set([
  'beginning',
  'completing',
  'continuing',
  'ending',
  'finishing',
  'starting',
  'stopping',
  'waiting',
  'pending',
  'asking',
  'calling',
  'coming',
  'doing',
  'getting',
  'giving',
  'going',
  'having',
  'keeping',
  'knowing',
  'letting',
  'looking',
  'making',
  'meaning',
  'putting',
  'saying',
  'seeing',
  'showing',
  'telling',
  'trying',
  'turning',
  'wanting',
  'working',
  'reading',
  'searching',
  'anything',
  'everything',
  'nothing',
  'something',
  'thing',
  'according',
  'assuming',
  'concerning',
  'considering',
  'depending',
  'during',
  'excluding',
  'following',
  'including',
  'providing',
  'regarding',
  'supposing',
  'amazing',
  'annoying',
  'boring',
  'confusing',
  'corresponding',
  'exciting',
  'existing',
  'frustrating',
  'interesting',
  'missing',
  'lint-missing',
  'outstanding',
  'overwhelming',
  'remaining',
  'surprising',
  'surrounding',
  'underlying',
  'bring',
  'cling',
  'fling',
  'king',
  'ring',
  'sing',
  'sling',
  'spring',
  'sting',
  'string',
  'swing',
  'wing',
  'wring',
  'sibling',
  'being',
  'needing',
  'running',
  'thinking',
  'using',
],);

/**
 Minimum word length to consider as a gerund candidate.
 */
const MIN_GERUND_LENGTH = 5;

/**
 Finds the last meaningful gerund in text.
 
 @param value - text to scan
 
 @returns capitalized gerund, or empty string when none is found
 
 @example
 ```ts
 findGerundInText('I am compiling and testing');
 ```
 */
export function findGerundInText(value: string,): string {
  /**
   Lowercased text for case-insensitive matching.
   */
  const lowercaseValue = value.toLowerCase();
  /**
   Raw `-ing` matches across lowercased text.
   */
  const matches = scanGerundMatches(lowercaseValue,);
  /**
   Matches that survive length and noise filters.
   */
  const candidates = matches
    .filter(function isLongEnough(word,): boolean {
      return word.length >= MIN_GERUND_LENGTH;
    },)
    .filter(function isNotNoise(word,): boolean {
      return !NOISE_GERUNDS.has(word,);
    },);

  /**
   Last surviving candidate.
   */
  const last = candidates.at(-1,);
  if (last === undefined)
    return '';

  return last.charAt(0,)
    .toUpperCase()
    + last.slice(1,);
}

//endregion Gerund filters
