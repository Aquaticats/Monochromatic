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

 A backslash before a line break joins the two lines into one word,
 an ANSI-C quoted word (`$'…'`) is read as the text it spells,
 and a `&` that belongs to a redirection (`>&2`, `&>/dev/null`) stays inside its word.
 A scan that ends inside a quote is followed by a second scan that ignores quoting,
 because an unbalanced quote in a command that failed to parse otherwise hides every command
 after it: `cat <<'EOF'` whose body spells `don't` would hide a loop written below the body.

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
 Characters that stand on either side of a redirection and keep its `&` in the same word.
 */
const REDIRECTION_CHARACTERS: ReadonlySet<string> = new Set([
  '<',
  '>',
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

/**
 Character that opens an ANSI-C quoted word when a single quote follows it.
 */
const DOLLAR_SIGN = '$';

/**
 Character that ends a line, which a backslash before it joins to the next one.
 */
const LINE_BREAK = '\n';

/**
 Character a CRLF line ending writes before its line break.
 */
const CARRIAGE_RETURN = '\r';

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
   Whether the open quote is an ANSI-C one, where a backslash still escapes.
   */
  ansiCQuoted: boolean;
  /**
   Whether the previous character was a backslash, which makes the current one literal.
   */
  escaped: boolean;
};

/**
 One pass over a source text: the runs it found and whether it reached the end inside a quote.
 */
type RawRunScan = {
  /**
   Runs of command words the pass reported.
   */
  readonly runs: readonly (readonly string[])[];
  /**
   Whether the source ended inside a quote, which leaves the text after that quote unread.
   */
  readonly endedInsideQuote: boolean;
};

//endregion Types

//region Word runs

/**
 Reads one source text once, either honouring quotes or treating every quote character as text.
 
 @param source - shell source text, usually one the parser rejected
 
 @param honorQuotes - whether quote characters open a quoted region
 
 @returns the runs the pass found and whether it ended inside a quote
 
 @example
 ```ts
 scanCommandWordRuns({ source: 'bun test "', honorQuotes: true, }).endedInsideQuote; // true
 ```
 */
function scanCommandWordRuns({
  source,
  honorQuotes,
}: {
  readonly source: string;
  readonly honorQuotes: boolean;
},): RawRunScan {
  /**
   State accumulated while the source is read.
   */
  const state: RawRunScanState = {
    runs: [],
    run: [],
    segments: [],
    segmentStart: -1,
    openQuote: '',
    ansiCQuoted: false,
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

  /**
   Checks whether the `&` at one offset belongs to a redirection rather than separating commands.
   
   @param offset - source offset holding the `&`
   
   @returns whether a `<`, `>` or `>` neighbour makes the `&` part of a redirection
   
   @example
   ```ts
   ampersandBelongsToRedirection(7); // true for `>&2`, false for `cmd & cmd`
   ```
   */
  function ampersandBelongsToRedirection(offset: number,): boolean {
    /**
     Character before the `&`, or empty text at the start of the source.
     */
    const previous = source[offset - 1];
    /**
     Character after the `&`, or empty text at the end of the source.
     */
    const next = source[offset + 1];
    return ((previous !== undefined) && (REDIRECTION_CHARACTERS.has(previous,)))
      || ((next !== undefined) && (next === '>'));
  }

  /**
   Checks whether one character ends the current run instead of staying in its word.
   
   @param character - character at the current offset
   
   @param offset - source offset holding that character
   
   @returns whether the run ends here, which the `&` of a redirection does not
   
   @example
   ```ts
   characterEndsRun({ character: ';', offset: 7, }); // true
   ```
   */
  function characterEndsRun({
    character,
    offset,
  }: {
    readonly character: string;
    readonly offset: number;
  },): boolean {
    if (!COMMAND_SEPARATOR_CHARACTERS.has(character,))
      return false;
    if (character !== '&')
      return true;
    return !ampersandBelongsToRedirection(offset,);
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
      // A backslash before a line break joins the two lines into one word.
      if ((character === LINE_BREAK) || (character === CARRIAGE_RETURN)) {
        if ((character === CARRIAGE_RETURN) && (source[offset + 1] === LINE_BREAK))
          offset += 1;
        continue;
      }
      if (state.segmentStart === (-1))
        state.segmentStart = offset;
      continue;
    }
    if ((character === BACKSLASH)
      && ((state.openQuote !== SINGLE_QUOTE) || state.ansiCQuoted)) {
      // The backslash itself is not word text: it makes the next character literal.
      sealSegment(offset,);
      state.escaped = true;
      continue;
    }
    if (state.openQuote !== '') {
      if (character === state.openQuote) {
        sealSegment(offset,);
        state.openQuote = '';
        state.ansiCQuoted = false;
        continue;
      }
      if (state.segmentStart === (-1))
        state.segmentStart = offset;
      continue;
    }
    if (honorQuotes
      && (character === DOLLAR_SIGN)
      && (source[offset + 1] === SINGLE_QUOTE)) {
      sealSegment(offset,);
      state.openQuote = SINGLE_QUOTE;
      state.ansiCQuoted = true;
      offset += 1;
      continue;
    }
    if (honorQuotes
      && ((character === SINGLE_QUOTE) || (character === DOUBLE_QUOTE))) {
      sealSegment(offset,);
      state.openQuote = character;
      continue;
    }
    if (WORD_SEPARATOR_CHARACTERS.has(character,)) {
      endWord(offset,);
      continue;
    }
    if (characterEndsRun({
      character,
      offset,
    },)) {
      endRun(offset,);
      continue;
    }
    if (state.segmentStart === (-1))
      state.segmentStart = offset;
  }
  endRun(source.length,);

  return {
    runs: state.runs,
    endedInsideQuote: state.openQuote !== '',
  };
}

/**
 Splits one source text into runs of command words.
 
 A scan that ends inside a quote is followed by a scan that treats quote characters as text,
 and both sets of runs are reported,
 so a command written after an unbalanced quote is still read.
 
 @param source - shell source text, usually one the parser rejected
 
 @returns word runs in source order, each beginning where a command can begin
 
 @example
 ```ts
 rawCommandWordRuns({ source: 'cd x && bun test "', }); // [['cd', 'x'], ['bun', 'test', '"']]
 ```
 */
function rawCommandWordRuns({ source, }: { readonly source: string; },): readonly (readonly string[])[] {
  /**
   Scan that honours quotes, which is the only one a well-quoted source needs.
   */
  const quoted = scanCommandWordRuns({
    source,
    honorQuotes: true,
  },);
  if (!quoted.endedInsideQuote)
    return quoted.runs;
  /**
   Scan that treats every quote character as text, which reads the commands a stray quote hid.
   */
  const unquoted = scanCommandWordRuns({
    source,
    honorQuotes: false,
  },);
  return [
    ...quoted.runs,
    ...unquoted.runs,
  ];
}

//endregion Word runs

export { rawCommandWordRuns, };
