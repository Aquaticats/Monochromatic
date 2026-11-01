import type { StopInput, } from '@monochromatic-dev/claude-code-plugin-hook-type/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  mkdtemp,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { randomUUID, } from 'node:crypto';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  CONTINUATION_MARKER,
  FEEDBACK_PREFIX,
} from './continuation-depth.ts';
import { createStopRemindersHandler, } from './index.ts';
import { questionLead, } from './reminder-lead.ts';
import {
  LUNA_MODEL,
  LunaKeyMissingError,
  SITUATIONS,
  type Situation,
} from './luna-situation.ts';

/**
 Credential the fake gateway expects; never a real key.
 */
const KEY = 'fixture-key-7f3a91';

/**
 Scratch directory holding transcripts and verdict logs, one file per case.
 */
const SCRATCH = await mkdtemp(join(scratchRoot(), 'stop-reminder-luna-',),);

/**
 Transcript line for a turn the user typed, which closes the counting window.
 */
const HUMAN = JSON.stringify({ type: 'user', origin: { kind: 'human', }, message: { content: 'go', }, },);

/**
 Transcript line for one forced-continuation block.
 */
const BLOCK = JSON.stringify({
  type: 'user',
  message: { content: `${FEEDBACK_PREFIX}:\n${CONTINUATION_MARKER}.`, },
},);

/**
 Transcript line for an assistant turn that issued a tool call.
 */
const TOOL = JSON.stringify({ type: 'assistant', message: { content: [{ type: 'tool_use', name: 'Bash', },], }, },);

/**
 Transcript line for an assistant turn that only produced prose.
 */
const TEXT = JSON.stringify({ type: 'assistant', message: { content: [{ type: 'text', text: 'blocked', },], }, },);

/**
 Transcript line for a tool result returned to the model.
 */
const TOOL_RESULT = JSON.stringify({
  type: 'user',
  message: { content: [{ type: 'tool_result', tool_use_id: 'x', },], },
},);

/**
 Text Claude Code queues when a background task finishes.
 */
const NOTIFICATION = '<task-notification>\n<task-id>b1</task-id>\n<status>completed</status>\n</task-notification>';

/**
 Transcript line for the finished task's notification entering the queue.
 */
const ENQUEUE = JSON.stringify({ type: 'queue-operation', operation: 'enqueue', content: NOTIFICATION, },);

/**
 Transcript line for that notification being delivered.
 */
const REMOVE = JSON.stringify({ type: 'queue-operation', operation: 'remove', content: NOTIFICATION, },);

/**
 Builds the gateway response for one chosen label.

 @param situation - label Luna chooses

 @param probability - probability of that label; the rest is shared evenly

 @returns JSON body text

 @example
 ```ts
 gatewayBody({ situation: 'work_closed', probability: 0.9 });
 ```
 */
function gatewayBody(
  {
    situation,
    probability,
  }: {
    readonly situation: Situation;
    readonly probability: number;
  },
): string {
  /**
   Probability each unchosen label shares.
   */
  const rest = (1 - probability) / (SITUATIONS.length - 1);
  return JSON.stringify({
    model: LUNA_MODEL,
    answers: [
      {
        type: 'choice',
        name: 'situation',
        choice: situation,
        confidence: 0.5,
        probabilities: SITUATIONS.map((value,) => ({
          value,
          probability: (value === situation) ? probability : rest,
        }),),
      },
    ],
    usage: { input_tokens: 1, output_tokens: 0, },
  },);
}

/**
 Stop event fields a case varies; anything omitted keeps the default.
 */
type StopOverrides = {
  readonly stop_hook_active?: boolean;
  readonly permission_mode?: StopInput['permission_mode'];
  readonly session_crons?: NonNullable<StopInput['session_crons']>;
  readonly last_assistant_message?: string;
  readonly background_tasks?: NonNullable<StopInput['background_tasks']>;
};

/**
 Everything one case needs to run the handler.
 */
type Harness = {
  readonly run: (overrides?: StopOverrides,) => ReturnType<ReturnType<typeof createStopRemindersHandler>>;
  readonly requests: string[];
  readonly logPath: string;
};

