/**
 Splits shell source the parser rejected into runs of command words.

 A command that fails to parse still shows the words an agent typed,
 and a guard that reads nothing from it answers "no" for a ban the visible text violates.
 This reader is deliberately crude:
 it tracks quotes,
 escapes,
 and the characters Bash reads as command separators,
 and it reports each run of words that begins where a command can begin.
 Anything it cannot classify stays inside the run it was found in,
 so prose such as `echo "bun test` never reads as a command.
 
 A here-document body is read as command text rather than skipped,
 because this reader never follows the redirect that declares it;
 a caller reads this fallback as a reason to block, never as proof of what runs.

 @module
 */

//region Constants

/**
 Characters that end the current run of words and let the next word start a command.
 */
const COMMAND_SEPARATOR_CHARACTERS: ReadonlySet<string> = new Set([
  '\n',
  '\r',
  '&',
  '(',
  ')',
  ';',
  '|',
  '{',
  '}',
],);

/**
 Characters that end one word without ending the run.
 */
const WORD_SEPARATOR_CHARACTERS: ReadonlySet<string> = new Set([
  '\t',
  ' ',
],);

/**
 Quote character that opens the region where no character is special but the closing quote.
 */
const SINGLE_QUOTE = '\'';

/**
 Quote character that opens the region where a backslash still escapes.
 */
const DOUBLE_QUOTE = '"';

/**
 Character that makes the next one literal outside single quotes.
 */
const BACKSLASH = '\\';

//endregion Constants

//region Types

/**
 Reader state for one source text: finished runs, the run and word being read, and quoting.
 */
type RawRunScanState = {
  /**
   Complete runs of words, each one starting where a command can begin.
   */
  runs: string[][];
  /**
   Words of the run still being read.
   */
  run: string[];
  /**
   Text segments of the word still being read, with the quote characters left out.
   */
  segments: string[];
  /**
   Offset where the segment being read starts, or -1 when none is open.
   */
  segmentStart: number;
  /**
   Quote character the reader is inside, or empty text outside quotes.
   */
  openQuote: string;
  /**
   Whether the previous character was a backslash, which makes the current one literal.
   */
  escaped: boolean;
};

//endregion Types

//region Word runs

/**
 Splits one source text into runs of command words.
 
 @param source - shell source text, usually one the parser rejected
 
 @returns word runs in source order, each beginning where a command can begin
 
 @example
 ```ts
 rawCommandWordRuns({ source: 'cd x && bun test "', }); // [['cd', 'x'], ['bun', 'test']]
 ```
 */
function rawCommandWordRuns({ source, }: { readonly source: string; },): readonly (readonly string[])[] {
  /**
   State accumulated while the source is read.
   */
  const state: RawRunScanState = {
    runs: [],
    run: [],
    segments: [],
    segmentStart: -1,
    openQuote: '',
    escaped: false,
  };

  /**
   Keeps the text read so far from the segment being collected.
   
   @param offset - source offset that ends the segment
   */
  function sealSegment(offset: number,): void {
    /**
     Offset where the segment being read starts, or -1 when none is open.
     */
    const start = state.segmentStart;
    if (start === (-1))
      return;
    /**
     Segments of the word still being read.
     */
    const { segments, } = state;
    /**
     Segment text between its start and the character that ended it.
     */
    const text = source.slice(
      start,
      offset,
    );
    segments.push(text,);
    state.segmentStart = -1;
  }

  /**
   Emits the word being read into the current run when it holds text.
   
   @param offset - source offset that ends the word
   */
  function endWord(offset: number,): void {
    sealSegment(offset,);
    /**
     Segments of the word still being read.
     */
    const { segments, } = state;
    if (segments.length === 0)
      return;
    /**
     Word text with the quote characters left out.
     */
    const word = segments.join('',);
    /**
     Words of the run still being read.
     */
    const { run, } = state;
    run.push(word,);
    state.segments = [];
  }

  /**
   Emits the current run and lets the next word start a fresh one.
   
   @param offset - source offset that ends the run
   */
  function endRun(offset: number,): void {
    endWord(offset,);
    /**
     Words of the run still being read.
     */
    const { run, } = state;
    if (run.length === 0)
      return;
    /**
     Runs read so far.
     */
    const { runs, } = state;
    runs.push(run,);
    state.run = [];
  }

  for (let offset = 0; offset < source.length; offset += 1) {
    /**
     Character at the current offset.
     */
    const character = source[offset];
    if (character === undefined)
      break;

    if (state.escaped) {
      state.escaped = false;
      if (state.segmentStart === (-1))
        state.segmentStart = offset;
      continue;
    }
    if ((character === BACKSLASH) && (state.openQuote !== SINGLE_QUOTE)) {
      // The backslash itself is not word text: it makes the next character literal.
      sealSegment(offset,);
      state.escaped = true;
      continue;
    }
    if (state.openQuote !== '') {
      if (character === state.openQuote) {
        sealSegment(offset,);
        state.openQuote = '';
        continue;
      }
      if (state.segmentStart === (-1))
        state.segmentStart = offset;
      continue;
    }
    if ((character === SINGLE_QUOTE) || (character === DOUBLE_QUOTE)) {
      sealSegment(offset,);
      state.openQuote = character;
      continue;
    }
    if (WORD_SEPARATOR_CHARACTERS.has(character,)) {
      endWord(offset,);
      continue;
    }
    if (COMMAND_SEPARATOR_CHARACTERS.has(character,)) {
      endRun(offset,);
      continue;
    }
    if (state.segmentStart === (-1))
      state.segmentStart = offset;
  }
  endRun(source.length,);

  return state.runs;
}

//endregion Word runs

export { rawCommandWordRuns, };
