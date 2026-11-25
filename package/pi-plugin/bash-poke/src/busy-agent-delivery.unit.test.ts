/**
 Delivery of a bash-poke result by a real Pi host that is busy, idle, or clearing its queue.

 Each case starts a disposable, credential-free Pi in RPC mode with the built
 extension and `busy-agent-host-fixture.ts`, whose scripted model keeps calling
 a slow tool. Pi's RPC `bash` command emits the same `user_bash` event that
 the TUI emits for `!`, so the extension runs as it does for a typed command.
 No case skips inside a Pi session: the scripted model has no network
 transport, so nothing can spend tokens.

 Background: `doc/troubleshooting/pi-follow-up-messages-wait-for-the-whole-agent-run.md`.

 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  mkdir,
  mkdtemp,
  rm,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';
import { createInterface, } from 'node:readline';
import { fileURLToPath, } from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import { constants, } from '../dist/final/node/index.mjs';

import {
  FIXTURE_FINAL_TEXT,
  FIXTURE_GUARD_VARIABLE,
  FIXTURE_MODEL,
  FIXTURE_PROVIDER,
  FIXTURE_SPARE_TEXT,
  FIXTURE_STEP_MS_VARIABLE,
  FIXTURE_STEPS_VARIABLE,
} from './busy-agent-host-fixture.ts';

//region Constants

/**
 Built extension bundle the host loads, the artifact users install.
 */
const builtExtension = join(import.meta.dirname, '..', 'dist', 'final', 'node', 'index.mjs', );

/**
 Fixture extension registering the scripted model and its slow tool.
 */
const fixtureExtension = join(import.meta.dirname, 'busy-agent-host-fixture.ts', );

/**
 Main module of the Pi package this package depends on.
 */
const piPackageEntry = fileURLToPath(import.meta.resolve('@earendil-works/pi-coding-agent', ), );

/**
 Pi's CLI entry, the package's `bin`, beside its main module.
 */
const piCli = join(
  dirname(piPackageEntry, ),
  'bundle',
  'cli.js',
);

/**
 Text the background command prints, so its poke is recognizable.
 */
const POKED_OUTPUT = 'poked';

/**
 Background command each case sends, which exits within milliseconds.
 */
const POKED_COMMAND = `printf ${POKED_OUTPUT}`;

/**
 Transcript label of a bash-poke message.
 */
const POKE_LABEL = `custom:${constants.POKE_CUSTOM_TYPE}`;

/**
 Tool batches in the busy case: enough that a poke held to the end of the run
 lands far from one delivered after the first batch.
 */
const BUSY_STEPS = 6;

/**
 Milliseconds per tool call in the busy case: long enough that the command,
 sent when the first call starts, exits before that call ends.
 */
const BUSY_STEP_MS = 300;

/**
 Tool batches in the idle case, where only the start of the run matters.
 */
const IDLE_STEPS = 1;

/**
 Tool batches in the queue-clearing case: one aborted, the rest for the second prompt.
 */
const CLEAR_STEPS = 3;

/**
 Milliseconds per tool call in the queue-clearing case: long enough that the
 queue is cleared while the poke still waits for the first call to end.
 */
const CLEAR_STEP_MS = 1_500;

/**
 Milliseconds between sending the command and clearing the queue: long enough
 for the command to exit and its poke to be queued, and shorter than one call.
 */
const CLEAR_AFTER_MS = BUSY_STEP_MS + BUSY_STEP_MS;

/**
 Milliseconds one host may run before the case fails with its stderr.
 */
const HOST_DEADLINE_MS = 30_000;

/**
 Case timeout: twice the host deadline, so a hang reports the host's own
 deadline error, which carries its stderr, instead of a bare harness timeout.
 */
const CASE_TIMEOUT_MS = HOST_DEADLINE_MS + HOST_DEADLINE_MS;

//endregion Constants

//region Types

/**
 One RPC command written to the host's stdin.
 */
type HostCommand = {
  readonly id: string;
  readonly type: string;
  readonly message?: string;
  readonly command?: string;
};

/**
 One event the host printed, read only as far as the cases need.
 */
type HostEvent = {
  readonly type: string;
  readonly message?: unknown;
  readonly command?: unknown;
  readonly success?: unknown;
};

/**
 One transcript message, labelled by role or by custom type.
 */
type HostMessage = {
  /**
   Role, or `custom:<type>` for an extension message.
   */
  readonly label: string;

  /**
   Concatenated text parts.
   */
  readonly text: string;
};

/**
 Handles a case script uses to drive its host.
 */
