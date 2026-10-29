import { rm, writeFile, } from 'node:fs/promises';
import { setTimeout as wait, } from 'node:timers/promises';

import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';

import { createRequestRegistry, requestExternalAnswer, } from '../dist/final/node/index.mjs';
import { runHelperCommand, } from './helper-process-fixture.ts';

//region End-to-end cancellation and operational failures

await describe({
  name: 'answer helper lifetime',
  children: [
    it({
      name: 'cancels while launcher is pending and safely runs the late command',
      fn: async () => {
        const controller = new AbortController();
        const state: { late?: Promise<void>; finished: boolean; } = { finished: false, };
        let caught: unknown;
        try {
          await requestExternalAnswer({
            cwd: process.cwd(),
            registry: createRequestRegistry(),
            signal: controller.signal,
            resolveTerminalEntryId: async () => 'fixture-terminal',
            launch: async ({ command, },) => {
              state.late = (async function delayedLaunch(): Promise<void> {
                await wait(100,);
                const output = await runHelperCommand({ command, },);
                expect(output.stderr,).toBe('',);
                expect(output.stdout,).toContain('This question is no longer active',);
                state.finished = true;
              })();
              controller.abort();
              await state.late;
            },
          },);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(Error,);
        expect(state.finished,).toBe(false,);
        await state.late;
        expect(state.finished,).toBe(true,);
      },
    },),
    it({
      name: 'cancels an active editor without bare shutdown errors',
      fn: async () => {
        const controller = new AbortController();
        const state: { child?: ReturnType<typeof runHelperCommand>; } = {};
        let caught: unknown;
        try {
          await requestExternalAnswer({
            cwd: process.cwd(),
            registry: createRequestRegistry(),
            signal: controller.signal,
            resolveTerminalEntryId: async () => 'fixture-terminal',
            editorCommand: [process.execPath, '--input-type=module', '--eval',
              "console.log('FIXTURE_EDITOR_READY'); setInterval(() => {}, 1000);",],
            launch: async ({ command, },) => {
              state.child = runHelperCommand({
                command,
                onOutput(output,) {
                  if (output.includes('FIXTURE_EDITOR_READY',))
                    controller.abort();
                },
              },);
              await state.child;
            },
          },);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(Error,);
        const output = await state.child;
        expect(output?.stdout,).toContain('FIXTURE_EDITOR_READY',);
        expect(output?.stdout,).toContain('This question is no longer active',);
        expect(output?.stderr,).toBe('',);
      },
    },),
    it({
      name: 'reports an unavailable editor through the tool instead of hanging',
      fn: async () => {
        let caught: unknown;
        try {
          await requestExternalAnswer({
            cwd: process.cwd(),
            registry: createRequestRegistry(),
            resolveTerminalEntryId: async () => 'fixture-terminal',
            editorCommand: ['fixture-editor-does-not-exist',],
            launch: async ({ command, },) => {
              const output = await runHelperCommand({ command, },);
              expect(output.stderr,).not.toContain('Unhandled',);
            },
          },);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(Error,);
        if (!Error.isError(caught,))
          throw new Error('Expected unavailable-editor diagnostic.',);
        expect(caught.message,).toContain('fixture-editor-does-not-exist',);
        expect(caught.message,).toContain('ENOENT',);
      },
    },),
    ...['missing', 'malformed',].map(function requestFileCase(kind,) {
      return it({
        name: `bootstrap classifies ${kind} request files without hiding operational failures`,
        fn: async () => {
          const controller = new AbortController();
          const state: { output?: string; error?: unknown; } = {};
          try {
            await requestExternalAnswer({
              cwd: process.cwd(),
              registry: createRequestRegistry(),
              signal: controller.signal,
              resolveTerminalEntryId: async () => 'fixture-terminal',
              launch: async ({ command, },) => {
                const requestPath = command.at(-1,);
                if (requestPath === undefined)
                  throw new Error('Missing fixture request path.',);
                await (kind === 'missing'
                  ? rm(requestPath,)
                  : writeFile(requestPath, '{invalid JSON',));
                try {
                  const output = await runHelperCommand({ command, },);
                  expect(output.stderr,).toBe('',);
                  state.output = output.stdout;
                }
                catch (error: unknown) {
                  state.error = error;
                }
                controller.abort();
              },
            },);
          }
          catch (error: unknown) {
            expect(error,).toBeInstanceOf(Error,);
          }
          if (kind === 'missing') {
            expect(state.error,).toBeUndefined();
            expect(state.output,).toContain('This question is no longer active',);
          }
          else {
            expect(state.output,).toBeUndefined();
            expect(state.error,).toBeInstanceOf(Error,);
            if (!Error.isError(state.error,))
              throw new Error('Expected malformed request to remain an error.',);
            expect(state.error.message,).toContain('SyntaxError',);
          }
        },
      },);
    },),
  ],
},);

//endregion End-to-end cancellation and operational failures
