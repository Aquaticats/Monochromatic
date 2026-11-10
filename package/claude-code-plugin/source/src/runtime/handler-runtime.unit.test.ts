/**
 Hook runtime protocol-channel tests: the writer's bytes are the only stdout
 the host sees, even when the handler logs.

 Codex parses hook stdout strictly, so a trailing log line fails every event
 as `hook returned invalid post-tool-use JSON output`; these cases pin the
 routing the runtime installs.

 @module
 */

import {
  spawnSync,
} from 'node:child_process';
import {
  fileURLToPath,
} from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  makeHookInputParser,
  makeJsonWriter,
  makeOwnedFileParser,
} from './handler-runtime.ts';

/**
 Shape one factory case parses, standing in for a hook event.
 */
type ProbeEvent = {
  readonly hook_event_name: 'Probe';
  readonly payload: number;
};

/**
 Shape one factory case writes, standing in for a handler response.
 */
type ProbeResponse = {
  readonly decision: 'allow';
};

/**
 Shape whose every field is optional, as a pass-through hook response is.
 */
type ProbeOptionalResponse = {
  readonly decision?: 'allow';
};

/**
 Absolute path of the runtime under test, resolved from this file so the child
 imports the same source regardless of the harness working directory.
 */
const RUNTIME_PATH = fileURLToPath(new URL('handler-runtime.ts', import.meta.url,),);

/**
 Directory whose `node_modules` resolves `@monochromatic-dev/logger-core` for
 the child script.
 */
const PACKAGE_ROOT = fileURLToPath(new URL('../..', import.meta.url,),);

/**
 Child script: one handler that logs a diagnostic and returns empty output,
 run through `runHookPlugin` exactly as a plugin entry would.
 */
const CHILD_SCRIPT = [
  `import { runHookPlugin } from ${JSON.stringify(RUNTIME_PATH,)};`,
  `import { tagged, } from '@monochromatic-dev/logger-core/ts';`,
  `const logger = tagged({ tag: 'handler-runtime-probe', },);`,
  `await runHookPlugin({`,
  `  parser: function parse(raw,) { return JSON.parse(raw,); },`,
  `  handler: async function handle() {`,
  `    logger.info('diagnostic line',);`,
  `    return {};`,
  `  },`,
  `  writer: function write(output,) { return JSON.stringify(output,); },`,
  `});`,
].join('\n',);

/**
 Runs {@link CHILD_SCRIPT} in a child process with one hook event on its stdin.

 @param environment - Extra environment for the child on top of the parent's.

 @returns Captured stdout, stderr, and exit status.
 */
function runChild(
  {
    environment,
  }: {
    readonly environment: Readonly<Record<string, string>>;
  },
): {
  readonly stderr: string;
  readonly stdout: string;
} {
  /**
   Child run with a bounded timeout so a hung import cannot stall the suite.
   */
  const result = spawnSync(
    'node',
    [
      '--input-type=module-typescript',
      '-e',
      CHILD_SCRIPT,
    ],
    {
      cwd: PACKAGE_ROOT,
      encoding: 'utf8',
      env: {
        ...process.env,
        ...environment,
      },
      input: '{"probe":true}',
      timeout: 30_000,
    },
  );
  return {
    stderr: result.stderr,
    stdout: result.stdout,
  };
}

await describe({
  name: 'hook runtime stdout protocol channel',
  children: [
    it({
      name: 'stdout carries exactly the writer bytes while the handler logs',
      fn: async () => {
        /**
         Child run with no routing override, so the runtime installs its own.
         */
        const run = runChild({ environment: {}, },);
        expect(run.stdout,)
          .toBe('{}',);
        expect(run.stderr,)
          .toContain('diagnostic line',);
      },
    },),

    it({
      name: 'an explicit MONOCHROMATIC_LOG_CONSOLE=off wins over the runtime default',
      fn: async () => {
        /**
         Child run with console-sink records dropped entirely.
         */
        const run = runChild({
          environment: {
            MONOCHROMATIC_LOG_CONSOLE: 'off',
          },
        },);
        expect(run.stdout,)
          .toBe('{}',);
        expect(run.stderr,)
          .not
          .toContain('diagnostic line',);
      },
    },),

    describe({
      name: makeHookInputParser.name,
      children: [
        it({
          name: 'parses a hook payload into the asserted shape',
          fn: async () => {
            /**
             Parser under test, typed for {@link ProbeEvent}.
             */
            const parse = makeHookInputParser<ProbeEvent>();
            expect(parse('{"hook_event_name":"Probe","payload":7}',),)
              .toEqual({
                hook_event_name: 'Probe',
                payload: 7,
              },);
          },
        },),
        it({
          name: 'throws when the payload is not JSON',
          fn: async () => {
            /**
             Parser under test, typed for {@link ProbeEvent}.
             */
            const parse = makeHookInputParser<ProbeEvent>();
            expect(function parseTruncated() {
              parse('{"hook_event_name":"Probe"',);
            },)
              .toThrow();
          },
        },),
      ],
    },),

    describe({
      name: makeOwnedFileParser.name,
      children: [
        it({
          name: 'parses a record this package wrote into the asserted shape',
          fn: async () => {
            /**
             Parser under test, typed for {@link ProbeEvent}.
             */
            const parse = makeOwnedFileParser<ProbeEvent>();
            expect(parse('{"hook_event_name":"Probe","payload":3}',),)
              .toEqual({
                hook_event_name: 'Probe',
                payload: 3,
              },);
          },
        },),
        it({
          name: 'throws when the record is truncated',
          fn: async () => {
            /**
             Parser under test, typed for {@link ProbeEvent}.
             */
            const parse = makeOwnedFileParser<ProbeEvent>();
            expect(function parseTruncated() {
              parse('{"hook_event_name":"Pro',);
            },)
              .toThrow();
          },
        },),
      ],
    },),

    describe({
      name: makeJsonWriter.name,
      children: [
        it({
          name: 'serializes a response with no trailing newline',
          fn: async () => {
            /**
             Writer under test, typed for {@link ProbeResponse}.
             */
            const write = makeJsonWriter<ProbeResponse>();
            expect(write({ decision: 'allow', },),)
              .toBe('{"decision":"allow"}',);
          },
        },),
        it({
          name: 'serializes a response whose every property is optional as an empty object',
          fn: async () => {
            /**
             Writer under test, over a hook response that carries no fields.
             */
            const write = makeJsonWriter<ProbeOptionalResponse>();
            expect(write({},),)
              .toBe('{}',);
          },
        },),
      ],
    },),
  ],
},);