type HostControl = {
  /**
   Writes one command now.
   */
  readonly send: (command: HostCommand) => void;

  /**
   Writes commands in order once a delay has passed.
   */
  readonly sendAfter: (input: {
    readonly delayMs: number;
    readonly commands: readonly HostCommand[];
  }) => void;

  /**
   Ends the case successfully.
   */
  readonly finish: () => void;
};

/**
 What one case does to its host.
 */
type HostScript = {
  /**
   Runs once, after the host is spawned.
   */
  readonly start: (control: HostControl) => void;

  /**
   Runs for every event the host prints.
   */
  readonly onEvent: (input: {
    readonly event: HostEvent;
    readonly control: HostControl;
  }) => void;
};

/**
 Everything a finished host left for assertions.
 */
type HostRecord = {
  /**
   Transcript messages in the order the host ended them.
   */
  readonly messages: readonly HostMessage[];

  /**
   Everything the host printed to stderr before the case finished.
   */
  readonly stderr: string;
};

//endregion Types

//region Event reading

/**
 Parses one stdout line of the host.

 @param line - one JSON line printed by Pi's RPC mode

 @returns event with its type checked

 @throws when the line is not a JSON object with a string `type`

 @example
 ```ts
 parseHostEvent('{"type":"agent_settled"}',);
 ```
 */
function parseHostEvent(line: string, ): HostEvent {
  /**
   Parsed value, checked before use.
   */
  const value: unknown = JSON.parse(line, );
  if (((typeof value) !== 'object') || (value === null) || (!('type' in value))
    || ((typeof value.type) !== 'string'))
    throw new Error(`Pi RPC printed a line that is not an event: ${line}`, );
  return value as HostEvent;
}

/**
 Reads the text parts of a message's content.

 @param content - message content, a string or an array of parts

 @returns concatenated text, empty when no part carries text

 @example
 ```ts
 contentText([{ type: 'text', text: 'hi', },],);
 ```
 */
function contentText(content: unknown, ): string {
  if ((typeof content) === 'string')
    return content;
  if (!Array.isArray(content, ))
    return '';
  return content
    .map(function partText(part: unknown, ): string {
      return (((typeof part) === 'object') && (part !== null) && ('text' in part)
        && ((typeof part.text) === 'string'))
        ? part.text
        : '';
    }, )
    .join('', );
}

/**
 Labels the message a `message_end` event carries.

 @param message - message object from the event

 @returns label and text

 @throws when the message has no string role

 @example
 ```ts
 readHostMessage({ role: 'assistant', content: [], },);
 ```
 */
function readHostMessage(message: unknown, ): HostMessage {
  if (((typeof message) !== 'object') || (message === null) || (!('role' in message))
    || ((typeof message.role) !== 'string'))
    throw new Error(`message_end event carried no role: ${JSON.stringify(message, )}`, );

  /**
   Custom type when this is an extension message.
   */
  const customType = ('customType' in message) && ((typeof message.customType) === 'string')
    ? message.customType
    : message.role;
  return {
    label: message.role === 'custom' ? `custom:${customType}` : message.role,
    text: contentText(('content' in message) ? message.content : [], ),
  };
}

//endregion Event reading

//region Host

/**
 Runs one disposable Pi host until its script finishes it.

 @param steps - tool batches the scripted model runs before its final reply

 @param stepMs - milliseconds each scripted tool call takes

 @param script - what the case does to the host

 @returns transcript messages and stderr

 @throws when the host exits early, prints a non-event line, or misses the deadline

 @example
 ```ts
 await runHost({ steps: 1, stepMs: 100, script, },);
 ```
 */
