/**
 Strict argument grammar for the search-fetch CLI.

 `node:util` `parseArgs` in strict mode owns flag spelling, and this module owns
 the two structures `parseArgs` cannot express: the `--` boundary and verb
 dispatch. Tokens after the first bare `--` are always data, so the token stream
 is split there before option parsing; `parseArgs` positionals alone cannot tell
 `search-fetch search -- a b` from `search-fetch search a b`.

 @module
 */

import { parseArgs, } from 'node:util';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import {
  fetchVerb,
  programName,
  searchFetchParseArgsOptions,
  searchFetchVerbs,
  searchVerb,
  type SearchFetchVerb,
} from './constants.ts';
import { SearchFetchCliUsageError, } from './errors.ts';

//region Types

/**
 What one command line asks the CLI to do.
 */
export type SearchFetchCliInvocation = {
  /**
   Dispatch target: shared web search or shared web fetch.
   */
  readonly kind: 'run';
  /**
   Verb selecting the shared tool.
   */
  readonly verb: SearchFetchVerb;
  /**
   Raw tool params assembled from the one argument and the accepted flags.
   */
  readonly rawParams: Readonly<Record<string, unknown>>;
} | {
  /**
   Explicit `--help`: print usage and exit 0.
   */
  readonly kind: 'help';
  /**
   Verb whose usage to print, when a known verb came before `--help`.
   */
  readonly verb?: SearchFetchVerb;
} | {
  /**
   Explicit `--version`: print the package version and exit 0.
   */
  readonly kind: 'version';
};

/**
 Option-side facts this grammar reads out of `parseArgs`.
 */
type ParsedOptionSide = {
  /**
   Whether `--help` appeared before the `--` boundary.
   */
  readonly help: boolean;
  /**
   Whether `--version` appeared before the `--` boundary.
   */
  readonly version: boolean;
  /**
   `--include-domains` value as parsed, present only when the flag appeared.
   */
  readonly includeDomainsValue?: string;
  /**
   `--from-date` value as parsed, present only when the flag appeared.
   */
  readonly fromDateValue?: string;
  /**
   `--to-date` value as parsed, present only when the flag appeared.
   */
  readonly toDateValue?: string;
  /**
   Positional tokens from the option side, verb first.
   */
  readonly positionals: readonly string[];
  /**
   Names of flags that appeared more than once.
   */
  readonly repeatedOptionNames: readonly string[];
};

//endregion Types

//region Public API

/**
 Parse one search-fetch command line into its invocation.

 @param argv - arguments after the script path, exactly as `process.argv` carried them

 @returns help, version, or run invocation

 @throws SearchFetchCliUsageError when the command line violates the documented grammar

 @example
 ```ts
 parseSearchFetchInvocation({ argv: ['search', '--', 'cachyos', 'kernel'] });
 ```
 */
export function parseSearchFetchInvocation(
  { argv, }: {
    readonly argv: readonly string[];
  },
): SearchFetchCliInvocation {
  /**
   Index of the first bare `--`, the boundary between options and data.
   */
  const separatorIndex = argv.indexOf('--',);
  /**
   Tokens before the boundary: flags, the verb, and at most one positional.
   */
  const leftTokens = separatorIndex === (-1)
    ? argv
    : argv.slice(
      0,
      separatorIndex,
    );
  /**
   Words after the boundary, joined with single spaces into the one argument.
   */
  const dataWords = separatorIndex === (-1)
    ? []
    : argv.slice(separatorIndex + 1,);
  /**
   Strict parse of the option side.
   */
  const parsed = parseOptionSide(leftTokens);
  if (parsed.repeatedOptionNames
    .length
    > 0)
    throw new SearchFetchCliUsageError({
      message: `Options ${
        parsed
          .repeatedOptionNames
          .map(function nameFlag(optionName,) {
            return `--${optionName}`;
          },)
          .join(', ',)
      } are single-valued and may appear at most once; repeats are refused, not last-wins.`,
    },);

  if (parsed.help) {
    /**
     Known verb among the option-side positionals, when the caller named one.
     */
    const helpVerb = parsed
      .positionals
      .find(function isKnownVerb(token,) {
        return isSearchFetchVerb(token,);
      },);
    return helpVerb === undefined
      ? { kind: 'help', }
      : {
        kind: 'help',
        verb: helpVerb,
      };
  }

  if (parsed.version)
    return { kind: 'version', };

  /**
   Verb token, the one argument positional, and any further positional.
   */
  const [verbToken, firstArgument, ...extraArguments] = parsed.positionals;
  if (verbToken === undefined)
    throw new SearchFetchCliUsageError({
      message: `${programName} needs a verb: ${searchVerb} or ${fetchVerb}.`,
    },);
  if (!isSearchFetchVerb(verbToken,))
    throw new SearchFetchCliUsageError({
      message: `\`${verbToken}\` is not a ${programName} verb. Use ${searchVerb} or ${fetchVerb}.`,
    },);
  if ((firstArgument !== undefined) && (dataWords.length > 0))
    throw new SearchFetchCliUsageError({
      message: `Pass the ${verbToken} argument in exactly one form: one positional before --, or words after --. Supplying both is unsupported.`,
    },);
  if (extraArguments.length > 0)
    throw new SearchFetchCliUsageError({
      message: `${programName} ${verbToken} takes exactly one positional before --; got ${String(extraArguments.length + 1,)}.`,
    },);
  if (firstArgument === undefined) {
    if (dataWords.length === 0)
      throw new SearchFetchCliUsageError({
        message: `${programName} ${verbToken} needs exactly one argument: one positional before --, or words after --.`,
      },);
    return runInvocation({
      verb: verbToken,
      argument: dataWords.join(' ',),
      parsed,
    },);
  }
  return runInvocation({
    verb: verbToken,
    argument: firstArgument,
    parsed,
  },);
}

