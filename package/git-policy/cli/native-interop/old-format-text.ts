/**
 Text and process helpers shared by the checks of `old-format-fixtures.mjs`.
 */
/// <reference types="node" />
import {
  REAL_GIT,
  run,
  runEnvironment,
} from './support.mjs';

/** Longest excerpt of a status or fsck output that a check message carries. */
export const SHORT_EXCERPT = 300;
/** Longest excerpt of the standard error of a native command that a check message carries. */
export const LONG_EXCERPT = 400;

/**
 The start of a text, trimmed, for a check message.

 @param {{ text: string, limit: number }} request - text and the longest excerpt wanted
 @returns {string} the excerpt
 */
export function excerpt({
  text,
  limit,
}) {
  return text
    .trim()
    .slice(
    0,
    limit,
    );
}

/**
 Lines of a text that are not empty.

 @param {string} text - text to split
 @returns {string[]} the non-empty lines
 */
export function linesOf(text) {
  return text
    .split('\n')
    .filter(function isNonEmpty(line) {
      return line.length > 0;
    });
}

/**
 Run `git fsck` through real Git in a restored fixture.

 @param {{ repository: string, home: string }} fixture - restored fixture
 @returns {Promise<import('./support.mjs').RunOutcome>} the outcome
 */
export function fsckOf(fixture) {
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
