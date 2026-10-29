/**
 Tests for Claude Code terminal title handler helpers.
 */

import type { HookInput, } from '@monochromatic-dev/claude-code-plugin-hook-type/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  MAX_TERMINAL_TITLE_UTF8_BYTES,
  terminalTitleUtf8ByteLength,
} from '@monochromatic-dev/agent-harness-shared-terminal-title/ts';
import { terminalTitleForEvent, } from './index.ts';

/**
 Shared event fields required by every Claude Code hook payload.
 */
const BASE_HOOK_INPUT = {
  session_id: 'session-1',
  transcript_path: '/tmp/transcript.jsonl',
  cwd: '/repo',
  permission_mode: 'default',
} as const;

/**
 Builds a typed user-prompt hook input for title tests.
 
 @param prompt - because prompt payload is display text under test
 
 @returns complete Claude Code hook input
 */
function userPromptSubmitEvent(prompt: string,): HookInput {
  return {
    ...BASE_HOOK_INPUT,
    hook_event_name: 'UserPromptSubmit',
    prompt,
  };
}

/**
 Builds a typed pre-tool hook input for title tests.
 
 @param filePath - because Read tool path text is display text under test
 
 @returns complete Claude Code hook input
 */
function readPreToolUseEvent(filePath: string,): HookInput {
  return {
    ...BASE_HOOK_INPUT,
    hook_event_name: 'PreToolUse',
    tool_name: 'Read',
    tool_input: { file_path: filePath, },
    tool_use_id: 'tool-1',
  };
}

/**
 Builds a typed pre-tool hook input for bash title tests.
 
 @param command - because command payload is display text under test
 
 @returns complete Claude Code hook input
 */
function bashPreToolUseEvent(command: string,): HookInput {
  return {
    ...BASE_HOOK_INPUT,
    hook_event_name: 'PreToolUse',
    tool_name: 'Bash',
    tool_input: { command, },
    tool_use_id: 'tool-1',
  };
}

await describe({
  name: terminalTitleForEvent.name,
  children: [
    it({
      name: 'renames prompt titles with lifecycle wording',
      fn: async () => {
        expect(
          terminalTitleForEvent(userPromptSubmitEvent('Refactor auth',),),
        ).toBe(
          '✳ Received prompt: Refactor auth',
        );
      },
    },),
    it({
      name: 'uses smart relative path in tool titles',
      fn: async () => {
        expect(
          terminalTitleForEvent(readPreToolUseEvent('/repo/src/index.ts',),),
        ).toBe(
          '✳ Reading src/index.ts',
        );
      },
    },),
    it({
      name: 'uses meaningful command suffix in bash titles',
      fn: async () => {
        expect(
          terminalTitleForEvent(bashPreToolUseEvent('env timeout 10 npm test',),),
        ).toBe(
          '✳ Running npm test',
        );
      },
    },),
    it({
      name: 'sanitizes OSC-breaking controls in prompt titles',
      fn: async () => {
        expect(
          terminalTitleForEvent(userPromptSubmitEvent('Fix\u001Bauth\u0007bug',),),
        ).toBe(
          '✳ Received prompt: Fix␛auth␇bug',
        );
      },
    },),
    it({
      name: 'byte-caps emitted prompt titles',
      fn: async () => {
        /**
         Prompt exceeding Ghostty byte-safe title payload length.
         */
        const title = terminalTitleForEvent(
          userPromptSubmitEvent('😀'.repeat(MAX_TERMINAL_TITLE_UTF8_BYTES,),),
        );
        expect(terminalTitleUtf8ByteLength(title,) <= MAX_TERMINAL_TITLE_UTF8_BYTES,)
          .toBe(true,);
      },
    },),
  ],
},);
