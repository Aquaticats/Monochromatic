import {
  lstat,
  open,
  readFile,
} from 'node:fs/promises';

import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { OpenSnitchConfigError, } from './errors.ts';
import { parseOpenSnitchConfig, } from './opensnitch-config-tree.ts';

/**
 Module logger for OpenSnitch config persistence.
 */
const l = tagged({ tag: 'opensnitch-config-file', },);

/**
 Sentinel indicating OpenSnitch system-firewall file is absent.
 */
export const OPENSNITCH_CONFIG_ABSENT: unique symbol = Symbol(
  'OpenSnitch system-firewall config absent',
);

/**
 Reports whether unknown failure carries Node filesystem code.
 
 @param error - Unknown caught value.
 
 @returns Whether value is Node error with optional code.
 
 @example
 ```ts
 isErrnoException({ code: 'ENOENT' });
 ```
 */
function isErrnoException(error: unknown,): error is NodeJS.ErrnoException {
  return Error.isError(error,);
}

/**
 Narrows config-read result to exact absence sentinel.
 
 @param value - Config text or absence symbol.
 
 @returns Whether OpenSnitch config path was absent.
 
 @example
 ```ts
 isOpenSnitchConfigAbsent(OPENSNITCH_CONFIG_ABSENT);
 ```
 */
export function isOpenSnitchConfigAbsent(
  value: string | typeof OPENSNITCH_CONFIG_ABSENT,
): value is typeof OPENSNITCH_CONFIG_ABSENT {
  if ((typeof value) !== 'symbol')
    return false;
  return value === OPENSNITCH_CONFIG_ABSENT;
}

/**
 Reads config when installed and rejects non-regular paths.
 
 @param path - OpenSnitch system-firewall config path.
 
 @returns Existing text or absence sentinel when OpenSnitch is not installed.
 
 @throws {@link OpenSnitchConfigError} when existing path cannot be read safely.
 
 @example
 ```ts
 await readOpenSnitchConfig({ path: '/etc/opensnitchd/system-fw.json' });
 ```
 */
export async function readOpenSnitchConfig(
  { path, }: { readonly path: string; },
): Promise<string | typeof OPENSNITCH_CONFIG_ABSENT> {
  try {
    /**
     Link metadata rejects symlink and hard-link replacement targets.
     */
    const metadata = await lstat(path,);
    if ((!metadata.isFile()) || (metadata.nlink !== 1)) {
      throw new OpenSnitchConfigError(
        `OpenSnitch system-firewall config must be one regular file with one link: ${path}`,
      );
    }
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error) {
    if (isErrnoException(error,) && (error.code === 'ENOENT'))
      return OPENSNITCH_CONFIG_ABSENT;
    throw new OpenSnitchConfigError(
      `Cannot read OpenSnitch system-firewall config: ${path}`,
      { cause: error, },
    );
  }
}

/**
 Writes one payload without truncating watched inode.
 
 Shorter JSON is padded with valid trailing whitespace to retain file size.
 One write event avoids duplicate OpenSnitch 1.8 reloads from truncate-plus-write.
 
 @param path - OpenSnitch system-firewall config path.
 
 @param text - Valid JSON text written at offset zero.
 
 @param minimumSize - Existing byte size retained through whitespace padding.
 
 @throws {@link OpenSnitchConfigError} when one write syscall is incomplete.
 
 @example
 ```ts
 await writeThroughWatchedInode({ path, text, minimumSize: 1024 });
 ```
 */
async function writeThroughWatchedInode(
  {
    path,
    text,
    minimumSize,
  }: {
    readonly path: string;
    readonly text: string;
    readonly minimumSize: number;
  },
): Promise<void> {
  /**
   UTF-8 replacement bytes.
   */
  const replacement = Buffer.from(
    text,
    'utf8',
  );
  /**
   Trailing JSON whitespace retaining existing inode size.
   */
  const padding = Buffer.alloc(
    Math.max(
      0,
      minimumSize - replacement.length,
    ),
    ' ',
  );
  /**
   Single payload passed through one positional write.
   */
  const payload = Buffer.concat([
    replacement,
    padding,
  ],);
  /**
   Watched file descriptor retaining inode through write.
   */
  await using handle = await open(
    path,
    'r+',
  );
  /**
   Positional write result used to reject a partial config.
   */
  const { bytesWritten, } = await handle.write(
    payload,
    0,
    payload.length,
    0,
  );
  if (bytesWritten !== payload.length) {
    throw new OpenSnitchConfigError(
      `OpenSnitch system-firewall write was incomplete: ${path}`,
    );
  }
  await handle.sync();
}

/**
 Writes validated JSON through watched inode and restores original text after write failure.
 
 OpenSnitch 1.8 watches file inode for `Write`;
 replacing path by rename can race its nftables reload.
 
 @param path - OpenSnitch system-firewall config path.
 
 @param original - Exact source text retained for recovery.
 
 @param rendered - Validated replacement text.
 
 @throws {@link OpenSnitchConfigError} when write or recovery fails.
 
 @example
 ```ts
 await writeOpenSnitchConfig({ path, original, rendered });
 ```
 */
export async function writeOpenSnitchConfig(
  {
    path,
    original,
    rendered,
  }: {
    readonly path: string;
    readonly original: string;
    readonly rendered: string;
  },
): Promise<void> {
  parseOpenSnitchConfig({
    text: rendered,
    path,
  },);
  /**
   Original byte length retained when replacement is shorter.
   */
  const originalSize = Buffer.byteLength(
    original,
    'utf8',
  );
  try {
    await writeThroughWatchedInode({
      path,
      text: rendered,
      minimumSize: originalSize,
    },);
  }
  catch (error) {
    l.error(`OpenSnitch config write failed at ${path}: ${String(error,)}`,);
    try {
      /**
       Current length may exceed original after failed extending write.
       */
      const recoverySize = Math.max(
        originalSize,
        Buffer.byteLength(
          rendered,
          'utf8',
        ),
      );
      await writeThroughWatchedInode({
        path,
        text: original,
        minimumSize: recoverySize,
      },);
    }
    catch (restoreError) {
      l.error(`OpenSnitch config recovery failed at ${path}: ${String(restoreError,)}`,);
      throw new OpenSnitchConfigError(
        `OpenSnitch system-firewall write and recovery failed: ${path}`,
        { cause: restoreError, },
      );
    }
    throw new OpenSnitchConfigError(
      `Cannot write OpenSnitch system-firewall config: ${path}`,
      { cause: error, },
    );
  }
  /**
   Persisted content parsed again from disk before network setup continues.
   */
  const persisted = await readFile(
    path,
    'utf8',
  );
  parseOpenSnitchConfig({
    text: persisted,
    path,
  },);
}
