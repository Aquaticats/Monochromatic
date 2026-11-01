/**
 Exercise the built extension inside real, credential-free, disposable Pi hosts. @module
 */
import assert from 'node:assert/strict';
import { execFile, } from 'node:child_process';
import { promisify, } from 'node:util';
import { randomUUID, } from 'node:crypto';
import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  dirname,
  join,
} from 'node:path';
import { fileURLToPath, } from 'node:url';
import { fauxAssistantMessage, } from '@earendil-works/pi-ai';
import { AdvisorCompletionError, } from '@monochromatic-dev/agent-harness-shared-advisor/ts';
import {
  parseAdvisorHostJsonl,
  verifyAdvisorHostEvidence,
} from './verify-host-results.ts';

/**
 Preserve Node's ChildProcess-returning execFile signature when selecting its promisify overload.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;
/**
 Native asynchronous adapter retains captured output, failures, and the child handle.
 */
const executeFile = promisifyExecFile(execFile,);

/**
 Fixed scenarios prevent a caller or model from supplying an unbounded task queue.
 */
const SCENARIOS = [
  'serial-credit',
  'explicit-credit',
  'collect',
  'straggler',
  'slash',
  'slash-error',
] as const;
/**
 Parent watchdog detects broken operation termination rather than supplying the product deadline.
 */
const HOST_WATCHDOG_MS = 15_000;
/**
 Bound all captured child output, including a malfunctioning host loop.
 */
const MAX_HOST_OUTPUT_BYTES = 8_388_608;
/**
 Fixed prompt never includes the live agent conversation.
 */
const FIXTURE_PROMPT = 'Run the scripted Advisor fixture.';
/**
 Reject accidental expansion of the verification prompt.
 */
const MAX_FIXTURE_PROMPT_CHARS = 200;

/**
 Run one isolated host and inspect its real persisted tool or slash-command boundary.
 
 @param mode - fixed fixture scenario
 
 @throws when registration, execution, accounting, or cancellation differs from the accepted contract
 */
