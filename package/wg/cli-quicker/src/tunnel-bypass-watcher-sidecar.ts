import {
  readFile,
  rename,
  rm,
  writeFile,
} from 'node:fs/promises';

import {
  BypassRouteError,
  isErrnoException,
} from './errors.ts';

/**
 Persisted watcher process identity resistant to PID reuse.
 */
export type WatcherProcessIdentity = {
  readonly ownerId: string;
  readonly pid: number;
  readonly startTime: string;
};

/**
 What the watcher record says.

 `unreadable` is a record that is there and holds no identity:
 a kill during the write left a fragment there in builds that wrote the record in place,
 and no watcher of that record is running any more.
 A caller treats it as no identity at all, and says so,
 because refusing the whole lifecycle over a fragment wedged `down` and `up` for good.
 */
export type WatcherRecord =
  | {
    /**
     A record holding an identity.
     */
    readonly kind: 'identity';

    /**
     The persisted identity.
     */
    readonly identity: WatcherProcessIdentity;
  }
    | {
    /**
     No record at all.
     */
      readonly kind: 'absent';
    }
    | {
    /**
     A record that holds no identity.
     */
      readonly kind: 'unreadable';

    /**
     The record's path,
     for the report.
     */
      readonly path: string;

    /**
     What is wrong with it,
     in the words of the parser.
     */
      readonly reason: string;
    };

/**
 What reading the record's text found:
 its text, or no file at all.
 */
type WatcherRecordText =
  | {
    /**
     The record's text.
     */
    readonly kind: 'text';

    /**
     What the file held.
     */
    readonly text: string;
  }
  | {
    /**
     No record file stands at the path.
     */
    readonly kind: 'absent';
  };

/**
 Resolves watcher identity sidecar for state path.
 
 @param statePath - Persisted bypass state path.
 
 @returns Watcher identity path.
 
 @example
 ```ts
 watcherIdentityPath({ statePath: '/run/wg-quicker/interface.json' });
 ```
 */
export function watcherIdentityPath(
  { statePath, }: { readonly statePath: string; },
): string {
  return `${statePath}.watcher.json`;
}

/**
 Parses watcher sidecar JSON with lifecycle diagnostics.
 
 @param text - Sidecar JSON text.
 
 @param statePath - State path named in diagnostics.
 
 @returns Parsed unknown value.
 
 @throws {@link BypassRouteError} when JSON is malformed.
 
 @example
 ```ts
 parseWatcherJson({ text: '{}', statePath: '/tmp/state' });
 ```
 */
function parseWatcherJson(
  {
    text,
    statePath,
  }: {
    readonly text: string;
    readonly statePath: string;
  },
): unknown {
  try {
    return JSON.parse(text,);
  }
  catch (error) {
    throw new BypassRouteError(`Invalid bypass watcher identity for ${statePath}: ${String(error,)}`,);
  }
}

/**
 Parses watcher sidecar JSON and shape.
 
 @param text - Sidecar JSON text.
 
 @param statePath - State path named in diagnostics.
 
 @returns Validated watcher identity.
 
 @throws {@link BypassRouteError} when JSON or fields are invalid.
 
 @example
 ```ts
 parseWatcherIdentity({ text: '{"pid":1}', statePath: '/tmp/state' });
 ```
 */
function parseWatcherIdentity(
  {
    text,
    statePath,
  }: {
    readonly text: string;
    readonly statePath: string;
  },
): WatcherProcessIdentity {
  /**
   Parsed sidecar before shape checks.
   */
  const value: unknown = parseWatcherJson({
    text,
    statePath,
  },);
  if (((typeof value) !== 'object')
    || (value === null)
    || (!('ownerId' in value))
    || (!('pid' in value))
    || (!('startTime' in value))
    || ((typeof value.ownerId) !== 'string')
    || (value.ownerId === '')
    || ((typeof value.pid) !== 'number')
    || (!Number.isSafeInteger(value.pid,))
    || (value.pid <= 0)
    || ((typeof value.startTime) !== 'string')
    || (value.startTime === '')) {
    throw new BypassRouteError(`Invalid bypass watcher identity for ${statePath}.`,);
  }
  return {
    ownerId: value.ownerId,
    pid: value.pid,
    startTime: value.startTime,
  };
}

/**
 The text of the watcher record, or that there is no record.

 @param path - The record's path.

 @returns The text, or absence.

 @example
 ```ts
 await recordText({ path: '/run/wg-quicker/interface-x.json.watcher.json' });
 ```
 */
async function recordText(
  { path, }: { readonly path: string; },
): Promise<WatcherRecordText> {
  try {
    return {
      kind: 'text',
      text: await readFile(
        path,
        'utf8',
      ),
    };
  }
  catch (error) {
    if (isErrnoException(error,) && (error.code === 'ENOENT'))
      return { kind: 'absent', };
    throw error;
  }
}

/**
 Reads the watcher record and says which of its three states it is in.
 
 A read error other than absence is thrown:
 a record that cannot be read is not a record that holds no identity.

 @param statePath - Persisted bypass state path.
 
 @returns The identity, its absence, or that the record holds none.

 @example
 ```ts
 await readWatcherRecord({ statePath: '/run/wg-quicker/interface.json' });
 ```
 */
export async function readWatcherRecord(
  { statePath, }: { readonly statePath: string; },
): Promise<WatcherRecord> {
  /**
   The record's path,
   for the unreadable answer.
   */
  const path = watcherIdentityPath({ statePath, },);
  /**
   What the record holds.
   */
  const record = await recordText({ path, },);
  if (record.kind === 'absent')
    return { kind: 'absent', };
  try {
    return {
      kind: 'identity',
      identity: parseWatcherIdentity({
        text: record.text,
        statePath,
      },),
    };
  }
  catch (error) {
    if (error instanceof BypassRouteError)
      return {
        kind: 'unreadable',
        path,
        reason: error.message,
      };
    throw error;
  }
}

/**
 Writes the watcher record so that a kill cannot leave half a record behind.
 
 The text goes to a temporary file beside the record and is renamed over it:
 a rename is atomic, so a reader sees the whole record or none of it.
 A record that is already there is replaced, which is what a start does after it stopped the watcher of the record before.

 @param statePath - Persisted bypass state path.
 
 @param identity - What the watcher records about itself.

 @param path - Record path to write,
 replaceable in tests.
 
 @example
 ```ts
 await writeWatcherRecord({ statePath: '/run/wg-quicker/interface.json', identity });
 ```
 */
export async function writeWatcherRecord(
  {
    statePath,
    identity,
    path = watcherIdentityPath({ statePath, },),
  }: {
    readonly statePath: string;
    readonly identity: WatcherProcessIdentity;
    readonly path?: string;
  },
): Promise<void> {
  /**
   Temporary file beside the record,
   named after this process so two writers never share one.
   */
  const temporary = `${path}.${String(process.pid,)}.tmp`;
  await writeFile(
    temporary,
    JSON.stringify(identity,),
    {
      flag: 'wx',
      mode: 0o600,
    },
  );
  try {
    await rename(
      temporary,
      path,
    );
  }
  catch (error) {
    await rm(
      temporary,
      { force: true, },
    );
    throw error;
  }
}
