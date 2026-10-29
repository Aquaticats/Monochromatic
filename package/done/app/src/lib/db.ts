/**
 Database initialization module; imported as a side-effect by `server.ts`.
 
 On import, this module:
 1. Resolves the database path from CLI args / env / default
 2. Ensures the directory exists
 3. Opens the SQLite database with WAL mode
 4. Runs migrations (creates tables, indexes, and the native FTS index)
 
 The default export (`db`) is the open Database instance used by `lib/db/tasks.ts`.
 */
import {
  connect,
  type Database,
} from '@tursodatabase/database';
import { mkdir, } from 'node:fs/promises';
import { dirname, } from 'node:path';
import {
  ARGUMENT_ABSENT,
  getArgumentValue,
} from './args.ts';
import { runMigrations, } from './db-migrations.ts';

/**
 Default database file path when neither `--db=` nor `DB_PATH` env var is provided.
 */
const DEFAULT_DATABASE_PATH = './data/done.db';

/**
 Strips the `file:` URI prefix if present, returning a plain filesystem path.
 
 @param value - Raw path that may use `file:` scheme
 
 @returns Plain filesystem path without URI prefix
 */
function normalizeDatabasePath(value: string,): string {
  if (!value.startsWith('file:',))
    return value;
  return value.slice('file:'.length,);
}

/**
 Resolves the database file path from CLI arguments, environment, or default.
 Priority: `--db=PATH` \> `DB_PATH` env var \> {@link DEFAULT_DATABASE_PATH}.
 
 @returns Resolved filesystem path to the database file; reads the flag via
 {@link getArgumentValue}, falling through the priority chain when it
 returns {@link ARGUMENT_ABSENT}
 */
function resolveDatabasePath(): string {
  /**
   Highest-priority source: explicit `--db=` flag.
   */
  const argumentPath = getArgumentValue('db',);
  /**
   Mid-priority source: `DB_PATH` env var when no flag is given.
   */
  const environmentPath = process.env
    .DB_PATH;
  /**
   Falls back to the default constant when neither source supplies a value.
   */
  const rawPath = argumentPath === ARGUMENT_ABSENT
    ? (environmentPath ?? DEFAULT_DATABASE_PATH)
    : argumentPath;
  return normalizeDatabasePath(rawPath,);
}

/**
 Creates the parent directory for the database file if it does not exist.
 Skips creation for `:memory:` databases.
 
 @param databasePath - Resolved filesystem path
 
 @example
 ```ts
 ensureDatabaseDirectoryExists('/home/user/.local/share/done/tasks.db');
 ```
 */
async function ensureDatabaseDirectoryExists(databasePath: string,): Promise<void> {
  if (databasePath === ':memory:')
    return;
  /**
   Parent directory of the database file, created recursively below.
   */
  const directoryPath = dirname(databasePath,);
  await mkdir(
    directoryPath,
    { recursive: true, },
  );
}

/**
 Resolved database file path.
 */
const databasePath = resolveDatabasePath();
await ensureDatabaseDirectoryExists(databasePath,);

/**
 Open SQLite database connection with WAL mode and foreign keys.
 */
const db: Database = await connect(
  databasePath,
  { experimental: ['index_method',], },
);
await db.exec('PRAGMA journal_mode = WAL',);
await db.exec('PRAGMA foreign_keys = ON',);

await runMigrations(db,);

export default db;
