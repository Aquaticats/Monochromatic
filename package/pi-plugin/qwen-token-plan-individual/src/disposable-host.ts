/**
 Disposable pi host machinery for gated verification.

 Split out of `verify-host.ts` so the assertion file stays inside the line
 budget.
 Everything here is verification-only: pi loads `verify-probe.ts` directly from
 source,
 and the package entry never imports this module.
 The host runs against a throwaway home under the module-test scratch root,
 never the developer's own `~/.pi`
 (throwaway-only rule in `AGENTS.md`).

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
 Wall-clock bound for one disposable host run,
 the same bound `package/pi-plugin/model-retirement/src/disposable-host.ts`
 sizes a host boot at.
 */
export const HOST_TIMEOUT_MS = 120_000;

/**
 Provider route whose override the assertions inspect.
 */
export const TARGET_PROVIDER = 'qwen-token-plan-individual';

/**
 Marker the probe prefixes onto every record it emits,
 derived from the probe's own JSON encoding of {@link TARGET_PROVIDER}.
 */
const PROBE_MARKER = `"probe":${JSON.stringify(TARGET_PROVIDER,)}`;

/**
 Timestamp the guarded fixture files carry,
 fixed so the byte-identity guard compares literal fixtures.
 */
const FIXTURE_TIMESTAMP = '2000-01-01T00:00:00.000Z';

/**
 Agent-directory files the never-persist guard pins byte-for-byte,
 each written with content shaped like the real pi stores
 (`~/.pi/agent` measured 2026-11-11:
 `models-store.json` keys providers with `models`/`checkedAt`/`lastModified`/`etag`,
 `models.json` carries `providers`,
 `settings.json` carries `enabledModels`,
 `auth.json` keys one type-tagged credential per provider).
 The credential is a disposable fixture key,
 so `pi --list-models` treats the route as configured
 and no real secret ever reaches a throwaway home.
 */
export const GUARDED_AGENT_FILES: Readonly<Record<string, string>> = {
  'auth.json': `${JSON.stringify({
    [TARGET_PROVIDER]: {
      type: 'api_key',
      key: 'fixture-key',
    },
  }, null, 1)}\n`,
  'models-store.json': `${JSON.stringify({
    [TARGET_PROVIDER]: {
      models: [],
      lastModified: FIXTURE_TIMESTAMP,
      etag: 'fixture',
      checkedAt: FIXTURE_TIMESTAMP,
    },
  }, null, 1)}\n`,
  'models.json': `${JSON.stringify({ providers: {}, }, null, 1)}\n`,
  'settings.json': `${JSON.stringify({
    enabledModels: [`${TARGET_PROVIDER}/qwen3.8-max`],
  }, null, 1)}\n`,
};

//endregion Constants

//region Types

/**
 One catalog model as the registry reports it.
 */
type ProbeModel = {
  readonly id: string;
  readonly contextWindow: number;
  readonly maxTokens: number;
  readonly reasoning: boolean;
  readonly thinkingLevelCount: number;
};

/**
 One catalog snapshot the probe reported.
 */
export type Snapshot = {
  /**
   Lifecycle point the snapshot was taken at.
   */
  readonly phase: string;
  /**
   Chat models the registry reported in total.
   */
  readonly chatCount: number;
  /**
   Models the registry reports under the override route,
   in registry order.
   */
  readonly routeModels: readonly ProbeModel[];
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
 Resolve the installed pi CLI entry from the package's own bin metadata
 layout.

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
 Create one throwaway home whose agent directory holds the guarded fixture
 files.

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
    'pi-qwen-token-plan-individual-host-',
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
  await Promise.all(Object.entries(GUARDED_AGENT_FILES,)
    .map(function writeGuarded([name, content,]) {
      return writeFile(
        join(agentDir, name,),
        content,
        'utf8',
      );
    },),);
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
 Test whether an unknown value is one probe route model.

 @param value - parsed JSON record from host stderr

 @returns whether the record carries every field the assertions read
 */
function isProbeModel(value: unknown,): value is ProbeModel {
  return isRecord(value,)
    && ((typeof value.id) === 'string')
    && ((typeof value.contextWindow) === 'number')
    && ((typeof value.maxTokens) === 'number')
    && ((typeof value.reasoning) === 'boolean')
    && ((typeof value.thinkingLevelCount) === 'number');
}

/**
 Test whether an unknown value is one probe snapshot.

 @param value - parsed JSON record from host stderr

 @returns whether the record carries every field the assertions read

 @example
 ```typescript
 isSnapshot({ phase: 'session_start', chatCount: 1532, routeModels: [] }); // false
 ```
 */
function isSnapshot(value: unknown,): value is Snapshot {
  if (!isRecord(value,))
    return false;
  return (value.probe === TARGET_PROVIDER)
    && ((typeof value.phase) === 'string')
    && ((typeof value.chatCount) === 'number')
    && (Array.isArray(value.routeModels,))
    && (value.routeModels
      .every(isProbeModel));
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

 Closing stdin is what makes RPC mode start a session and then exit on its own,
 since an idle pipe would hold the host open until the watchdog fired.
 Both output streams are consumed to end of file,
 which is when the child has exited.

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
