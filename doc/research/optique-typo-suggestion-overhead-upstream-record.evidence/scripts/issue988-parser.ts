// The reproduction from upstream issue 988: thirty optional options and a
// thousand long distinct positional paths, parsed through the arm's own
// @optique/core. Copied into every arm directory at image build time so the
// bare `@optique/core` specifiers resolve to that arm's version.

import { object, } from '@optique/core/constructs';
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

//region Parser

/**
 * Number of `--option-N` declarations the issue-988 reproduction declares.
 */
const OPTION_COUNT = 30;

/**
 * Shape of the value the issue-988 parser returns, reduced to what this
 * measurement inspects.
 */
export type Issue988Value = {
  readonly paths: readonly string[];
};

/**
 * Builds the root parser from the issue text: thirty optional value-taking
 * options followed by a positional path list.
 *
 * @returns Parser accepting the thirty options plus positional paths.
 */
function buildParser() {
  /** One optional value-taking option per index. */
  const optionParsers = Array.from(
    { length: OPTION_COUNT, },
    function parseOption(_value, index,) {
      return optional(option(`--option-${index}`, string(),),);
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

//endregion Parser

//region Runner

/**
 * A callable that parses one argv and reports only what validation needs.
 */
export type Issue988Runner = (paths: readonly string[]) => {
  readonly success: boolean;
  readonly handled: number;
};

/**
 * Builds a runner that parses the issue-988 shape with this arm's Optique.
 *
 * @returns Runner reporting parse success and the positional count.
 */
export function createIssue988Runner(): Issue988Runner {
  /** Parser instance reused across timed parses. */
  const parser = buildParser();

  return function run(paths,) {
    /** Parse result for one argv. */
    const result = parseSync(
      parser,
      paths,
    );

    if (!result.success) {
      return {
        success: false,
        handled: 0,
      };
    }

    /** Parsed value narrowed to the fields this measurement reads. */
    const value = result.value as Issue988Value;

    return {
      success: true,
      handled: value.paths.length,
    };
  };
}

//endregion Runner
