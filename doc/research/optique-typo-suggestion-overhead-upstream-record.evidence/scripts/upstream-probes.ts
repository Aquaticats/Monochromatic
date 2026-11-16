// The two minimal parsers from upstream issues 990 and 991, run against the
// arm's own @optique/core. Copied into every arm directory at image build
// time. These record what the library changed underneath the commit parser,
// independent of the wrapper's own fact record.

import { object, } from '@optique/core/constructs';
import {
  multiple,
  optional,
} from '@optique/core/modifiers';
import { parseSync, } from '@optique/core/parser';
import {
  argument,
  option,
  passThrough,
} from '@optique/core/primitives';
import { string, } from '@optique/core/valueparser';

//region Probe shapes

/**
 * Compact description of one parse result, keeping only what the issue text
 * asserts about it.
 */
export type ProbeOutcome = {
  readonly success: boolean;
  /** Positional values the parse accepted, when it succeeded. */
  readonly paths?: readonly string[];
  /** Pass-through values the parse kept, when the parser declares them. */
  readonly extra?: readonly string[];
  /** Option value the parse produced, when the parser declares it. */
  readonly message?: string | undefined;
};

//endregion Probe shapes

//region Parsers

/**
 * Builds the parser from upstream issue 990: positional paths plus
 * next-token pass-through.
 *
 * @returns Parser whose positionals and pass-through values are recorded.
 */
function buildPositionalParser() {
  return object({
    paths: multiple(argument(string(),),),
    extra: passThrough({ format: 'nextToken', },),
  },);
}

/**
 * Builds the parser from upstream issue 991: one optional value option plus
 * next-token pass-through.
 *
 * @returns Parser whose message value and pass-through values are recorded.
 */
function buildMessageParser() {
  return object({
    message: optional(option(
      '-m',
      '--message',
      string(),
    ),),
    extra: passThrough({ format: 'nextToken', },),
  },);
}

//endregion Parsers

//region Entry point

/**
 * Runs both upstream reproductions with the two spellings each issue compares.
 *
 * @returns One outcome per issue and spelling.
 */
export function runUpstreamProbes(): Readonly<Record<string, ProbeOutcome>> {
  /** Parser for the joined-token issue. */
  const positionalParser = buildPositionalParser();
  /** Parser for the missing-value issue. */
  const messageParser = buildMessageParser();
  /** Outcome per probe name. */
  const outcomes: Record<string, ProbeOutcome> = {};

  /** Joined unknown long option, the spelling issue 990 reports as misread. */
  const positionalJoined = parseSync(
    positionalParser,
    [
      '--unknown=v',
      'a.txt',
    ],
  );
  outcomes['join/positional-joined'] = positionalJoined.success
    ? {
      success: true,
      paths: positionalJoined.value.paths,
      extra: positionalJoined.value.extra,
    }
    : { success: false, };

  /** Same option as two tokens, the spelling both sides already agreed on. */
  const positionalSeparated = parseSync(
    positionalParser,
    [
      '--unknown',
      'v',
      'a.txt',
    ],
  );
  outcomes['join/positional-separated'] = positionalSeparated.success
    ? {
      success: true,
      paths: positionalSeparated.value.paths,
      extra: positionalSeparated.value.extra,
    }
    : { success: false, };

  /** Joined token after the `--` separator, where it stays a positional. */
  const positionalAfterSeparator = parseSync(
    positionalParser,
    [
      '--',
      '--unknown=v',
      'a.txt',
    ],
  );
  outcomes['join/positional-after-separator'] = positionalAfterSeparator.success
    ? {
      success: true,
      paths: positionalAfterSeparator.value.paths,
      extra: positionalAfterSeparator.value.extra,
    }
    : { success: false, };

  /** Known value option with no value, short spelling, issue 991. */
  const shortMissingValue = parseSync(
    messageParser,
    ['-m',],
  );
  outcomes['missing/short'] = shortMissingValue.success
    ? {
      success: true,
      message: shortMissingValue.value.message,
      extra: shortMissingValue.value.extra,
    }
    : { success: false, };

  /** Known value option with no value, long spelling, issue 991. */
  const longMissingValue = parseSync(
    messageParser,
    ['--message',],
  );
  outcomes['missing/long'] = longMissingValue.success
    ? {
      success: true,
      message: longMissingValue.value.message,
      extra: longMissingValue.value.extra,
    }
    : { success: false, };

  return outcomes;
}

//endregion Entry point
