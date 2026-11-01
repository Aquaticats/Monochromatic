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
 Finds wrapped command cursor for `env` arguments.
 
 @param args - because env accepts options and assignments before command
 
 @returns cursor for wrapped command or sentinel when absent
 */
function envWrappedCursor(args: readonly string[],): number | typeof COMMAND_TOKENS_MISSING {
  {
    /**
     Cursor over env arguments before wrapped command.
     */
    let cursor = 0;
    while (cursor
      < args.length)
      {
      /**
       Current env argument token.
       */
      const token = args[cursor];
      if (token === undefined)
        return COMMAND_TOKENS_MISSING;
      if (isAssignmentToken(token,)) {
        cursor += 1;
        continue;
      }
      if (token.startsWith('-',)) {
        cursor = skipWrapperOption({
          args,
          cursor,
          optionsWithArgument: ENV_OPTIONS_WITH_ARGUMENT,
        },);
        continue;
      }
      return cursor;
    }
  }
  return COMMAND_TOKENS_MISSING;
}

/**
 Finds wrapped command cursor for `nice` arguments.
 
 @param args - because nice accepts options before command
 
 @returns cursor for wrapped command or sentinel when absent
 */
function niceWrappedCursor(args: readonly string[],): number | typeof COMMAND_TOKENS_MISSING {
  {
    /**
     Cursor over nice arguments before wrapped command.
     */
    let cursor = 0;
    while (cursor
      < args.length)
      {
      /**
       Current nice argument token.
       */
      const token = args[cursor];
      if (token === undefined)
        return COMMAND_TOKENS_MISSING;
      if (token.startsWith('-',)) {
        cursor = skipWrapperOption({
          args,
          cursor,
          optionsWithArgument: NICE_OPTIONS_WITH_ARGUMENT,
        },);
        continue;
      }
      return cursor;
    }
  }
  return COMMAND_TOKENS_MISSING;
}

/**
 Finds wrapped command cursor for `timeout` arguments.
 
 @param args - because timeout accepts options and one duration before command
 
 @returns cursor for wrapped command or sentinel when absent
 */
function timeoutWrappedCursor(args: readonly string[],): number | typeof COMMAND_TOKENS_MISSING {
  {
    /**
     Cursor over timeout options and duration.
     */
    let cursor = 0;
    while (cursor
      < args.length)
      {
      /**
       Current timeout argument token.
       */
      const token = args[cursor];
      if (token === undefined)
        return COMMAND_TOKENS_MISSING;
      if (token.startsWith('-',)) {
        cursor = skipWrapperOption({
          args,
          cursor,
          optionsWithArgument: TIMEOUT_OPTIONS_WITH_ARGUMENT,
        },);
        continue;
      }
      return cursor + 1;
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
  if (tokens.name === 'env') {
    return tokensFromCursor({
      args: tokens.args,
      cursor: envWrappedCursor(tokens.args,),
    },);
  }
  if (tokens.name === 'nice') {
    return tokensFromCursor({
      args: tokens.args,
      cursor: niceWrappedCursor(tokens.args,),
    },);
  }
  if (tokens.name === 'timeout') {
    return tokensFromCursor({
      args: tokens.args,
      cursor: timeoutWrappedCursor(tokens.args,),
    },);
  }
  if (tokens.name === 'nohup') {
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
