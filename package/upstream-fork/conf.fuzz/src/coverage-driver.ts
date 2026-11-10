/**
 Deterministic coverage driver:
 exercises the conf-fork public store surface and its error paths with
 fixed inputs,
 so the V8 coverage it produces is reproducible.
 Run under `NODE_V8_COVERAGE` by the `fuzz:coverage` task,
 then summarized by `coverage-report.ts`.

 The blocks live beside this file, split to stay inside the file-length budget:
 fixed inputs and shared helpers in `coverage-fixtures.ts`,
 the item surface in `coverage-exercise-store.ts`,
 and the store options in `coverage-exercise-options.ts`.

 @module
 */

import {
  exerciseConstruction,
  exerciseEncryption,
  exerciseFailingMigration,
  exerciseMigrations,
  exerciseSchema,
} from './coverage-exercise-options.ts';
import { exerciseStoreField, } from './coverage-exercise-store.ts';

//region Exercise

/**
 Exercises construction,
 defaults,
 the item surface with its error paths,
 store replacement with bookkeeping preservation,
 item counts and iteration,
 change subscriptions,
 schema validation,
 encryption round-trips,
 and migrations.
 
 @example
 ```ts
 await exercise();
 ```
 */
async function exercise(): Promise<void> {
  await exerciseConstruction();
  await exerciseStoreField();
  await exerciseSchema();
  await exerciseEncryption();
  await exerciseMigrations();
  await exerciseFailingMigration();
}

//endregion Exercise

await exercise();