/**
 Builds a handler wired to a fake gateway and a scratch transcript and log.

 @param lines - transcript lines, oldest first

 @param reply - gateway response, or `undefined` for a transport failure

 @param env - environment overrides on top of the opted-in default

 @param readKey - credential reader override

 @returns the runner, the request bodies the fake saw, and the verdict log path

 @example
 ```ts
 const { run } = harness({ lines: [HUMAN, TOOL], reply: new Response(body) });
 ```
 */
async function harness(
  {
    lines,
    reply,
    env = {},
    readKey = async () => KEY,
  }: {
    readonly lines: readonly string[];
    readonly reply?: Response;
    readonly env?: NodeJS.ProcessEnv;
    readonly readKey?: () => Promise<string>;
  },
): Promise<Harness> {
  /**
   Unique per case, since cases run concurrently and share the scratch directory.
   */
  const id = randomUUID();
  /**
   Transcript the handler reads for depth and progress.
   */
  const transcript = join(SCRATCH, `transcript-${id}.jsonl`,);
  await writeFile(transcript, `${lines.join('\n',)}\n`,);
  /**
   Where this case's verdicts land.
   */
  const logPath = join(SCRATCH, `verdicts-${id}.jsonl`,);
  /**
   Request bodies the fake gateway received.
   */
  const requests: string[] = [];
  /**
   Handler under test.
   */
  const handler = createStopRemindersHandler({
    env: {
      MONOCHROMATIC_STOP_AUTO_CONTINUE: 'on',
      MONOCHROMATIC_STOP_VERDICT_LOG: logPath,
      ...env,
    },
    readKey,
    fetchImpl: async function gateway(_input: Parameters<typeof fetch>[0], init: RequestInit = {},) {
      requests.push(((typeof init.body) === 'string') ? init.body : '',);
      if (reply === undefined) {
        throw new TypeError('offline',);
      }
      return reply.clone();
    },
  },);
  return {
    requests,
    logPath,
    run: async (overrides = {},) => await handler({
      session_id: 'test-session',
      transcript_path: transcript,
      cwd: SCRATCH,
      permission_mode: 'default',
      hook_event_name: 'Stop',
      stop_hook_active: true,
      last_assistant_message: 'Next: run the tests.',
      ...overrides,
    },),
  };
}

/**
 Reads the verdict records a case appended.

 @param logPath - verdict log path

 @returns parsed records, oldest first

 @example
 ```ts
 const [record] = await records(logPath);
 ```
 */
async function records(logPath: string,): Promise<readonly Record<string, unknown>[]> {
  try {
    return (await readFile(logPath, 'utf8',))
      .split('\n',)
      .filter(Boolean,)
      .map((line,) => JSON.parse(line,) as Record<string, unknown>,);
  }
  catch (error) {
    void error;
    return [];
  }
}

/**
 Gateway reply for one label.

 @param situation - label Luna chooses

 @param probability - its probability, 0.9 unless a case needs another

 @returns response the fake gateway returns

 @example
 ```ts
 verdictReply('work_closed');
 ```
 */
function verdictReply(situation: Situation, probability = 0.9,): Response {
  return new Response(gatewayBody({ situation, probability, },),);
}

/**
 Gateway reply for an explicit distribution over the six labels.

 @param scores - probability per label; the largest is the chosen label

 @returns response the fake gateway returns

 @example
 ```ts
 distributionReply({ ...none, waits_on_user: 0.54, leaves_open_work: 0.37 });
 ```
 */
function distributionReply(scores: Readonly<Record<Situation, number>>,): Response {
  /**
   Label with the largest probability, the one the fake gateway reports as chosen.
   */
  const choice = SITUATIONS.reduce((best, label,) => ((scores[label] > scores[best]) ? label : best));
  return Response.json({
    model: LUNA_MODEL,
    answers: [
      {
        type: 'choice',
        name: 'situation',
        choice,
        confidence: 0.5,
        probabilities: SITUATIONS.map((value,) => ({ value, probability: scores[value], }),),
      },
    ],
    usage: { input_tokens: 1, output_tokens: 0, },
  },);
}

