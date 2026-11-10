/**
 Reads the clearnet host list that the installed build carries, again at every refresh.

 A route watcher keeps the bypass rules of the update hosts and the model provider hosts current,
 and a new list of those hosts reaches a tunnel without a restart because the watcher reads this data file
 at every refresh instead of only the list its own process was built with.
 The file is the copy the build emits beside the route-watcher bundle,
 so replacing the build replaces the list the running watcher applies.
 What the file must contain is read by `clearnet-hosts-data-values.ts`.

 @module
 */

import {
  constants,
  type Stats,
} from 'node:fs';
import { open as openFile, } from 'node:fs/promises';
import { fileURLToPath, } from 'node:url';

import {
  isClearnetHostsDataError,
  parseClearnetHostsData,
} from './clearnet-hosts-data-values.ts';
import { CLEARNET_HOSTS_DATA_FILENAME, } from './clearnet-hosts-data-file.ts';
import { isErrnoException, } from './errors.ts';

export {
  CLEARNET_HOSTS_DATA_SCHEMA_VERSION,
  type ClearnetHostsDataError,
  clearnetHostsFromData,
  isBareHostname,
  isClearnetHostsDataError,
  parseClearnetHostsData,
  providerHost,
} from './clearnet-hosts-data-values.ts';

/**
 Writable mode bits a data file read by a root watcher never carries.
 */
const UNSAFE_WRITE_BITS = 0o022;

/**
 Outcome of one reading of the host data of the installed build.
 */
export type HostListRead =
  | {
    /**
     The file was read,
     and these are its hosts.
     */
    readonly kind: 'read';

    /**
     Hosts every rule is installed for,
     the update hosts followed by the model provider hosts.
     */
    readonly hosts: readonly string[];
  }
    | {
    /**
     The file could not be read,
     so the caller keeps the rules of the list it read last.
     */
      readonly kind: 'failed';

    /**
     What is wrong with the file,
     naming the path.
     */
      readonly failure: string;

    /**
     Path that was read.
     */
      readonly path: string;
    };

/**
 Reads the clearnet host list the installed build carries.

 The watcher decides what a failure means,
 so the reader reports it instead of installing a different list.

 @example
 ```ts
 const read = createInstalledClearnetHostsReader({});
 const result = await read();
 ```
 */
export type HostListReader = () => Promise<HostListRead>;

/**
 Resolves the data file that sits beside the running bundle.

 The build copies the file into each output directory beside the chunks,
 so a bundle finds it wherever it runs,
 and the watcher finds the copy of the build that was installed over the one it started from.

 @returns Absolute path of the data file.

 @example
 ```ts
 defaultClearnetHostsDataPath();
 ```
 */
export function defaultClearnetHostsDataPath(): string {
  return fileURLToPath(new URL(
    CLEARNET_HOSTS_DATA_FILENAME,
    import.meta.url,
  ),);
}

/**
 Reads which identities may own the data file this tunnel's watcher reads.

 A root watcher reads a file only when its owner is root or the user the tunnel was started for,
 so no other account on the machine can choose which hosts leave the tunnel.
 Without a `sudo` caller only root qualifies,
 and `up` started by root directly then falls back to the list compiled into this build and says so.

 @param recordedCaller - Caller identity `sudo` recorded,
 absent when there was none.

 @returns Owner uids allowed in addition to root.

 @example
 ```ts
 allowedDataOwnerUids({ recordedCaller: '1000' }); // [0, 1000]
 ```
 */
export function allowedDataOwnerUids(
  { recordedCaller, }: { readonly recordedCaller?: string; },
): readonly number[] {
  if ((recordedCaller === undefined) || (recordedCaller === ''))
    return [0,];
  /**
   Parsed caller identity.
   */
  const uid = Number(recordedCaller,);
  if ((!Number.isSafeInteger(uid,)) || (uid < 1))
    return [0,];
  return [
    0,
    uid,
  ];
}

/**
 Reads which identities may own the data file of this run.

 @returns Owner uids allowed in addition to root.

 @example
 ```ts
 allowedOwnerUidsForThisRun();
 ```
 */
