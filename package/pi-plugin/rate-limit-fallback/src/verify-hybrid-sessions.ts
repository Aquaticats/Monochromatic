/**
 Real Pi sessions against stand-in writer members, with the built package installed from a `packages`
 list: the shape the owner's `~/.pi/agent/settings.json` gives it.
 @module
 */

import assert from 'node:assert/strict';
import { mkdtemp, } from 'node:fs/promises';
import { join, } from 'node:path';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  assistantMessages,
  installPackageList,
  recordTypes,
  runPiSession,
  WRITER_POOL_MODEL,
} from './verify-pi-command-line-session.ts';
import { overflowSession, } from './verify-overflow-session.ts';

//region What the sessions select and who answers

/**
 Model the session selects: the package's public writer virtual model.
 */
const WRITER_SELECTION = 'rate-limit-fallback/writer';

/**
 Provider Pi must report for the selection, whose name is the package's.
 */
const PACKAGE_PROVIDER = 'rate-limit-fallback';

/**
 Member id the writer pool asks first.
 */
const HEAD_MEMBER = 'deepseek-v4.1-flash';

/**
 Member id the writer pool asks second, which answers whatever the head does.
 */
const SECOND_MEMBER = 'deepseek-v4.1-flash-flex';

/**
 Provider both stand-in members are registered under, which the writer pool asks.
 */
const MEMBER_PROVIDER = 'neuralwatt';

/**
 Selection line the stand-in extension writes on stderr, whose model half this harness checks.
 */
const SELECTION_MARKER = '[stand-in-selection]';

/**
 Arguments every arm shares: JSON events, no tools and no project resources.

 @returns Pi arguments up to the prompt

 @example
 ```ts
 baseArguments();
 ```
 */
function baseArguments(): string[] {
  return [
    '--mode',
    'json',
    '--model',
    WRITER_SELECTION,
    '--no-tools',
    '--no-context-files',
    '--no-skills',
    '--no-prompt-templates',
    '--no-mcp',
  ];
}

//endregion What the sessions select and who answers

//region Single-prompt arms

/**
 One prompt against one member behaviour.
 */
type SinglePromptArm = Readonly<{
  name: string;
  behaviour: string;
  answers: string
}>;

/**
 Prompts whose member behaviour answers, refuses, or overflows.
 */
const SINGLE_PROMPT_ARMS: readonly SinglePromptArm[] = [
  {
    name: 'answers a turn through the writer pool',
    behaviour: 'answer',
    answers: HEAD_MEMBER,
  },
  {
    name: 'moves on from a member’s 429 and still answers',
    behaviour: 'refuse-429',
    answers: SECOND_MEMBER,
  },
  {
    name: 'moves on from a member’s 503 and still answers',
    behaviour: 'refuse-503',
    answers: SECOND_MEMBER,
  },
  {
    name: 'moves on from Cloudflare 523 and still answers',
    behaviour: 'refuse-523',
    answers: SECOND_MEMBER,
  },
];

//endregion Single-prompt arms


/**
 Disposable home this run writes under the test scratch root.
 */
const home = await mkdtemp(join(
  scratchRoot(),
  'rate-limit-fallback-hybrid-',
),);
await installPackageList(join(
  home,
  'agent',
),);

for (const arm of SINGLE_PROMPT_ARMS) {
  // One Pi process per arm, so the next arm starts only after this one's settings and logs settle.
  /**
   That arm's session.
   */
  // oxlint-disable-next-line no-await-in-loop -- arms share one disposable home, one process at a time
  const run = await runPiSession({
    home,
    behaviour: arm.behaviour,
    args: [
      ...baseArguments(),
      '--no-session',
      'Reply with the single word ready.',
    ],
  },);
  assert.equal(
    run.exitCode,
    0,
    `${arm.name}: Pi exited ${run.exitCode}: ${run.stderr}`,
  );
  assert.ok(
    run.stderr
      .includes(`${SELECTION_MARKER} ${PACKAGE_PROVIDER}/writer`,),
    `${arm.name}: the selection must stay the writer virtual model`,
  );
  /**
   Final assistant messages the arm produced.
   */
  const answered = assistantMessages(run.records,);
  assert.equal(
    answered.length,
    1,
    `${arm.name}: one assistant record, found ${answered.length}`,
  );
  assert.equal(
    answered[0]
      ?.stopReason,
    'stop',
  );
  assert.equal(
    answered[0]
      ?.provider,
    MEMBER_PROVIDER,
  );
  assert.equal(
    answered[0]
      ?.model,
    arm.answers,
    `${arm.name}: the recorded member answers`,
  );
  assert.deepEqual(
    recordTypes(run.records,)
      .filter(function retriedOrCompacted(type,) {
        return type.startsWith('auto_retry',) || type.startsWith('compaction',);
      },),
    [],
    `${arm.name}: the pool must consume the refusal, so Pi never retries or compacts`,
  );
  process.stdout
    .write(`PASS hybrid session: ${arm.name}\n`,);
}