async function verifyScenario(mode: typeof SCENARIOS[number],): Promise<void> {
  /**
   Each host owns its home, session, configuration, and dispatch trace.
   */
  const root = await mkdtemp(join(
    scratchRoot(),
    'advisor-host-',
  ),);
  /**
   Remove only this freshly created fixture after the child has exited.
   */
  await using cleanup = {
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        root,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
  /**
   Fake home prevents global extension discovery or credential access.
   */
  const home = join(
    root,
    'home',
  );
  /**
   Advisor reads only this generated configuration.
   */
  const agentDir = join(
    home,
    '.pi',
    'agent',
  );
  await mkdir(
    join(
      agentDir,
      'extensions',
    ),
    { recursive: true, },
  );
  await writeFile(
    join(
      agentDir,
      'extensions',
      'pi-advisor.json',
    ),
    JSON.stringify({
      enabled: true,
      timeoutMs: 3_000,
      maxAdvisorOutputTokens: 1_000,
      hedgingEnabled: (mode === 'collect') || (mode === 'straggler')
        || (mode === 'explicit-credit'),
      hedgeDelayMs: 5,
      collectionGraceMs: 30,
    },),
    { flag: 'wx', },
  );
  /**
   A seeded assistant makes slash-only session persistence observable too.
   */
  const sessionFile = join(
    root,
    'session.jsonl',
  );
  /**
   Synthetic session timestamp, unrelated to user session metadata.
   */
  const timestamp = new Date().toISOString();
  await writeFile(
    sessionFile,
    `${[
    {
      type: 'session',
      version: 3,
      id: randomUUID(),
      timestamp,
      cwd: root,
    },
    {
      type: 'message',
      id: '00000001',
      parentId: null,
      timestamp,
      message: {
        ...fauxAssistantMessage('Synthetic session seed',),
        provider: 'fixture-main',
        model: 'main',
        api: 'fixture-main-api',
      },
    },
  ].map(function jsonLine(value): string { return JSON.stringify(value,); },)
      .join('\n',)  }\n`,
    { flag: 'wx', },
  );
  /**
   Installed package's actual CLI entry, verified from its bin metadata.
   */
  const piCli = join(
    dirname(
      fileURLToPath(import.meta.resolve('@earendil-works/pi-coding-agent'),),
    ),
    'bundle',
    'cli.js',
  );
  /**
   Built consumer artifact, not sibling source.
   */
  const extension = fileURLToPath(new URL(
    '../dist/final/node/index.mjs',
    import.meta.url,
  ),);
  /**
   Only this guarded fixture supplies model providers.
   */
  const fixture = fileURLToPath(new URL(
    'verify-host-fixture.ts',
    import.meta.url,
  ),);
  /**
   Slash scenarios exercise the registered command rather than the tool.
   */
  const prompt = mode === 'slash' ? '/advisor' : mode === 'slash-error' ? '/advisor fixture-credit/a' : FIXTURE_PROMPT;
  /**
   Explicit argument vector avoids shell expansion and preserves the scoped-model glob.
   */
  const args = [
    piCli,
    '--offline',
    '--no-extensions',
    '--no-skills',
    '--no-prompt-templates',
    '--no-context-files',
    '--no-builtin-tools',
    '--no-themes',
    '--approve',
    '--mode',
    'json',
    '--print',
    '--session',
    sessionFile,
    '--provider',
    'fixture-main',
    '--model',
    'main',
    '--models',
    'fixture-*/*',
    '--extension',
    extension,
    '--extension',
    fixture,
    '--system-prompt',
    'Fixture execution only.',
    '--',
    prompt,
  ];
  /**
   Child inherits no ambient provider credentials or live-session metadata.
   */
  const execution = executeFile(
    process.execPath,
    args,
    {
      cwd: root,
      encoding: 'utf8',
      timeout: HOST_WATCHDOG_MS,
      maxBuffer: MAX_HOST_OUTPUT_BYTES,
      env: {
        PATH: process.env
          .PATH
          ?? '',
        HOME: home,
        PI_CODING_AGENT_DIR: agentDir,
        PI_OFFLINE: '1',
        PI_TELEMETRY: '0',
        PI_SKIP_VERSION_CHECK: '1',
        TERM: 'dumb',
        NO_COLOR: '1',
        PI_ADVISOR_VERIFY_HOST: '1',
        PI_ADVISOR_FIXTURE_ROOT: root,
        PI_ADVISOR_FIXTURE_MODE: mode,
      },
    },
  );
  // Pi print mode reads piped stdin; close it rather than leaving the fixture waiting for input.
  execution.child
    .stdin
    ?.end();
  /**
   Both output streams are retained, and nonzero exits reject instead of being ignored.
   */
  const result = await execution;
  assert.equal(
    result.stderr,
    '',
    `Pi host ${mode} emitted stderr`,
  );
  /**
   Complete host event and persistence evidence.
   */
  const events = parseAdvisorHostJsonl(result.stdout,);
  /**
   Complete persisted session, including tool-result hooks.
   */
  const entries = parseAdvisorHostJsonl(await readFile(
    sessionFile,
    'utf8',
  ),);
  /**
   Independent provider trace proves actual dispatch and cancellation.
   */
  const trace = parseAdvisorHostJsonl(await readFile(
    join(
      root,
      'dispatch.jsonl',
    ),
    'utf8',
  ),);
  verifyAdvisorHostEvidence({
    mode,
    events,
    entries,
    trace,
  },);
  if ((mode !== 'slash') && (mode !== 'slash-error'))
    assert.ok(
      result.stdout
        .includes('HOST_FIXTURE_COMPLETE',),
      'primary agent did not continue after Advisor',
    );
  console.log(`Advisor real Pi host verified: ${mode}`,);
}

if (process.env
  .PI_ADVISOR_VERIFY_HOST
  !== '1')
  throw new AdvisorCompletionError('Run the explicitly guarded verify:host task, not an ambient agent session',);
if (FIXTURE_PROMPT.length > MAX_FIXTURE_PROMPT_CHARS)
  throw new AdvisorCompletionError('Advisor host verification prompt exceeded its fixed transcript-size guard',);
for (const mode of SCENARIOS) {
  // oxlint-disable-next-line no-await-in-loop -- Keep fixture hosts sequential so only one agent process is live at a time.
  await verifyScenario(mode,);
}
