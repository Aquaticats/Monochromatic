/**
 Disposable pi host machinery for gated verification.

 Split out of `verify-host.ts` so the assertion file stays inside the line budget.
 Everything here is verification-only: pi loads `verify-probe.ts` directly from source,
 and the package entry never imports this module.

 @module
 */

import { spawn, } from 'node:child_process';
import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  dirname,
  join,
} from 'node:path';
import { text, } from 'node:stream/consumers';
import { fileURLToPath, } from 'node:url';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

//region Constants

/**
 Wall-clock bound for one disposable host run.
 */
const HOST_TIMEOUT_MS = 120_000;

/**
 Context window the disposable host overrides, against the bundled 272000.
 */
export const OVERRIDDEN_CONTEXT_WINDOW = 750_000;

/**
 Provider whose metadata the assertions inspect.
 */
export const TARGET_PROVIDER = 'openai-codex';

/**
 Model the rule retires inside the target provider.
 */
export const RETIRED_MODEL = 'gpt-6-sol';

/**
 Model the rule keeps as that family's winner.
 */
export const KEEPER_MODEL = 'gpt-6.1-sol';

/**
 Provider whose models carry no endpoint, so only wrapping can filter it.
 */
export const WRAPPED_PROVIDER = 'azure-openai-responses';

/**
 Model the rule retires inside the endpoint-less provider.
 */
export const WRAPPED_RETIRED = 'gpt-4.1';

/**
 Model the rule keeps inside the endpoint-less provider.
 */
export const WRAPPED_KEPT = 'gpt-5.5';

/**
 Marker the probe prefixes onto every record it emits.
 */
const PROBE_MARKER = '"probe":"model-retirement"';

//endregion Constants

//region Types

/**
 One catalog snapshot the probe reported.
 */
export type Snapshot = {
  /**
   Lifecycle point the snapshot was taken at.
   */
  readonly phase: string;
  /**
   Chat models the registry reported.
   */
  readonly chatCount: number;
  /**
   Image models the registry reported.
   */
  readonly imageCount: number;
  /**
   Classifier models the registry reported.
   */
  readonly classifierCount: number;
  /**
   Context window of the overridden model, or -1 when it is absent.
   */
  readonly targetContextWindow: number;
  /**
   Thinking levels the overridden model still carries.
   */
  readonly targetThinkingLevels: number;
  /**
   Whether the model the rule retires was still listed.
   */
  readonly hasRetired: boolean;
  /**
   Whether the family winner was still listed.
   */
  readonly hasKeeper: boolean;
  /**
   Whether the model retired inside the endpoint-less provider was still listed.
   */
  readonly hasWrappedRetired: boolean;
  /**
   Whether the family winner inside the endpoint-less provider was still listed.
   */
  readonly hasWrappedKeeper: boolean;
};

/**
 Outcome of parsing one stderr line.
 */
type LineParse = {
  /**
   Parsed record, when the line was JSON.
   */
  readonly record?: unknown;
  /**
   Text of the parse failure, when the line was not JSON.
   */
  readonly malformed?: string;
};

/**
 Output one disposable host run produced.
 */
export type HostRun = {
  /**
   Everything the host wrote to stdout.
   */
  readonly stdout: string;
  /**
   Everything the host wrote to stderr.
   */
  readonly stderr: string;
};

//endregion Types

//region Host setup

/**
 Build the `models.json` a disposable host starts with.

 @returns JSON text carrying one context-window override and no credentials

 @example
 ```typescript
 modelsJsonFixture(); // '{ "providers": { "openai-codex": ... } }'
 */
export function modelsJsonFixture(): string {
  return JSON.stringify(
    {
      providers: {
        [TARGET_PROVIDER]: {
          modelOverrides: {
            'gpt-6-luna': { contextWindow: OVERRIDDEN_CONTEXT_WINDOW, },
            'gpt-6.1-sol': { contextWindow: OVERRIDDEN_CONTEXT_WINDOW, },
          },
        },
      },
    },
    null,
    2,
  );
}

/**
 Resolve the installed pi CLI entry from the package's own bin metadata layout.

 @returns absolute path of pi's bundled CLI script

 @example
 ```typescript
 piCliPath(); // '/.../pi-coding-agent/dist/bundle/cli.js'
 ```
 */
export function piCliPath(): string {
  /**
   Package entry the resolver reports, which sits beside the bundled CLI.
   */
  const packageEntry = fileURLToPath(import.meta.resolve('@earendil-works/pi-coding-agent'),);
  return join(
    dirname(packageEntry,),
    'bundle',
    'cli.js',
  );
}

/**
 Create one throwaway home with an agent directory holding the fixture models file.

 @returns the root to delete afterwards, the home, and the agent directory

 @example
 ```typescript
 const host = await createDisposableHome();
 ```
 */
