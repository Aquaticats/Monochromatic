/**
 Tests for the violations one file change would add to a shell script.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  literalRmPathFindingsAddedToScriptFile,
  textAfterEdits,
} from '@monochromatic-dev/agent-harness-shared-shell-command-analyzer';

//region Fixtures

/**
 Path of the script every fixture under test changes.
 */
const SCRIPT_PATH = '/repo/clean.sh';

/**
 A script that already deletes a computed path.
 */
const SCRIPT_WITH_OLD_VIOLATION = '#!/bin/sh\nrm -rf "$old"\n';

//endregion Fixtures

//region Expectations

/**
 Reads the words of every violation one change would add.
 
 @param before - file text before the change
 
 @param after - file text after the change
 
 @returns offending words of the added violations, in order
 
 @example
 ```ts
 addedWords({ before: '', after: 'rm -rf "$d"\n' }); // ['"$d"']
 ```
 */
function addedWords({
  before,
  after,
}: {
  readonly before: string;
  readonly after: string;
},): readonly string[] {
  return literalRmPathFindingsAddedToScriptFile({
    path: SCRIPT_PATH,
    before,
    after,
  },)
    .map(function wordOf(finding,): string {
      return finding.word;
    },);
}

//endregion Expectations

await describe({
  name: literalRmPathFindingsAddedToScriptFile.name,
  children: [
    //region Refused changes
    describe({
      name: 'a change that adds a computed path',
      children: [
        it({
          name: 'refuses a new script whose deletion names its path at run time',
          fn: async function testNewScript() {
            expect(addedWords({
              before: '',
              after: '#!/bin/sh\nrm -rf "$d"\n',
            },),).toEqual(['"$d"',],);
          },
        },),
        it({
          name: 'refuses an edit that turns a literal path into a computed one',
          fn: async function testEditedPath() {
            expect(addedWords({
              before: '#!/bin/sh\nrm -rf /home/user/temp/agent/scratch\n',
              after: '#!/bin/sh\nrm -rf "$d"\n',
            },),).toEqual(['"$d"',],);
          },
        },),
        it({
          name: 'refuses a change that adds a second copy of an existing violation',
          fn: async function testSecondCopy() {
            expect(addedWords({
              before: SCRIPT_WITH_OLD_VIOLATION,
              after: `${SCRIPT_WITH_OLD_VIOLATION}rm -rf "$old"\n`,
            },),).toEqual(['"$old"',],);
          },
        },),
        it({
          name: 'refuses a violation two edits could split between them',
          fn: async function testSplitViolation() {
            /**
             File whose placeholder words hold no deletion yet.
             */
            const before = 'COMMAND OPERAND\n';
            /**
             Text the two edits leave, which neither edit spells on its own.
             */
            const after = textAfterEdits({
              text: before,
              edits: [
                {
                  oldText: 'COMMAND',
                  newText: 'rm -rf',
                },
                {
                  oldText: 'OPERAND',
                  newText: '"$d"',
                },
              ],
            },);
            expect(after,).toBe('rm -rf "$d"\n',);
            expect(addedWords({
              before,
              after,
            },),).toEqual(['"$d"',],);
          },
        },),
      ],
    },),
    //endregion Refused changes

    //region Allowed changes
    describe({
      name: 'a change that adds no violation',
      children: [
        it({
          name: 'allows a repair of an old script, which keeps no violation',
          fn: async function testRepair() {
            expect(addedWords({
              before: SCRIPT_WITH_OLD_VIOLATION,
              after: '#!/bin/sh\nrm -rf /home/user/temp/agent/scratch\n',
            },),).toEqual([],);
          },
        },),
        it({
          name: 'allows a repair that leaves another old violation in place',
          fn: async function testPartialRepair() {
            /**
             Script whose first deletion is repaired and second is untouched.
             */
            const before = `${SCRIPT_WITH_OLD_VIOLATION}rm -rf "$other"\n`;
            expect(addedWords({
              before,
              after: before.replace('"$old"', '/home/user/temp/agent/scratch',),
            },),).toEqual([],);
          },
        },),
        it({
          name: 'allows a change that only moves an existing violation down the file',
          fn: async function testMovedViolation() {
            expect(addedWords({
              before: SCRIPT_WITH_OLD_VIOLATION,
              after: `#!/bin/sh\n\n${SCRIPT_WITH_OLD_VIOLATION.slice('#!/bin/sh\n'.length,)}`,
            },),).toEqual([],);
          },
        },),
        it({
          name: 'allows a change to a file no shell reads',
          fn: async function testOtherFile() {
            expect(
              literalRmPathFindingsAddedToScriptFile({
                path: '/repo/notes.md',
                before: '',
                after: 'rm -rf "$d"\n',
              },),
            ).toEqual([],);
          },
        },),
        it({
          name: 'allows a script change that writes every path out',
          fn: async function testLiteralScript() {
            expect(addedWords({
              before: '',
              after: '#!/bin/sh\nrm --recursive /home/user/temp/agent/scratch\n',
            },),).toEqual([],);
          },
        },),
      ],
    },),
    //endregion Allowed changes

    //region Reported locations
    describe({
      name: 'what a refused change reports',
      children: [
        it({
          name: 'names the file and the line of the added violation',
          fn: async function testReportedLocation() {
            /**
             Findings of one change that adds a violation below two existing lines.
             */
            const findings = literalRmPathFindingsAddedToScriptFile({
              path: SCRIPT_PATH,
              before: '#!/bin/sh\necho hi\n',
              after: '#!/bin/sh\necho hi\nrm -rf "$d"\n',
            },);
            expect(findings[0]?.file,).toBe(SCRIPT_PATH,);
            expect(findings[0]?.line,).toBe(3,);
          },
        },),
      ],
    },),
    //endregion Reported locations
  ],
},);
