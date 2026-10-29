/**
 Text and process helpers shared by the checks of `old-format-fixtures.ts`.
 */
import {
  REAL_GIT,
  run,
  runEnvironment,
} from './support.ts';
import type { RunOutcome } from './support.ts';

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

 @param request - text and the longest excerpt wanted
 
 @returns the excerpt
 */
export function excerpt({
  text,
  limit,
}: {
  text: string;
  limit: number
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
 */
export function fsckOf(fixture: {
  repository: string;
  home: string
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