export async function createDisposableHome(): Promise<{
  readonly root: string;
  readonly home: string;
  readonly agentDir: string;
}> {
  /**
   Throwaway root holding everything this run writes.
   */
  const root = await mkdtemp(join(
    scratchRoot(),
    'pi-model-retirement-host-',
  ),);
  /**
   Home the host process sees.
   */
  const home = join(
    root,
    'home',
  );
  /**
   Agent directory the host reads settings and models from.
   */
  const agentDir = join(
    home,
    '.pi',
    'agent',
  );
  await mkdir(
    agentDir,
    { recursive: true, },
  );
  await writeFile(
    join(
      agentDir,
      'models.json',
    ),
    modelsJsonFixture(),
  );
  return {
    root,
    home,
    agentDir,
  };
}

/**
 Delete a throwaway root, ignoring a directory that is already gone.

 @param root - directory to remove

 @example
 ```typescript
 await removeDisposableHome({ root });
 ```
 */
export async function removeDisposableHome(
  {
    root,
  }: {
    readonly root: string;
  },
): Promise<void> {
  await rm(
    root,
    {
      recursive: true,
      force: true,
    },
  );
}

//endregion Host setup

//region Output parsing

/**
 Test whether an unknown value is a plain object record.

 @param value - parsed JSON value

 @returns whether the value can be read by key

 @example
 ```typescript
 isRecord({}); // true
 ```
 */
function isRecord(value: unknown,): value is Record<string, unknown> {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Parse one stderr line, keeping the failure text instead of discarding it.

 @param line - one line of host stderr

 @returns the parsed record, or the reason the line was not JSON

 @example
 ```typescript
 parseJsonLine({ line: '{}' }); // { record: {} }
 ```
 */
function parseJsonLine({ line, }: { readonly line: string; },): LineParse {
  try {
    return { record: JSON.parse(line,) as unknown, };
  } catch (error) {
    return { malformed: caughtValueText(error,), };
  }
}

/**
 Test whether an unknown value is one probe snapshot.

 @param value - parsed JSON record from host stderr

 @returns whether the record carries every field the assertions read

 @example
 ```typescript
 isSnapshot({ phase: 'session_start', chatCount: 1532 }); // false
 ```
 */
function isSnapshot(value: unknown,): value is Snapshot {
  if (!isRecord(value,))
    return false;
  return (value.probe === 'model-retirement')
    && ((typeof value.phase) === 'string')
    && ((typeof value.chatCount) === 'number')
    && ((typeof value.imageCount) === 'number')
    && ((typeof value.classifierCount) === 'number')
    && ((typeof value.targetContextWindow) === 'number')
    && ((typeof value.targetThinkingLevels) === 'number')
    && ((typeof value.hasRetired) === 'boolean')
    && ((typeof value.hasKeeper) === 'boolean')
    && ((typeof value.hasWrappedRetired) === 'boolean')
    && ((typeof value.hasWrappedKeeper) === 'boolean');
}

/**
 Read the snapshots a host run reported.

 @param stderr - everything the host wrote to stderr

 @returns snapshots in emission order

 @example
 ```typescript
 readSnapshots({ stderr: run.stderr });
 ```
 */
export function readSnapshots({ stderr, }: { readonly stderr: string; },): Snapshot[] {
  /**
   Snapshots collected so far.
   */
  const snapshots: Snapshot[] = [];
  for (const line of stderr.split('\n',)) {
    if (!line.includes(PROBE_MARKER,))
      continue;
    /**
     Parse outcome for this line.
     */
    const parsed = parseJsonLine({ line, },);
    if (isSnapshot(parsed.record,))
      snapshots.push(parsed.record,);
  }
  return snapshots;
}

//endregion Output parsing

//region Host run

/**
 Run one disposable pi host with its stdin closed.

 Closing stdin is what makes RPC mode start a session and then exit on its own, since an
 idle pipe would hold the host open until the watchdog fired. Both output streams are
 consumed to end of file, which is when the child has exited.

 @param args - pi CLI arguments following the node executable

 @param cwd - directory the host runs in

 @param env - environment the host sees

 @returns everything the host wrote to both streams

 @example
 ```typescript
 await runDisposableHost({ args, cwd, env });
 ```
 */
export async function runDisposableHost(
  {
    args,
    cwd,
    env,
  }: {
    readonly args: readonly string[];
    readonly cwd: string;
    readonly env: NodeJS.ProcessEnv;
  },
): Promise<HostRun> {
  /**
   Host child process, killed by the watchdog signal when it overruns.
   */
  const child = spawn(
    process.execPath,
    args,
    {
      cwd,
      env,
      signal: AbortSignal.timeout(HOST_TIMEOUT_MS,),
    },
  );
  /**
   Captured output, started before stdin closes so nothing is missed.
   */
  const outputs = Promise.all([
    text(child.stdout,),
    text(child.stderr,),
  ],);
  child.stdin
    .end();
  /**
   Both streams read to end of file.
   */
  const [stdout, stderr,] = await outputs;
  return {
    stdout,
    stderr,
  };
}

//endregion Host run
