import { randomUUID, } from 'node:crypto';
import { constants, } from 'node:fs';
import {
  mkdir,
  open,
  rename,
  unlink,
  writeFile,
} from 'node:fs/promises';
import {
  isAbsolute,
  join,
} from 'node:path';

import { OpenSnitchConfigError, } from './errors.ts';
import {
  bypassRuntimeDirectory,
  bypassStateKey,
} from './tunnel-bypass-path.ts';

/**
 Current persisted OpenSnitch lifecycle-state schema.
 */
const STATE_VERSION = 1;

/**
 Lowest valid UDP port.
 */
const MIN_UDP_PORT = 1;

/**
 Highest valid UDP port.
 */
const MAX_UDP_PORT = 65_535;

/**
 Sentinel indicating no persisted interface lifecycle state.
 */
export const OPENSNITCH_STATE_ABSENT: unique symbol = Symbol('OpenSnitch lifecycle state absent',);

/**
 OpenSnitch resources owned by one interface lifecycle.
 */
export type OpenSnitchState = {
  /**
   Namespace-specific ownership identity.
   */
  readonly networkNamespaceKey: string;

  /**
   Exact system-firewall file containing managed rules.
   */
  readonly path: string;

  /**
   Exact endpoint ports potentially owned by interface.
   */
  readonly ports: readonly number[];
};

/**
 Reports non-null JSON object.
 
 @param value - Unknown parsed value.
 
 @returns Whether value has object shape.
 
 @example
 ```ts
 isRecord({ Version: 1 });
 ```
 */
function isRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  if ((typeof value) !== 'object')
    return false;
  if (value === null)
    return false;
  return !Array.isArray(value,);
}

/**
 Reports whether unknown failure carries Node filesystem code.
 
 @param error - Unknown caught value.
 
 @returns Whether value is Node filesystem error.
 
 @example
 ```ts
 isErrnoException({ code: 'ENOENT' });
 ```
 */
function isErrnoException(error: unknown,): error is NodeJS.ErrnoException {
  return Error.isError(error,);
}

/**
 Resolves interface-specific lifecycle-state path.
 
 @param interfaceName - WireGuard interface identity.
 
 @param networkNamespaceKey - Namespace-specific ownership identity.
 
 @returns State path under private runtime directory.
 
 @example
 ```ts
 openSnitchStatePath({ interfaceName: 'wg0', networkNamespaceKey: 'abc123' });
 ```
 */
function openSnitchStatePath(
  {
    interfaceName,
    networkNamespaceKey,
  }: {
    readonly interfaceName: string;
    readonly networkNamespaceKey: string;
  },
): string {
  return join(
    bypassRuntimeDirectory(),
    `opensnitch-interface-${bypassStateKey({
      interfaceName: `${interfaceName}\0${networkNamespaceKey}`,
    },)}.json`,
  );
}

/**
 Parses and validates persisted lifecycle state.
 
 @param text - Raw state JSON.
 
 @param path - State path for diagnostics.
 
 @param interfaceName - Expected WireGuard interface identity.
 
 @param networkNamespaceKey - Expected namespace-specific ownership identity.
 
 @returns Validated state.
 
 @throws {@link OpenSnitchConfigError} when state cannot safely drive cleanup.
 
 @example
 ```ts
 parseOpenSnitchState({
   text: '{"Version":1,"InterfaceName":"wg0","NetworkNamespaceKey":"abc123","Path":"/tmp/fw.json","Ports":[]}',
   path,
   interfaceName: 'wg0',
   networkNamespaceKey: 'abc123',
 });
 ```
 */
