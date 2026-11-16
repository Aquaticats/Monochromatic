/**
 Reads the two lines of a review's own file that say what it covered: the ranges it was reviewed over, and how many of the files
 its brief listed it read in full.

 The brief asks for them at the top of the review file, the first copied from the brief and the second the reviewer's count:

 ```text
 Ranges reviewed: `main...fix/a`
 Files read in full: 12 of 12
 ```

 A range is a pair of Git references, and a reference may hold a comma, a semicolon, an underscore or a backtick, so the line is
 read from the code spans the brief writes every range in: two spans beside each other are two ranges, with nothing removed from
 inside either. Text no code span holds is read best effort, one word per range, because a reference cannot contain a space.

 @module
 */

import { codeSpansIn, } from './markdown-code-spans.ts';
import {
  REVIEW_COMPLETE_LINE,
  REVIEW_IN_PROGRESS_LINE,
} from './review-completion-lines.ts';

/**
 The label the line that names the ranges a review read begins with, as the brief writes it.
 */
export const RANGES_REVIEWED_LABEL = 'Ranges reviewed:';

/**
 The label the line that names how many of the listed files a review read begins with, as the brief writes it.
 */
export const FILES_READ_LABEL = 'Files read in full:';

/**
 The word a stated count puts between how many files were read and how many the brief listed.
 */
const COUNT_JOINER = 'of';

/**
 The word that joins one stated range to the next in text no code span holds, which a reference cannot contain.
 */
const RANGE_JOINER = ' and ';

/**
 The characters that end one stated range in text no code span holds when a space does not.
 */
const RANGE_SEPARATORS: readonly string[] = [
  ',',
  ';',
];

/**
 The runs of emphasis a reviewer may put around a whole stated range in text no code span holds.
 */
const EMPHASIS_RUNS: readonly string[] = [
  '**',
  '__',
  '*',
  '_',
];

/**
 The characters that may end a sentence after a stated range in text no code span holds, which a reference cannot end with.
 */
const SENTENCE_ENDINGS: ReadonlySet<string> = new Set<string>([
  ',',
  ';',
  '.',
]);

/**
 A space, which a reference cannot contain and which therefore separates one stated range from the next.
 */
const A_SPACE = ' ';

/**
 What a review's own file states about what it covered.
 */
export type ReviewStatement = {
  /**
   The ranges the review says it read, absent when its file states none.
   */
  readonly ranges?: readonly string[];

  /**
   How many of the listed files the review says it read in full, absent when its file states no count.
   */
  readonly filesRead?: number;

  /**
   How many files the review says its brief listed, absent when its file states no count.
   */
  readonly filesTotal?: number;
};

/**
 What a review's own file states about the files it read.
 */
type StatedFileCount =
  | {
    /**
     The review states both numbers of a count.
     */
    readonly kind: 'counted';

    /**
     How many of the listed files the review says it read in full.
     */
    readonly filesRead: number;

    /**
     How many files the review says its brief listed.
     */
    readonly filesTotal: number;
  }
    | {
      /**
       The review states no count.
       */
      readonly kind: 'no-count';
    };

/**
 Reads the text a label is followed by on one line, whatever markdown decorates the line.

 @param line - one line of a review's own file

 @param label - the label, with its colon

 @returns the text after the label, or empty text when the line does not carry the label

 @example
 ```ts
 const text = textAfterLabelIn({ line: '**Ranges reviewed:** `main...fix/a`', label: RANGES_REVIEWED_LABEL, },);
 ```
 */
function textAfterLabelIn({
  line,
  label,
}: {
  readonly line: string;
  readonly label: string;
}): string {
  /**
   Where the label is written, whatever case the reviewer wrote it in.
   */
  const at = line.toLowerCase()
    .indexOf(label.toLowerCase(),);
  return (at === (-1))
    ? ''
    : line.slice(at + label.length,);
}

