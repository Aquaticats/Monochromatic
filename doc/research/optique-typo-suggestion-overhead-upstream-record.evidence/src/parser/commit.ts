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

import { PATHSPEC_SEPARATOR, } from '../escape-hatch.ts';
import {
  DRY_RUN_COMMIT_ALIASES,
  INCLUDE_ALIASES,
  LONG_ALIASES,
  NULL_ALIASES,
  PORCELAIN_ALIASES,
  SHORT_ALIASES,
} from './commit-flag-aliases.ts';
import {
  extractCommitPathspecs,
  hasCommitPathspec,
  normaliseCommitArgs,
} from './commit-normalise.ts';

//region Commit escape hatch

/**
 * Wrapper-only flag that suppresses `-o` injection for one commit invocation.
 */
export const COMMIT_ESCAPE_HATCH = '--no-enforce-only';

//endregion Commit escape hatch

//region Commit post-subcommand optique parser

/**
 * Optique parser for the post-`commit` argv region.
 *
 * Declares the flags and value-taking options the commit-only rule needs to
 * decide whether to inject `-o`. Pathspec presence is detected separately by
 * {@link hasCommitPathspec}'s arity-aware scanner because Optique passthrough
 * cannot know whether unknown no-value flags such as `-q` consume the
 * following token.
 */
