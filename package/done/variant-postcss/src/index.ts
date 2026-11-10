/**
 Server-side library surface of the Done postcss variant.

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
} from './server/api/task-validation.ts';

/**
 Task update parsing, exported for built-artifact verification.

 @internal
 */
export { parseTaskUpdateInput, } from './server/api/task-validation-update.ts';

/**
 Schema migrations and the guarded native FTS index, exported for
 built-artifact verification.

 @internal
 */
export {
  runMigrations,
  tryEnableFts,
} from './lib/db/migrations.ts';

/**
 Search statements the migration test's FTS and LIKE paths assert on, exported
 for built-artifact verification.

 @internal
 */
export {
  SQL_SEARCH_FTS,
  SQL_SEARCH_LIKE,
} from './lib/db/task-sql.ts';

/**
 Drawer navigation and header builders, exported for built-artifact
 verification: the drawer must not offer a destination that only moves the
 scroll position, nor a placeholder identity.

 @internal
 */
export {
  buildHeader,
  buildNav,
} from './client/component/side-drawer-nav.ts';