/**
 Says whether a character is one of the digits a stated count is written with.

 @param character - the character to read

 @returns whether it is a digit

 @example
 ```ts
 const isADigit = isDigit('4');
 ```
 */
function isDigit(character: string,): boolean {
  return (character >= '0') && (character <= '9');
}

/**
 Reads where the first number of a stated count begins.

 @param text - the text a count is written in

 @returns the index of its first digit, or minus one when it holds none

 @example
 ```ts
 const from = firstDigitIn(' 9 of 12');
 ```
 */
function firstDigitIn(text: string,): number {
  for (let index = 0; index < text.length; index += 1) {
    if (isDigit(text.charAt(index,),)) {
      return index;
    }
  }
  return -1;
}

/**
 Reads where the run of digits that starts at a place ends.

 @param text - the text the run is in

 @param from - where the run starts, which is a digit

 @returns the index after its last digit

 @example
 ```ts
 const end = endOfDigitsIn({ text: ' 9 of 12', from: 1, },);
 ```
 */
function endOfDigitsIn({
  text,
  from,
}: {
  readonly text: string;
  readonly from: number;
}): number {
  for (let index = from; index < text.length; index += 1) {
    if (!isDigit(text.charAt(index,),)) {
      return index;
    }
  }
  return text.length;
}

/**
 Reads the count of files a review states, out of the text after the label.

 The two numbers must be joined by the word a count puts between them, so a line that happens to hold two numbers while stating no
 count is not read as one.

 @param text - the text after the label

 @returns the count the line states, or that it states none

 @example
 ```ts
 const counted = countStatedIn(' 9 of 12 listed files read in full');
 ```
 */
function countStatedIn(text: string,): StatedFileCount {
  /**
   Where the number of files read begins.
   */
  const readStart = firstDigitIn(text,);
  if (readStart === (-1)) {
    return { kind: 'no-count', };
  }
  /**
   Where the number of files read ends.
   */
  const readEnd = endOfDigitsIn({
    text,
    from: readStart,
  },);
  /**
   What follows the number of files read.
   */
  const rest = text.slice(readEnd,);
  /**
   Where the number of files listed begins.
   */
  const listedStart = firstDigitIn(rest,);
  if (listedStart === (-1)) {
    return { kind: 'no-count', };
  }
  if (rest.slice(
    0,
    listedStart,
  )
    .trim()
    !== COUNT_JOINER) {
    return { kind: 'no-count', };
  }
  return {
    kind: 'counted',
    filesRead: Number(text.slice(
      readStart,
      readEnd,
    ),),
    filesTotal: Number(rest.slice(
      listedStart,
      endOfDigitsIn({
        text: rest,
        from: listedStart,
      },),
    ),),
  };
}

/**
 Reads a stated range in text no code span holds, without the emphasis or the sentence's end a reviewer may have put around it.

 Such text is read best effort: a reference may hold a comma, a semicolon or an underscore, so a range that holds one is only read
 byte for byte when it is written in a code span, which is how the brief writes every range.

 @param piece - one word of the text after the ranges label

 @returns the range it states

 @example
 ```ts
 const range = statedRangeIn('**main...fix/a**,');
 ```
 */
function statedRangeIn(piece: string,): string {
  return withoutEmphasisAround(withoutSentenceEnding(piece,),);
}

/**
 Reads a line without the emphasis a reviewer may have put around the whole of it.

 @param text - the line's text, trimmed

 @returns the text without a matched run of emphasis at its two ends

 @example
 ```ts
 const line = withoutEmphasisAround('**Review complete: every run it started has finished.**');
 ```
 */
function withoutEmphasisAround(text: string,): string {
  for (const run of EMPHASIS_RUNS) {
    if ((text.length > (2 * run.length)) && text.startsWith(run,)
      && text.endsWith(run,)) {
      return text.slice(
        run.length,
        -run.length,
      );
    }
  }
  return text;
}