function parseOpenSnitchState(
  {
    text,
    path,
    interfaceName,
    networkNamespaceKey,
  }: {
    readonly text: string;
    readonly path: string;
    readonly interfaceName: string;
    readonly networkNamespaceKey: string;
  },
): OpenSnitchState {
  /**
   Parsed unknown state value.
   */
  const parsed = (function parseUnknown(): unknown {
    try {
      return JSON.parse(text,);
    }
    catch (error) {
      throw new OpenSnitchConfigError(
        `wg-quicker OpenSnitch lifecycle state is invalid JSON: ${path}`,
        { cause: error, },
      );
    }
  })();
  if (!isRecord(parsed,))
    throw new OpenSnitchConfigError(`wg-quicker OpenSnitch lifecycle state has invalid root: ${path}`,);
  if (parsed.Version !== STATE_VERSION)
    throw new OpenSnitchConfigError(`wg-quicker OpenSnitch lifecycle state has unsupported version: ${path}`,);
  if (parsed.InterfaceName !== interfaceName)
    throw new OpenSnitchConfigError(`wg-quicker OpenSnitch lifecycle state has wrong interface: ${path}`,);
  if (parsed.NetworkNamespaceKey !== networkNamespaceKey)
    throw new OpenSnitchConfigError(`wg-quicker OpenSnitch lifecycle state has wrong network namespace: ${path}`,);
  if (((typeof parsed.Path) !== 'string') || (!isAbsolute(parsed.Path,)))
    throw new OpenSnitchConfigError(`wg-quicker OpenSnitch lifecycle state has invalid config path: ${path}`,);
  if (!Array.isArray(parsed.Ports,))
    throw new OpenSnitchConfigError(`wg-quicker OpenSnitch lifecycle state has invalid ports: ${path}`,);
  /**
   Valid port subset proving every persisted value before use.
   */
  const ports = parsed
    .Ports
    .filter(function validPort(port,): port is number {
      if ((typeof port) !== 'number')
        return false;
      if (!Number.isSafeInteger(port,))
        return false;
      return (port >= MIN_UDP_PORT) && (port <= MAX_UDP_PORT);
    },);
  /**
   Original unvalidated port count.
   */
  const originalPortCount = parsed
    .Ports
    .length;
  if (ports.length !== originalPortCount)
    throw new OpenSnitchConfigError(`wg-quicker OpenSnitch lifecycle state has invalid port value: ${path}`,);
  return {
    networkNamespaceKey,
    path: parsed.Path,
    ports: [...new Set(ports,),]
      .toSorted(function ascending(
        a,
        b,
      ): number {
        return a - b;
      },),
  };
}

/**
 Reads one interface's persisted OpenSnitch lifecycle state without following links.
 
 @param interfaceName - WireGuard interface identity.
 
 @param networkNamespaceKey - Namespace-specific ownership identity.
 
 @returns Validated state or absence sentinel.
 
 @throws {@link OpenSnitchConfigError} when state path is unsafe or unreadable.
 
 @example
 ```ts
 await readOpenSnitchState({ interfaceName: 'wg0', networkNamespaceKey: 'abc123' });
 ```
 */
export async function readOpenSnitchState(
  {
    interfaceName,
    networkNamespaceKey,
  }: {
    readonly interfaceName: string;
    readonly networkNamespaceKey: string;
  },
): Promise<OpenSnitchState | typeof OPENSNITCH_STATE_ABSENT> {
  /**
   Interface-and-namespace-specific state path.
   */
  const path = openSnitchStatePath({
    interfaceName,
    networkNamespaceKey,
  },);
  try {
    /**
     Link-refusing descriptor for race-resistant state read.
     */
    await using handle = await open(
      path,
      constants.O_RDONLY | constants.O_NOFOLLOW,
    );
    /**
     Open descriptor metadata resistant to path replacement.
     */
    const metadata = await handle.stat();
    if ((!metadata.isFile()) || (metadata.nlink !== 1))
      throw new OpenSnitchConfigError(`Unsafe wg-quicker OpenSnitch lifecycle state path: ${path}`,);
    return parseOpenSnitchState({
      text: await handle.readFile('utf8',),
      path,
      interfaceName,
      networkNamespaceKey,
    },);
  }
  catch (error) {
    if (isErrnoException(error,) && (error.code === 'ENOENT'))
      return OPENSNITCH_STATE_ABSENT;
    if (error instanceof OpenSnitchConfigError)
      throw error;
    throw new OpenSnitchConfigError(
      `Cannot read wg-quicker OpenSnitch lifecycle state: ${path}`,
      { cause: error, },
    );
  }
}

