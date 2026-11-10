/**
 Cases for the response helpers every mvm tool answers through.

 These helpers are reachable only through the bin entry, whose import starts a stdio
 server loop, so the cases import the built module that re-exports them.
 */

import type { TextContent, } from '@monochromatic-dev/mcp-stdio/ts';
import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  errorResponse,
  formatExecResult,
  invalidArgumentsResponse,
  textResponse,
} from '../dist/final/node/exports-for-built-artifact-tests.mjs';

await describe({
  name: '',
  children: [
    //region textResponse: the one success shape every tool returns

    describe({
      name: textResponse.name,
      children: [
        it({
          name: 'wraps the message in exactly one text content item',
          fn: async () => {
            const response = textResponse('VM created.',);

            expect(response.content,).toHaveLength(1,);
            expect(response.content[0],).toEqual({
              type: 'text',
              text: 'VM created.',
            },);
            // A success response must not carry the error flag: a client that saw
            // `isError` here would report a completed operation as a failure.
            expect(response,).not.toHaveProperty('isError',);
          },
        },),
      ],
    },),

    //endregion textResponse

    //region errorResponse: a caught exception rendered for the client

    describe({
      name: errorResponse.name,
      children: [
        it({
          name: 'marks the response as an error and preserves the caught message',
          fn: async () => {
            const response = errorResponse({
              tag: 'exec_in_vm',
              err: new Error('guest agent is not responding',),
            },);

            expect(response.isError,).toBe(true,);
            expect(response.content,).toHaveLength(1,);
            expect(response.content[0]?.text,).toBe('Error: guest agent is not responding',);
          },
        },),
        it({
          name: 'renders a thrown value that is not an Error as its own text',
          fn: async () => {
            // Handlers catch `unknown`, and a backend that rejects with a string must
            // still produce a readable message rather than `undefined`.
            const response = errorResponse({
              tag: 'push_to_vm',
              err: 'host disk is full',
            },);

            expect(response.content[0]?.text,).toBe('Error: host disk is full',);
            expect(response.isError,).toBe(true,);
          },
        },),
        it({
          name: 'reports the tag to the log rather than to the client',
          fn: async () => {
            const response = errorResponse({
              tag: 'destroy_vm',
              err: new Error('no such VM',),
            },);

            // The tag names the tool in this server's own logging; the caller only
            // reads what went wrong.
            expect(response.content[0]?.text,).toBe('Error: no such VM',);
            expect(response.content[0]?.text,).not.toContain('destroy_vm',);
          },
        },),
      ],
    },),

    //endregion errorResponse

    //region invalidArgumentsResponse: a refusal that caught nothing

    describe({
      name: invalidArgumentsResponse.name,
      children: [
        it({
          name: 'returns the refusal text verbatim under the error flag',
          fn: async () => {
            const refusal = 'Provide either `name` or `all: true`, not both.';
            const response = invalidArgumentsResponse({
              tag: 'destroy_vm',
              text: refusal,
            },);

            expect(response.isError,).toBe(true,);
            expect(response.content[0]?.text,).toBe(`Error: ${refusal}`,);
          },
        },),
      ],
    },),

    //endregion invalidArgumentsResponse

    //region formatExecResult: the three sections a client reads back

    describe({
      name: formatExecResult.name,
      children: [
        it({
          name: 'reports only the exit code when the command produced no output',
          fn: async () => {
            expect(
              formatExecResult({
                exitCode: 0,
                stderr: '',
                stdout: '',
              },),
            ).toBe('exit code: 0',);
          },
        },),
        it({
          name: 'reports stdout alone when nothing was written to stderr',
          fn: async () => {
            const formatted = formatExecResult({
              exitCode: 0,
              stderr: '',
              stdout: 'Linux dev-01',
            },);

            // An empty-stderr section would read as a command that failed silently.
            expect(formatted,).toBe('stdout:\nLinux dev-01\n\nexit code: 0',);
            expect(formatted,).not.toContain('stderr:',);
          },
        },),
        it({
          name: 'reports stderr alone when nothing was written to stdout',
          fn: async () => {
            const formatted = formatExecResult({
              exitCode: 1,
              stderr: 'bash: nope: command not found',
              stdout: '',
            },);

            expect(formatted,).toBe('stderr:\nbash: nope: command not found\n\nexit code: 1',);
            expect(formatted,).not.toContain('stdout:',);
          },
        },),
        it({
          name: 'orders stdout, then stderr, then the exit code with blank lines between',
          fn: async () => {
            expect(
              formatExecResult({
                exitCode: 2,
                stderr: 'warning: no such flag',
                stdout: 'partial output',
              },),
            ).toBe('stdout:\npartial output\n\nstderr:\nwarning: no such flag\n\nexit code: 2',);
          },
        },),
        it({
          name: 'keeps the trailing newline a command wrote',
          fn: async () => {
            // Command output normally ends in a newline, and the formatter passes it
            // through rather than trimming it, so the separator blank line follows it.
            expect(
              formatExecResult({
                exitCode: 0,
                stderr: '',
                stdout: 'Linux dev-01\n',
              },),
            ).toBe('stdout:\nLinux dev-01\n\n\nexit code: 0',);
          },
        },),
      ],
    },),

    //endregion formatExecResult

    //region Protocol types: the builders answer with the owned content type

    describe({
      name: 'response content types',
      children: [
        it({
          name: 'narrows textResponse to a single protocol text item',
          fn: async () => {
            // A local copy of this shape could drift from the protocol's own, and it did:
            // mvm's copy carried no `annotations` field and made every member mutable.
            expectTypeOf<ReturnType<typeof textResponse>['content'][number]>()
              .toEqualTypeOf<TextContent>();
          },
        },),
        it({
          name: 'accepts an annotated text item the local copy could not express',
          fn: async () => {
            const annotated: TextContent = {
              text: 'VM created.',
              type: 'text',
              annotations: { audience: ['user',], },
            };

            expectTypeOf<typeof annotated>()
              .toExtend<ReturnType<typeof textResponse>['content'][number]>();
          },
        },),
      ],
    },),

    //endregion Protocol types
  ],
},);
