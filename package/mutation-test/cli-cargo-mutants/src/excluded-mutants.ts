/**
 Mutant kinds that no campaign in this repository tries, as regular expressions matched against the names
 `cargo mutants --list` prints.

 Turning `+= 1` into `*= 1`, or `-= 1` into `/= 1`, leaves a counter unchanged, so a loop stepping one never ends
 and the mutant only reaches the test timeout.
 cargo-mutants exits with its timeout status whenever any mutant timed out and has no option to change that,
 so one such mutant fails a whole campaign; see `doc/troubleshooting/cargo-mutants-timeout-exit-status.md`.
 The human accepted on 2026-10-05 that these two kinds are never tried, for every product, and decided on
 2026-10-29 that this list stays one repository-wide constant: a product adds exclusions on top of it and never
 removes one.
 `replace += with -=` and `replace -= with +=` stay active as the check on every counter.
 */
export const excludedMutantKinds: readonly string[] = [
  String.raw`replace \+= with \*=`,
  'replace -= with /=',
];

/**
 Build the `--exclude-re` arguments of a campaign: the repository-wide kinds first, then the product's own.

 @param additional - patterns a product excludes on top of the repository-wide kinds, such as proven-equivalent mutants
 
 @returns one `--exclude-re <pattern>` pair per pattern, in that order
 
 @example
 ```ts
 exclusionArguments(['replace foo with bar']);
 ```
 */
export function exclusionArguments(additional: readonly string[]): string[] {
  return [
    ...excludedMutantKinds,
    ...additional,
  ].flatMap(function excludePair(pattern) {
    return [
      '--exclude-re',
      pattern,
    ];
  });
}
