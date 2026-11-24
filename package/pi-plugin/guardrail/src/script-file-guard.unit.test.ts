/**
 Tests for pi's shell-script guard on `write` and `edit`.
 
 @module
 */

import {
  mkdtemp,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  evaluateScriptFileGuard,
  GUARDRAIL_NOT_BLOCKED,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Disposable working directory the guard resolves tool paths against.
 */
const fixtureCwd = await mkdtemp(join(scratchRoot(), 'pi-file-guard-literal-rm-',),);

/**
 Script holding an old computed deletion, which a change may repair.
 */
const REPAIRABLE_SCRIPT = join(fixtureCwd, 'repairable.sh',);

await writeFile(REPAIRABLE_SCRIPT, '#!/bin/sh\nrm -rf "$old"\n',);

//endregion Fixtures

//region Expectations

/**
 Reads the refusal reason of a guard decision.
 
 @param decision - decision the guard returned
 
 @returns the reason text, empty when the call is not blocked
 
 @example
 ```ts
 reasonOf(decision);
 ```
 */
function reasonOf(decision: unknown,): string {
  if (decision === GUARDRAIL_NOT_BLOCKED)
    return '';
  if ((typeof decision) !== 'object')
    return '';
  if (decision === null)
    return '';
  return ('reason' in decision) && ((typeof decision.reason) === 'string')
    ? decision.reason
    : '';
}

//endregion Expectations

await describe({
  name: evaluateScriptFileGuard.name,
  children: [
    //region write
    describe({
      name: 'a write call',
      children: [
        it({
          name: 'refuses a new script whose deletion computes its path',
          fn: async function testWriteViolatingScript() {
            expect(
              reasonOf(await evaluateScriptFileGuard({
                toolName: 'write',
                input: {
                  path: 'new-script.sh',
                  content: '#!/bin/sh\nrm -rf "$d"\n',
                },
                cwd: fixtureCwd,
              },),),
            ).toContain('(word `"$d"`;',);
          },
        },),
        it({
          name: 'refuses a script whose first line asks for a shell',
          fn: async function testWriteShebangScript() {
            expect(
              reasonOf(await evaluateScriptFileGuard({
                toolName: 'write',
                input: {
                  path: 'cleanup',
                  content: '#!/usr/bin/env bash\nrm -f "$d/$n"\n',
                },
                cwd: fixtureCwd,
              },),),
            ).toContain('(word `"$d/$n"`;',);
          },
        },),
        it({
          name: 'refuses a new script whose rmdir computes its path',
          fn: async function testWriteRmdirScript() {
            expect(
              reasonOf(await evaluateScriptFileGuard({
                toolName: 'write',
                input: {
                  path: 'cleanup.sh',
                  content: '#!/bin/sh\nrmdir -p "$d"\n',
                },
                cwd: fixtureCwd,
              },),),
            ).toContain('(word `"$d"`;',);
          },
        },),
        it({
          name: 'refuses a new script whose target collapsed to the filesystem root',
          fn: async function testWriteCollapsedScript() {
            /**
             Reason the guard gave for a script that deletes the filesystem root.
             */
            const reason = reasonOf(await evaluateScriptFileGuard({
              toolName: 'write',
              input: {
                path: 'wipe.sh',
                content: '#!/bin/sh\nrm -rf /\n',
              },
              cwd: fixtureCwd,
            },),);
            expect(reason,).toContain('the filesystem root',);
          },
        },),
        it({
          name: 'allows a script that writes every path out',
          fn: async function testWriteLiteralScript() {
            expect(
              await evaluateScriptFileGuard({
                toolName: 'write',
                input: {
                  path: 'literal.sh',
                  content: '#!/bin/sh\nrm --recursive /home/user/temp/agent/scratch\n',
                },
                cwd: fixtureCwd,
              },),
            ).toBe(GUARDRAIL_NOT_BLOCKED,);
          },
        },),
        it({
          name: 'allows a file that is no shell script',
          fn: async function testWriteOtherFile() {
            expect(
              await evaluateScriptFileGuard({
                toolName: 'write',
                input: {
                  path: 'notes.md',
                  content: 'rm -rf "$d"\n',
                },
                cwd: fixtureCwd,
              },),
            ).toBe(GUARDRAIL_NOT_BLOCKED,);
          },
        },),
        it({
          name: 'allows a rewrite that keeps the one old violation',
          fn: async function testWriteKeepsOldViolation() {
            expect(
              await evaluateScriptFileGuard({
                toolName: 'write',
                input: {
                  path: 'repairable.sh',
                  content: '#!/bin/sh\nrm -rf "$old"\n',
                },
                cwd: fixtureCwd,
              },),
            ).toBe(GUARDRAIL_NOT_BLOCKED,);
          },
        },),
      ],
    },),
    //endregion write

    //region edit
    describe({
      name: 'an edit call',
      children: [
        it({
          name: 'refuses an edit that adds a computed deletion',
          fn: async function testEditAddsViolation() {
            expect(
              reasonOf(await evaluateScriptFileGuard({
                toolName: 'edit',
                input: {
                  path: 'repairable.sh',
                  edits: [{
                    oldText: '"$old"',
                    newText: '"$new"',
                  },],
                },
                cwd: fixtureCwd,
              },),),
            ).toContain('(word `"$new"`;',);
          },
        },),
        it({
          name: 'refuses two edits that split one violation between them',
          fn: async function testEditSplits() {
            /**
             Script whose placeholder words hold no deletion yet.
             */
            const path = join(fixtureCwd, 'placeholder.sh',);
            await writeFile(path, '#!/bin/sh\nCOMMAND OPERAND\n',);
            expect(
              reasonOf(await evaluateScriptFileGuard({
                toolName: 'edit',
                input: {
                  path,
                  edits: [
                    {
                      oldText: 'COMMAND',
                      newText: 'rm -rf',
                    },
                    {
                      oldText: 'OPERAND',
                      newText: '"$split"',
                    },
                  ],
                },
                cwd: fixtureCwd,
              },),),
            ).toContain('(word `"$split"`;',);
          },
        },),
        it({
          name: 'allows an edit that repairs the old violation',
          fn: async function testEditRepairs() {
            expect(
              await evaluateScriptFileGuard({
                toolName: 'edit',
                input: {
                  path: 'repairable.sh',
                  edits: [{
                    oldText: 'rm -rf "$old"',
                    newText: 'rm --recursive /home/user/temp/agent/scratch',
                  },],
                },
                cwd: fixtureCwd,
              },),
            ).toBe(GUARDRAIL_NOT_BLOCKED,);
          },
        },),
      ],
    },),
    //endregion edit

    //region other tools
    describe({
      name: 'a tool that changes no file',
      children: [
        it({
          name: 'allows a tool this guard does not read',
          fn: async function testOtherTool() {
            expect(
              await evaluateScriptFileGuard({
                toolName: 'read',
                input: { path: 'repairable.sh', },
                cwd: fixtureCwd,
              },),
            ).toBe(GUARDRAIL_NOT_BLOCKED,);
          },
        },),
        it({
          name: 'allows a write whose content is not text',
          fn: async function testWriteWithoutContent() {
            expect(
              await evaluateScriptFileGuard({
                toolName: 'write',
                input: {
                  path: 'x.sh',
                  content: 1,
                },
                cwd: fixtureCwd,
              },),
            ).toBe(GUARDRAIL_NOT_BLOCKED,);
          },
        },),
        it({
          name: 'allows an edit whose replacement list holds no replacement',
          fn: async function testEditWithoutEdits() {
            expect(
              await evaluateScriptFileGuard({
                toolName: 'edit',
                input: {
                  path: 'x.sh',
                  edits: [{ oldText: 'a', },],
                },
                cwd: fixtureCwd,
              },),
            ).toBe(GUARDRAIL_NOT_BLOCKED,);
          },
        },),
        it({
          name: 'allows an input that is not an object',
          fn: async function testNotAnObject() {
            expect(
              await evaluateScriptFileGuard({
                toolName: 'write',
                input: 'not an object',
                cwd: fixtureCwd,
              },),
            ).toBe(GUARDRAIL_NOT_BLOCKED,);
          },
        },),
      ],
    },),
    //endregion other tools
  ],
},);
