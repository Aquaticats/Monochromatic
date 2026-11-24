/**
 Tests for the literal-`rm`-path guardrail on the Bash, Write, Edit and MultiEdit tools.
 
 Every case names the surface and the branch it covers,
 so a passing run can be compared against the surfaces the guardrail claims to read.
 
 @module
 */

import {
  mkdir,
  mkdtemp,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import type { PreToolUseInput, } from '@monochromatic-dev/claude-code-plugin-hook-type/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  guardrailHandler,
  type GuardrailOutput,
} from './guardrail.ts';

//region Fixtures

/**
 Disposable folder every file fixture lives in.
 */
const fixtureRoot = await mkdtemp(join(scratchRoot(), 'cc-guardrail-literal-rm-',),);

/**
 Script holding a computed deletion, which a Bash call runs.
 */
const VIOLATING_SCRIPT = join(fixtureRoot, 'deploy.sh',);

/**
 Script holding an old computed deletion, which a change may repair.
 */
const REPAIRABLE_SCRIPT = join(fixtureRoot, 'repairable.sh',);

/**
 Notes file that is no shell script, whose text a change may hold anything.
 */
const NOTES_FILE = join(fixtureRoot, 'notes.md',);

/**
 Folder a here-document writes a script into.
 */
const GENERATED_FOLDER = join(fixtureRoot, 'generated',);

/**
 Refusal sentence naming the spelling the rule asks for.
 */
const REMEDY_SENTENCE = '`rm --recursive /home/user/temp/agent/<folder>/scratch`';

await writeFile(VIOLATING_SCRIPT, '#!/bin/sh\nrm -rf "$d"\n',);
await writeFile(REPAIRABLE_SCRIPT, '#!/bin/sh\nrm -rf "$old"\n',);
await writeFile(NOTES_FILE, 'plain notes\n',);
await mkdir(GENERATED_FOLDER,);

//endregion Fixtures

//region Event builders

/**
 Builds a PreToolUse event for one tool call.
 
 @param toolName - tool the model called
 
 @param toolInput - that tool's input
 
 @returns event Claude Code would dispatch
 
 @example
 ```ts
 makeEvent({ toolName: 'Bash', toolInput: { command: 'rm -rf "$d"' } });
 ```
 */
function makeEvent({
  toolName,
  toolInput,
}: {
  readonly toolName: string;
  readonly toolInput: Record<string, unknown>;
},): PreToolUseInput {
  return {
    hook_event_name: 'PreToolUse',
    session_id: 'session-test',
    transcript_path: '/tmp/transcript.jsonl',
    cwd: fixtureRoot,
    permission_mode: 'default',
    tool_name: toolName,
    tool_input: toolInput,
    tool_use_id: 'tool-use-test',
  };
}

//endregion Event builders

//region Expectations

/**
 Reads the refusal reason of a deny response.
 
 @param output - response to inspect
 
 @returns the reason text, or empty text when the output allows the call
 
 @example
 ```ts
 denialReason(isDeny(output) ? output : {});
 ```
 */
function denialReason(output: GuardrailOutput,): string {
  if (!('hookSpecificOutput' in output))
    return '';
  return output.hookSpecificOutput.permissionDecisionReason ?? '';
}

//endregion Expectations

