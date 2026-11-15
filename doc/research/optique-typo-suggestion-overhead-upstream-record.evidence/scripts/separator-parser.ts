// The separator shape: a parser shaped like the old commit parser's option set,
// with the thousand positional paths placed after the `--` separator, so the
// library itself sees every token and only the commit wrapper would have cut
// them off. Parser text follows coordinator-check/separator-variant.ts exactly.
// Copied into every arm directory at image build time.

import { object, } from '@optique/core/constructs';
import {
  multiple,
  optional,
} from '@optique/core/modifiers';
import { parseSync, } from '@optique/core/parser';
import {
  argument,
  flag,
  option,
  passThrough,
} from '@optique/core/primitives';
import { string, } from '@optique/core/valueparser';

//region Parser

/**
 * Expands a long option into the abbreviations git accepts, so each flag
 * declares every spelling.
 *
 * @param name - Full long option spelling.
 *
 * @param shortestStem - Shortest accepted stem length.
 *
 * @returns Option names in descending length.
 */
function prefixes(name: string, shortestStem: number,): [unknown, ...unknown[]] {
  /** Option name without the leading dashes. */
  const stem = name.slice(2,);
  /** Accepted spellings. */
  const names: string[] = [];

  for (let length = shortestStem; length <= stem.length; length += 1) {
    names.push(`--${stem.slice(
      0,
      length,
    )}`,);
  }

  return names as [unknown, ...unknown[]];
}

/**
 * Parser for the separator shape: the same option set the old commit parser
 * declared, with pass-through for unknown options.
 */
const separatorParser = object({
  all: multiple(flag(
    '-a',
    '--all',
  ),),
  only: multiple(flag(
    '-o',
    '--only',
  ),),
  noOnly: multiple(flag('--no-only',),),
  include: multiple(flag(
    '-i',
    ...prefixes(
      '--include',
      3,
    ),
  ),),
  interactive: multiple(flag('--interactive',),),
  patch: multiple(flag(
    '-p',
    '--patch',
  ),),
  amend: multiple(flag('--amend',),),
  allowEmpty: multiple(flag('--allow-empty',),),
  dryRun: multiple(flag(...prefixes(
    '--dry-run',
    2,
  ),),),
  short: multiple(flag(...prefixes(
    '--short',
    2,
  ),),),
  porcelain: multiple(flag(...prefixes(
    '--porcelain',
    3,
  ),),),
  long: multiple(flag(...prefixes(
    '--long',
    1,
  ),),),
  nul: multiple(flag(
    '-z',
    ...prefixes(
      '--null',
      2,
    ),
  ),),
  message: optional(option(
    '-m',
    '--message',
    string(),
  ),),
  file: optional(option(
    '-F',
    '--file',
    string(),
  ),),
  reuseMessage: optional(option(
    '-C',
    '--reuse-message',
    string(),
  ),),
  reeditMessage: optional(option(
    '-c',
    '--reedit-message',
    string(),
  ),),
  squash: optional(option(
    '--squash',
    string(),
  ),),
  fixup: optional(option(
    '--fixup',
    string(),
  ),),
  author: optional(option(
    '--author',
    string(),
  ),),
  date: optional(option(
    '--date',
    string(),
  ),),
  cleanup: optional(option(
    '--cleanup',
    string(),
  ),),
  trailer: multiple(option(
    '--trailer',
    string(),
  ),),
  template: optional(option(
    '-t',
    '--template',
    string(),
  ),),
  unified: optional(option(
    '-U',
    '--unified',
    string(),
  ),),
  interHunkContext: optional(option(
    '--inter-hunk-context',
    string(),
  ),),
  pathspecFromFile: optional(option(
    '--pathspec-from-file',
    string(),
  ),),
  pathspecFileNul: multiple(flag('--pathspec-file-nul',),),
  paths: multiple(argument(string(),),),
  unknown: passThrough({ format: 'nextToken', },),
},);

//endregion Parser

//region Runner

/**
 * What one separator parse did.
 */
export type SeparatorOutcome = {
  readonly success: boolean;
  readonly tokens: number;
};

/**
 * Builds a runner for the separator shape.
 *
 * @param tokenCount - How many paths follow the separator.
 *
 * @returns Runner reporting parse success and accepted positional count.
 */
export function createSeparatorRunner(tokenCount: number,): () => SeparatorOutcome {
  /** Paths the argv carries after the separator. */
  const paths = Array.from(
    { length: tokenCount, },
    function path(_value, index,) {
      return `package/group-${index % 37}/package-${index}/src/feature-${index}.unit.test.ts`;
    },
  );
  /** Argv with the separator before the paths. */
  const args = [
    '--message',
    'x',
    '--',
    ...paths,
  ];

  return function run() {
    /** Parse result for the prepared argv. */
    const result = parseSync(
      separatorParser,
      args,
    );

    if (!result.success) {
      return {
        success: false,
        tokens: 0,
      };
    }

    /** Positional values the parse accepted. */
    const accepted = (result.value as { readonly paths: readonly string[]; }).paths.length;

    return {
      success: accepted === tokenCount,
      tokens: accepted,
    };
  };
}

//endregion Runner
