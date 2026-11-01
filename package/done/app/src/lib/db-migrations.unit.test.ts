/**
 Tests for the schema migration runner and the guarded native FTS index.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  connect,
  type Database,
} from '@tursodatabase/database';

import {
  runMigrations,
  tryEnableFts,
} from './db-migrations.ts';
import {
  SQL_SEARCH_FTS,
  SQL_SEARCH_LIKE,
} from './db/tasks-sql.ts';

/**
 Column values for a single seeded task row; `created_at` mirrors `updatedAt`.
 */
type TaskSeed = {
  id: string;
  title: string;
  description: string;
  tags: string;
  updatedAt: string;
};

/**
 Opens a fresh in-memory database with the FTS index method enabled and every
 migration applied.
 
 @returns Migrated in-memory database
 
 @example
 ```ts
 const database = await freshDatabase();
 ```
 */
async function freshDatabase(): Promise<Database> {
  /**
   In-memory connection opened with the experimental index method Turso FTS needs.
   */
  const database = await connect(
    ':memory:',
    { experimental: ['index_method',], },
  );
  await runMigrations(database,);
  return database;
}

/**
 Inserts one task row.
 
 @param database - Target database
 @param seed - Column values to write
 
 @example
 ```ts
 await insertTask({ database, seed });
 ```
 */
async function insertTask({ database, seed, }: { database: Database; seed: TaskSeed; },): Promise<void> {
  await (await database.prepare(
    'INSERT INTO tasks (id, title, description, tags, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)',
  ))
    .run(
    seed.id,
    seed.title,
    seed.description,
    seed.tags,
    seed.updatedAt,
    seed.updatedAt,
    );
}

/**
 Runs the native FTS search and returns only the matched task IDs, in order.
 
 @param database - Migrated database
 @param query - Raw query bound to the FTS predicate
 
 @returns Matched task IDs, ordered by the search SQL
 
 @example
 ```ts
 const ids = await ftsSearchIds({ database, query: 'groceries' });
 ```
 */
async function ftsSearchIds({ database, query, }: { database: Database; query: string; },): Promise<string[]> {
  /**
   Matched rows narrowed to the id projection the assertions compare against.
   */
  const rows = (await (await database.prepare(SQL_SEARCH_FTS,)).all(query,)) as { id: string; }[];
  return rows.map(function toId(row,) {
    return row.id;
  },);
}