const commitRegionParser = object({
  allFlags: multiple(flag(
    '-a',
    '--all',
  ),),
  explicitOnlyFlags: multiple(flag(
    '-o',
    '--only',
  ),),
  noOnlyFlags: multiple(flag('--no-only',),),
  includeFlags: multiple(flag(
    '-i',
    ...INCLUDE_ALIASES,
  ),),
  interactiveFlags: multiple(flag('--interactive',),),
  patchFlags: multiple(flag(
    '-p',
    '--patch',
  ),),
  amendFlags: multiple(flag('--amend',),),
  allowEmptyFlags: multiple(flag('--allow-empty',),),
  dryRunFlags: multiple(flag(...DRY_RUN_COMMIT_ALIASES,),),
  shortFlags: multiple(flag(...SHORT_ALIASES,),),
  porcelainFlags: multiple(flag(...PORCELAIN_ALIASES,),),
  longFlags: multiple(flag(...LONG_ALIASES,),),
  nullFlags: multiple(flag(
    '-z',
    ...NULL_ALIASES,
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
  pathspecFileNulFlags: multiple(flag('--pathspec-file-nul',),),
  escape: multiple(flag(COMMIT_ESCAPE_HATCH,),),
  positionals: multiple(
    argument(string(),),
  ),
  unknownOptions: passThrough({ format: 'nextToken', },),
},);

//endregion Commit post-subcommand optique parser

//region Commit region facts

/**
 * Facts about the post-`commit` argv region used by commit-only policy.
 */
export type CommitRegion = {
  /**
   * Whether argv asks git to stage every tracked modification before committing.
   */
  readonly hasAllFlag: boolean;
  /**
   * Whether argv explicitly enables or disables only mode.
   */
  readonly hasExplicitOnlyFlag: boolean;
  /**
   * Whether argv explicitly disables only mode (`--no-only`).
   */
  readonly hasNoOnlyFlag: boolean;
  /**
   * Whether argv chooses include mode (`-i`/`--include`), which git forbids
   * combining with `--only`.
   */
  readonly hasIncludeFlag: boolean;
  /**
   * Whether argv requests interactive selection.
   */
  readonly hasInteractiveFlag: boolean;
  /**
   * Whether argv requests patch selection.
   */
  readonly hasPatchFlag: boolean;
  /**
   * Whether argv is a dry run that records no commit: `--dry-run` or any of
   * the output-format flags git documents as implying it (`--short`,
   * `--porcelain`, `--long`, `-z`/`--null`), in any accepted abbreviation.
   */
  readonly isDryRun: boolean;
  /**
   * Whether argv includes a mode where git permits no positional pathspec.
   */
  readonly hasPathlessAllowedFlag: boolean;
  /**
   * Whether argv amends the previous commit (`--amend`).
   */
  readonly hasAmendFlag: boolean;
  /**
   * Whether argv permits a commit recording no change (`--allow-empty`).
   */
  readonly hasAllowEmptyFlag: boolean;
  /**
   * Whether argv asks git to read pathspecs from file or stdin.
   */
  readonly hasPathspecFromFile: boolean;
  /**
   * Exact pathspec source spelling when present.
   */
  readonly pathspecFile?: string;
  /**
   * Whether pathspec file uses NUL delimiters.
   */
  readonly hasPathspecFileNul: boolean;
  /**
   * Whether argv includes at least one positional pathspec (before or after `--`).
   */
  readonly hasPathspec: boolean;
  /**
   * Whether wrapper-only escape hatch appears as a real flag.
   */
  readonly hasEscapeHatch: boolean;
  /**
   * Parsed positional pathspecs for supported explicit-path transactions.
   */
  readonly pathspecs: readonly string[];
};

/**
 * Parses the post-`commit` argv region into a structured fact set used by
 * the commit-only rule. Inline short-cluster values are normalised by
 * {@link normaliseCommitArgs} before optique parsing so `-mhello` and
 * `-mhello file.ts` are interpreted the same way real git would. Pathspec
 * presence is detected by {@link hasCommitPathspec} over the same normalised
 * argv because pass-through unknown options may otherwise consume no-value
 * flag pathspecs.
 *
 * @param postSubcommandArgs - Arguments strictly after `commit` subcommand.
 *
 * @returns Fact record consumed by commit-only policy.
 *
 * @example
 * ```ts
 * parseCommitRegion(['-am', 'hello']).hasAllFlag;
 * // => true
 * ```
 */
export function parseCommitRegion(
  postSubcommandArgs: readonly string[],
): CommitRegion {
  /**
   * Normalised argv where inline short-cluster values are split apart.
   */
  const normalised = normaliseCommitArgs(postSubcommandArgs,);
  /**
   * Position of pathspec separator after normalisation.
   */
  const separatorIndex = normalised.indexOf(PATHSPEC_SEPARATOR,);
  /**
   * Argv slice handed to optique; pathspec region is excluded.
   */
  const region = separatorIndex === (-1)
    ? normalised
    : normalised.slice(
      0,
      separatorIndex,
    );
  /**
   * Whether the normalised argv supplies a positional pathspec.
   */
  const hasPathspec = hasCommitPathspec(normalised,);

  /**
   * Optique parse result over the cleaned option region.
   */
  const parseResult = parseSync(
    commitRegionParser,
    region,
  );

  if (!parseResult.success) {
    return {
      hasAllFlag: false,
      hasExplicitOnlyFlag: false,
      hasNoOnlyFlag: false,
      hasIncludeFlag: false,
      hasInteractiveFlag: false,
      hasPatchFlag: false,
      isDryRun: false,
      hasPathlessAllowedFlag: false,
      hasAmendFlag: false,
      hasAllowEmptyFlag: false,
      hasPathspecFromFile: false,
      hasPathspecFileNul: false,
      hasPathspec: false,
      hasEscapeHatch: false,
      pathspecs: [],
    };
  }

  /**
   * Successful parse value with optique-inferred shape.
   */
  const { value, } = parseResult;
  /**
   * Sum of explicit only-mode flag occurrences (`-o`, `--only`, `--no-only`).
   */
  const explicitOnlyCount = value.explicitOnlyFlags
    .length
    + value
    .noOnlyFlags
    .length;
  /**
   * Sum of pathless-allowed flag occurrences (`--amend`, `--allow-empty`).
   */
  const pathlessAllowedCount = value.amendFlags
    .length
    + value
    .allowEmptyFlags
    .length;
  /**
   * Sum of dry-run flag occurrences, counting the output-format flags git
   * documents as implying `--dry-run`.
   */
  const dryRunCount = value.dryRunFlags
    .length
    + value
    .shortFlags
    .length
    + value
    .porcelainFlags
    .length
    + value
    .longFlags
    .length
    + value
    .nullFlags
    .length;

  return {
    hasAllFlag: value.allFlags
      .length
      > 0,
    hasExplicitOnlyFlag: explicitOnlyCount > 0,
    hasNoOnlyFlag: value.noOnlyFlags
      .length
      > 0,
    hasIncludeFlag: value.includeFlags
      .length
      > 0,
    hasInteractiveFlag: value.interactiveFlags
      .length
      > 0,
    hasPatchFlag: value.patchFlags
      .length
      > 0,
    isDryRun: dryRunCount > 0,
    hasPathlessAllowedFlag: pathlessAllowedCount > 0,
    hasAmendFlag: value.amendFlags
      .length
      > 0,
    hasAllowEmptyFlag: value.allowEmptyFlags
      .length
      > 0,
    hasPathspecFromFile: value.pathspecFromFile
      !== undefined,
    ...(value.pathspecFromFile === undefined ? {} : { pathspecFile: value.pathspecFromFile, }),
    hasPathspecFileNul: value.pathspecFileNulFlags
      .length
      > 0,
    hasPathspec,
    hasEscapeHatch: value.escape
      .length
      > 0,
    pathspecs: extractCommitPathspecs(normalised,),
  };
}

//endregion Commit region facts