/**
 Reads a word without the comma, semicolon or full stop a sentence may put at either end of it, which a reference cannot end with.

 @param text - the word to read

 @returns the word without them

 @example
 ```ts
 const word = withoutSentenceEnding('main...fix/a,');
 ```
 */
function withoutSentenceEnding(text: string,): string {
  return text.slice(
    startAfterSentenceEndings(text,),
    endBeforeSentenceEndings(text,),
  );
}

/**
 Reads where a word begins, past the sentence's end a reviewer may have put before it.

 @param text - the word to read

 @returns the index of its first character that is not a sentence's end

 @example
 ```ts
 const from = startAfterSentenceEndings(',main...fix/a');
 ```
 */
function startAfterSentenceEndings(text: string,): number {
  for (let index = 0; index < text.length; index += 1) {
    if (!SENTENCE_ENDINGS.has(text.charAt(index,),)) {
      return index;
    }
  }
  return text.length;
}

/**
 Reads where a word ends, before the sentence's end a reviewer may have put after it.

 @param text - the word to read

 @returns the index after its last character that is not a sentence's end

 @example
 ```ts
 const to = endBeforeSentenceEndings('main...fix/a.');
 ```
 */
function endBeforeSentenceEndings(text: string,): number {
  for (let index = text.length - 1; index >= 0; index -= 1) {
    if (!SENTENCE_ENDINGS.has(text.charAt(index,),)) {
      return index + 1;
    }
  }
  return 0;
}

/**
 Reads the ranges a review states, out of the text after the label.

 A reference may hold a comma, a semicolon, an underscore or a backtick, so those cannot separate two ranges. The brief writes
 every range in a code span, and two spans beside each other are read as two ranges with nothing removed from inside either. Text
 no code span holds is read best effort, one word per range, because a reference cannot contain a space.

 @param text - the text after the label

 @returns the ranges stated, each once and in the order written, and nothing when none is left

 @example
 ```ts
 const ranges = rangesStatedIn(' `main...fix/a`, `main...fix/b`');
 ```
 */
function rangesStatedIn(text: string,): readonly string[] {
  // A code span is copied text: the only characters this may drop are the fence and the one padding space CommonMark adds for a
  // text that begins or ends with a backtick, both of which `codeSpansIn` has already removed. A reference may end in any byte
  // Git allows, U+00A0 included, and `trim()` would eat it, so nothing inside a span is cut.
  /**
   The code spans the text holds, which are the ranges the brief wrote.
   */
  const spans = codeSpansIn({ text, },)
    .filter(function isARange(span,) {
      return span !== '';
    },);
  if (spans.length > 0) {
    return spans;
  }
  return text.split(A_SPACE,)
    .map(function asARange(piece,) {
      return statedRangeIn(piece,);
    },)
    .filter(function isARange(piece,) {
      return piece !== '';
    },);
}

/**
 The characters a fenced code block is made of, either of which opens and closes one.
 */
const FENCE_CHARACTERS: ReadonlySet<string> = new Set<string>([
  '`',
  '~',
]);

/**
 How many of a fence's characters a fenced block is made of at least.
 */
const FENCE_LENGTH = 3;

/**
 How many spaces a line may begin with before it is an indented code block.
 */
const INDENTED_CODE_SPACES = 4;

/**
 How many spaces a block quote's marker may follow, and a fenced block's fence or an indented block's first character.
 */
const SPACES_BEFORE_A_MARKER = 3;

/**
 One line of a message, read as what it is in CommonMark.
 */
type ReadLine =
  | {
    /**
     The line begins a fenced code block, or closes one.
     */
    readonly kind: 'fence';

    /**
     The character its fence is made of.
     */
    readonly character: string;

    /**
     How many of that character its fence holds.
     */
    readonly length: number;

    /**
     Whether nothing but spaces follows the fence, which is what closing one takes.
     */
    readonly restIsBlank: boolean;
  }
    | {
      /**
       The line is no fence.
       */
      readonly kind: 'no-fence';
    };

