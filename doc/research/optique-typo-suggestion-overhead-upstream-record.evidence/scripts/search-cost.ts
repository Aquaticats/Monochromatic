// Part 2: what one suggestion search costs on @optique/core 0.7.0, the first
// release with the feature, at the design note's own parameters: 50 options
// whose names are 20 characters long, and one mistyped 20-character option.
// Copied into the 0.7.0 arm directory at image build time so the bare
// `@optique/core` specifiers resolve to that version.
//
// `findSimilar` is not reachable through the package's exports map in 0.7.0,
// so the search itself is timed through the shipped file path.

import { object, } from '@optique/core/constructs';
import { message, } from '@optique/core/message';
import {
  multiple,
  optional,
} from '@optique/core/modifiers';
import { parse, } from '@optique/core/parser';
import {
  argument,
  option,
} from '@optique/core/primitives';
import { string, } from '@optique/core/valueparser';

//region Parameters

/**
 * Option count the design note names.
 */
const OPTION_COUNT = 50;

/**
 * Declared option name length in characters.
 */
const NAME_LENGTH = 20;

/**
 * Mistyped option length in characters.
 */
const MISTYPED_LENGTH = 20;

/**
 * Paths the counted successful parse carries.
 */
const PATH_COUNT = 1000;

/**
 * How many times the isolated search runs.
 */
const SEARCH_REPEATS = 20_000;

/**
 * How many times the failing and succeeding parses run.
 */
const PARSE_REPEATS = 2000;

//endregion Parameters

//region Inputs

/**
 * Builds the long-option name for one option parser, exactly NAME_LENGTH
 * characters long.
 *
 * @param index - Zero-based option index.
 *
 * @returns Option name of NAME_LENGTH characters.
 */
function optionName(index: number,): string {
  /** Numbered middle section. */
  const numbered = `opt-${String(index,).padStart(
    3,
    '0',
  )}`;

  return `--${numbered}${'x'.repeat(NAME_LENGTH - (2 + numbered.length))}`;
}

/**
 * The one mistyped option: one character away from the first declared name,
 * and the same length as every declared name.
 *
 * @returns Mistyped option token of MISTYPED_LENGTH characters.
 */
function mistypedOption(): string {
  /** Declared name it resembles. */
  const resembling = optionName(0,);

  return `${resembling.slice(
    0,
    MISTYPED_LENGTH - 1,
  )}y`;
}

//endregion Inputs

//region Helpers

/**
 * Interpolated median.
 *
 * @param values - Numbers to summarise.
 *
 * @returns Median value.
 */
function median(values: readonly number[],): number {
  /** Sorted copy. */
  const sorted = [...values].sort(function ascending(left, right,) {
    return left - right;
  },);
  /** Middle index. */
  const middle = Math.floor(sorted.length / 2,);

  return sorted.length % 2 === 0
    ? ((sorted[middle - 1] ?? Number.NaN) + (sorted[middle] ?? Number.NaN)) / 2
    : (sorted[middle] ?? Number.NaN);
}

//endregion Helpers

//region Entry point

/**
 * Measures the isolated search, the failing parse, the search-free parse, and
 * counts the searches one thousand-path successful parse performs.
 */