//endregion Public API

//region Helpers

/**
 Parse the option side strictly, turning parser refusals into usage errors.

 @param leftTokens - tokens before the `--` boundary

 @returns option-side facts for grammar dispatch

 @throws SearchFetchCliUsageError when `parseArgs` refuses the tokens

 @example
 ```ts
 parseOptionSide(['search', '--from-date', '2026-01-01']);
 ```
 */
function parseOptionSide(leftTokens: readonly string[],): ParsedOptionSide {
  try {
    /**
     Strict parse with token access, the only spelling that reports repeats.
     */
    const parsed = parseArgs({
      args: leftTokens,
      strict: true,
      allowPositionals: true,
      tokens: true,
      options: searchFetchParseArgsOptions,
    },);
    /**
     Option occurrence counts read from parser tokens; `parseArgs` alone is
     last-wins for repeats and this grammar refuses them instead.
     */
    const optionOccurrences = new Map<string, number>();
    for (const token of parsed.tokens) {
      if (token.kind === 'option')
        optionOccurrences.set(
          token.name,
          (optionOccurrences.get(token.name,) ?? 0) + 1,
        );
    }
    /**
     Flags that appeared more than once, in first-repeat order.
     */
    const repeatedOptionNames: string[] = [];
    for (const [optionName, occurrenceCount,] of optionOccurrences) {
      if (occurrenceCount > 1)
        repeatedOptionNames.push(optionName,);
    }
    return {
      help: parsed.values
        .help
        === true,
      version: parsed.values
        .version
        === true,
      ...(parsed.values['include-domains'] === undefined
        ? {}
        : { includeDomainsValue: parsed.values['include-domains'], }),
      ...(parsed.values['from-date'] === undefined
        ? {}
        : { fromDateValue: parsed.values['from-date'], }),
      ...(parsed.values['to-date'] === undefined
        ? {}
        : { toDateValue: parsed.values['to-date'], }),
      positionals: parsed.positionals,
      repeatedOptionNames,
    };
  }
  catch (error: unknown) {
    throw new SearchFetchCliUsageError({
      message: caughtValueText(error,),
    },);
  }
}

/**
 Assemble the run invocation for one verb from the one argument and the flags.

 @param verb - verb selecting the shared tool

 @param argument - the one query or URL argument

 @param parsed - parsed option side carrying the accepted flags

 @returns run invocation carrying raw tool params

 @example
 ```ts
 runInvocation({ verb: searchVerb, argument: 'docs', parsed: optionSide });
 ```
 */
function runInvocation(
  {
    verb,
    argument,
    parsed,
  }: {
    readonly verb: SearchFetchVerb;
    readonly argument: string;
    readonly parsed: ParsedOptionSide;
  },
): SearchFetchCliInvocation {
  /**
   Allow-list entries after trimming and empty-entry dropping.
   */
  const includeDomains = trimmedIncludeDomains(parsed.includeDomainsValue,);
  return {
    kind: 'run',
    verb,
    rawParams: {
      ...(verb === searchVerb ? { query: argument, } : { url: argument, }),
      ...(includeDomains.length === 0 ? {} : { includeDomains, }),
      ...(parsed.fromDateValue === undefined
        ? {}
        : { fromDate: parsed.fromDateValue, }),
      ...(parsed.toDateValue === undefined
        ? {}
        : { toDate: parsed.toDateValue, }),
    },
  };
}

/**
 Report whether a positional token names a supported verb.

 @param token - positional token under test

 @returns whether token is a verb name

 @example
 ```ts
 isSearchFetchVerb('search');
 ```
 */
function isSearchFetchVerb(token: string,): token is SearchFetchVerb {
  return searchFetchVerbs.some(function matchesVerb(verb,) {
    return verb === token;
  },);
}

/**
 Normalize the `--include-domains` comma list.

 @param rawValue - raw flag value as parsed, when the flag appeared

 @returns trimmed non-empty entries in caller order; empty when the flag was absent or every entry was empty

 @example
 ```ts
 trimmedIncludeDomains('a.com, ,b.com');
 ```
 */
function trimmedIncludeDomains(rawValue?: string,): string[] {
  if (rawValue === undefined)
    return [];
  return rawValue
    .split(',',)
    .map(function trimEntry(entry,) {
      return entry.trim();
    },)
    .filter(function isNonEmptyEntry(entry,) {
      return entry !== '';
    },);
}

//endregion Helpers