/**
 Writes what a line is as far as fenced code blocks go.

 @param line - one line of a message

 @returns the fence it opens or closes, or that it is no fence

 @example
 ```ts
 const read = lineReadAsAFence('```text');
 ```
 */
function lineReadAsAFence(line: string,): ReadLine {
  /**
   How many spaces the line begins with.
   */
  const spaces = leadingSpacesIn(line,);
  if (spaces > SPACES_BEFORE_A_MARKER) {
    return { kind: 'no-fence', };
  }
  /**
   The character the line's marker is made of, when it is one a fence is made of.
   */
  const character = line.charAt(spaces,);
  if (!FENCE_CHARACTERS.has(character,)) {
    return { kind: 'no-fence', };
  }
  for (let at = spaces; line.charAt(at,) === character; at += 1) {
    /**
     How many of the character the run under this step holds.
     */
    const length = (at - spaces) + 1;
    if (length >= FENCE_LENGTH) {
      return {
        kind: 'fence',
        character,
        length,
        restIsBlank: line.slice(at + 1,)
          .trim()
          === '',
      };
    }
  }
  return { kind: 'no-fence', };
}

/**
 Reads how many spaces a line begins with.

 @param line - one line of a message

 @returns the number of spaces before its first other character

 @example
 ```ts
 const spaces = leadingSpacesIn('    indented');
 ```
 */
function leadingSpacesIn(line: string,): number {
  for (let at = 0; at < line.length; at += 1) {
    if (line.charAt(at,) !== ' ') {
      return at;
    }
  }
  return line.length;
}

/**
 Says whether a line stands as the message's own text: a paragraph line or a list item, which is a line that is no block quote, no
 fence and no indented code block.

 @param line - one line of a message

 @returns whether the line is the message's own text

 @example
 ```ts
 const counts = standsAsItsOwnText('Ranges reviewed: `main...fix/a`');
 ```
 */
function standsAsItsOwnText(line: string,): boolean {
  if (line.startsWith('\t',)) {
    return false;
  }
  /**
   How many spaces the line begins with.
   */
  const spaces = leadingSpacesIn(line,);
  if (spaces >= INDENTED_CODE_SPACES) {
    return false;
  }
  return line.charAt(spaces,) !== '>';
}

/**
 Reads the lines of a message that stand as its own text.

 A wanted line counts only where it stands as the message's own text: a paragraph line or a list item. It does not count inside a
 fenced code block, whose lines are quoted from elsewhere and run from a fence of three or more backticks or tildes to the next
 fence of the same character and at least that length; nor inside a block quote, whose lines begin with `>`; nor inside an indented
 code block, whose lines begin with four spaces or a tab. A fence nothing closes runs to the end of the message, as CommonMark says.

 @param shown - the message or file to read

 @returns its own lines, in the order they stand

 @example
 ```ts
 const lines = statementLinesIn({ shown: 'Review complete: every run it started has finished.\n', },);
 ```
 */
export function statementLinesIn({ shown, }: { readonly shown: string },): readonly string[] {
  /**
   The fence that is open, and what closes it.
   */
  const fence = {
    open: false,
    character: '',
    length: 0,
  };
  /**
   The lines that stand as the message's own text.
   */
  const lines: string[] = [];
  for (const line of shown.split('\n',)) {
    /**
     What this line is as far as fences go.
     */
    const read = lineReadAsAFence(line,);
    if (fence.open) {
      // A line inside a fence is quoted text, whether or not it closes the fence.
      if ((read.kind === 'fence') && (read.character === fence.character)
        && (read.length >= fence.length)
        && read.restIsBlank) {
        fence.open = false;
      }
      continue;
    }
    if (read.kind === 'fence') {
      fence.open = true;
      fence.character = read.character;
      fence.length = read.length;
      continue;
    }
    if (!standsAsItsOwnText(line,)) {
      continue;
    }
    lines.push(line,);
  }
  return lines;
}