async function main(): Promise<void> {
  /** Declared option names in declaration order. */
  const names = Array.from(
    { length: OPTION_COUNT, },
    function name(_value, index,) {
      return optionName(index,);
    },
  );
  /** Mistyped token. */
  const mistyped = mistypedOption();
  /** Shipped search module, reached by file path because the exports map omits it. */
  const suggestionModule = await import(
    '/bench/arms/0.7.0/node_modules/@optique/core/dist/suggestion.js',
  ) as {
    findSimilar: (
      input: string,
      candidates: Iterable<string>,
      options?: unknown,
    ) => readonly string[];
  };
  /** Parser for the failing and succeeding single-option parses. */
  const singleOptionParser = object({
    ...Object.fromEntries(
      names.map(function field(name, index,) {
        return [
          `option${index}`,
          optional(option(name, string(),),),
        ];
      },),
    ),
  });

  for (let warmup = 0; warmup < 200; warmup += 1) {
    suggestionModule.findSimilar(
      mistyped,
      names,
    );
    parse(
      singleOptionParser,
      [mistyped,],
    );
  }

  /** Isolated search durations in milliseconds. */
  const searchSamples: number[] = [];

  for (let index = 0; index < SEARCH_REPEATS; index += 1) {
    /** Monotonic start. */
    const startedAt = performance.now();
    suggestionModule.findSimilar(
      mistyped,
      names,
    );
    searchSamples.push(performance.now() - startedAt,);
  }

  /** Failing-parse durations in milliseconds. */
  const failingSamples: number[] = [];

  for (let index = 0; index < PARSE_REPEATS; index += 1) {
    /** Monotonic start. */
    const startedAt = performance.now();
    parse(
      singleOptionParser,
      [mistyped,],
    );
    failingSamples.push(performance.now() - startedAt,);
  }

  /** Succeeding-parse durations in milliseconds. */
  const succeedingSamples: number[] = [];
  /** A declared option with a valid value. */
  const validArgv = [
    optionName(0,),
    'value',
  ];

  for (let index = 0; index < PARSE_REPEATS; index += 1) {
    /** Monotonic start. */
    const startedAt = performance.now();
    parse(
      singleOptionParser,
      validArgv,
    );
    succeedingSamples.push(performance.now() - startedAt,);
  }

  /** Mismatches counted through a function-valued `errors.noMatch`. */
  let probes = 0;
  /** Counting parser: the same 50 options plus a positional path list. */
  const countingParser = object({
    ...Object.fromEntries(
      names.map(function field(name, index,) {
        return [
          `option${index}`,
          optional(option(
            name,
            string(),
            {
              errors: {
                noMatch: function countMismatch() {
                  probes += 1;
                  return message`No match.`;
                },
              },
            },
          ),),
        ];
      },),
    ),
    paths: multiple(argument(string(),),),
  });
  /** Paths the counted parse carries. */
  const paths = Array.from(
    { length: PATH_COUNT, },
    function path(_value, index,) {
      return `package/group-${index % 37}/package-${index}/src/feature-${index}.unit.test.ts`;
    },
  );
  /** Monotonic start of the counted parse. */
  const countedStart = performance.now();
  /** Counted parse result. */
  const counted = parse(
    countingParser,
    paths,
  );
  /** Wall clock of the counted parse in milliseconds. */
  const countedMs = performance.now() - countedStart;

  process.stdout.write(`${
    JSON.stringify({
      kind: 'search-cost',
      arm: '0.7.0',
      optionCount: OPTION_COUNT,
      nameLength: NAME_LENGTH,
      mistyped,
      editDistanceCells: OPTION_COUNT * MISTYPED_LENGTH * NAME_LENGTH,
      search: {
        repeats: SEARCH_REPEATS,
        medianMs: median(searchSamples,),
        minMs: Math.min(...searchSamples,),
        suggestionsForMistyped: suggestionModule.findSimilar(
          mistyped,
          names,
        ),
      },
      failingParse: {
        repeats: PARSE_REPEATS,
        medianMs: median(failingSamples,),
        minMs: Math.min(...failingSamples,),
      },
      succeedingParse: {
        repeats: PARSE_REPEATS,
        medianMs: median(succeedingSamples,),
        minMs: Math.min(...succeedingSamples,),
      },
      countedParse: {
        pathCount: PATH_COUNT,
        success: counted.success,
        probes,
        acceptedPaths: counted.success
          ? (counted.value as { readonly paths: readonly string[]; }).paths.length
          : 0,
        ms: countedMs,
      },
      node: process.version,
    },)
  }\n`,);
}

await main();

//endregion Entry point
