/**
 File, backup, and shell-profile helpers for the one-time credential migration.
 
 The migration is a sequential CLI: it reads old configs, writes the store,
 verifies, then edits the old files. Helpers here keep the orchestration in
 `migrate.ts` small and make every failure path value-free.
 
 @module
 */

import {
  chmod,
  copyFile,
  mkdir,
  readFile,
  rename,
  stat,
  writeFile,
} from 'node:fs/promises';
import { CONFIG_JSON_INDENT_SPACES, } from './config-constants.ts';

//region Constants

/**
 Mode for a disposable fixture or backup directory.
 */
const PRIVATE_DIRECTORY_MODE = 0o700;

/**
 Mode for private migration files.
 */
const PRIVATE_FILE_MODE = 0o600;

/**
 Mode for a shell profile that already exists.
 */
const SHELL_PROFILE_MODE = 0o644;

/**
 Permission mask for an existing file mode.
 */
const FILE_MODE_MASK = 0o777;

/**
 Backup root below home, matching the repo's scratch convention.
 */
export const BACKUP_ROOT = 'temp/agent';

//endregion Constants

//region Types

/**
 One backed-up file and its original path.
 */
export type BackupEntry = {
  /**
   File name inside the backup directory.
   */
  readonly name: string;
  /**
   Original absolute path restored on failure.
   */
  readonly source: string;
};

/**
 Presence-tagged value; the repo models absence without nullish unions.
 */
export type Present<T> = {
  /**
   Value was absent.
   */
  readonly present: false;
} | {
  /**
   Value was present.
   */
  readonly present: true;
  /**
   The value itself.
   */
  readonly value: T;
};

/**
 Error raised when a file cannot be read or parsed for migration.
 */
export class MigrationReadError extends Error {
  /**
   Creates a read or parse failure with the affected path.
 
   @param message - value-free failure description
 
   @param path - affected path
 
   @param options - original failure, never carrying credential output
   */
  public constructor(
    message: string,
    path: string,
    options?: ErrorOptions,
  ) {
    super(
      `${message}: ${path}`,
      options,
    );
    this.name = 'MigrationReadError';
  }
}

//endregion Types

//region Files

/**
 Require an existing regular file.
 
 @param path - candidate path
 
 @param description - human-readable file kind
 
 @throws when the path is absent or not a regular file
  
 @example
 ```ts
 await requireFile({ path: '/repo/.env.local.json', description: 'store' });
 ```
 */
export async function requireFile(
  {
    path,
    description,
  }: {
    readonly path: string;
    readonly description: string;
  },
): Promise<void> {
  try {
    /**
     Local value for metadata.
     */
    const metadata = await stat(path,);
    if (metadata.isFile())
      return;
  }
  catch (error: unknown) {
    if (!isMissingFile(error,))
      throw error;
  }
  throw new MigrationReadError(
    `missing ${description}`,
    path,
  );
}

/**
 Read optional JSON text and parse it as an object.
 
 @param path - JSON file path
 
 @returns presence-tagged parsed record
 
 @throws when the file is unreadable or not a JSON object
  
 @example
 ```ts
 const record = await readJsonIfPresent({ path: '/tmp/config.json' });
 ```
 */
export async function readJsonIfPresent(
  { path, }: { readonly path: string; },
): Promise<Present<Readonly<Record<string, unknown>>>> {
  /**
   Local value for text.
   */
  const text = await readTextIfPresent({ path, },);
  if (!text.present)
    return { present: false, };
  return {
    present: true,
    value: parseJsonObject({
      text: text.value,
      path,
    }),
  };
}

/**
 Parse JSON text as an object without exposing parser input in diagnostics.
 
 @param text - raw file text
 
 @param path - path used in diagnostics
 
 @returns parsed object
 
 @throws when the text is not a JSON object
 */
