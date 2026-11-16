import { PATHSPEC_SEPARATOR, } from '../escape-hatch.ts';
import {
  type ArgvSpec,
  parseArgv,
} from './argv.ts';
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

//region Commit post-subcommand region parser

/**
 * Declared option surface of the post-`commit` argv region.
 *
 * Declares the flags the commit-only rule reads, plus every value-taking option
 * git accepts here. Most declared values are never read: declaring them is what
 * stops a value such as the `-m` message from being counted as a pathspec.
 * Pathspec presence is still detected separately by {@link hasCommitPathspec}'s
 * arity-aware scanner, because an undeclared option's arity is unknowable and no
 * region parser can decide whether `-q` consumes the following token.
 */
const commitRegionSpec: ArgvSpec = {
  flags: {
    allFlags: { names: [
      '-a',
      '--all',
    ], },
    explicitOnlyFlags: { names: [
      '-o',
      '--only',
    ], },
    noOnlyFlags: { names: ['--no-only',], },
    includeFlags: { names: [
      '-i',
      ...INCLUDE_ALIASES,
    ], },
    interactiveFlags: { names: ['--interactive',], },
    patchFlags: { names: [
      '-p',
      '--patch',
    ], },
    amendFlags: { names: ['--amend',], },
    allowEmptyFlags: { names: ['--allow-empty',], },
    dryRunFlags: { names: [...DRY_RUN_COMMIT_ALIASES,], },
    shortFlags: { names: [...SHORT_ALIASES,], },
    porcelainFlags: { names: [...PORCELAIN_ALIASES,], },
    longFlags: { names: [...LONG_ALIASES,], },
    nullFlags: { names: [
      '-z',
      ...NULL_ALIASES,
    ], },
    pathspecFileNulFlags: { names: ['--pathspec-file-nul',], },
    escape: { names: [COMMIT_ESCAPE_HATCH,], },
  },
  valueOptions: {
    message: { names: [
      '-m',
      '--message',
    ], },
    file: { names: [
      '-F',
      '--file',
    ], },
    reuseMessage: { names: [
      '-C',
      '--reuse-message',
    ], },
    reeditMessage: { names: [
      '-c',
      '--reedit-message',
    ], },
    squash: { names: ['--squash',], },
    fixup: { names: ['--fixup',], },
    author: { names: ['--author',], },
    date: { names: ['--date',], },
    cleanup: { names: ['--cleanup',], },
    trailer: { names: ['--trailer',], },
    template: { names: [
      '-t',
      '--template',
    ], },
    unified: { names: [
      '-U',
      '--unified',
    ], },
    interHunkContext: { names: ['--inter-hunk-context',], },
    pathspecFromFile: { names: ['--pathspec-from-file',], },
  },
};

//endregion Commit post-subcommand region parser

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
 * {@link normaliseCommitArgs} before region parsing so `-mhello` and
 * `-mhello file.ts` are interpreted the same way real git would. Pathspec
 * presence is detected by {@link hasCommitPathspec} over the same normalised
 * argv because pass-through unknown options may otherwise consume no-value
 * flag pathspecs.
 *
 * A region this parser refuses to read raises rather than degrading to an
 * empty fact set: every fact here decides whether the commit-only rule injects
 * `-o`, so guessing from a misread region weakens the guard it feeds.
 *
 * @param postSubcommandArgs - Arguments strictly after `commit` subcommand.
 *
 * @returns Fact record consumed by commit-only policy.
 *
 * @throws ArgvParseError when region names a token no reading can settle.
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
   * Argv slice handed to region parser; pathspec region is excluded.
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
   * Parsed facts over the cleaned option region.
   */
  const {
    flagCounts,
    optionValues,
  } = parseArgv({
    args: region,
    spec: commitRegionSpec,
  },);
  /**
   * Exact pathspec source spelling, taking git's last-wins order.
   */
  const pathspecFile = (optionValues.pathspecFromFile ?? []).at(-1,);
  /**
   * Sum of explicit only-mode flag occurrences (`-o`, `--only`, `--no-only`).
   */
  const explicitOnlyCount = (flagCounts.explicitOnlyFlags ?? 0)
    + (flagCounts.noOnlyFlags ?? 0);
  /**
   * Sum of pathless-allowed flag occurrences (`--amend`, `--allow-empty`).
   */
  const pathlessAllowedCount = (flagCounts.amendFlags ?? 0)
    + (flagCounts.allowEmptyFlags ?? 0);
  /**
   * Sum of dry-run flag occurrences, counting the output-format flags git
   * documents as implying `--dry-run`.
   */
  const dryRunCount = (flagCounts.dryRunFlags ?? 0)
    + (flagCounts.shortFlags ?? 0)
    + (flagCounts.porcelainFlags ?? 0)
    + (flagCounts.longFlags ?? 0)
    + (flagCounts.nullFlags ?? 0);

  return {
    hasAllFlag: (flagCounts.allFlags ?? 0) > 0,
    hasExplicitOnlyFlag: explicitOnlyCount > 0,
    hasNoOnlyFlag: (flagCounts.noOnlyFlags ?? 0) > 0,
    hasIncludeFlag: (flagCounts.includeFlags ?? 0) > 0,
    hasInteractiveFlag: (flagCounts.interactiveFlags ?? 0) > 0,
    hasPatchFlag: (flagCounts.patchFlags ?? 0) > 0,
    isDryRun: dryRunCount > 0,
    hasPathlessAllowedFlag: pathlessAllowedCount > 0,
    hasAmendFlag: (flagCounts.amendFlags ?? 0) > 0,
    hasAllowEmptyFlag: (flagCounts.allowEmptyFlags ?? 0) > 0,
    hasPathspecFromFile: pathspecFile !== undefined,
    ...(pathspecFile === undefined ? {} : { pathspecFile, }),
    hasPathspecFileNul: (flagCounts.pathspecFileNulFlags ?? 0) > 0,
    hasPathspec,
    hasEscapeHatch: (flagCounts.escape ?? 0) > 0,
    pathspecs: extractCommitPathspecs(normalised,),
  };
}

//endregion Commit region facts
