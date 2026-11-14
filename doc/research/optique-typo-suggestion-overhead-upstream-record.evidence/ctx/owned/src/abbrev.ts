//region Long-option abbreviation expansion

/**
 * Long-option prefix used by git command-line conventions.
 */
const LONG_OPTION_PREFIX = '--';

/**
 * Default shortest stem accepted by git for unambiguous abbreviations.
 */
const DEFAULT_SHORTEST_STEM_LENGTH = 1;

/**
 * Options for expanding a long option into its accepted abbreviations.
 */
type ExpandAbbreviationsOptions = {
  /**
   * Full long-option spelling such as `--dry-run`.
   */
  readonly longOption: string;
  /**
   * Shortest stem length accepted by git for this option, excluding the `--`.
   */
  readonly minStemLength?: number;
};

/**
 * Optique-compatible long-option alias literal.
 */
export type LongOptionAlias = `--${string}`;

/**
 * Expands a long option into the descending list of abbreviation aliases that
 * git's argument parser accepts when the abbreviation is unambiguous. Used to
 * declare option aliases for optique parsers so each declared option matches
 * every form git would accept, closing parser-mismatch bypasses.
 *
 * @param longOption - Full long-option spelling such as `--dry-run`.
 *
 * @param minStemLength - Shortest stem length accepted by git for this option.
 *
 * @returns Aliases in descending length, starting with the full option spelling.
 *
 * @throws When long option does not start with `--` or stem length is invalid.
 *
 * @example
 * ```ts
 * expandAbbreviations({ longOption: '--dry-run', minStemLength: 1 });
 * // => ['--dry-run', '--dry-ru', '--dry-r', '--dry-', '--dry', '--dr', '--d']
 * ```
 */
export function expandAbbreviations({
  longOption,
  minStemLength = DEFAULT_SHORTEST_STEM_LENGTH,
}: ExpandAbbreviationsOptions,): readonly [
  LongOptionAlias,
  ...LongOptionAlias[],
] {
  if (!longOption.startsWith(LONG_OPTION_PREFIX,)) {
    throw new Error(
      `expandAbbreviations: long option must start with -- (got ${longOption}).`,
    );
  }

  /**
   * Option name without leading `--`.
   */
  const stem = longOption.slice(LONG_OPTION_PREFIX.length,);

  if (minStemLength <= 0) {
    throw new Error(
      `expandAbbreviations: minStemLength must be >= 1 (got ${minStemLength}).`,
    );
  }

  if (minStemLength > stem
    .length) {
    throw new Error(
      `expandAbbreviations: minStemLength ${minStemLength} exceeds stem length ${stem.length}.`,
    );
  }

  /**
   * Number of aliases generated; equals the stem-length range.
   */
  const aliasCount = (stem.length
    - minStemLength) + 1;
  /**
   * Aliases generated in descending stem length, full spelling first.
   */
  const aliases = Array.from(
    { length: aliasCount, },
    function alias(
      _v,
      i,
    ): LongOptionAlias {
      return `${LONG_OPTION_PREFIX}${
        stem.slice(
          0,
          stem.length
            - i,
        )
      }`;
    },
  );

  /* oxlint-disable typescript/no-unsafe-type-assertion -- aliasCount >= 1 guarantees non-empty array; the tuple form is the shape optique's option()/flag() require. */
  /* oxlint-disable stylistic/tuple-per-line -- the cast target is metadata about the tuple shape, not a literal; one-line layout keeps the cast adjacent to the return. */
  return aliases as [LongOptionAlias, ...LongOptionAlias[],];
  /* oxlint-enable stylistic/tuple-per-line */
  /* oxlint-enable typescript/no-unsafe-type-assertion */
}

//endregion Long-option abbreviation expansion