function parseJsonObject(
  {
    text,
    path,
  }: {
    readonly text: string;
    readonly path: string;
  },
): Readonly<Record<string, unknown>> {
  try {
    /**
     Local value for parsed.
     */
    const parsed = JSON.parse(text,) as unknown;
    if (!isRecord(parsed,))
      throw new MigrationReadError(
        'JSON root must be an object',
        path,
      );
    return parsed;
  }
  catch (error: unknown) {
    if (error instanceof MigrationReadError)
      throw error;
    throw new MigrationReadError(
      'cannot parse JSON',
      path,
      { cause: error, },
    );
  }
}

/**
 Read optional text.
 
 @param path - text file path
 
 @returns presence-tagged text
 
 @throws when the file is unreadable for a reason other than absence
  
 @example
 ```ts
 const text = await readTextIfPresent({ path: '/tmp/profile' });
 ```
 */
export async function readTextIfPresent(
  { path, }: { readonly path: string; },
): Promise<Present<string>> {
  try {
    return {
      present: true,
      value: await readFile(
        path,
        'utf8',
      ),
    };
  }
  catch (error: unknown) {
    if (isMissingFile(error,))
      return { present: false, };
    throw new MigrationReadError(
      'cannot read text',
      path,
      { cause: error, },
    );
  }
}

/**
 Read one optional credential string from a config record.
 
 @param record - presence-tagged config record
 
 @param key - top-level key to read
 
 @param path - config path used in diagnostics
 
 @returns presence-tagged trimmed credential
 
 @throws when the field is present but not a non-empty string
  
 @example
 ```ts
 const value = credentialValue({ record, key: 'exaApiKey', path });
 ```
 */
export function credentialValue(
  {
    record,
    key,
    path,
  }: {
    readonly record: Present<Readonly<Record<string, unknown>>>;
    readonly key: string;
    readonly path: string;
  },
): Present<string> {
  if (!record.present)
    return { present: false, };
  /**
   Local value for value.
   */
  const value = record.value[key];
  if (value === undefined)
    return { present: false, };
  if ((typeof value) !== 'string')
    throw new MigrationReadError(
      `${key} must be a string`,
      path,
    );
  /**
   Local value for trimmed.
   */
  const trimmed = value.trim();
  if (trimmed === '')
    throw new MigrationReadError(
      `${key} must not be blank`,
      path,
    );
  return {
    present: true,
    value: trimmed,
  };
}

//endregion Files

//region Backups

/**
 Copy every existing path into the backup directory, preserving its mode.
 
 @param backupDir - backup directory
 
 @param paths - candidate source paths
 
 @returns backup entries in path order
  
 @example
 ```ts
 const entries = await backupFiles({ backupDir, paths: [storePath] });
 ```
 */
export async function backupFiles(
  {
    backupDir,
    paths,
  }: {
    readonly backupDir: string;
    readonly paths: readonly string[];
  },
): Promise<readonly BackupEntry[]> {
  await mkdir(
    backupDir,
    {
      recursive: true,
      mode: PRIVATE_DIRECTORY_MODE,
    },
  );
  /**
   Backup promises; each resolves to a presence-tagged entry.
   */
  const results = await Promise.all(paths.map(async function backupOne(source,) {
    try {
      /**
       Local value for metadata.
       */
      const metadata = await stat(source,);
      if (!metadata.isFile())
        return { present: false, } as const;
      /**
       Local value for name.
       */
      const name = `${String(
        paths.indexOf(source,),
      )}-${source.split('/',)
        .at(-1,)
        ?? 'file'}`;
      await copyFile(
        source,
        `${backupDir}/${name}`,
      );
      await chmod(
        `${backupDir}/${name}`,
        metadata.mode & FILE_MODE_MASK,
      );
      return {
        present: true,
        value: {
          name,
          source,
        },
      } as const;
    }
    catch (error: unknown) {
      if (isMissingFile(error,))
        return { present: false, } as const;
      throw error;
    }
  },),);
  /**
   Successfully backed-up entries in source order.
   */
  return results.flatMap(function presentEntry(result,) {
    return result.present ? [result.value,] : [];
  },);
}

/**
 Restore every backup after a failed migration.
 
 @param backupDir - backup directory
 
 @param backedUp - backup entries created earlier
  
 @example
 ```ts
 await restoreBackups({ backupDir, backedUp });
 ```
 */