async function runHost(
  {
    steps,
    stepMs,
    script,
  }: {
    readonly steps: number;
    readonly stepMs: number;
    readonly script: HostScript;
  },
): Promise<HostRecord> {
  /**
   Private directory holding the host's home, agent directory, and logs.
   */
  const root = await mkdtemp(join(scratchRoot(), 'bash-poke-busy-host-', ), );
  await using cleanup = {
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(root, { recursive: true, force: true, }, );
    },
  };

  /**
   Fake home, so no global extension, setting, or credential is read.
   */
  const home = join(root, 'home', );

  /**
   Empty agent directory of the host.
   */
  const agentDir = join(home, '.pi', 'agent', );
  await mkdir(agentDir, { recursive: true, }, );

  /**
   Host process; its environment carries nothing from the parent but `PATH`.
   */
  const child = spawn(
    process.execPath,
    [
      piCli,
      '--offline',
      '--no-extensions',
      '--no-skills',
      '--no-prompt-templates',
      '--no-context-files',
      '--no-builtin-tools',
      '--no-themes',
      '--approve',
      '--no-session',
      '--mode',
      'rpc',
      '--provider',
      FIXTURE_PROVIDER,
      '--model',
      FIXTURE_MODEL,
      '--extension',
      builtExtension,
      '--extension',
      fixtureExtension,
      '--system-prompt',
      'Fixture execution only.',
    ],
    {
      cwd: root,
      stdio: ['pipe', 'pipe', 'pipe', ],
      env: {
        PATH: process.env.PATH ?? '',
        HOME: home,
        PI_CODING_AGENT_DIR: agentDir,
        PI_OFFLINE: '1',
        PI_TELEMETRY: '0',
        PI_SKIP_VERSION_CHECK: '1',
        TERM: 'dumb',
        NO_COLOR: '1',
        MONOCHROMATIC_LOG_DIRECTORY: join(root, 'logs', ),
        [FIXTURE_GUARD_VARIABLE]: '1',
        [FIXTURE_STEPS_VARIABLE]: String(steps, ),
        [FIXTURE_STEP_MS_VARIABLE]: String(stepMs, ),
      },
    },
  );
  await using stopChild = {
    async [Symbol.asyncDispose](): Promise<void> {
      if ((child.exitCode !== null) || (child.signalCode !== null))
        return;
      child.kill('SIGTERM', );
      await once(child, 'exit', );
    },
  };

  /**
   Stderr chunks, joined into failure messages and the returned record.
   */
  const stderrChunks: string[] = [];
  child.stderr
    .setEncoding('utf8', );
  child.stderr
    .on('data', function collectStderr(chunk: string, ): void {
      stderrChunks.push(chunk, );
    }, );

  /**
   Transcript messages in the order the host ended them.
   */
  const messages: HostMessage[] = [];

  /**
   Settles when the script finishes the case or something fails.
   */
  const finished = Promise.withResolvers<void>();

  /**
   Delayed sends still pending, cleared when the case ends.
   */
  const timers = new Set<NodeJS.Timeout>();

  /**
   Handles the script drives the host with.
   */
  const control: HostControl = {
    send(command: HostCommand, ): void {
      child.stdin
        .write(`${JSON.stringify(command, )}\n`, );
    },
    sendAfter({ delayMs, commands, }, ): void {
      /**
       Timer writing the commands once the delay has passed.
       */
      const timer = setTimeout(function sendDelayed(): void {
        timers.delete(timer, );
        for (const command of commands)
          control.send(command, );
      }, delayMs, );
      timers.add(timer, );
    },
    finish(): void {
      finished.resolve();
    },
  };

  /**
   Line reader over the host's stdout.
   */
  const lines = createInterface({ input: child.stdout, }, );
  lines.on('line', function onLine(line: string, ): void {
    try {
      /**
       Event this line carries.
       */
      const event = parseHostEvent(line, );
      if (event.type === 'message_end')
        messages.push(readHostMessage(event.message, ), );
      script.onEvent({ event, control, }, );
    }
    catch (error: unknown) {
      finished.reject(error, );
    }
  }, );
  child.once('exit', function onEarlyExit(code: unknown, ): void {
    finished.reject(new Error(`Pi host exited before the case finished (code ${String(code, )}); stderr: ${stderrChunks.join('', )}`, ), );
  }, );

  /**
   Deadline after which the case fails with the host's stderr.
   */
  const deadline = setTimeout(function failAtDeadline(): void {
    finished.reject(new Error(`Pi host did not finish within ${String(HOST_DEADLINE_MS, )} ms; stderr: ${stderrChunks.join('', )}`, ), );
  }, HOST_DEADLINE_MS, );

  using stopWaiting = {
    [Symbol.dispose](): void {
      clearTimeout(deadline, );
      for (const timer of timers)
        clearTimeout(timer, );
      lines.close();
    },
  };

  script.start(control, );
  await finished.promise;
  return {
    messages,
    stderr: stderrChunks.join('', ),
  };
}

//endregion Host

//region Cases

