/**
 Text and process helpers shared by the checks of `old-format-driver.ts`.
 */
import {
  REAL_GIT,
  run,
  runEnvironment,
  type RunOutcome,
} from './support.ts';

/**
 Longest excerpt of a status or fsck output that a check message carries.
 */
export const SHORT_EXCERPT = 300;
/**
 Longest excerpt of the standard error of a native command that a check message carries.
 */
export const LONG_EXCERPT = 400;

/**
 The start of a text, trimmed, for a check message.

 @param text - text to excerpt

 @param limit - longest excerpt wanted

 @returns the excerpt

 @example
 ```ts
 const shown = excerpt({ text: output, limit: SHORT_EXCERPT });
 ```
 */
export function excerpt({
  text,
  limit,
}: {
  readonly text: string;
  readonly limit: number;
}): string {
  return text
    .trim()
    .slice(
    0,
    limit,
    );
}

/**
 Lines of a text that are not empty.

 @param text - text to split
 
 @returns the non-empty lines

 @example
 ```ts
 const lines = linesOf('a\n\nb');
 ```
 */
export function linesOf(text: string): string[] {
  return text
    .split('\n')
    .filter(function isNonEmpty(line) {
      return line.length > 0;
    });
}

/**
 Run `git fsck` through real Git in a restored fixture.

 @param fixture - restored fixture
 
 @returns the outcome

 @example
 ```ts
 const outcome = await fsckOf(fixture);
 ```
 */
export function fsckOf(fixture: {
  readonly repository: string;
  readonly home: string;
}): Promise<RunOutcome> {
  return run({
    command: REAL_GIT,
    args: [
      'fsck',
      '--no-dangling'
    ],
    cwd: fixture.repository,
    env: runEnvironment({ home: fixture.home }),
  });
}
