/**
 Explicit testing seam for the task REST handlers.

 The `index` entry carries no database code: importing it must not open a
 database, run migrations, or touch a data file. Importing any module that
 reaches `lib/db.ts` does, because that module connects while it evaluates.

 This entry exists so unit tests can drive the handlers through the bundle
 this package builds, which is what `test-import/require-eventual-artifact`
 enforces, while the database side effect stays confined to a test process
 that points `DB_PATH` at an in-memory database before the import resolves.

 @module
 */

/**
 Task create and update handlers, exported for built-artifact verification.

 @internal
 */
export {
  handleCreateTask,
  handleUpdateTask,
} from './server/api/tasks.ts';