await describe({
  name: 'literal rm path guardrail',
  children: [
    //region Bash
    describe({
      name: 'a Bash call',
      children: [
        it({
          name: 'refuses a deletion whose path the command computes',
          fn: async function testComputedPathCommand() {
            const output = await guardrailHandler(makeEvent({
              toolName: 'Bash',
              toolInput: { command: 'rm -rf "$d"', },
            },),);
            expect(denialReason(output,),).toContain('`rm -rf "$d"`',);
            expect(denialReason(output,),).toContain(REMEDY_SENTENCE,);
          },
        },),
        it({
          name: 'refuses a deletion inside the script file the command runs',
          fn: async function testScriptFileCommand() {
            const output = await guardrailHandler(makeEvent({
              toolName: 'Bash',
              toolInput: { command: `bash ${VIOLATING_SCRIPT}`, },
            },),);
            expect(denialReason(output,),).toContain(VIOLATING_SCRIPT,);
          },
        },),
        it({
          name: 'refuses a deletion inside a here-document body the command writes',
          fn: async function testHereDocumentCommand() {
            const output = await guardrailHandler(makeEvent({
              toolName: 'Bash',
              toolInput: { command: 'bash <<\'EOF\'\nrm -rf "$d"\nEOF', },
            },),);
            expect(denialReason(output,),).toContain('(word `"$d"`)',);
          },
        },),
        it({
          name: 'refuses a deletion reached through a program wrapper',
          fn: async function testWrappedCommand() {
            const output = await guardrailHandler(makeEvent({
              toolName: 'Bash',
              toolInput: { command: 'sudo rm -rf "$litter"', },
            },),);
            expect(denialReason(output,),).toContain('(word `"$litter"`)',);
          },
        },),
        it({
          name: 'allows a deletion that writes its path out',
          fn: async function testLiteralCommand() {
            const output = await guardrailHandler(makeEvent({
              toolName: 'Bash',
              toolInput: { command: 'rm --recursive /home/user/temp/agent/scratch', },
            },),);
            expect(denialReason(output,),).toBe('',);
          },
        },),
        it({
          name: 'allows a single-quoted path, which the shell copies',
          fn: async function testSingleQuotedCommand() {
            const output = await guardrailHandler(makeEvent({
              toolName: 'Bash',
              toolInput: { command: 'rm -rf \'$d\'', },
            },),);
            expect(denialReason(output,),).toBe('',);
          },
        },),
        it({
          name: 'allows a Git subcommand that deletes, which is not rm',
          fn: async function testGitRmCommand() {
            const output = await guardrailHandler(makeEvent({
              toolName: 'Bash',
              toolInput: { command: 'git rm -r "$d"', },
            },),);
            expect(denialReason(output,),).toBe('',);
          },
        },),
      ],
    },),
    //endregion Bash

    //region Write
    describe({
      name: 'a Write call',
      children: [
        it({
          name: 'refuses a new script whose deletion computes its path',
          fn: async function testWriteViolatingScript() {
            const output = await guardrailHandler(makeEvent({
              toolName: 'Write',
              toolInput: {
                file_path: join(fixtureRoot, 'new-script.sh',),
                content: '#!/bin/sh\nrm -rf "$d"\n',
              },
            },),);
            expect(denialReason(output,),).toContain('(word `"$d"`)',);
          },
        },),
        it({
          name: 'refuses a script recognized by its first line alone',
          fn: async function testWriteShebangScript() {
            const output = await guardrailHandler(makeEvent({
              toolName: 'Write',
              toolInput: {
                file_path: join(fixtureRoot, 'cleanup',),
                content: '#!/usr/bin/env bash\nrm -f "$d/$n"\n',
              },
            },),);
            expect(denialReason(output,),).toContain('(word `"$d/$n"`)',);
          },
        },),
        it({
          name: 'allows a script that writes every path out',
          fn: async function testWriteLiteralScript() {
            const output = await guardrailHandler(makeEvent({
              toolName: 'Write',
              toolInput: {
                file_path: join(fixtureRoot, 'new-clean.sh',),
                content: '#!/bin/sh\nrm --recursive /home/user/temp/agent/scratch\n',
              },
            },),);
            expect(denialReason(output,),).toBe('',);
          },
        },),
        it({
          name: 'allows a file that is no shell script',
          fn: async function testWriteOtherFile() {
            const output = await guardrailHandler(makeEvent({
              toolName: 'Write',
              toolInput: {
                file_path: NOTES_FILE,
                content: 'rm -rf "$d"\n',
              },
            },),);
            expect(denialReason(output,),).toBe('',);
          },
        },),
        it({
          name: 'allows a rewrite that keeps the one old violation',
          fn: async function testWriteKeepsOldViolation() {
            /**
             Script whose own deletion is the violation it keeps.
             */
            const content = '#!/bin/sh\nrm -rf "$old"\n';
            const output = await guardrailHandler(makeEvent({
              toolName: 'Write',
              toolInput: {
                file_path: REPAIRABLE_SCRIPT,
                content,
              },
            },),);
            expect(denialReason(output,),).toBe('',);
          },
        },),
        it({
          name: 'allows a rewrite that repairs the old violation',
          fn: async function testWriteRepairs() {
            const output = await guardrailHandler(makeEvent({
              toolName: 'Write',
              toolInput: {
                file_path: REPAIRABLE_SCRIPT,
                content: '#!/bin/sh\nrm --recursive /home/user/temp/agent/scratch\n',
              },
            },),);
            expect(denialReason(output,),).toBe('',);
          },
        },),
      ],
    },),
    //endregion Write

    //region Edit
    describe({
      name: 'an Edit call',
      children: [
        it({
          name: 'refuses an edit that turns a written path into a computed one',
          fn: async function testEditAddsViolation() {
            const output = await guardrailHandler(makeEvent({
              toolName: 'Edit',
              toolInput: {
                file_path: REPAIRABLE_SCRIPT,
                old_string: '"$old"',
                new_string: '"$new"',
              },
            },),);
            expect(denialReason(output,),).toContain('(word `"$new"`)',);
          },
        },),
        it({
          name: 'allows an edit that repairs the old violation',
          fn: async function testEditRepairs() {
            const output = await guardrailHandler(makeEvent({
              toolName: 'Edit',
              toolInput: {
                file_path: REPAIRABLE_SCRIPT,
                old_string: 'rm -rf "$old"',
                new_string: 'rm --recursive /home/user/temp/agent/scratch',
              },
            },),);
            expect(denialReason(output,),).toBe('',);
          },
        },),
        it({
          name: 'allows an edit whose new text is no command',
          fn: async function testEditOtherText() {
            const output = await guardrailHandler(makeEvent({
              toolName: 'Edit',
              toolInput: {
                file_path: NOTES_FILE,
                old_string: 'plain notes',
                new_string: 'rm -rf "$d"',
              },
            },),);
            expect(denialReason(output,),).toBe('',);
          },
        },),
      ],
    },),
    //endregion Edit

    //region MultiEdit
    describe({
      name: 'a MultiEdit call',
      children: [
        it({
          name: 'refuses two edits that split one violation between them',
          fn: async function testMultiEditSplits() {
            /**
             File whose placeholder words hold no deletion yet.
             */
            const path = join(fixtureRoot, 'placeholder.sh',);
            await writeFile(path, '#!/bin/sh\nCOMMAND OPERAND\n',);
            const output = await guardrailHandler(makeEvent({
              toolName: 'MultiEdit',
              toolInput: {
                file_path: path,
                edits: [
                  {
                    old_string: 'COMMAND',
                    new_string: 'rm -rf',
                  },
                  {
                    old_string: 'OPERAND',
                    new_string: '"$split"',
                  },
                ],
              },
            },),);
            expect(denialReason(output,),).toContain('(word `"$split"`)',);
          },
        },),
        it({
          name: 'allows two edits that add no deletion',
          fn: async function testMultiEditInnocent() {
            const output = await guardrailHandler(makeEvent({
              toolName: 'MultiEdit',
              toolInput: {
                file_path: REPAIRABLE_SCRIPT,
                edits: [
                  {
                    old_string: 'rm -rf "$old"',
                    new_string: 'rm --recursive /home/user/temp/agent/scratch',
                  },
                  {
                    old_string: '#!/bin/sh',
                    new_string: '#!/bin/bash',
                  },
                ],
              },
            },),);
            expect(denialReason(output,),).toBe('',);
          },
        },),
      ],
    },),
    //endregion MultiEdit

    //region Input shapes
    describe({
      name: 'input this guardrail cannot read',
      children: [
        it({
          name: 'allows a Write call whose content is not text',
          fn: async function testWriteWithoutContent() {
            const output = await guardrailHandler(makeEvent({
              toolName: 'Write',
              toolInput: { file_path: join(fixtureRoot, 'x.sh',), content: 1, },
            },),);
            expect(denialReason(output,),).toBe('',);
          },
        },),
        it({
          name: 'allows an Edit call whose replaced text is not text',
          fn: async function testEditWithoutStrings() {
            const output = await guardrailHandler(makeEvent({
              toolName: 'Edit',
              toolInput: {
                file_path: join(fixtureRoot, 'x.sh',),
                old_string: 1,
                new_string: 2,
              },
            },),);
            expect(denialReason(output,),).toBe('',);
          },
        },),
        it({
          name: 'allows a MultiEdit call whose edit list holds no replacement',
          fn: async function testMultiEditWithoutEdits() {
            const output = await guardrailHandler(makeEvent({
              toolName: 'MultiEdit',
              toolInput: {
                file_path: join(fixtureRoot, 'x.sh',),
                edits: [{ old_string: 'a', },],
              },
            },),);
            expect(denialReason(output,),).toBe('',);
          },
        },),
        it({
          name: 'allows a Write call whose file path is not text',
          fn: async function testWriteWithoutPath() {
            const output = await guardrailHandler(makeEvent({
              toolName: 'Write',
              toolInput: {
                file_path: 7,
                content: 'rm -rf "$d"\n',
              },
            },),);
            expect(denialReason(output,),).toBe('',);
          },
        },),
      ],
    },),
    //endregion Input shapes
  ],
},);