export async function restoreBackups(
  {
    backupDir,
    backedUp,
  }: {
    readonly backupDir: string;
    readonly backedUp: readonly BackupEntry[];
  },
): Promise<void> {
  await Promise.all(backedUp.map(async function restoreOne({
    name,
    source,
  },) {
    try {
      await copyFile(
        `${backupDir}/${name}`,
        source,
      );
    }
    catch (error: unknown) {
      console.error(`migration: could not restore ${source}: ${String(error,)}`,);
    }
  },),);
  console.error(
    `migration: failed; original files restored from ${backupDir}`,
  );
}

//endregion Backups

//region Writes

/**
 Write JSON atomically with two-space indentation and a trailing newline.
 
 @param path - destination path
 
 @param value - JSON-serializable value
 
 @param mode - file mode
  
 @example
 ```ts
 await writeJsonAtomic({ path, value: {}, mode: 0o600 });
 ```
 */
export async function writeJsonAtomic(
  {
    path,
    value,
    mode,
  }: {
    readonly path: string;
    readonly value: unknown;
    readonly mode: number;
  },
): Promise<void> {
  await writeTextAtomic({
    path,
    text: `${JSON.stringify(
      value,
      null,
      CONFIG_JSON_INDENT_SPACES,
    )}\n`,
    mode,
  },);
}

/**
 Write text atomically in the destination directory.
 
 @param path - destination path
 
 @param text - text to write
 
 @param mode - file mode
  
 @example
 ```ts
 await writeTextAtomic({ path, text: '', mode: 0o600 });
 ```
 */
export async function writeTextAtomic(
  {
    path,
    text,
    mode,
  }: {
    readonly path: string;
    readonly text: string;
    readonly mode: number;
  },
): Promise<void> {
  /**
   Local value for temporaryPath.
   */
  const temporaryPath = `${path}.migration-${String(process.pid,)}`;
  await writeFile(
    temporaryPath,
    text,
    { mode, },
  );
  await chmod(
    temporaryPath,
    mode,
  );
  await rename(
    temporaryPath,
    path,
  );
}

/**
 Read an existing file mode, defaulting to the shell-profile mode when absent.
 
 @param path - file path
 
 @returns numeric mode bits
  
 @example
 ```ts
 const mode = await fileMode('/home/user/.bashrc');
 ```
 */
export async function fileMode(path: string,): Promise<number> {
  try {
    return (await stat(path,)).mode & FILE_MODE_MASK;
  }
  catch (error: unknown) {
    if (isMissingFile(error,))
      return SHELL_PROFILE_MODE;
    throw error;
  }
}

//endregion Writes

//region Helpers

/**
 Return whether value is a non-null object record.
 
 @param value - unknown value
 
 @returns whether value is a record
 */
function isRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  return (value !== null)
    && ((typeof value) === 'object')
    && (!Array.isArray(value,));
}

/**
 Return whether a filesystem error is a missing-file error.
 
 @param error - unknown error
 
 @returns whether error has the ENOENT code
 */
function isMissingFile(error: unknown,): boolean {
  return Error.isError(error,)
    && ('code' in error)
    && (error.code === 'ENOENT');
}

/**
 Join a backup root under home.
 
 @param home - home directory
 
 @param name - backup directory name
 
 @returns absolute backup path
  
 @example
 ```ts
 const dir = backupPath({ home, name: 'backup' });
 ```
 */
export function backupPath(
  {
    home,
    name,
  }: {
    readonly home: string;
    readonly name: string;
  },
): string {
  return `${home}/${BACKUP_ROOT}/${name}`;
}

/**
 Directory name for a migration backup.
 
 @returns timestamped directory name
  
 @example
 ```ts
 const name = backupDirectoryName();
 ```
 */
export function backupDirectoryName(): string {
  return `env-local-json-migration-${new Date().toISOString()
    .replaceAll(
      ':',
      '-',
    )}`;
}

//endregion Helpers
