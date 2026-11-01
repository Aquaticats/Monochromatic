/**
 Post-install mutations for the catalog-tighten matrix.
 
 Each scenario applies one mutation after install and before the tool runs:
 seeding a stale orphan, or removing a file or directory that catalog-tighten
 depends on, to exercise its graceful behaviour (tighten anyway, MISS, UNDCL,
 or fail cleanly).
 */

import {
  mkdir,
  rm,
  writeFile,
} from 'node:fs/promises';
import {
  join,
} from 'node:path';

import {
  CONSUMER_DIRS,
  FIXTURE_ORPHAN,
  FIXTURE_PACKAGE,
  FIXTURE_STORE_DIR,
  type Scenario,
} from './combos.ts';
import {
  PNPM_BIN_DIR,
  WORK_DIR,
} from './container-paths.ts';

//region Mutations

/**
 Indentation passed to `JSON.stringify` for the seeded orphan manifest.
 */
const JSON_INDENT = 2;

/**
 Removes a path, ignoring absence.
 
 @param path - absolute path to remove
 
 @example
 ```ts
 await removePath('/work/pnpm-lock.yaml');
 ```
 */
async function removePath(path: string,): Promise<void> {
  await rm(
    path,
    {
      recursive: true,
      force: true,
    },
  );
}

/**
 Seeds {@link FIXTURE_PACKAGE} at the higher {@link FIXTURE_ORPHAN} version
 into the virtual store, with no symlink pointing at it, reproducing the
 post-downgrade leftover from `doc/troubleshooting/pnpm-modules-cache.md`.
 The resolver must ignore it.
 
 @example
 ```ts
 await seedStaleOrphan();
 ```
 */
async function seedStaleOrphan(): Promise<void> {
  /**
   Virtual-store package directory for the orphan version, mirroring pnpm's `.pnpm` layout.
   */
  const orphanDir = join(
    WORK_DIR,
    'node_modules',
    '.pnpm',
    `${FIXTURE_PACKAGE}@${FIXTURE_ORPHAN}`,
    'node_modules',
    FIXTURE_PACKAGE,
  );
  await mkdir(
    orphanDir,
    { recursive: true, },
  );
  await writeFile(
    join(
      orphanDir,
      'package.json',
    ),
    `${
      JSON.stringify(
        {
          name: FIXTURE_PACKAGE,
          version: FIXTURE_ORPHAN,
        },
        undefined,
        JSON_INDENT,
      )
    }\n`,
  );
}

/**
 Removes every `node_modules` in the fixture: the root and both
 {@link CONSUMER_DIRS}.
 
 @example
 ```ts
 await removeAllModules();
 ```
 */
async function removeAllModules(): Promise<void> {
  await removePath(join(
    WORK_DIR,
    'node_modules',
  ),);
  await Promise.all(CONSUMER_DIRS.map(async function removeConsumerModules(dir,): Promise<void> {
    await removePath(join(
      WORK_DIR,
      dir,
      'node_modules',
    ),);
  },),);
}

/**
 Turns the catalog package into a store-only orphan: rewrites both
 {@link CONSUMER_DIRS} manifests to drop the {@link FIXTURE_PACKAGE}
 dependency and removes their `node_modules`, while leaving the root
 `node_modules` (and its `.pnpm` virtual store) intact. The package is then
 present in the store but declared by no importer and reachable through no
 symlink, which is the layout catalog-tighten classifies as `UNDCL` (rather
 than `MISS`, which also requires the store copy to be absent, or a plain
 unresolved miss, which keeps the declaration).
 
 @example
 ```ts
 await orphanStoreCopy();
 ```
 */
async function orphanStoreCopy(): Promise<void> {
  await Promise.all(CONSUMER_DIRS.map(async function orphanOneConsumer(dir,): Promise<void> {
    /**
     Absolute consumer directory whose manifest is undeclared and `node_modules` removed.
     */
    const consumerDir = join(
      WORK_DIR,
      dir,
    );
    await writeFile(
      join(
        consumerDir,
        'package.json',
      ),
      `${
        JSON.stringify(
          {
            name: dir,
            private: true,
            version: '0.0.0',
            dependencies: {},
          },
          undefined,
          JSON_INDENT,
        )
      }\n`,
    );
    await removePath(join(
      consumerDir,
      'node_modules',
    ),);
  },),);
}

/**
 Applies the scenario's post-install mutation before the tool runs. An if/else
 chain maps each mutation to its action (lint rule no-restricted-syntax/no-switch),
 delegating to {@link seedStaleOrphan} and {@link removeAllModules} for the
 multi-step cases.
 
 @param scenario - scenario whose mutation to apply
 
 @example
 ```ts
 await applyMutation(SCENARIOS[0]);
 ```
 */
export async function applyMutation(scenario: Scenario,): Promise<void> {
  /**
   Mutation to apply.
   */
  const { mutation, } = scenario;
  if (mutation === 'none')
    return;
  if (mutation === 'stale-orphan') {
    await seedStaleOrphan();
    return;
  }
  if (mutation === 'remove-all-modules') {
    await removeAllModules();
    return;
  }
  if (mutation === 'remove-lockfile') {
    await removePath(join(
      WORK_DIR,
      'pnpm-lock.yaml',
    ),);
    return;
  }
  if (mutation === 'remove-workspace-yaml') {
    await removePath(join(
      WORK_DIR,
      'pnpm-workspace.yaml',
    ),);
    return;
  }
  if (mutation === 'remove-some-modules') {
    await removePath(join(
      WORK_DIR,
      CONSUMER_DIRS[0],
      'node_modules',
    ),);
    return;
  }
  if (mutation === 'orphan-store-copy') {
    await orphanStoreCopy();
    return;
  }
  if (mutation === 'remove-virtual-store') {
    await removePath(join(
      WORK_DIR,
      'node_modules',
      '.pnpm',
    ),);
    return;
  }
  if (mutation === 'remove-store') {
    await removePath(FIXTURE_STORE_DIR,);
    return;
  }
  if (mutation === 'remove-pnp-cjs') {
    await removePath(join(
      WORK_DIR,
      '.pnp.cjs',
    ),);
    return;
  }
  if (mutation === 'remove-pnpm') {
    await removePath(PNPM_BIN_DIR,);
    return;
  }
  throw new Error('unhandled mutation',);
}

//endregion Mutations
