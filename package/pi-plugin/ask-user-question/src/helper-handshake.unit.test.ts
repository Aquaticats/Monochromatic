import { once, } from 'node:events';
import { writeFile, } from 'node:fs/promises';
import { createServer, type Socket, } from 'node:net';

import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';

import { createRequestRegistry, readHelperRequest, requestExternalAnswer, } from '../dist/final/node/index.mjs';
import { runHelperCommand, } from './helper-process-fixture.ts';

//region Requester acknowledgement boundary

await describe({
  name: 'helper acknowledgement before editing',
  children: ['ready', 'fragmented', 'silent', 'invalid',].map(function acknowledgementCase(kind,) {
    return it({
      name: `handles ${kind} requester without opening an unaccepted editor`,
      fn: async () => {
        const sockets = new Set<Socket>();
        const timers = new Set<ReturnType<typeof setTimeout>>();
        await using server = createServer(function fakeRequester(socket,) {
          sockets.add(socket,);
          socket.once('data', function receiveToken(): void {
            if (kind === 'ready') {
              socket.write('ready\n',);
              return;
            }
            if (kind === 'invalid') {
              socket.write('invalid\n',);
              return;
            }
            if (kind === 'fragmented') {
              socket.write('re',);
              timers.add(setTimeout(function finishAcknowledgement(): void {
                socket.write('ady\n',);
              }, 20,),);
              return;
            }
            timers.add(setTimeout(function closeWithoutAcknowledgement(): void {
              socket.end();
            }, 500,),);
          },);
        },);
        using cleanup = { [Symbol.dispose](): void {
          for (const timer of timers)
            clearTimeout(timer,);
          for (const socket of sockets)
            socket.destroy();
        }, };
        server.listen({ host: '127.0.0.1', port: 0, },);
        await once(server, 'listening',);
        const address = server.address();
        if ((address === null) || ((typeof address) === 'string'))
          throw new Error('Missing fixture endpoint.',);
        const controller = new AbortController();
        const state: { output?: Awaited<ReturnType<typeof runHelperCommand>>; } = {};
        try {
          await requestExternalAnswer({
            cwd: process.cwd(),
            registry: createRequestRegistry(),
            signal: controller.signal,
            resolveTerminalEntryId: async () => 'fixture-terminal',
            editorCommand: [process.execPath, '--input-type=module', '--eval',
              "console.log('FIXTURE_EDITOR_WAS_STARTED');",],
            launch: async ({ command, },) => {
              const requestPath = command.at(-1,);
              if (requestPath === undefined)
                throw new Error('Missing fixture request.',);
              const request = await readHelperRequest({ requestPath, },);
              await writeFile(requestPath, JSON.stringify({ ...request, port: address.port, },),);
              state.output = await runHelperCommand({ command, },);
              controller.abort();
            },
          },);
        }
        catch (error: unknown) {
          expect(error,).toBeInstanceOf(Error,);
        }
        const {output} = state;
        if (output === undefined)
          throw new Error('Helper did not complete acknowledgement fixture.',);
        if ((kind === 'ready') || (kind === 'fragmented'))
          expect(output.stdout,).toContain('FIXTURE_EDITOR_WAS_STARTED',);
        else
          expect(output.stdout,).not.toContain('FIXTURE_EDITOR_WAS_STARTED',);
        if (kind === 'invalid')
          expect(output.stderr,).toContain('invalid startup acknowledgement',);
        else
          expect(output.stderr,).toBe('',);
      },
    },);
  },),
},);

//endregion Requester acknowledgement boundary