{
  /**
   The hybrid session's third turn and every turn's messages.
   */
  const {
    run,
    turns,
  } = await overflowSession({
    home,
    sessionId: 'hybrid-overflow',
    model: WRITER_SELECTION,
  },);
  assert.equal(
    run.exitCode,
    0,
    `the overflow turn must finish: Pi exited ${run.exitCode}: ${run.stderr}`,
  );
  assert.ok(
    run.stderr
      .includes(`${SELECTION_MARKER} ${PACKAGE_PROVIDER}/writer`,),
    'the selection must stay the writer virtual model through the recovery',
  );
  assert.equal(
    turns[1]
      ?.at(-1,)
      ?.stopReason,
    'stop',
    'the second turn must answer before the padded third turn',
  );
  /**
   Compaction runs Pi reported in the recovered turn.
   */
  const compactions = run.records
    .filter(function compactionEnd(record,) {
      return record.type === 'compaction_end';
    },);
  assert.equal(
    compactions.length,
    1,
    'the overflow must be recovered by exactly one compaction',
  );
  assert.equal(
    compactions[0]
      ?.reason,
    'overflow',
  );
  assert.equal(
    compactions[0]
      ?.willRetry,
    true,
    'Pi must retry the turn after compacting it',
  );
  assert.deepEqual(
    run.records
      .filter(function autoRetry(record,) {
        return record.type
          .startsWith('auto_retry',);
      },),
    [],
    'the recovery is Pi’s compaction retry, never a provider retry',
  );
  assert.equal(
    turns.at(-1,)
      ?.length,
    2,
    'the recovered turn holds the failed attempt and the answer that replaced it',
  );
  assert.equal(
    turns.at(-1,)
      ?.[0]
      ?.stopReason,
    'error',
    'the first record is the member’s overflow',
  );
  assert.ok(
    (turns.at(-1,)
      ?.[0]
      ?.errorMessage
      ?? '').includes("exceeds model's maximum context length",),
    'the failed record is the member’s own overflow',
  );
  assert.equal(
    turns.at(-1,)
      ?.at(-1,)
      ?.stopReason,
    'stop',
    'the retried turn must end in an answer',
  );
  assert.equal(
    turns.at(-1,)
      ?.at(-1,)
      ?.model,
    HEAD_MEMBER,
  );
  process.stdout
    .write('PASS hybrid session: compacts and retries a member’s context overflow\n',);
}

{
  // The control: the same three prompts with the pool selected directly. Pi resolves a message's
  // limits from the selected model alone unless that selection is virtual
  // (`@earendil-works/pi-coding-agent` dist/core/agent-session.js:313-320), so this arm must not
  // recover; without it the hybrid arm could pass for a reason the route does not own.
  /**
   The control session's third turn and every turn's messages.
   */
  const control = await overflowSession({
    home,
    sessionId: 'direct-pool-overflow',
    model: `${PACKAGE_PROVIDER}/${WRITER_POOL_MODEL}`,
  },);
  assert.ok(
    control.run
      .stderr
      .includes(`${SELECTION_MARKER} ${PACKAGE_PROVIDER}/${WRITER_POOL_MODEL}`,),
    'the control must select the pool model itself',
  );
  assert.deepEqual(
    control.run
      .records
      .filter(function compaction(record,) {
        return record.type
          .startsWith('compaction',);
      },),
    [],
    'the pool selected directly gets no compact-and-retry',
  );
  assert.ok(
    (control.turns
      .at(-1,)
      ?.at(-1,)
      ?.errorMessage
      ?? '').includes("exceeds model's maximum context length",),
    'the control turn ends on the member’s own overflow error',
  );
  process.stdout
    .write('PASS hybrid session: the pool selected directly recovers nothing, so the arm above measures the route\n',);
}

process.stdout
  .write('PASS hybrid sessions: installed from a packages list, selection stays the virtual model, refusals and the overflow recovery happen inside the package route\n',);