/**
 Transcript line for the feedback record Claude Code writes when this hook blocks on a trailing question.

 @param sentence - the question sentence the reminder named

 @returns one JSONL line

 @example
 ```ts
 questionFeedback('Want me to commit?');
 ```
 */
function questionFeedback(sentence: string,): string {
  return JSON.stringify({
    type: 'user',
    message: { content: `${FEEDBACK_PREFIX}:\n${questionLead(sentence,)} Use the AskUserQuestion tool.`, },
  },);
}

/**
 Transcript in which the agent worked since the last block, so state allows continuation.
 */
const OPEN = [HUMAN, TOOL,] as const;

/**
 Transcript of a repeated stop: an earlier stop was blocked, the agent worked, and the final message is the newest record.
 */
const HISTORY = [
  HUMAN,
  JSON.stringify({ type: 'assistant', message: { content: [{ type: 'text', text: 'earlier stop words', },], }, },),
  BLOCK,
  TOOL,
  JSON.stringify({ type: 'assistant', message: { content: [{ type: 'text', text: 'Final words.', },], }, },),
] as const;

await describe({
  name: 'stop-reminder Luna situation check',
  children: [
    describe({
      name: 'decision table',
      children: [
        it({
          name: 'blocks an announced next step and says to do it now',
          fn: async () => {
            const h = await harness({ lines: OPEN, reply: verdictReply('announces_own_next_step',), },);

            const output = await h.run();

            expect(output.decision,).toBe('block',);
            expect(output.reason,).toContain('Resume the next item now',);
            expect(output.systemMessage,).toBeUndefined();
            expect(h.requests,).toHaveLength(1,);
          },
        },),
        it({
          name: 'blocks open work the agent could do itself and says to do it now',
          fn: async () => {
            const h = await harness({ lines: OPEN, reply: verdictReply('leaves_open_work',), },);

            const output = await h.run({ last_assistant_message: 'Still open: file the issue for the flaky test.', },);

            expect(output.decision,).toBe('block',);
            expect(output.reason,).toContain('Do them now',);
            expect(output.reason,).toContain('AskUserQuestion',);
          },
        },),
        it({
          name: 'blocks open work while a shell command runs, since no task will do it',
          fn: async () => {
            const h = await harness({ lines: OPEN, reply: verdictReply('leaves_open_work',), },);

            const output = await h.run({ background_tasks: [{ id: '1', type: 'shell', status: 'running', },], },);

            expect(output.decision,).toBe('block',);
          },
        },),
        it({
          name: 'blocks a wait on the user and routes it through AskUserQuestion',
          fn: async () => {
            const h = await harness({ lines: OPEN, reply: verdictReply('waits_on_user',), },);

            const output = await h.run({ last_assistant_message: 'I will wait for your direction.', },);

            expect(output.decision,).toBe('block',);
            expect(output.reason,).toContain('AskUserQuestion',);
          },
        },),
        it({
          name: 'blocks a background wait when no background task is running',
          fn: async () => {
            const h = await harness({ lines: OPEN, reply: verdictReply('waits_on_background_work',), },);

            const output = await h.run({ background_tasks: [], },);

            expect(output.decision,).toBe('block',);
            expect(output.reason,).toContain('no background task is running',);
          },
        },),
        it({
          name: 'blocks a background wait when the only tasks have finished',
          fn: async () => {
            const h = await harness({ lines: OPEN, reply: verdictReply('waits_on_background_work',), },);

            const output = await h.run({ background_tasks: [{ id: '1', type: 'subagent', status: 'completed', },], },);

            expect(output.decision,).toBe('block',);
          },
        },),
        it({
          name: 'releases a background wait while a background command is running',
          fn: async () => {
            const h = await harness({ lines: OPEN, reply: verdictReply('waits_on_background_work',), },);

            const output = await h.run({ background_tasks: [{ id: '1', type: 'shell', status: 'running', },], },);

            expect(output.decision,).toBeUndefined();
            expect((await records(h.logPath,))[0]?.action,).toBe('release',);
          },
        },),
        ...(['pauses_for_handoff', 'work_closed',] as const).map(function releases(situation,) {
          return it({
            name: `releases ${situation}`,
            fn: async () => {
              const h = await harness({ lines: OPEN, reply: verdictReply(situation,), },);

              const output = await h.run();

              expect(output.decision,).toBeUndefined();
              expect(output.systemMessage,).toBeUndefined();
            },
          },);
        },),
        it({
          name: 'releases a verdict below the cutoff even for a blocking label',
          fn: async () => {
            const h = await harness({ lines: OPEN, reply: verdictReply('announces_own_next_step', 0.4,), },);

            expect((await h.run()).decision,).toBeUndefined();
          },
        },),
        it({
          name: 'blocks at exactly the cutoff',
          fn: async () => {
            const h = await harness({ lines: OPEN, reply: verdictReply('announces_own_next_step', 0.6,), },);

            expect((await h.run()).decision,).toBe('block',);
          },
        },),
        it({
          name: 'leads every block reason with the marker the depth guard counts',
          fn: async () => {
            const reasons = await Promise.all((
              ['announces_own_next_step', 'leaves_open_work', 'waits_on_user', 'waits_on_background_work',] as const
            ).map(async function reasonFor(situation,) {
              const h = await harness({ lines: OPEN, reply: verdictReply(situation,), },);

              return (await h.run({ background_tasks: [], },)).reason;
            },),);

            expect(reasons.every((reason,) => reason?.includes(CONTINUATION_MARKER,) === true),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: 'request',
      children: [
        it({
          name: 'sends the final message and never the credential',
          fn: async () => {
            const h = await harness({ lines: OPEN, reply: verdictReply('work_closed',), },);

            await h.run({ last_assistant_message: 'All done, tests pass.', },);

            expect((JSON.parse(h.requests[0] ?? '',) as { input: string }).input,).toBe('All done, tests pass.',);
            expect(h.requests[0],).not.toContain(KEY,);
          },
        },),
        it({
          name: 'asks even inside a blocked chain, where the quality detectors are suppressed',
          fn: async () => {
            const h = await harness({ lines: OPEN, reply: verdictReply('announces_own_next_step',), },);

            await h.run({ stop_hook_active: true, },);

            expect(h.requests,).toHaveLength(1,);
          },
        },),
      ],
    },),
    describe({
      name: 'skips Luna without spending a request',
      children: [
        it({
          name: 'when a subagent is running, releasing even a blocking verdict and logging the skip',
          fn: async () => {
            const h = await harness({ lines: OPEN, reply: verdictReply('announces_own_next_step',), },);

            const output = await h.run({
              background_tasks: [{ id: '1', type: 'subagent', status: 'running', },],
            },);

            expect(output.decision,).toBeUndefined();
            expect(h.requests,).toHaveLength(0,);
            const [record,] = await records(h.logPath,);
            expect(record?.outcome,).toBe('skipped',);
            expect(record?.action,).toBe('release',);
            expect(record?.detail,).toBe('a subagent is running',);
          },
        },),
        it({
          name: 'when a finished task\'s notification is queued, though the task list is already empty',
          fn: async () => {
            const h = await harness({
              lines: [...OPEN, TOOL_RESULT, ENQUEUE,],
              reply: verdictReply('waits_on_background_work',),
            },);

            const output = await h.run({ background_tasks: [], },);

            expect(output.decision,).toBeUndefined();
            expect(h.requests,).toHaveLength(0,);
            const [record,] = await records(h.logPath,);
            expect(record?.outcome,).toBe('skipped',);
            expect(record?.action,).toBe('release',);
            expect(record?.detail,).toBe('a task notification is queued',);
          },
        },),
        it({
          name: 'not when the queued notification was already delivered or a later tool result outdates it',
          fn: async () => {
            const counts = await Promise.all([
              [...OPEN, TOOL_RESULT, ENQUEUE, REMOVE,],
              [...OPEN, ENQUEUE, TOOL_RESULT,],
            ].map(async function askedFor(lines,) {
              const h = await harness({ lines, reply: verdictReply('announces_own_next_step',), },);

              await h.run({ background_tasks: [], },);
              return h.requests.length;
            },),);

            expect(counts,).toStrictEqual([1, 1,],);
          },
        },),
        it({
          name: 'in plan mode, where the agent cannot perform what it announces, and logs the skip',
          fn: async () => {
            const h = await harness({ lines: OPEN, reply: verdictReply('announces_own_next_step',), },);

            const output = await h.run({ permission_mode: 'plan', },);

            expect(output.decision,).toBeUndefined();
            expect(h.requests,).toHaveLength(0,);
            const [record,] = await records(h.logPath,);
            expect(record?.outcome,).toBe('skipped',);
            expect(record?.detail,).toContain('plan mode',);
            expect(record?.permissionMode,).toBe('plan',);
          },
        },),
        it({
          name: 'when a subagent is running, still giving a trailing question its own reason',
          fn: async () => {
            const h = await harness({ lines: OPEN, reply: verdictReply('work_closed',), },);

            const output = await h.run({
              stop_hook_active: false,
              last_assistant_message: 'Which package should I migrate first?',
              background_tasks: [{ id: '1', type: 'subagent', status: 'running', },],
            },);

            expect(output.reason,).toContain('AskUserQuestion',);
            expect(h.requests,).toHaveLength(0,);
          },
        },),
        it({
          name: 'not when only a command, a monitor, or a finished subagent is in the task list',
          fn: async () => {
            const lists = [
              [{ id: '1', type: 'shell', status: 'running', },],
              [{ id: '1', type: 'monitor', status: 'running', },],
              [{ id: '1', type: 'subagent', status: 'completed', },],
            ];
            const counts = await Promise.all(lists.map(async function askedFor(list,) {
              const h = await harness({ lines: OPEN, reply: verdictReply('work_closed',), },);
              await h.run({ background_tasks: list, },);
              return h.requests.length;
            },),);

            expect(counts,).toEqual([1, 1, 1,],);
          },
        },),
        it({
          name: 'when forced continuation is not opted in',
          fn: async () => {
            const h = await harness({
              lines: OPEN,
              reply: verdictReply('announces_own_next_step',),
              env: { MONOCHROMATIC_STOP_AUTO_CONTINUE: 'off', },
            },);

            expect(await h.run(),).toEqual({},);
            expect(h.requests,).toHaveLength(0,);
          },
        },),
        it({
          name: 'when the Luna switch is off, using the state-based block',
          fn: async () => {
            const h = await harness({
              lines: OPEN,
              reply: verdictReply('work_closed',),
              env: { MONOCHROMATIC_STOP_LUNA: 'off', },
            },);

            const output = await h.run();

            expect(output.decision,).toBe('block',);
            expect(output.systemMessage,).toBeUndefined();
            expect(h.requests,).toHaveLength(0,);
          },
        },),
        it({
          name: 'when the depth limit is reached',
          fn: async () => {
            const chain = [HUMAN, ...Array.from({ length: 3, },).flatMap(() => [TOOL, BLOCK,]), TOOL,];
            const h = await harness({
              lines: chain,
              reply: verdictReply('announces_own_next_step',),
              env: { MONOCHROMATIC_STOP_AUTO_CONTINUE_MAX: '3', },
            },);

            expect((await h.run()).decision,).toBeUndefined();
            expect(h.requests,).toHaveLength(0,);
          },
        },),
        it({
          name: 'when the last forced continuation issued no tool call',
          fn: async () => {
            const h = await harness({ lines: [HUMAN, TOOL, BLOCK, TEXT,], reply: verdictReply('announces_own_next_step',), },);

            expect((await h.run()).decision,).toBeUndefined();
            expect(h.requests,).toHaveLength(0,);
          },
        },),
        it({
          name: 'when a trailing question already owns the stop',
          fn: async () => {
            const h = await harness({ lines: OPEN, reply: verdictReply('work_closed',), },);

            const output = await h.run({ stop_hook_active: false, last_assistant_message: 'Which package should I migrate first?', },);

            expect(output.reason,).toContain('AskUserQuestion',);
            expect(h.requests,).toHaveLength(0,);
          },
        },),
        it({
          name: 'when there is no message to read',
          fn: async () => {
            const h = await harness({ lines: OPEN, reply: verdictReply('work_closed',), },);

            expect((await h.run({ last_assistant_message: '   ', },)).decision,).toBe('block',);
            expect(h.requests,).toHaveLength(0,);
          },
        },),
      ],
    },),
    describe({
      name: 'falls back to the state-based rules and says so',
      children: [
        it({
          name: 'blocks and warns when the gateway is unreachable',
          fn: async () => {
            const h = await harness({ lines: OPEN, },);

            const output = await h.run();

            expect(output.decision,).toBe('block',);
            expect(output.reason,).toContain('Resume the next item now',);
            expect(output.systemMessage,).toContain('Luna check was unavailable',);
          },
        },),
        it({
          name: 'keeps the background-task release in the fallback',
          fn: async () => {
            const h = await harness({ lines: OPEN, },);

            const output = await h.run({ background_tasks: [{ id: '1', type: 'shell', status: 'running', },], },);

            expect(output.decision,).toBeUndefined();
            expect(output.systemMessage,).toContain('Luna check was unavailable',);
          },
        },),
        it({
          name: 'warns on a non-200 status',
          fn: async () => {
            const h = await harness({ lines: OPEN, reply: new Response('busy', { status: 503, },), },);

            expect((await h.run()).systemMessage,).toContain('503',);
          },
        },),
        it({
          name: 'warns on a response that breaks the contract',
          fn: async () => {
            const h = await harness({ lines: OPEN, reply: new Response('{}',), },);

            const output = await h.run();

            expect(output.decision,).toBe('block',);
            expect(output.systemMessage,).toContain('unavailable',);
          },
        },),
        it({
          name: 'warns when no credential can be read',
          fn: async () => {
            const h = await harness({
              lines: OPEN,
              reply: verdictReply('work_closed',),
              readKey: async () => { throw new LunaKeyMissingError('STOP_REMINDER_LLMGATEWAY_API_KEY is unavailable',); },
            },);

            const output = await h.run();

            expect(output.systemMessage,).toContain('STOP_REMINDER_LLMGATEWAY_API_KEY',);
            expect(h.requests,).toHaveLength(0,);
          },
        },),
      ],
    },),
    describe({
      name: 'verdict log',
      children: [
        it({
          name: 'records the label, probabilities, task summary, depth, and message tail',
          fn: async () => {
            const h = await harness({ lines: OPEN, reply: verdictReply('waits_on_user', 0.8,), },);

            await h.run({
              last_assistant_message: 'Needs your approval.',
              session_crons: [{ id: 'cron-1', },],
              background_tasks: [{ id: '1', type: 'shell', status: 'running', command: 'secret-command', },],
            },);

            const [record,] = await records(h.logPath,);
            expect(record?.outcome,).toBe('luna',);
            expect(record?.situation,).toBe('waits_on_user',);
            expect(record?.action,).toBe('block',);
            expect(record?.depth,).toBe(0,);
            expect(record?.message,).toBe('Needs your approval.',);
            expect(record?.sessionId,).toBe('test-session',);
            expect(record?.permissionMode,).toBe('default',);
            expect(record?.sessionCrons,).toBe(1,);
            expect(record?.backgroundTasks,).toEqual([{ type: 'shell', status: 'running', },],);
            expect(JSON.stringify(record,),).not.toContain('secret-command',);
            expect(record?.probabilities,).toMatchObject({ waits_on_user: 0.8, },);
          },
        },),
        it({
          name: 'records a fallback with its cause',
          fn: async () => {
            const h = await harness({ lines: OPEN, },);

            await h.run();

            const [record,] = await records(h.logPath,);
            expect(record?.outcome,).toBe('fallback',);
            expect(String(record?.detail,),).toContain('offline',);
          },
        },),
        it({
          name: 'keeps only the end of a long message',
          fn: async () => {
            const h = await harness({ lines: OPEN, reply: verdictReply('work_closed',), },);

            await h.run({ last_assistant_message: `${'x'.repeat(2_000,)}END`, },);

            const [record,] = await records(h.logPath,);
            expect(String(record?.message,),).toHaveLength(600,);
            expect(String(record?.message,).endsWith('END',),).toBe(true,);
          },
        },),
        it({
          name: 'does not change the decision when the log cannot be written',
          fn: async () => {
            const h = await harness({
              lines: OPEN,
              reply: verdictReply('announces_own_next_step',),
              env: { MONOCHROMATIC_STOP_VERDICT_LOG: join(SCRATCH, 'transcript-1.jsonl', 'impossible', 'log.jsonl',), },
            },);

            expect((await h.run()).decision,).toBe('block',);
          },
        },),
        it({
          name: 'records whether the hook had already blocked this turn',
          fn: async () => {
            const repeated = await harness({ lines: OPEN, reply: verdictReply('work_closed',), },);
            const first = await harness({ lines: OPEN, reply: verdictReply('work_closed',), },);

            await repeated.run({ stop_hook_active: true, },);
            await first.run({ stop_hook_active: false, },);

            expect((await records(repeated.logPath,))[0]?.stopHookActive,).toBe(true,);
            expect((await records(first.logPath,))[0]?.stopHookActive,).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: 'split verdicts',
      children: [
        it({
          name: 'blocks a verdict split between blocking labels that together reach 0.9, with the largest label\'s reason',
          fn: async () => {
            const h = await harness({
              lines: OPEN,
              reply: distributionReply({
                announces_own_next_step: 0.04,
                leaves_open_work: 0.37,
                waits_on_background_work: 0.02,
                waits_on_user: 0.54,
                pauses_for_handoff: 0.02,
                work_closed: 0.01,
              },),
            },);

            const output = await h.run({ last_assistant_message: 'I haven\'t started on that.', },);

            expect(output.decision,).toBe('block',);
            expect(output.reason,).toContain('AskUserQuestion',);
            const [record,] = await records(h.logPath,);
            expect(record?.action,).toBe('block',);
            expect(String(record?.detail,),).toContain('blocking labels together 0.95',);
            expect(String(record?.detail,),).toContain('largest waits_on_user',);
          },
        },),
        it({
          name: 'uses the open-work reason when that label is the largest of the split',
          fn: async () => {
            const h = await harness({
              lines: OPEN,
              reply: distributionReply({
                announces_own_next_step: 0.02,
                leaves_open_work: 0.5,
                waits_on_background_work: 0.03,
                waits_on_user: 0.4,
                pauses_for_handoff: 0.03,
                work_closed: 0.02,
              },),
            },);

            const output = await h.run();

            expect(output.decision,).toBe('block',);
            expect(output.reason,).toContain('Do them now',);
          },
        },),
        it({
          name: 'releases a split verdict whose blocking labels reach only 0.85',
          fn: async () => {
            const h = await harness({
              lines: OPEN,
              reply: distributionReply({
                announces_own_next_step: 0.05,
                leaves_open_work: 0.3,
                waits_on_background_work: 0.05,
                waits_on_user: 0.5,
                pauses_for_handoff: 0.05,
                work_closed: 0.05,
              },),
            },);

            const output = await h.run();

            expect(output.decision,).toBeUndefined();
          },
        },),
        it({
          name: 'counts the combined level in whole percent, so three labels at 0.3 reach 0.9',
          fn: async () => {
            const h = await harness({
              lines: OPEN,
              reply: distributionReply({
                announces_own_next_step: 0.3,
                leaves_open_work: 0.3,
                waits_on_background_work: 0.05,
                waits_on_user: 0.3,
                pauses_for_handoff: 0.03,
                work_closed: 0.02,
              },),
            },);

            expect((await h.run()).decision,).toBe('block',);
          },
        },),
        it({
          name: 'does not count a label that releases toward the combined level',
          fn: async () => {
            const h = await harness({
              lines: OPEN,
              reply: distributionReply({
                announces_own_next_step: 0.02,
                leaves_open_work: 0.2,
                waits_on_background_work: 0.03,
                waits_on_user: 0.2,
                pauses_for_handoff: 0,
                work_closed: 0.55,
              },),
            },);

            expect((await h.run()).decision,).toBeUndefined();
          },
        },),
      ],
    },),
    describe({
      name: 'wording checks inside a chain',
      children: [
        it({
          name: 'blocks a trailing question the first time without asking Luna',
          fn: async () => {
            const h = await harness({ lines: OPEN, reply: verdictReply('work_closed',), },);

            const output = await h.run({ stop_hook_active: true, last_assistant_message: 'Done. Do you want me to?', },);

            expect(output.decision,).toBe('block',);
            expect(output.reason,).toContain('ends with a question to the user',);
            expect(h.requests,).toHaveLength(0,);
          },
        },),
        it({
          name: 'asks Luna once that question was already raised in this turn',
          fn: async () => {
            const h = await harness({
              lines: [HUMAN, questionFeedback('Do you want me to?',), TOOL,],
              reply: verdictReply('work_closed',),
            },);

            const output = await h.run({ stop_hook_active: true, last_assistant_message: 'Done. Do you want me to?', },);

            expect(output.decision,).toBeUndefined();
            expect(h.requests,).toHaveLength(1,);
          },
        },),
        it({
          name: 'adds the hedge reminder to a Luna release',
          fn: async () => {
            const h = await harness({ lines: OPEN, reply: verdictReply('work_closed',), },);

            const output = await h.run({ stop_hook_active: true, last_assistant_message: 'It is probably done.', },);

            expect(output.decision,).toBe('block',);
            expect(output.reason,).toContain('uncertain language',);
            expect(h.requests,).toHaveLength(1,);
          },
        },),
      ],
    },),
    describe({
      name: 'turn history',
      children: [
        it({
          name: 'sends the earlier stops of the turn on a repeated stop while a task runs',
          fn: async () => {
            const h = await harness({ lines: HISTORY, reply: verdictReply('work_closed',), },);

            await h.run({
              stop_hook_active: true,
              last_assistant_message: 'Final words.',
              background_tasks: [{ id: '1', type: 'shell', status: 'running', },],
            },);

            const { input, } = JSON.parse(h.requests[0] ?? '{}',) as { input: string };
            expect(input,).toContain('[stop 1] earlier stop words',);
            expect(input.endsWith('Final message:\nFinal words.',),).toBe(true,);
          },
        },),
        it({
          name: 'sends the final message alone on a repeated stop when no task runs',
          fn: async () => {
            const h = await harness({ lines: HISTORY, reply: verdictReply('work_closed',), },);

            await h.run({ stop_hook_active: true, last_assistant_message: 'Final words.', },);

            const { input, } = JSON.parse(h.requests[0] ?? '{}',) as { input: string };
            expect(input,).toBe('Final words.',);
          },
        },),
        it({
          name: 'sends the final message alone on a repeated stop when only a finished task is listed',
          fn: async () => {
            const h = await harness({ lines: HISTORY, reply: verdictReply('work_closed',), },);

            await h.run({
              stop_hook_active: true,
              last_assistant_message: 'Final words.',
              background_tasks: [{ id: '1', type: 'shell', status: 'completed', },],
            },);

            const { input, } = JSON.parse(h.requests[0] ?? '{}',) as { input: string };
            expect(input,).toBe('Final words.',);
          },
        },),
        it({
          name: 'sends the final message alone on a first stop even while a task runs',
          fn: async () => {
            const h = await harness({ lines: HISTORY, reply: verdictReply('work_closed',), },);

            await h.run({
              stop_hook_active: false,
              last_assistant_message: 'Final words.',
              background_tasks: [{ id: '1', type: 'shell', status: 'running', },],
            },);

            const { input, } = JSON.parse(h.requests[0] ?? '{}',) as { input: string };
            expect(input,).toBe('Final words.',);
          },
        },),
        it({
          name: 'sends the final message alone on a repeated stop that follows no earlier stop',
          fn: async () => {
            const h = await harness({ lines: OPEN, reply: verdictReply('work_closed',), },);

            await h.run({
              stop_hook_active: true,
              last_assistant_message: 'Final words.',
              background_tasks: [{ id: '1', type: 'shell', status: 'running', },],
            },);

            const { input, } = JSON.parse(h.requests[0] ?? '{}',) as { input: string };
            expect(input,).toBe('Final words.',);
          },
        },),
      ],
    },),
  ],
},);
