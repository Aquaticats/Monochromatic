/**
 Shell wrapper removal for terminal title command summaries.
 
 @module
 */

import {
  ENV_OPTIONS_WITH_ARGUMENT,
  isAssignmentToken,
  NICE_OPTIONS_WITH_ARGUMENT,
  skipWrapperOption,
  TIMEOUT_OPTIONS_WITH_ARGUMENT,
} from './command-options.ts';
import {
  COMMAND_TOKENS_MISSING,
  type CommandTokens,
  wrappedTokensFromArgs,
} from './command-tokens.ts';

//region Wrapper constants

/**
 Shell wrapper command names whose own invocation is title noise.
 */
const WRAPPER_COMMAND_NAMES: ReadonlySet<string> = new Set([
  'env',
  'nice',
  'nohup',
  'timeout',
],);

/**
 Result of unwrapping one shell wrapper.
 */
type CommandTokensResult = CommandTokens | typeof COMMAND_TOKENS_MISSING;

//endregion Wrapper constants

//region Cursor scanners

/**
 How one wrapper lays out its own arguments before the command it wraps.
 */
type WrapperArgumentGrammar = Readonly<{
  /**
   Options of this wrapper that consume the token after them.
   */
  optionsWithArgument: ReadonlySet<string>;

  /**
   Whether a `NAME=value` assignment may stand before the command.
   */
  skipsAssignments?: boolean;

  /**
   Whether one bare value, such as a duration, stands before the command.
   */
  skipsValue?: boolean;
}>;

/**
 The wrapper commands whose own arguments this module walks past, and how.
 */
const WRAPPER_ARGUMENT_GRAMMARS: ReadonlyMap<string, WrapperArgumentGrammar> = new Map([
  [
    'env',
    {
      optionsWithArgument: ENV_OPTIONS_WITH_ARGUMENT,
      skipsAssignments: true,
    },
  ],
  [
    'nice',
    {
      optionsWithArgument: NICE_OPTIONS_WITH_ARGUMENT,
    },
  ],
  [
    'timeout',
    {
      optionsWithArgument: TIMEOUT_OPTIONS_WITH_ARGUMENT,
      skipsValue: true,
    },
  ],
],);

/**
 Finds the cursor of the command a wrapper runs.
 
 @param args - because a wrapper's own options, assignments and values precede its command
 
 @param grammar - because each wrapper declares which arguments are its own
 
 @returns cursor for wrapped command or sentinel when absent
 */
function wrappedCursor(
  {
    args,
    grammar,
  }: Readonly<{
    args: readonly string[];
    grammar: WrapperArgumentGrammar;
  }>,
): number | typeof COMMAND_TOKENS_MISSING {
  {
    /**
     Cursor over the wrapper's own arguments before the wrapped command.
     */
    let cursor = 0;
    while (cursor
      < args.length)
      {
      /**
       Current wrapper argument token.
       */
      const token = args[cursor];
      if (token === undefined)
        return COMMAND_TOKENS_MISSING;
      if ((grammar.skipsAssignments === true) && isAssignmentToken(token,)) {
        cursor += 1;
        continue;
      }
      if (token.startsWith('-',)) {
        cursor = skipWrapperOption({
          args,
          cursor,
          optionsWithArgument: grammar.optionsWithArgument,
        },);
        continue;
      }
      if (grammar.skipsValue
        === true)
        return cursor + 1;
      return cursor;
    }
  }
  return COMMAND_TOKENS_MISSING;
}

//endregion Cursor scanners

//region Wrapper implementations

/**
 Converts wrapper cursor result to command tokens.
 
 @param args - because wrapper command passes remaining tokens as wrapped command
 
 @param cursor - because scanner determines wrapped command position
 
 @returns wrapped command tokens or sentinel
 */
function tokensFromCursor(
  {
    args,
    cursor,
  }: Readonly<{
    args: readonly string[];
    cursor: number | typeof COMMAND_TOKENS_MISSING;
  }>,
): CommandTokensResult {
  if (((typeof cursor) === 'symbol') && (cursor === COMMAND_TOKENS_MISSING))
    return COMMAND_TOKENS_MISSING;
  return wrappedTokensFromArgs({
    args,
    cursor,
  },);
}

/**
 Unwraps one recognized wrapper command.
 
 @param tokens - because wrappers should not dominate command titles
 
 @returns wrapped command tokens or sentinel when tokens are not a wrapper
 */
function unwrapWrapper(tokens: CommandTokens,): CommandTokensResult {
  /**
   Grammar of this wrapper's own arguments, when this module walks past them.
   */
  const grammar = WRAPPER_ARGUMENT_GRAMMARS.get(tokens.name,);
  if (grammar !== undefined) {
    return tokensFromCursor({
      args: tokens.args,
      cursor: wrappedCursor({
        args: tokens.args,
        grammar,
      },),
    },);
  }
  if (tokens.name === 'nohup') {
    /**
     `nohup` takes only `--help` and `--version`,
     so the command it runs starts at the first token whatever that token is.
     */
    return wrappedTokensFromArgs({
      args: tokens.args,
      cursor: 0,
    },);
  }
  return COMMAND_TOKENS_MISSING;
}

//endregion Wrapper implementations

//region Public wrapper API

/**
 Removes leading shell wrappers from parsed command tokens.
 
 @param tokens - because users care about meaningful work, not wrapper mechanics
 
 @returns innermost meaningful command tokens
 
 @example
 ```ts
 unwrapCommandTokens({ name: 'timeout', args: ['10', 'npm', 'test'] });
 // { name: 'npm', args: ['test'] }
 ```
 */
function unwrapCommandTokens(tokens: CommandTokens,): CommandTokens {
  {
    /**
     Current command tokens after each wrapper removal.
     */
    let current = tokens;
    while (WRAPPER_COMMAND_NAMES.has(current.name,)) {
      /**
       Next wrapped command tokens.
       */
      const next = unwrapWrapper(current,);
      if (((typeof next) === 'symbol') && (next === COMMAND_TOKENS_MISSING))
        return current;
      current = next;
    }
    return current;
  }
}

//endregion Public wrapper API

export { unwrapCommandTokens, };