/**
 Persists cleanup ownership before modifying external OpenSnitch state.
 
 @param interfaceName - WireGuard interface identity.
 
 @param state - Exact config path and potential owned ports.
 
 @throws {@link OpenSnitchConfigError} when private state cannot be persisted.
 
 @example
 ```ts
 await writeOpenSnitchState({
   interfaceName: 'wg0',
   state: { networkNamespaceKey: 'abc123', path, ports: [51820] },
 });
 ```
 */
export async function writeOpenSnitchState(
  {
    interfaceName,
    state,
  }: {
    readonly interfaceName: string;
    readonly state: OpenSnitchState;
  },
): Promise<void> {
  /**
   Private runtime root.
   */
  const directory = bypassRuntimeDirectory();
  /**
   Interface-and-namespace-specific final state path.
   */
  const path = openSnitchStatePath({
    interfaceName,
    networkNamespaceKey: state.networkNamespaceKey,
  },);
  /**
   Unique same-directory path for atomic replacement.
   */
  const temporaryPath = `${path}.${randomUUID()}.tmp`;
  /**
   Serializable state document.
   */
  const stateDocument = {
    Version: STATE_VERSION,
    InterfaceName: interfaceName,
    NetworkNamespaceKey: state.networkNamespaceKey,
    Path: state.path,
    Ports: [...state.ports,],
  };
  /**
   Complete state JSON prepared before filesystem mutation.
   */
  const serialized = `${JSON.stringify(
    stateDocument,
    null,
    2,
  )}\n`;
  await mkdir(
    directory,
    {
      mode: 0o700,
      recursive: true,
    },
  );
  try {
    await writeFile(
      temporaryPath,
      serialized,
      {
        encoding: 'utf8',
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
    try {
      await unlink(temporaryPath,);
    }
    catch (cleanupError) {
      if (!(isErrnoException(cleanupError,) && (cleanupError.code === 'ENOENT')))
        throw new OpenSnitchConfigError(
          `Cannot remove temporary wg-quicker OpenSnitch lifecycle state: ${temporaryPath}`,
          { cause: cleanupError, },
        );
    }
    throw new OpenSnitchConfigError(
      `Cannot persist wg-quicker OpenSnitch lifecycle state: ${path}`,
      { cause: error, },
    );
  }
}

/**
 Clears lifecycle ownership after config and live-chain cleanup succeed.
 
 @param interfaceName - WireGuard interface identity.
 
 @param networkNamespaceKey - Namespace-specific ownership identity.
 
 @throws {@link OpenSnitchConfigError} when state cannot be removed.
 
 @example
 ```ts
 await removeOpenSnitchState({ interfaceName: 'wg0', networkNamespaceKey: 'abc123' });
 ```
 */
export async function removeOpenSnitchState(
  {
    interfaceName,
    networkNamespaceKey,
  }: {
    readonly interfaceName: string;
    readonly networkNamespaceKey: string;
  },
): Promise<void> {
  /**
   Interface-and-namespace-specific state path.
   */
  const path = openSnitchStatePath({
    interfaceName,
    networkNamespaceKey,
  },);
  try {
    await unlink(path,);
  }
  catch (error) {
    if (isErrnoException(error,) && (error.code === 'ENOENT'))
      return;
    throw new OpenSnitchConfigError(
      `Cannot clear wg-quicker OpenSnitch lifecycle state: ${path}`,
      { cause: error, },
    );
  }
}