export function allowedOwnerUidsForThisRun(): readonly number[] {
  /**
   Caller identity `sudo` records, when it recorded one.
   */
  const recordedCaller = process.env
    .SUDO_UID;
  return allowedDataOwnerUids(recordedCaller === undefined ? {} : { recordedCaller, },);
}

/**
 Names what makes a data file one a root watcher must not read.

 @param stats - Metadata of the file that was opened.

 @param path - Path named in the refusal.

 @param allowedOwnerUids - Identities that may own the file.

 @returns A reason that follows the path,
 or the empty string when the file may be read.

 @example
 ```ts
 dataFileRefusal({ stats, path: 'data.json', allowedOwnerUids: [0] });
 ```
 */
function dataFileRefusal(
  {
    stats,
    path,
    allowedOwnerUids,
  }: {
    readonly stats: Stats;
    readonly path: string;
    readonly allowedOwnerUids: readonly number[];
  },
): string {
  if (!stats.isFile())
    return `${path} is not a regular file, and only a regular file is read`;
  if ((stats.mode & UNSAFE_WRITE_BITS) !== 0) {
    return `${path} is writable by its group or by another user, `
      + 'and a root watcher reads only a file nobody else can change';
  }
  if (!allowedOwnerUids.includes(stats.uid,)) {
    return `${path} is owned by uid ${String(stats.uid,)}, `
      + `and this tunnel reads only a file owned by uid ${allowedOwnerUids.join(', ',)}`;
  }
  return '';
}

/**
 Reads the failure of a data file that cannot be opened or read.

 @param error - Caught value.

 @param path - Path that was read.

 @returns What is wrong with the file.

 @example
 ```ts
 readFailureText({ error, path: 'data.json' });
 ```
 */
function readFailureText({
  error,
  path,
}: {
  readonly error: unknown;
  readonly path: string
},): string {
  if (isClearnetHostsDataError({ error, },) && Error.isError(error,))
    return error.message;
  if (isErrnoException(error,) && (error.code === 'ELOOP')) {
    return `${path} is a symbolic link, or sits below one, and no link is followed`;
  }
  return `${path} cannot be read: ${String(error,)}`;
}

/**
 Builds the reader the route watcher uses at every refresh.

 The file is opened without following a link and validated on the handle that is read,
 so a link or another file planted at the path is refused instead of handed to root.

 @param path - Data file to read,
 the copy beside the running bundle by default.

 @param allowedOwnerUids - Identities that may own the file,
 root and the tunnel's caller by default.

 @returns Reader of the installed host list.

 @example
 ```ts
 const read = createInstalledClearnetHostsReader({ path: '/usr/local/lib/wg-quicker/clearnet-hosts.data.json' });
 ```
 */
export function createInstalledClearnetHostsReader(
  {
    path = defaultClearnetHostsDataPath(),
    allowedOwnerUids = allowedOwnerUidsForThisRun(),
  }: {
    readonly path?: string;
    readonly allowedOwnerUids?: readonly number[];
  } = {},
): HostListReader {
  return async function readInstalledClearnetHosts(): Promise<HostListRead> {
    try {
      /**
       Open handle of the data file,
       which no link is followed to reach.
       */
      await using handle = await openFile(
        path,
        constants.O_RDONLY | constants.O_NOFOLLOW,
      );
      /**
       What refuses the file before its text reaches a list.
       */
      const refusal = dataFileRefusal({
        stats: await handle.stat(),
        path,
        allowedOwnerUids,
      },);
      if (refusal !== '') {
        return {
          kind: 'failed',
          failure: refusal,
          path,
        };
      }
      return {
        kind: 'read',
        hosts: parseClearnetHostsData({
          text: await handle.readFile({ encoding: 'utf8', },),
          path,
        },),
      };
    }
    catch (error) {
      return {
        kind: 'failed',
        failure: readFailureText({
          error,
          path,
        },),
        path,
      };
    }
  };
}
