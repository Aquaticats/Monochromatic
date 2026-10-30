import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  adversarialLines,
  controlCharacterValues,
} from './adversarial-fixture.ts';
import { buildCommitMessage } from './commit-message.ts';
import { maximumNoteLength } from './constants.ts';
import { CrateReleaseError } from './error.ts';

/**
 Message for the usual scope and version with the given note.

 @param note - release note
 @returns the commit message
 */
function messageFor(note: string): string {
  return buildCommitMessage({ scope: 'git-policy/cli', version: '0.4.10', note });
}

/**
 Run a build that must throw and return what it threw.

 @param action - build under test
 @returns the thrown value
 */
function thrownBy(action: () => unknown): unknown {
  try {
    action();
  }
  catch (error) {
    return error;
  }
  return undefined;
}

await describe({
  name: '',
  children: [
    describe({
      name: buildCommitMessage.name,
      children: [
        it({
          name: 'writes the conventional subject with scope, version and note',
          fn: async () => {
            expect(messageFor('fix the scanner')).toBe('chore(git-policy/cli): release 0.4.10: fix the scanner');
          },
        }),
        ...adversarialLines.map(function noteCase(note) {
          return it({
            name: `keeps the note ${JSON.stringify(note)} verbatim as the end of one line`,
            fn: async () => {
              const message = messageFor(note);
              expect(message).toBe(`chore(git-policy/cli): release 0.4.10: ${note}`);
              expect(message.includes('\n')).toBe(false);
            },
          });
        }),
        ...controlCharacterValues.map(function controlCase(note) {
          return it({
            name: `rejects the note ${JSON.stringify(note)}`,
            fn: async () => {
              expect(thrownBy(() => messageFor(note))).toBeInstanceOf(CrateReleaseError);
            },
          });
        }),
        it({
          name: 'rejects an empty note',
          fn: async () => {
            expect(thrownBy(() => messageFor(''))).toBeInstanceOf(CrateReleaseError);
          },
        }),
        it({
          name: 'rejects a note of only spaces',
          fn: async () => {
            expect(thrownBy(() => messageFor('   '))).toBeInstanceOf(CrateReleaseError);
          },
        }),
        it({
          name: 'accepts a note of exactly the maximum length',
          fn: async () => {
            expect(messageFor('a'.repeat(maximumNoteLength))).toContain('a'.repeat(maximumNoteLength));
          },
        }),
        it({
          name: 'rejects a note one character over the maximum length',
          fn: async () => {
            expect(thrownBy(() => messageFor('a'.repeat(maximumNoteLength + 1)))).toBeInstanceOf(CrateReleaseError);
          },
        }),
        ...[
          '',
          'has space',
          'a(b',
          'a)b',
          'a:b',
          'a\nb',
          '../x',
          '/abs',
          "a'b",
        ].map(function scopeCase(scope) {
          return it({
            name: `rejects the scope ${JSON.stringify(scope)}`,
            fn: async () => {
              expect(thrownBy(() => buildCommitMessage({ scope, version: '0.4.10', note: 'n' }))).toBeInstanceOf(CrateReleaseError);
            },
          });
        }),
        ...[
          'pkg',
          'git-policy/cli',
          'a.b_c-d',
        ].map(function acceptedScopeCase(scope) {
          return it({
            name: `accepts the scope ${JSON.stringify(scope)}`,
            fn: async () => {
              expect(buildCommitMessage({ scope, version: '1.0.0-rc.1', note: 'n' })).toBe(`chore(${scope}): release 1.0.0-rc.1: n`);
            },
          });
        }),
        it({
          name: 'rejects a version that is not a release version',
          fn: async () => {
            expect(thrownBy(() => buildCommitMessage({ scope: 's', version: '1.0', note: 'n' }))).toBeInstanceOf(CrateReleaseError);
          },
        }),
      ],
    }),
  ],
});