/**
 Reads the ranges a review's own file states, from the first line that states any.

 @param shown - the review's own file, as it was written

 @returns the ranges the first line that states any names, and nothing when no line states one

 @example
 ```ts
 const ranges = rangesStatedFirst('Ranges reviewed: `main...fix/a`\n');
 ```
 */
function rangesStatedFirst(shown: string,): readonly string[] {
  for (const line of statementLinesIn({ shown, },)) {
    /**
     The ranges this line states.
     */
    const ranges = rangesStatedIn(textAfterLabelIn({
      line,
      label: RANGES_REVIEWED_LABEL,
    },),);
    if (ranges.length > 0) {
      return ranges;
    }
  }
  return [];
}

/**
 Reads the count of files a review's own file states, from the first line that states one.

 @param shown - the review's own file, as it was written

 @returns the count the first line that states one names, or that no line states one

 @example
 ```ts
 const counted = countStatedFirst('Files read in full: 12 of 12\n');
 ```
 */
function countStatedFirst(shown: string,): StatedFileCount {
  for (const line of statementLinesIn({ shown, },)) {
    /**
     The count this line states.
     */
    const counted = countStatedIn(textAfterLabelIn({
      line,
      label: FILES_READ_LABEL,
    },),);
    if (counted.kind === 'counted') {
      return counted;
    }
  }
  return { kind: 'no-count', };
}

/**
 Reads what a review's own file states about what it covered.

 @param shown - the review's own file, as it was written

 @returns the ranges and the count it states, each absent when its file states none

 @example
 ```ts
 const stated = statementIn({ shown: 'Ranges reviewed: `main...fix/a`\nFiles read in full: 12 of 12\n', },);
 ```
 */
export function statementIn({ shown, }: { readonly shown: string },): ReviewStatement {
  /**
   The ranges the review states, which are none when it states none.
   */
  const ranges = rangesStatedFirst(shown,);
  /**
   What the review states about the files it read.
   */
  const counted = countStatedFirst(shown,);
  /**
   The statement so far, with the ranges when the review states any.
   */
  const stated: ReviewStatement = (ranges.length === 0) ? {} : { ranges, };
  if (counted.kind === 'no-count') {
    return stated;
  }
  return {
    ...stated,
    filesRead: counted.filesRead,
    filesTotal: counted.filesTotal,
  };
}

/**
 What a review's own file says about whether its work is done: the line the brief gives it while its runs are still going, the line
 that replaces it when every run has finished, or neither.
 */
export type ReviewCompletion =
  | {
    /**
     The file still carries the line that opens a review whose runs are going.
     */
    readonly kind: 'in-progress';
  }
    | {
      /**
       The file carries the line that says every run has finished.
       */
      readonly kind: 'complete';
    }
    | {
      /**
       The file carries neither line.
       */
      readonly kind: 'neither';
    };

/**
 Reads whether a review's own file says its work is done, by the exact words the brief gives it.

 A file that still carries the line that opens a review whose runs are going decides, even when both lines are there: the reviewer
 replaced the file's text and left the opening behind, and a review that says its runs are going has not finished.

 @param shown - the review's own file, as it was written

 @returns which line the file carries, or that it carries neither

 @example
 ```ts
 const completion = completionIn({ shown: 'Review complete: every run it started has finished.\n', },);
 ```
 */
export function completionIn({ shown, }: { readonly shown: string },): ReviewCompletion {
  /**
   Every line that stands as the message's own text, read without the emphasis a reviewer may have put around a whole line.
   */
  const lines = new Set(statementLinesIn({ shown, },)
    .map(function withoutItsEmphasis(line,) {
      return withoutEmphasisAround(line.trim(),);
    },));
  if (lines.has(REVIEW_IN_PROGRESS_LINE,)) {
    return { kind: 'in-progress', };
  }
  return lines.has(REVIEW_COMPLETE_LINE,)
    ? { kind: 'complete', }
    : { kind: 'neither', };
}
