import { randomUUID, } from 'node:crypto';
import {
  mkdir,
  readFile,
  rename,
  rm,
  writeFile,
} from 'node:fs/promises';

import { BypassStateError, } from './errors.ts';
import {
  bypassRuntimeDirectory,
  bypassStatePath,
} from './tunnel-bypass-path.ts';
import { parseBypassState, } from './tunnel-bypass-state-parse.ts';
import {
  findFreeBypassPreference,
  findFreeBypassTable,
} from './tunnel-table.ts';
import type { BypassState, } from './tunnel-bypass-types.ts';

/**
 Sentinel representing absent state path.
 */
export const BYPASS_STATE_ABSENT: unique symbol = Symbol('bypass state path is absent',);

/**
 Narrows caught value to Node filesystem error.
 
 @param error - Caught value.
 
 @returns Whether value carries error code.
 
 @example
 ```ts
 isErrnoException({ code: 'ENOENT' }); // true
 ```
 */
function isErrnoException(error: unknown,): error is NodeJS.ErrnoException {
  return ((typeof error) === 'object')
    && (error !== null)
    && ('code' in error);
}

/**
 Reads UTF-8 path with explicit absence.
 
 @param path - Path to read.
 
 @returns Text or absence sentinel.
 
 @example
 ```ts
 await readIfExists('/run/wg-quicker/state.json');
 ```
 */
async function readIfExists(
  path: string,
): Promise<string | typeof BYPASS_STATE_ABSENT> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error) {
    if (isErrnoException(error,) && (error.code === 'ENOENT'))
      return BYPASS_STATE_ABSENT;
    throw error;
  }
}

/**
 Reads state from explicit path for watcher process.
 
 @param path - State path passed by detached watcher.
 
 @returns Validated state.
 
 @throws {@link BypassStateError} when absent or invalid.
 
 @example
 ```ts
 await readBypassStatePath({ path: '/run/wg-quicker/interface-key.json' });
 ```
 */
export async function readBypassStatePath(
  { path, }: { readonly path: string; },
): Promise<BypassState> {
  /**
   State text when path exists.
   */
  const text = await readIfExists(path,);
  if (text === BYPASS_STATE_ABSENT)
    throw new BypassStateError(`Application-bypass state does not exist: ${path}`,);
  return parseBypassState({
    text,
    path,
  },);
}

/**
 Reads state for interface when present.
 
 @param interfaceName - Interface identity.
 
 @returns State or absence sentinel.
 
 @example
 ```ts
 await readBypassState({ interfaceName: 'wg0' });
 ```
 */
export async function readBypassState(
  { interfaceName, }: { readonly interfaceName: string; },
): Promise<BypassState | typeof BYPASS_STATE_ABSENT> {
  /**
   Interface-specific path.
   */
  const path = bypassStatePath({ interfaceName, },);
  /**
   State text when path exists.
   */
  const text = await readIfExists(path,);
  if (text === BYPASS_STATE_ABSENT)
    return BYPASS_STATE_ABSENT;
  /**
   Validated state from interface path.
   */
  const state = parseBypassState({
    text,
    path,
  },);
  if (state.interfaceName !== interfaceName) {
    throw new BypassStateError(
      `Application-bypass state at ${path} belongs to ${state.interfaceName}, not ${interfaceName}.`,
    );
  }
  return state;
}

/**
 Ensures private runtime directory exists.
 
 @example
 ```ts
 await ensureRuntimeDirectory();
 ```
 */
async function ensureRuntimeDirectory(): Promise<void> {
  await mkdir(
    bypassRuntimeDirectory(),
    {
      mode: 0o700,
      recursive: true,
    },
  );
}

/**
 Selects collision-safe table and preference under caller-held allocation lock.
 
 @param interfaceName - Interface becoming owner.
 
 @param mark - Socket mark persisted with state.
 
 @returns Unpersisted state carrying resource identity.
 
 @example
 ```ts
 await claimBypassState({ interfaceName: 'wg0', mark: 100 });
 ```
 */
export async function claimBypassState(
  {
    interfaceName,
    mark,
  }: {
    readonly interfaceName: string;
    readonly mark: number;
  },
): Promise<BypassState> {
  await ensureRuntimeDirectory();
  /**
   Free routing resources observed while global kernel lock is held.
   */
  const [table, preference,] = await Promise.all([
    findFreeBypassTable({ minimum: 0, },),
    findFreeBypassPreference({ maximum: 0, },),
  ],);
  return {
    version: 2,
    interfaceName,
    mark,
    table,
    preference,
    ownerId: randomUUID(),
    routes: [],
  };
}

/**
 Atomically persists claimed state.
 
 @param state - Complete ownership state.
 
 @example
 ```ts
 await persistBypassState({ state });
 ```
 */
export async function persistBypassState(
  { state, }: { readonly state: BypassState; },
): Promise<void> {
  await ensureRuntimeDirectory();
  /**
   Final interface state path.
   */
  const path = bypassStatePath({ interfaceName: state.interfaceName, },);
  /**
   Unique same-directory temporary path.
   */
  const temporaryPath = `${path}.${randomUUID()}.tmp`;
  /**
   Fresh route JSON values detached from caller-owned route containers.
   */
  const serializedRoutes: {
    readonly proto: string;
    readonly tokens: readonly string[];
  }[] = [];
  for (const route of state.routes) {
    serializedRoutes.push({
      proto: route.proto,
      tokens: [...route.tokens,],
    },);
  }
  try {
    await writeFile(
      temporaryPath,
      JSON.stringify({
        version: state.version,
        interfaceName: state.interfaceName,
        mark: state.mark,
        table: state.table,
        preference: state.preference,
        ownerId: state.ownerId,
        routes: serializedRoutes,
      },),
      {
        flag: 'wx',
        mode: 0o600,
      },
    );
    await rename(
      temporaryPath,
      path,
    );
  }
  catch (error) {
    await rm(
      temporaryPath,
      { force: true, },
    );
    throw error;
  }
}

/**
 Removes state after watcher,
 rules,
 and routes are confirmed absent.
 
 @param state - Ownership state being released.
 
 @example
 ```ts
 await releaseBypassState({ state });
 ```
 */
export async function releaseBypassState(
  { state, }: { readonly state: BypassState; },
): Promise<void> {
  /**
   Current state validated before ownership record removal.
   */
  const current = await readBypassState({ interfaceName: state.interfaceName, },);
  if (current === BYPASS_STATE_ABSENT)
    return;
  if (current.ownerId !== state.ownerId) {
    throw new BypassStateError(
      `Refusing to remove application-bypass state owned by ${current.ownerId}.`,
    );
  }
  await rm(bypassStatePath({ interfaceName: state.interfaceName, },),);
}
