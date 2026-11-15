//region Worktree-enforcement escape hatch constants

/**
 * Wrapper-only flag that suppresses linked-worktree enforcement for one
 * guarded invocation. Stripped before forwarding to real git, which would
 * otherwise reject it.
 */
export const WORKTREE_ENFORCEMENT_ESCAPE_HATCH = '--no-enforce-worktree';

/**
 * Pathspec separator after which git treats every remaining token as a path.
 * Wrapper-only options are not recognised past this token.
 */
export const PATHSPEC_SEPARATOR = '--';

//endregion Worktree-enforcement escape hatch constants

//region Escape hatch token stripping

/**
 * Options for stripping the escape hatch from post-subcommand argv.
 */
type StripEscapeHatchOptions = {
  /**
   * Complete git argv.
   */
  readonly args: readonly string[];
  /**
   * Index where guarded subcommand appears.
   */
  readonly subcommandIndex: number;
  /**
   * Options that consume next argv token as value.
   */
  readonly separateValueOptions: ReadonlySet<string>;
};

/**
 * Removes flag-position escape-hatch tokens from the post-subcommand region.
 * Tokens that sit in the value position of a separated-value option are
 * preserved so the resulting argv is forwarded with values intact. Pathspec
 * tokens past {@link PATHSPEC_SEPARATOR} are preserved verbatim.
 *
 * Optique handles the detection upstream by parsing each guarded subcommand's
 * post-region with arity-aware option declarations; this helper only does the
 * destructive token removal, via {@link filterFlagEscapeHatch}, that the
 * parsed result cannot express directly.
 *
 * @param args - Complete git argv.
 *
 * @param subcommandIndex - Index where guarded subcommand appears.
 *
 * @param separateValueOptions - Options whose next argv token is value.
 *
 * @returns Git argv with flag-position escape-hatch tokens removed.
 *
 * @example
 * ```ts
 * stripEscapeHatch({
 *   args: ['stash', '--no-enforce-worktree', 'list'],
 *   subcommandIndex: 0,
 *   separateValueOptions: new Set(),
 * });
 * // => ['stash', 'list']
 * ```
 */
export function stripEscapeHatch({
  args,
  subcommandIndex,
  separateValueOptions,
}: StripEscapeHatchOptions,): readonly string[] {
  /**
   * Pre-subcommand region and subcommand kept verbatim so global options survive the strip.
   */
  const preAndSubcommand = args.slice(
    0,
    subcommandIndex + 1,
  );
  /**
   * Slice of args strictly after guarded subcommand.
   */
  const postSubcommandArgs = args.slice(subcommandIndex + 1,);
  /**
   * Position of pathspec separator inside post-subcommand region.
   */
  const separatorIndex = postSubcommandArgs.indexOf(PATHSPEC_SEPARATOR,);

  if (separatorIndex === (-1)) {
    return [
      ...preAndSubcommand,
      ...filterFlagEscapeHatch({
        args: postSubcommandArgs,
        separateValueOptions,
      },),
    ];
  }

  /**
   * Region before pathspec separator where wrapper-only options are recognized.
   */
  const wrapperArgs = postSubcommandArgs.slice(
    0,
    separatorIndex,
  );
  /**
   * Pathspec separator and user path text preserved verbatim.
   */
  const pathspecArgs = postSubcommandArgs.slice(separatorIndex,);

  return [
    ...preAndSubcommand,
    ...filterFlagEscapeHatch({
      args: wrapperArgs,
      separateValueOptions,
    },),
    ...pathspecArgs,
  ];
}

/**
 * Options for the arity-aware escape-hatch filter.
 */
type FilterFlagEscapeHatchOptions = {
  /**
   * Argv slice to filter.
   */
  readonly args: readonly string[];
  /**
   * Options whose next argv token is value.
   */
  readonly separateValueOptions: ReadonlySet<string>;
};

/**
 * Filters flag-position escape-hatch tokens out of an argv slice while leaving
 * value-position tokens intact. Recursive walk that mirrors the optique
 * arity model used by the per-subcommand parsers.
 *
 * @param args - Argv slice to filter.
 *
 * @param separateValueOptions - Options whose next argv token is value.
 *
 * @returns Argv slice with flag-position escape-hatch tokens removed.
 *
 * @example
 * ```ts
 * filterFlagEscapeHatch({
 *   args: ['-m', '--no-enforce-worktree', '--no-enforce-worktree'],
 *   separateValueOptions: new Set(['-m']),
 * });
 * // => ['-m', '--no-enforce-worktree']
 * ```
 */
function filterFlagEscapeHatch({
  args,
  separateValueOptions,
}: FilterFlagEscapeHatchOptions,): readonly string[] {
  /**
   * Current token at the head of args.
   */
  const [arg, ...remaining] = args;

  if (arg === undefined)
    return [];

  if (separateValueOptions.has(arg,)) {
    /**
     * Value token that follows a separated-value option; kept verbatim.
     */
    const [value, ...afterValue] = remaining;

    if (value === undefined)
      return [arg,];

    return [
      arg,
      value,
      ...filterFlagEscapeHatch({
        args: afterValue,
        separateValueOptions,
      },),
    ];
  }

  if (arg === WORKTREE_ENFORCEMENT_ESCAPE_HATCH) {
    return filterFlagEscapeHatch({
      args: remaining,
      separateValueOptions,
    },);
  }

  return [
    arg,
    ...filterFlagEscapeHatch({
      args: remaining,
      separateValueOptions,
    },),
  ];
}

//endregion Escape hatch token stripping
