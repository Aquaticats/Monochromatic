//region Nesting vocabulary
// THE BOUNDS AND THE WORDS A READING OF A BODY IS GIVEN IN, apart from the scan that
// makes it (`nesting-bound.ts`), so the lexing and counting modules and the parser
// entries read one definition.

/**
 How many levels of any one measure a body may nest before it is refused.
 */
export const NESTING_BOUND = 256;

/**
 How many emphasis delimiters one block may carry before it is refused.

 A DIFFERENT NUMBER FROM THE NESTING BOUND: delimiters pair across a block, and
 the parser's cost grows with their count rather than with a depth, so the
 count that costs a fraction of a second is the bound.
 */
export const DELIMITER_BOUND = 1_024;

/**
 The text the engine puts at the start of a stack exhaustion's message.
 V8 raises it as a `RangeError` reading "Maximum call stack size exceeded"
 (read off `Error.message` of a runaway recursion on the installed runtime);
 another engine may end it with a full stop, so only its start is compared.
 */
export const STACK_OVERFLOW_MESSAGE = 'Maximum call stack size exceeded';

/**
 The account of a stack exhaustion, in the same words as {@link nestingAccount}.
 */
export const STACK_OVERFLOW_ACCOUNT = 'nested too deeply to read: the parser exhausted its stack';

/**
 What a nesting bound counts.
 */
export type NestingMeasure =
  | 'container markers'
    | 'indentation'
    | 'brackets'
    | 'open tags'
    | 'open braces'
    | 'emphasis delimiters';

/**
 Where a body first passes the bound.
 */
export type NestingExcess = {
  /**
   What was counted.
   */
  readonly measure: NestingMeasure;

  /**
   The bound it passed.
   */
  readonly bound: number;

  /**
   One-based line of the item that passed the bound.
   */
  readonly line: number;

  /**
   One-based column of that item, counting from the first character the
   parser reads (a leading byte order mark is not counted).
   */
  readonly column: number;
};

/**
 What reading a body against the bound found.
 */
export type NestingReading =
  | { readonly kind: 'within'; }
    | ({ readonly kind: 'beyond'; } & NestingExcess);

/**
 Which grammar will read the body, since a plain markdown parse nests over
 neither tags nor braces.
 */
export type NestingGrammar = 'mdx' | 'markdown';

//endregion Nesting vocabulary
