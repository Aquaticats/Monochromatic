/**
 Verifies built spawn-pi extension and CLI artifacts.
 
 @module
 */

import { execFile, } from 'node:child_process';
import {
  access,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { fileURLToPath, } from 'node:url';
import { promisify, } from 'node:util';

import type { ExtensionFactory, } from '@earendil-works/pi-coding-agent';

import {
  createExtensionContext,
  createSessionShutdownEvent,
  createSessionStartEvent,
  fakePiApi,
  onlyHandler,
} from './pi-api-fixture.ts';
import {
  envVar,
  tempDir,
} from './test-support.ts';

//region Constants

/**
 Built extension path consumed by Pi.
 */
const BUILT_EXTENSION_PATH = '../dist/final/node/index.mjs';

/**
 Built CLI path exposed by package bin.
 */
const BUILT_CLI_PATH = '../dist/final/node/cli.mjs';

/**
 Source CLI path used by local source-mode symlink setup.
 */
const SOURCE_CLI_PATH = './cli.ts';

/**
 Node executable name used for source TypeScript CLI smoke check.
 */
const NODE_EXECUTABLE = 'node';

/**
 Expected Node shebang for built CLI artifact.
 */
const EXPECTED_NODE_SHEBANG = '#!/usr/bin/env node';

/**
 Expected extension registrations from built spawn-pi entry point.
 */
const EXPECTED_REGISTRATIONS = [
  'event:session_start',
  'event:session_shutdown',
  'event:agent_end',
] as const;

//endregion Constants

//region Types

/**
 Built spawn-pi extension module shape.
 */
type SpawnPiExtensionModule = {
  /**
   Pi extension factory.
   */
  readonly default: ExtensionFactory;
};

//endregion Types

//region Subprocess helpers

/* oxlint-disable typescript/strict-void-return -- node:util.promisify intentionally accepts execFile even though execFile also returns a ChildProcess handle; this wrapper only consumes the promise result. */
/**
 Promise-returning `execFile` used for inert CLI help checks.
 */
const execFileAsync = promisify(execFile,);
/* oxlint-enable typescript/strict-void-return */

/**
 Runs an inert CLI help invocation and throws with captured detail when it exits non-zero.
 
 @param command - executable to invoke.
 
 @param args - CLI arguments, expected to include an inert `--help`.
 
 @param label - description used in failure message.
 
 @throws when invocation exits non-zero or fails to spawn.
 
 @mutates args - `execFileAsync` may inspect or retain argument storage across native process launch.
 
 @example
 ```typescript
 await runHelpOrThrow({ command: '/pkg/cli.mjs', args: ['--help'], label: 'spawn-pi --help' });
 ```
 */
async function runHelpOrThrow(
  {
    command,
    args,
    label,
  }: {
    readonly command: string;
    args: string[];
    readonly label: string;
  },
): Promise<void> {
  try {
    await execFileAsync(
      command,
      args,
      { encoding: 'utf8', },
    );
  }
  catch (error: unknown) {
    throw new Error(
      `${label} failed: ${String(error,)}`,
      { cause: error, },
    );
  }
}

//endregion Subprocess helpers

//region Verification

/**
 Verifies built extension and CLI shape via {@link verifyBuiltExtension}, {@link verifyBuiltCli},
 and {@link verifySourceCli}.
 
 @returns verification result text.
 
 @throws when any built artifact check fails.
 
 @example
 ```typescript
 console.log(await verifyBuiltSpawnPi());
 ```
 */
async function verifyBuiltSpawnPi(): Promise<string> {
  await verifyBuiltExtension();
  await verifyBuiltCli();
  await verifySourceCli();

  return 'spawn-pi verified: extension registrations, Node CLI artifact, source CLI help, and inert built CLI help';
}

/**
 Verifies built extension registers expected events and starts cleanly.
 
 @throws when extension module shape or event registration is wrong.
 
 @example
 ```typescript
 await verifyBuiltExtension();
 ```
 */
async function verifyBuiltExtension(): Promise<void> {
  /**
   Built extension module namespace.
   */
  const mod: unknown = await import(BUILT_EXTENSION_PATH);
  if (!isSpawnPiExtensionModule(mod,))
    throw new Error('built spawn-pi extension does not export a default extension factory',);

  /**
   Fake Pi API harness capturing built extension registrations.
   */
  const harness = fakePiApi();
  await mod.default(harness.api,);

  for (const registration of EXPECTED_REGISTRATIONS) {
    /**
     Whether built extension registered expected lifecycle event.
     */
    const hasRegistration = harness
      .registrations
      .includes(registration,);

    if (!hasRegistration)
      throw new Error(`missing spawn-pi registration: ${registration}`,);
  }

  /**
   Temporary Pi agent directory for inert extension startup.
   */
  await using dir = await tempDir({ prefix: 'spawn-pi-verify-', },);
  /**
   Environment override routing extension state into temp directory.
   */
  using _agentDir = envVar({
    name: 'PI_CODING_AGENT_DIR',
    value: dir.path,
  },);

  /**
   Built extension event handlers captured by fake API.
   */
  const { handlers, } = harness;
  /**
   Built extension session-start handler.
   */
  const sessionStartHandler = onlyHandler(handlers.sessionStart,);
  /**
   Built extension session-shutdown handler.
   */
  const sessionShutdownHandler = onlyHandler(handlers.sessionShutdown,);

  await sessionStartHandler(
    createSessionStartEvent(),
    createExtensionContext({
      sessionId: 'verify-session',
      sessionFile: join(
        dir.path,
        'verify.jsonl',
      ),
      cwd: dir.path,
    },),
  );
  await sessionShutdownHandler(
    createSessionShutdownEvent(),
    createExtensionContext({ sessionId: 'verify-session', },),
  );
}

/**
 Verifies built CLI exists, carries Node shebang, and prints help without spawning a terminal.
 
 @throws when CLI artifact shape or help invocation is wrong.
 
 @example
 ```typescript
 await verifyBuiltCli();
 ```
 */
async function verifyBuiltCli(): Promise<void> {
  /**
   Absolute built CLI path.
   */
  const builtCliPath = fileURLToPath(new URL(
    BUILT_CLI_PATH,
    import.meta.url,
  ),);

  try {
    await access(builtCliPath,);
  }
  catch (error: unknown) {
    throw new Error(
      `missing built spawn-pi CLI: ${builtCliPath}`,
      { cause: error, },
    );
  }

  /**
   Built CLI source text for shebang verification.
   */
  const builtCliText = await readFile(
    builtCliPath,
    'utf8',
  );
  if (!builtCliText.startsWith(EXPECTED_NODE_SHEBANG,))
    throw new Error('built spawn-pi CLI does not use Node shebang',);

  await runHelpOrThrow({
    command: builtCliPath,
    args: ['--help',],
    label: 'built spawn-pi --help',
  },);
}

/**
 Verifies source CLI can be evaluated by Node in local development mode without spawning a terminal.
 
 @throws when source CLI help invocation fails.
 
 @example
 ```typescript
 await verifySourceCli();
 ```
 */
async function verifySourceCli(): Promise<void> {
  /**
   Absolute source CLI path.
   */
  const sourceCliPath = fileURLToPath(new URL(
    SOURCE_CLI_PATH,
    import.meta.url,
  ),);

  await runHelpOrThrow({
    command: NODE_EXECUTABLE,
    args: [
      sourceCliPath,
      '--help',
    ],
    label: 'source spawn-pi --help under Node',
  },);
}

/**
 Detects built spawn-pi extension module shape.
 
 @param value - imported module namespace.
 
 @returns whether module exports extension factory.
 
 @example
 ```typescript
 isSpawnPiExtensionModule(await import('../dist/final/node/index.mjs'));
 ```
 */
function isSpawnPiExtensionModule(value: unknown,): value is SpawnPiExtensionModule {
  if ((value === null) || ((typeof value) !== 'object'))
    return false;
  return ('default' in value) && ((typeof value.default) === 'function');
}

//endregion Verification

console.log(await verifyBuiltSpawnPi(),);