await describe({
  name: '',
  children: [
    describe({
      name: 'Pi host delivery',
      // One host at a time: each spawns a Pi process, and the cases do not share state.
      concurrency: 1,
      children: [
        it({
          name: 'lands a poke after the current tool batch while the agent works',
          timeout: CASE_TIMEOUT_MS,
          fn: async () => {
            /**
             Whether the command was sent, so it is sent once.
             */
            const state = { bashSent: false, };

            /**
             Host transcript after the run settled.
             */
            const record = await runHost({
              steps: BUSY_STEPS,
              stepMs: BUSY_STEP_MS,
              script: {
                start(control: HostControl, ): void {
                  control.send({ id: 'prompt', type: 'prompt', message: 'Run the busy fixture.', }, );
                },
                onEvent({ event, control, }, ): void {
                  if ((event.type === 'tool_execution_start') && (!state.bashSent)) {
                    state.bashSent = true;
                    control.send({ id: 'bash', type: 'bash', command: POKED_COMMAND, }, );
                  }
                  if (event.type === 'agent_settled')
                    control.finish();
                },
              },
            }, );

            /**
             Labels in transcript order.
             */
            const labels = record.messages.map(function label(message: HostMessage, ): string {
              return message.label;
            }, );

            /**
             Position of the final reply, which ends the run.
             */
            const finalIndex = record.messages.findIndex(function isFinal(message: HostMessage, ): boolean {
              return (message.label === 'assistant') && (message.text === FIXTURE_FINAL_TEXT);
            }, );
            expect(labels, ).toContain('toolResult', );
            expect(labels, ).toContain(POKE_LABEL, );
            expect(labels.indexOf(POKE_LABEL, ), ).toBeGreaterThan(labels.indexOf('toolResult', ), );
            expect(labels.indexOf(POKE_LABEL, ), ).toBeLessThan(finalIndex, );
            expect(record.messages[labels.indexOf(POKE_LABEL, )]?.text, ).toContain(POKED_OUTPUT, );
            // A follow-up would have added a turn after the final reply.
            expect(record.messages.at(-1, )?.text, ).toBe(FIXTURE_FINAL_TEXT, );
            expect(labels.filter(function isSpare(_label: string, index: number, ): boolean {
              return record.messages[index]?.text === FIXTURE_SPARE_TEXT;
            }, ), ).toEqual([], );
            expect(record.stderr, ).toBe('', );
          },
        }, ),
        it({
          name: 'starts a run with the poke when the agent is idle',
          timeout: CASE_TIMEOUT_MS,
          fn: async () => {
            /**
             Host transcript after the poke's run settled.
             */
            const record = await runHost({
              steps: IDLE_STEPS,
              stepMs: BUSY_STEP_MS,
              script: {
                start(control: HostControl, ): void {
                  control.send({ id: 'bash', type: 'bash', command: POKED_COMMAND, }, );
                },
                onEvent({ event, control, }, ): void {
                  if (event.type === 'agent_settled')
                    control.finish();
                },
              },
            }, );

            /**
             Labels after the system prompt.
             */
            const conversation = record.messages
              .map(function label(message: HostMessage, ): string {
                return message.label;
              }, )
              .filter(function isConversation(label: string, ): boolean {
                return label !== 'system';
              }, );
            expect(conversation[0], ).toBe(POKE_LABEL, );
            expect(conversation[1], ).toBe('assistant', );
            expect(record.messages.at(-1, )?.text, ).toBe(FIXTURE_FINAL_TEXT, );
            expect(record.stderr, ).toBe('', );
          },
        }, ),
        it({
          name: 'loses a queued poke when Pi clears its queue, as Escape does',
          timeout: CASE_TIMEOUT_MS,
          fn: async () => {
            /**
             Progress through the two runs.
             */
            const state = {
              bashSent: false,
              settled: 0,
              clearAnswered: false,
            };

            /**
             Host transcript after both runs settled.
             */
            const record = await runHost({
              steps: CLEAR_STEPS,
              stepMs: CLEAR_STEP_MS,
              script: {
                start(control: HostControl, ): void {
                  control.send({ id: 'prompt', type: 'prompt', message: 'Run the busy fixture.', }, );
                },
                onEvent({ event, control, }, ): void {
                  if ((event.type === 'tool_execution_start') && (!state.bashSent)) {
                    state.bashSent = true;
                    control.send({ id: 'bash', type: 'bash', command: POKED_COMMAND, }, );
                    // Interactive Escape while streaming clears the queue and then aborts.
                    control.sendAfter({
                      delayMs: CLEAR_AFTER_MS,
                      commands: [
                        { id: 'clear', type: 'clear_queue', },
                        { id: 'abort', type: 'abort', },
                      ],
                    }, );
                  }
                  if ((event.type === 'response') && (event.command === 'clear_queue'))
                    state.clearAnswered = event.success === true;
                  if (event.type !== 'agent_settled')
                    return;
                  state.settled += 1;
                  if (state.settled === 1)
                    control.send({ id: 'prompt-after-clear', type: 'prompt', message: 'Prompt after the clear.', }, );
                  else
                    control.finish();
                },
              },
            }, );
            expect(state.clearAnswered, ).toBe(true, );
            expect(record.messages.map(function label(message: HostMessage, ): string {
              return message.label;
            }, ), ).not.toContain(POKE_LABEL, );
            expect(record.stderr, ).toBe('', );
          },
        }, ),
      ],
    }, ),
  ],
}, );

//endregion Cases
