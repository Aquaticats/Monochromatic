import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  extensionArguments,
  initialSpawnState,
  piCommandArguments,
  SESSION_NOT_FOUND_WARNING,
  terminalInvocation,
  UNLINKED_SPAWN_TITLE,
} from './cli-core.ts';
import type { PidMapping, } from './paths.ts';

/**
 Parent mapping fixture used by CLI core tests.
 */
const IDENTITY: PidMapping = {
  sessionId: 'parent-session',
  sessionFile: '/tmp/parent.jsonl',
  cwd: '/repo',
  extensionPath: '/repo/packages/pi-plugin/spawn/dist/final/node/index.mjs',
};

await describe({
  name: '',
  children: [
    describe({
      name: initialSpawnState.name,
      children: [
        it({
          name: 'creates unclaimed running state linked to parent session',
          fn: async function testInitialState() {
            expect(initialSpawnState({
              spawnId: 'spawn-1',
              identity: IDENTITY,
              cwd: '/repo',
            },),).toEqual({
              spawnId: 'spawn-1',
              sessionId: '',
              sessionFile: '',
              parentSessionId: 'parent-session',
              parentSessionFile: '/tmp/parent.jsonl',
              cwd: '/repo',
              status: 'running',
              lastMessage: '',
            },);
          },
        },),
      ],
    },),
    describe({
      name: extensionArguments.name,
      children: [
        it({
          name: 'passes parent extension path to child Pi',
          fn: async function testExtensionArgs() {
            expect(extensionArguments({ identity: IDENTITY, },),).toEqual([
              '--extension',
              '/repo/packages/pi-plugin/spawn/dist/final/node/index.mjs',
            ],);
          },
        },),
        it({
          name: 'omits extension flag when parent mapping has no extension path',
          fn: async function testNoExtensionArgs() {
            expect(extensionArguments({
              identity: {
                ...IDENTITY,
                extensionPath: '',
              },
            },),).toEqual([],);
          },
        },),
        it({
          name: 'omits extension flag when parent mapping is absent',
          fn: async function testAbsentIdentityExtensionArgs() {
            expect(extensionArguments({},),).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: piCommandArguments.name,
      children: [
        it({
          name: 'combines pi executable, extension path, extra arguments, and prompt',
          fn: async function testPiArguments() {
            expect(piCommandArguments({
              args: {
                prompt: 'do work',
                extraArguments: ' --model openai/gpt-5.1   --thinking high ',
              },
              identity: IDENTITY,
            },),).toEqual([
              'pi',
              '--extension',
              '/repo/packages/pi-plugin/spawn/dist/final/node/index.mjs',
              '--model',
              'openai/gpt-5.1',
              '--thinking',
              'high',
              'do work',
            ],);
          },
        },),
        it({
          name: 'omits extension path while preserving extra arguments without identity',
          fn: async function testPiArgumentsWithoutIdentity() {
            expect(piCommandArguments({
              args: {
                prompt: 'do work',
                extraArguments: '--model openai/gpt-5.1',
              },
            },),).toEqual([
              'pi',
              '--model',
              'openai/gpt-5.1',
              'do work',
            ],);
          },
        },),
      ],
    },),
    describe({
      name: terminalInvocation.name,
      children: [
        it({
          name: 'wraps child pi invocation with terminal-exec delimiter and title',
          fn: async function testTerminalInvocation() {
            expect(terminalInvocation({
              spawnId: '12345678-90ab-cdef',
              args: {
                prompt: 'do work',
              },
              identity: IDENTITY,
            },),).toEqual({
              command: 'terminal-exec',
              args: [
                '--title=spawn-pi 12345678',
                '--',
                'pi',
                '--extension',
                '/repo/packages/pi-plugin/spawn/dist/final/node/index.mjs',
                'do work',
              ],
            },);
          },
        },),
        it({
          name: 'wraps unlinked child pi invocation without extension arguments',
          fn: async function testUnlinkedTerminalInvocation() {
            expect(terminalInvocation({
              args: {
                prompt: 'do work',
                extraArguments: '--thinking high',
              },
            },),).toEqual({
              command: 'terminal-exec',
              args: [
                `--title=${UNLINKED_SPAWN_TITLE}`,
                '--',
                'pi',
                '--thinking',
                'high',
                'do work',
              ],
            },);
          },
        },),
      ],
    },),
    describe({
      name: 'fallback warning',
      children: [
        it({
          name: 'explains that child Pi launches without result forwarding',
          fn: async function testSessionNotFoundWarning() {
            expect(SESSION_NOT_FOUND_WARNING,).toContain('Could not find calling Pi session',);
            expect(SESSION_NOT_FOUND_WARNING,).toContain('without result forwarding',);
            expect(SESSION_NOT_FOUND_WARNING,).toContain('spawn-pi extension',);
          },
        },),
      ],
    },),
  ],
},);
