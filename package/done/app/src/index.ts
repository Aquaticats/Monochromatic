/**
 Server-side library surface of the Done app.

 The server itself runs from source (`node src/server.ts`). This entry exists
 so unit tests load request parsing and schema migration code through the
 rolldown bundle under `dist/final/node` rather than through sibling source:
 a test that reaches source proves nothing about the artifact the package
 builds, which is what the `test-import/require-eventual-artifact` rule
 enforces.

 Every export is internal. The app declares no `exports` map and nothing
 outside the package imports this entry.

 @module
 */

/**
 Request-body validation and parsing, exported for built-artifact verification.

 @internal
 */
export {
  INVALID,
  isRecord,
  parseEnumValue,
  parseStringArray,
} from './server/api/tasks-parse.ts';

/**
 Task update parsing, exported for built-artifact verification.

 @internal
 */
export { parseTaskUpdateInput, } from './server/api/tasks-parse-update.ts';

/**
 Schema migrations and the guarded native FTS index, exported for
 built-artifact verification.

 @internal
 */
export {
  runMigrations,
  tryEnableFts,
} from './lib/db-migrations.ts';

/**
 Search statements the migration test's FTS and LIKE paths assert on, exported
 for built-artifact verification.

 @internal
 */
export {
  SQL_SEARCH_FTS,
  SQL_SEARCH_LIKE,
} from './lib/db/tasks-sql.ts';
