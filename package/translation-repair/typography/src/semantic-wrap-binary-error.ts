import {
  LINT_BINARY_NAME,
  PINNED_LINT_BINARY_VERSION,
} from './semantic-wrap-binary-constants.ts';

//region Semantic wrap binary errors
// WHAT TO SAY WHEN THE PROGRAM IS MISSING OR IS ANOTHER VERSION, which are the
// two failures an operator fixes by installing something. Both messages carry
// the same advice, written once.

/**
 What to do about a program that is missing or is another version.
 */
const INSTALL_ADVICE = `Install version ${PINNED_LINT_BINARY_VERSION} with "mise run toolchain:sync" (the tool is `
  + `"cargo:${LINT_BINARY_NAME}" in toolchain.lock.json) or with "cargo binstall `
  + `${LINT_BINARY_NAME}@${PINNED_LINT_BINARY_VERSION}", and start this program under "mise run" or with `
  + `the tree's ".cache/toolchain-bin" directory first on the PATH, so that the pinned tool is the one found.`;

/**
 Stands for a program's answer to `--version` that is not its name and one
 version token, so no text of that answer is repeated.
 */
export const UNREADABLE_ANSWER: unique symbol = Symbol('the program answered with no readable version',);

/**
 Words for the version a program answered with.

 @param found - version token it answered with, or the stand-in for an answer that was not a version line

 @returns The clause naming that version, or saying the answer could not be read

 @example
 ```ts
 const clause = versionClause({ found: '0.1.1', },); // 'version 0.1.1'
 ```
 */
export function versionClause({ found, }: { readonly found: string | typeof UNREADABLE_ANSWER; },): string {
  if (found === UNREADABLE_ANSWER)
    return 'not a version this package can read';
  return `version ${found}`;
}

/**
 Signals that the program is not where the PATH lookup looks.

 @example
 ```ts
 throw new LintBinaryMissingError({ binary: 'monochromatic-lint', },);
 ```
 */
export class LintBinaryMissingError extends Error {
  /**
   Declares this message safe to forward: it names the program, which a
   constant or an operator supplies, and writes the rest itself.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds the failure naming the program and every way to install it.

   @param binary - program that was looked up

   @example
   ```ts
   new LintBinaryMissingError({ binary: 'monochromatic-lint', },);
   ```
   */
  public constructor({ binary, }: { readonly binary: string; },) {
    super(
      `${binary} was not found on the PATH, and shipped text cannot be wrapped at its semantic boundaries `
        + `without it, because it is this repository's pinned Markdown linter. ${INSTALL_ADVICE}`,
    );
    this.name = 'LintBinaryMissingError';
  }
}

/**
 Signals that the program found is not the version this package is written
 against.

 @example
 ```ts
 throw new LintBinaryVersionError({ binary: 'monochromatic-lint', found: '0.1.1', },);
 ```
 */
export class LintBinaryVersionError extends Error {
  /**
   Declares this message safe to forward: it names the program and a version
   token read from its answer, checked to hold nothing but version characters.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds the failure naming both versions and the way to install the pinned one.

   @param binary - program that was asked

   @param found - version token it answered with, or the stand-in for an answer that was not a version line

   @example
   ```ts
   new LintBinaryVersionError({ binary: 'monochromatic-lint', found: '0.1.1', },);
   ```
   */
  public constructor(
    {
      binary,
      found,
    }: {
      readonly binary: string;
      readonly found: string | typeof UNREADABLE_ANSWER;
    },
  ) {
    super(
      `${binary} is ${versionClause({ found, },)}, but shipped text is wrapped by version `
        + `${PINNED_LINT_BINARY_VERSION} only, because the wrap is that version's rule. ${INSTALL_ADVICE}`,
    );
    this.name = 'LintBinaryVersionError';
  }
}

//endregion Semantic wrap binary errors
