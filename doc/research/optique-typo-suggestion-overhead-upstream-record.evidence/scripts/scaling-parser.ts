// The synthetic parser for the scaling experiment: an `object()` with P
// optional value-taking options, each declaring one fixed-length long name,
// followed by `multiple(argument(string()))`. No passThrough, so every
// positional token is offered to every option parser and rejected.
// Copied into every arm directory at image build time so the bare
// `@optique/core` specifiers resolve to that arm's version.

import { object, } from '@optique/core/constructs';
import { message, } from '@optique/core/message';
import {
  multiple,
  optional,
} from '@optique/core/modifiers';
import { parseSync, } from '@optique/core/parser';
import {
  argument,
  option,
} from '@optique/core/primitives';
import { string, } from '@optique/core/valueparser';

//region Shapes

/**
 * One point of the experiment: how many option parsers, how long each option
 * name is, how long each positional token is, and how many tokens arrive.
 */
export type ScalingShape = {
  readonly optionCount: number;
  readonly tokenLength: number;
  readonly tokenCount: number;
};

/**
 * What one parse did, beyond how long it took.
 */
export type ScalingOutcome = {
  readonly success: boolean;
  readonly tokens: number;
  readonly probes: number;
};

/**
 * Counter a function-valued `errors.noMatch` increments, so the number of
 * mismatch probes is counted rather than derived.
 */
type ProbeCounter = {
  count: number;
};

//endregion Shapes

//region Inputs

/**
 * Long-option name declared by the option parser at `index`. Every name has
 * exactly NAME_LENGTH characters, so option count and name length are separate
 * factors.
 */
const NAME_LENGTH = 12;

/**
 * Prefix shared by every generated option name.
 */
const NAME_PREFIX = '--option-';

/**
 * Builds the long-option name for one option parser.
 *
 * @param index - Zero-based option parser index.
 *
 * @returns Option name of exactly NAME_LENGTH characters.
 */
export function optionName(index: number,): string {
  /** Zero-padded index, sized so the whole name reaches NAME_LENGTH. */
  const paddedIndex = String(index,).padStart(NAME_LENGTH - NAME_PREFIX.length, '0',);

  return `${NAME_PREFIX}${paddedIndex}`;
}

/**
 * Builds one positional token of exactly `length` characters. Tokens never
 * start with a dash, so the argument parser accepts every one of them.
 *
 * @param index - Zero-based token index, which keeps tokens distinct.
 *
 * @param length - Token length in characters.
 *
 * @returns Token of exactly `length` characters.
 */
export function tokenOf(index: number, length: number,): string {
  return `p${String(index,).toString(36,).padStart(
    length - 1,
    '0',
  )}`;
}

//endregion Inputs

//region Builders

/**
 * Builds the synthetic parser, optionally counting every mismatch probe.
 *
 * @param shape - Option count and the token length that fixes the name space.
 *
 * @param counter - Counter to increment per mismatch, or undefined for the
 * default error path.
 *
 * @returns Parser over P optional options and a positional token list.
 */
function buildParser(
  shape: { readonly optionCount: number; },
  counter: ProbeCounter | undefined,
) {
  /** One optional value-taking option per index. */
  const optionParsers = Array.from(
    { length: shape.optionCount, },
    function buildOption(_value, index,) {
      /** Counting mismatch callback, used only by the probe runs. */
      const errors = counter === undefined
        ? undefined
        : {
          noMatch: function countMismatch() {
            counter.count += 1;
            return message`No match.`;
          },
        };

      return optional(
        errors === undefined
          ? option(optionName(index,), string(),)
          : option(
            optionName(index,),
            string(),
            { errors, },
          ),
      );
    },
  );

  return object({
    ...Object.fromEntries(
      optionParsers.map(function named(parser, index,) {
        return [
          `option${index}`,
          parser,
        ];
      },),
    ),
    paths: multiple(argument(string(),),),
  },);
}

/**
 * Builds a runner for one point of the scaling experiment.
 *
 * @param shape - Point to build.
 *
 * @param countProbes - Whether to attach the counting mismatch callback.
 *
 * @returns Runner reporting parse success, positional count and probe count.
 */
export function createScalingRunner(shape: ScalingShape, countProbes: boolean,): () => ScalingOutcome {
  /** Mismatch counter shared with the counting callback. */
  const counter: ProbeCounter = { count: 0, };
  /** Parser for this point. */
  const parser = buildParser(
    shape,
    countProbes ? counter : undefined,
  );
  /** Tokens this point parses. */
  const tokens = Array.from(
    { length: shape.tokenCount, },
    function buildToken(_value, index,) {
      return tokenOf(
        index,
        shape.tokenLength,
      );
    },
  );

  return function run() {
    /** Mismatches before this parse. */
    const before = counter.count;
    /** Parse result for the prepared argv. */
    const result = parseSync(
      parser,
      tokens,
    );
    /** Positional tokens the parse accepted. */
    const accepted = result.success
      ? (result.value as { readonly paths: readonly string[]; }).paths.length
      : 0;

    return {
      success: result.success && accepted === shape.tokenCount,
      tokens: accepted,
      probes: counter.count - before,
    };
  };
}

//endregion Builders
