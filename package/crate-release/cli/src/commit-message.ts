import {
  commitType,
  maximumNoteLength,
} from './constants.ts';
import { CrateReleaseError } from './error.ts';
import {
  hasControlCharacter,
  parseCommitScope,
} from './validate.ts';
import { assertReleaseVersion } from './version.ts';

/**
 Build the subject of a release commit: `chore(<scope>): release <version>: <note>`.

 The result is one line, so it is a complete conventional-commit subject. The note is kept verbatim, because the
 message travels to Git as one argument and no shell reads it; the note may hold quotes, `$`, `;` and backticks,
 but not a line break or another control character, which would turn the rest of the note into a commit body.

 @param scope - package scope of the subject, such as `git-policy/cli`

 @param version - version being released

 @param note - one-line summary of the release

 @returns the commit subject

 @throws CrateReleaseError when the scope or version is invalid, or the note is blank, longer than the maximum, or
 holds a control character

 @example
 ```ts
 buildCommitMessage({ scope: 'git-policy/cli', version: '0.4.10', note: 'fix the scanner' });
 ```
 */
export function buildCommitMessage({
  scope,
  version,
  note,
}: {
  readonly scope: string;
  readonly version: string;
  readonly note: string;
}): string {
  parseCommitScope({
    option: '--scope',
    value: scope
  });
  assertReleaseVersion({
    option: '<to>',
    value: version
  });
  if (note.trim() === '')
    throw new CrateReleaseError('The release <note> must not be empty or only spaces.');
  if (hasControlCharacter(note))
    throw new CrateReleaseError(
      `The release <note> must be one line without control characters (line breaks, tabs, NUL), got ${JSON.stringify(note)}.`,
    );
  if (note.length > maximumNoteLength)
    throw new CrateReleaseError(
      `The release <note> must be at most ${String(maximumNoteLength)} characters, got ${String(note.length)}.`,
    );
  return `${commitType}(${scope}): release ${version}: ${note}`;
}
