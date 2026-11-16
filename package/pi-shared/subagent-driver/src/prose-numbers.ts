/**
 Reads the counts and commit hashes a review's prose statement gives: an "N of M" coverage pair, and the hexadecimal hash a line
 names.

 @module
 */

/**
 The word between an "N of M" line's two counts.
 */
const COUNTS_SEPARATOR = ' of ';

/**
 The shortest and longest commit hashes git writes.
 */
const SHORTEST_COMMIT_HASH = 7;
/**
 The longest commit hash git writes.
 */
const LONGEST_COMMIT_HASH = 40;

/**
 No coverage counts on a line.
 */
export const NO_COUNTS = { kind: 'no counts', } as const;

/**
 Reads the two counts of an "N of M" line.

 @param line - one line of a review

 @returns the count read whole and the count given, or nothing when the line holds no such pair

 @example
 ```ts
 const counts = countsIn({ line: '**37 of 39**' });
 ```
 */
export function countsIn({ line, }: { readonly line: string },):
  | {
    readonly kind: 'the counts';
    readonly readWhole: number;
    readonly given: number
  }
    | { readonly kind: 'no counts' } {

  /**
   Each place the separator could sit, in order, skipping the ones whose neighbours are not both counts.
   */
  for (
    let at = line.indexOf(COUNTS_SEPARATOR,);
    at !== (-1);
    at = nextSeparatorIn({
      line,
      after: at,
    },)
  ) {
    /**
     The digits just before it.
     */
    const readWhole = digitsEndingAt({
      text: line,
      at,
    },);
    /**
     The digits just after it.
     */
    const given = digitsStartingAt({
      text: line,
      at: at + COUNTS_SEPARATOR.length,
    },);
    if ((readWhole.kind === 'digits') && (given.kind === 'digits')) {
      return {
        kind: 'the counts',
        readWhole: Number(readWhole.text,),
        given: Number(given.text,),
      };
    }
  }
  return NO_COUNTS;
}

/**
 Reads where the next separator sits after a position.

 @param line - the line

 @param after - where the search starts, exclusive

 @returns the separator's position, or minus one when none follows

 @example
 ```ts
 const at = nextSeparatorIn({ line: '37 of 39', after: 2 });
 ```
 */
function nextSeparatorIn({
  line,
  after,
}: {
  readonly line: string;
  readonly after: number
},): number {
  return line.indexOf(
    COUNTS_SEPARATOR,
    after + 1,
  );
}

/**
 Reads the digits that end at a position.

 @param text - the line

 @param at - where the digits end

 @returns the digits, or nothing when none end there

 @example
 ```ts
 const digits = digitsEndingAt({ text: '**37 of 39**', at: 4 });
 ```
 */
function digitsEndingAt({
  text,
  at,
}: {
  readonly text: string;
  readonly at: number
},):
  | {
    readonly kind: 'digits';
    readonly text: string
  }
    | { readonly kind: 'none' } {
  /**
   Where the digits start.
   */
  const start = startOfDigitsEndingAt({
    text,
    from: at,
  },);
  return (start === at)
    ? { kind: 'none', }
    : {
      kind: 'digits',
      text: text.slice(
        start,
        at,
      ),
    };
}

/**
 Reads where a run of digits that ends at a position starts.

 @param text - the line

 @param from - where the digits end

 @returns where the run starts, or the same position when no digit ends there

 @example
 ```ts
 const start = startOfDigitsEndingAt({ text: '**37 of 39**', from: 4 });
 ```
 */
function startOfDigitsEndingAt({
  text,
  from,
}: {
  readonly text: string;
  readonly from: number
},): number {
  for (let at = from; ; at -= 1) {
    if ((at === 0) || (!isADigit(text.charAt(at - 1,),))) {
      return at;
    }
  }
}

/**
 Reads the digits that start at a position.

 @param text - the line

 @param at - where the digits start

 @returns the digits, or nothing when none start there

 @example
 ```ts
 const digits = digitsStartingAt({ text: '**37 of 39**', at: 9 });
 ```
 */
function digitsStartingAt({
  text,
  at,
}: {
  readonly text: string;
  readonly at: number
},):
  | {
    readonly kind: 'digits';
    readonly text: string
  }
    | { readonly kind: 'none' } {
  /**
   Where the digits end.
   */
  const end = endOfDigitsStartingAt({
    text,
    from: at,
  },);
  return (end === at)
    ? { kind: 'none', }
    : {
      kind: 'digits',
      text: text.slice(
        at,
        end,
      ),
    };
}

/**
 Reads where a run of digits that starts at a position ends.

 @param text - the line

 @param from - where the digits start

 @returns where the run ends, or the same position when no digit starts there

 @example
 ```ts
 const end = endOfDigitsStartingAt({ text: '**37 of 39**', from: 9 });
 ```
 */
function endOfDigitsStartingAt({
  text,
  from,
}: {
  readonly text: string;
  readonly from: number
},): number {
  for (let at = from; ; at += 1) {
    if ((at === text.length) || (!isADigit(text.charAt(at,),))) {
      return at;
    }
  }
}

/**
 Says whether a character is a decimal digit.

 @param character - the character

 @returns whether it is a digit

 @example
 ```ts
 const digit = isADigit('3');
 ```
 */
function isADigit(character: string,): boolean {
  return (character >= '0') && (character <= '9');
}

/**
 Says whether a text is a commit hash: 7 to 40 hexadecimal characters.

 @param text - the text

 @returns whether it is a commit hash

 @example
 ```ts
 const isHash = isACommitHash('abc1234');
 ```
 */
export function isACommitHash(text: string,): boolean {
  if ((text.length < SHORTEST_COMMIT_HASH) || (text.length > LONGEST_COMMIT_HASH)) {
    return false;
  }
  for (const character of text) {
    /**
     Whether the character is a hexadecimal digit.
     */
    const hexadecimal = isADigit(character,)
      || ((character >= 'a') && (character <= 'f'))
      || ((character >= 'A') && (character <= 'F'));
    if (!hexadecimal) {
      return false;
    }
  }
  return true;
}