await describe({
  name: runMigrations.name,
  children: [
    it({
      name: 'resolves and leaves the tasks table queryable',
      fn: async function migratesTables(): Promise<void> {
        /**
         Freshly migrated database under test.
         */
        const database = await freshDatabase();
        /**
         Rows from the empty tasks table, proving the table exists after migration.
         */
        const rows = (await (await database.prepare('SELECT id FROM tasks',)).all()) as { id: string; }[];
        expect(rows,).toHaveLength(0,);
      },
    },),
    it({
      name: 'creates the native FTS index so search matches an inserted task',
      fn: async function matchesInserted(): Promise<void> {
        /**
         Freshly migrated database under test.
         */
        const database = await freshDatabase();
        await insertTask({
          database,
          seed: {
            id: 'a',
            title: 'Buy groceries',
            description: 'milk and eggs',
            tags: '["home"]',
            updatedAt: '2024-01-01',
          },
        },);
        expect(await ftsSearchIds({ database, query: 'groceries', },),).toEqual(['a',],);
      },
    },),
    it({
      name: 'keeps the index in sync on update and delete without triggers',
      fn: async function staysInSync(): Promise<void> {
        /**
         Freshly migrated database under test.
         */
        const database = await freshDatabase();
        await insertTask({
          database,
          seed: {
            id: 't',
            title: 'alpha widget',
            description: 'first',
            tags: '[]',
            updatedAt: '2024-01-01',
          },
        },);
        expect(await ftsSearchIds({ database, query: 'widget', },),).toEqual(['t',],);

        await (await database.prepare('UPDATE tasks SET title = ?, updated_at = ? WHERE id = ?',))
          .run('beta gadget', '2024-02-01', 't',);
        expect(await ftsSearchIds({ database, query: 'widget', },),).toEqual([],);
        expect(await ftsSearchIds({ database, query: 'gadget', },),).toEqual(['t',],);

        await (await database.prepare('DELETE FROM tasks WHERE id = ?',)).run('t',);
        expect(await ftsSearchIds({ database, query: 'gadget', },),).toEqual([],);
      },
    },),
    it({
      name: 'orders tied-score matches by most recent update',
      fn: async function ordersByRecency(): Promise<void> {
        /**
         Freshly migrated database under test.
         */
        const database = await freshDatabase();
        await insertTask({
          database,
          seed: {
            id: 'old',
            title: 'widget report',
            description: 'x',
            tags: '[]',
            updatedAt: '2021-01-01',
          },
        },);
        await insertTask({
          database,
          seed: {
            id: 'new',
            title: 'widget summary',
            description: 'y',
            tags: '[]',
            updatedAt: '2023-01-01',
          },
        },);
        expect(await ftsSearchIds({ database, query: 'widget', },),).toEqual(['new', 'old',],);
      },
    },),
    it({
      name: 'builds the index over rows that existed before creation',
      fn: async function indexesPreexistingRows(): Promise<void> {
        /**
         Freshly migrated database whose FTS index is dropped to simulate an
         older schema, then rebuilt over an already-present row.
         */
        const database = await freshDatabase();
        await (await database.prepare('DROP INDEX tasks_fts',)).run();
        await insertTask({
          database,
          seed: {
            id: 'kiwi',
            title: 'preexisting kiwi',
            description: 'fruit',
            tags: '[]',
            updatedAt: '2024-01-01',
          },
        },);
        expect(await tryEnableFts(database,),).toBe(true,);
        expect(await ftsSearchIds({ database, query: 'kiwi', },),).toEqual(['kiwi',],);
      },
    },),
  ],
},);

await describe({
  name: tryEnableFts.name,
  children: [
    it({
      name: 'returns true when the FTS index method is available',
      fn: async function reportsAvailable(): Promise<void> {
        /**
         Bare table the FTS index attaches to; the index method is available on
         the pinned build.
         */
        const database = await connect(':memory:', { experimental: ['index_method',], },);
        await database.exec('CREATE TABLE tasks (title TEXT, description TEXT, tags TEXT)',);
        expect(await tryEnableFts(database,),).toBe(true,);
      },
    },),
    it({
      name: 'returns false and does not throw when index creation fails',
      fn: async function degradesGracefully(): Promise<void> {
        /**
         Minimal stand-in whose `exec` rejects, mimicking a build without the
         experimental FTS index method.
         */
        const failingDatabase = {
          exec: async function exec(): Promise<void> {
            throw new Error('no such index method: fts',);
          },
        };
        /**
         Result of the guarded call against the failing stand-in.
         */
        const enabled = await tryEnableFts(failingDatabase as unknown as Database,);
        expect(enabled,).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: 'LIKE search fallback',
  children: [
    it({
      name: 'matches by substring where the FTS path would not',
      fn: async function matchesSubstring(): Promise<void> {
        /**
         Freshly migrated database under test.
         */
        const database = await freshDatabase();
        await insertTask({
          database,
          seed: {
            id: 'g',
            title: 'groceries',
            description: 'weekly shop',
            tags: '[]',
            updatedAt: '2024-01-01',
          },
        },);
        /**
         Substring-matched rows from the LIKE fallback query.
         */
        const rows = (await (await database.prepare(SQL_SEARCH_LIKE,)).all('%groc%', '%groc%',)) as { id: string; }[];
        expect(rows.map(function toId(row,) {
          return row.id;
        },),).toEqual(['g',],);
      },
    },),
  ],
},);
