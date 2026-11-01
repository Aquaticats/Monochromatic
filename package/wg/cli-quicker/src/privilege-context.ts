import { constants, } from 'node:fs';
import {
  mkdtemp,
  open,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { PrivilegeError, } from './errors.ts';
import {
  capturePrivilegeContext,
  MAX_PRIVILEGE_CONTEXT_BYTES,
  parsePrivilegeContext,
} from './privilege-context-data.ts';

/**
 Internal argument identifying private caller-context file.
 */
export const PRIVILEGE_CONTEXT_ARGUMENT = '--wg-quicker-privilege-context';

/**
 Permission bits forbidden on caller-context file.
 */
const PRIVATE_FILE_MODE_MASK = 0o077;

/**
 Private context file retained until sudo child closes.
 */
export type PrivilegeContextFile = {
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates private bounded context file for privileged child.
 
 @returns Disposable file contract carrying path.
 
 @throws {@link PrivilegeError} when context exceeds bound.
 
 @example
 ```ts
 await using context = await createPrivilegeContextFile();
 ```
 */
export async function createPrivilegeContextFile(): Promise<PrivilegeContextFile> {
  /**
   Validated context serialized without inherited object capabilities.
   */
  const serialized = JSON.stringify(capturePrivilegeContext(),);
  /**
   Serialized byte count checked before filesystem write.
   */
  const size = Buffer.byteLength(
    serialized,
    'utf8',
  );
  if (size > MAX_PRIVILEGE_CONTEXT_BYTES)
    throw new PrivilegeError('Caller context exceeds safe size bound.',);
  /**
   Configured temporary root or system default.
   */
  const temporaryRoot = process
    .env
    .TMPDIR
    ?? '/tmp';
  /**
   User-private directory retained while sudo child runs.
   */
  const directory = await mkdtemp(join(
    temporaryRoot,
    'wg-quicker-context-',
  ),);
  /**
   Fixed file name inside unpredictable private directory.
   */
  const path = join(
    directory,
    'context.json',
  );
  await writeFile(
    path,
    serialized,
    {
      encoding: 'utf8',
      flag: 'wx',
      mode: 0o600,
    },
  );
  return {
    path,
    /**
     Removes caller context even after sudo failure.
     */
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        directory,
        {
          force: true,
          recursive: true,
        },
      );
    },
  };
}

/**
 Parses decimal sudo caller UID.
 
 @returns Valid positive UID.
 
 @throws {@link PrivilegeError} when sudo identity is absent or malformed.
 
 @example
 ```ts
 sudoCallerUid();
 ```
 */
function sudoCallerUid(): number {
  /**
   Original user identity assigned by sudo.
   */
  const raw = process
    .env
    .SUDO_UID;
  if (raw === undefined)
    throw new PrivilegeError('Privilege context requires SUDO_UID.',);
  /**
   Numeric UID candidate.
   */
  const uid = Number(raw,);
  if ((!Number.isSafeInteger(uid,)) || (uid <= 0))
    throw new PrivilegeError('Privilege context requires valid SUDO_UID.',);
  return uid;
}

/**
 Reads validated private context and applies allowlisted environment.
 
 @param path - Private context file path from internal argument.
 
 @throws {@link PrivilegeError} when ownership,
 mode,
 size,
 identity,
 or content validation fails.
 
 @example
 ```ts
 await applyPrivilegeContextFile({ path: '/tmp/private/context.json' });
 ```
 */
async function applyPrivilegeContextFile({ path, }: { readonly path: string; },): Promise<void> {
  /**
   File opened without following final-component symlink.
   */
  await using handle = await open(
    path,
    constants.O_RDONLY | constants.O_NOFOLLOW,
  );
  /**
   Metadata checked on opened descriptor to avoid final-component race.
   */
  const stats = await handle.stat();
  /**
   Original sudo caller used as required file owner.
   */
  const expectedUid = sudoCallerUid();
  if (!stats.isFile())
    throw new PrivilegeError('Caller context path is not regular file.',);
  if ((stats.uid !== expectedUid) || (stats.nlink !== 1))
    throw new PrivilegeError('Caller context file ownership is invalid.',);
  if ((stats.mode & PRIVATE_FILE_MODE_MASK) !== 0)
    throw new PrivilegeError('Caller context file mode is not private.',);
  if (stats.size > MAX_PRIVILEGE_CONTEXT_BYTES)
    throw new PrivilegeError('Caller context file exceeds size bound.',);
  /**
   Parsed context tied to descriptor owner.
   */
  const context = parsePrivilegeContext({ text: await handle.readFile('utf8',), },);
  if (context.uid !== expectedUid)
    throw new PrivilegeError('Caller context UID does not match SUDO_UID.',);
  for (const [key, value,] of Object.entries(context.environment,))
    process.env[key] = value;
}

/**
 Restores internal caller context and returns public CLI arguments.
 
 @returns Arguments after removing internal context pair.
 
 @example
 ```ts
 await restorePrivilegeContext();
 ```
 */
export async function restorePrivilegeContext(): Promise<readonly string[]> {
  /**
   CLI arguments copied from runtime-owned process state.
   */
  const processArguments = process
    .argv
    .slice(2,);
  if ((process.geteuid?.() ?? 0) !== 0)
    return processArguments;
  /**
   Internal marker and path at root-child argument prefix.
   */
  const [marker, contextPath, ...publicArguments] = processArguments;
  if (marker !== PRIVILEGE_CONTEXT_ARGUMENT)
    return processArguments;
  if (contextPath === undefined)
    throw new PrivilegeError('Privilege context argument lacks file path.',);
  await applyPrivilegeContextFile({ path: contextPath, },);
  return publicArguments;
}
