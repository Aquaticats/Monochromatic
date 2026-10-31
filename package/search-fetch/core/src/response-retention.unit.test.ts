/**
 Unit tests for response temp directory retention and its write-path pruning.

 @module
 */

import {
  chmod,
  mkdir,
  mkdtemp,
  readdir,
  readFile,
  symlink,
  utimes,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { setTimeout as delay, } from 'node:timers/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  createJsonContent,
  pruneResponseTempDirs,
  RESPONSE_MAX_AGE_MILLISECONDS,
  RESPONSE_MAX_DIR_COUNT,
  RESPONSE_TEMP_DIR_PREFIX,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Age step between seeded response directories, one minute.
 */
const SEED_AGE_STEP_MILLISECONDS = 60_000;

/**
 Age just inside the 7 day retention window.
 */
const JUST_UNDER_MAX_AGE_MILLISECONDS = RESPONSE_MAX_AGE_MILLISECONDS - SEED_AGE_STEP_MILLISECONDS;

/**
 Age just outside the 7 day retention window.
 */
const JUST_OVER_MAX_AGE_MILLISECONDS = RESPONSE_MAX_AGE_MILLISECONDS + SEED_AGE_STEP_MILLISECONDS;

/**
 Age of the just-returned directory in the oldest-sorting count-cap case:
 6 days in milliseconds.
 */
const SIX_DAY_AGE_MILLISECONDS = 518_400_000;

/**
 Oversized response forcing the truncated full-response write.
 */
const OVERSIZED_RESPONSE = { markdown: 'x'.repeat(200_000,), };

/**
 Text written into foreign fixture entries.
 */
const FOREIGN_TEXT = 'foreign';

/**
 Bound on waiting for the delayed console warn delivery.
 */
const LOG_SETTLE_MILLISECONDS = 2_000;

//endregion Fixtures

//region Helpers

/**
 Create one disposable fixture directory under the shared scratch root.

 @param label - short fixture label used in the directory prefix

 @returns absolute path of the created fixture directory

 @example
 ```ts
 await makeFixture({ label: 'retention' });
 ```
 */
async function makeFixture({ label, }: { readonly label: string; }): Promise<string> {
  return mkdtemp(join(
    scratchRoot(),
    `search-fetch-retention-${label}-`,
  ),);
}

/**
 Create one disposable project fixture whose nearest `node_modules` resolves a
 scratch parent of its own.

 @param label - short fixture label used in the directory prefix

 @returns project directory and its resolved scratch parent

 @example
 ```ts
 await makeProject({ label: 'a' });
 ```
 */
async function makeProject({ label, }: { readonly label: string; }): Promise<{
  readonly projectDir: string;
  readonly parentDir: string;
}> {
  /**
   Project directory holding its own `node_modules` marker.
   */
  const projectDir = await makeFixture({ label, });
  await mkdir(join(projectDir, 'node_modules',), { recursive: true, },);
  return {
    projectDir,
    parentDir: join(
      projectDir,
      'node_modules',
      '.monochromatic',
      'search-fetch',
    ),
  };
}

/**
 Seed one owned response directory with a controlled modification age.

 @param parentDir - scratch parent receiving the seeded directory

 @param name - owned entry name carrying the owned prefix

 @param ageMs - age applied to the directory modification time

 @returns absolute path of the seeded directory

 @example
 ```ts
 await seedResponseDir({ parentDir, name: `${RESPONSE_TEMP_DIR_PREFIX}seed-a`, ageMs: 0 });
 ```
 */
async function seedResponseDir(
  {
    parentDir,
    name,
    ageMs,
  }: {
    readonly parentDir: string;
    readonly name: string;
    readonly ageMs: number;
  },
): Promise<string> {
  /**
   Absolute path of the seeded directory.
   */
  const dirPath = join(
    parentDir,
    name,
  );
  await mkdir(
    dirPath,
    { recursive: true, },
  );
  /**
   Modification time placing the directory at the requested age.
   */
  const when = new Date(Date.now() - ageMs);
  await utimes(
    dirPath,
    when,
    when,
  );
  return dirPath;
}

/**
 Seed the given number of owned response directories with distinct ages.

 @param parentDir - scratch parent receiving the seeded directories

 @param count - number of directories to seed

 @param ageOffsetMs - age of the oldest seeded directory

 @returns seeded directory paths ordered oldest first

 @example
 ```ts
 await seedAgedDirs({ parentDir, count: 64, ageOffsetMs: 64 * 60_000 });
 ```
 */
async function seedAgedDirs(
  {
    parentDir,
    count,
    ageOffsetMs,
  }: {
    readonly parentDir: string;
    readonly count: number;
    readonly ageOffsetMs: number;
  },
): Promise<string[]> {
  /**
   Seeding promises resolved in oldest-first order.
   */
  const seeding: Promise<string>[] = [];
  for (let index = 0; index < count; index += 1) {
    seeding.push(seedResponseDir({
      parentDir,
      name: `${RESPONSE_TEMP_DIR_PREFIX}seed-${String(index,)}`,
      ageMs: ageOffsetMs - (index * SEED_AGE_STEP_MILLISECONDS),
    },),);
  }
  return Promise.all(seeding,);
}

/**
 Write one foreign fixture file.

 @param path - absolute file path

 @returns the written path

 @example
 ```ts
 await seedFile({ path: join(parentDir, 'notes.txt') });
 ```
 */
async function seedFile({ path, }: { readonly path: string; }): Promise<string> {
  await writeFile(
    path,
    FOREIGN_TEXT,
    'utf8',
  );
  return path;
}

/**
 List owned response entry names in one scratch parent.

 @param parentDir - scratch parent to list

 @returns entry names carrying the owned prefix, sorted

 @example
 ```ts
 await ownedEntryNames({ parentDir });
 ```
 */
async function ownedEntryNames({ parentDir, }: { readonly parentDir: string; }): Promise<string[]> {
  return (await readdir(parentDir,))
    .filter(function isOwnedEntry(name,) {
      return name.startsWith(RESPONSE_TEMP_DIR_PREFIX,);
    },)
    .toSorted();
}

/**
 Read one entry name relative to its scratch parent.

 @param options - scratch parent and absolute entry path

 @returns entry name inside the scratch parent

 @example
 ```ts
 entryName({ parentDir, dirPath });
 ```
 */
function entryName(
  {
    parentDir,
    dirPath,
  }: {
    readonly parentDir: string;
    readonly dirPath: string;
  },
): string {
  return dirPath.slice(parentDir.length + 1,);
}

/**
 Read one seeded directory path by index, failing loudly when absent.

 @param seeded - seeded directory paths ordered oldest first

 @param index - zero-based index into the seeded paths

 @returns seeded directory path at the index

 @example
 ```ts
 seededAt({ seeded, index: 0 });
 ```
 */
function seededAt(
  {
    seeded,
    index,
  }: {
    readonly seeded: readonly string[];
    readonly index: number;
  },
): string {
  /**
   Seeded path at the index, expected present for every seeded fixture.
   */
  const seededPath = seeded[index];
  if (seededPath === undefined)
    throw new Error(`missing seeded directory at ${String(index,)}`,);
  return seededPath;
}

/**
 Console warn lines captured through a test stub, with a promise resolving at
 the first delivered line.
 */
type WarnRecorder = {
  /**
   Captured warn line text.
   */
  readonly lines: string[];
  /**
   Resolves when the first warn line is delivered.
   */
  readonly firstLine: Promise<void>;
  /**
   Stub fake recording one warn call.
   */
  readonly note: (text: unknown) => void;
};

/**
 Build a console warn recorder for a test's console stub.

 @returns recorder capturing warn text and signalling the first delivery

 @example
 ```ts
 const recorder = warnRecorder();
 ```
 */
function warnRecorder(): WarnRecorder {
  /**
   Captured warn line text.
   */
  const lines: string[] = [];
  /**
   First-delivery signal and its resolver.
   */
  const {
    promise: firstLine,
    resolve: noteFirstLine,
  } = Promise.withResolvers<void>();
  return {
    lines,
    firstLine,
    note(text: unknown) {
      lines.push(String(text,),);
      noteFirstLine();
    },
  };
}

/**
 Move the process working directory for one scoped test body.

 @param dir - project directory whose scratch parent a write resolves

 @returns disposable restoring the previous working directory

 @example
 ```ts
 using restored = scopedCwd({ dir: projectDir });
 ```
 */
function scopedCwd({ dir, }: { readonly dir: string; }): Disposable {
  /**
   Working directory restored when the scope exits.
   */
  const originalCwd = process.cwd();
  process.chdir(dir,);
  return {
    [Symbol.dispose]() {
      process.chdir(originalCwd,);
    },
  };
}

/**
 Make a fixture directory writable again when the scope exits.

 @param dirPath - fixture directory a test left non-writable

 @returns async disposable restoring the writable mode

 @example
 ```ts
 await using restored = restoringWritable({ dirPath: parentDir });
 ```
 */
function restoringWritable({ dirPath, }: { readonly dirPath: string; }): AsyncDisposable {
  return {
    async [Symbol.asyncDispose]() {
      await chmod(dirPath, 0o755,);
    },
  };
}

//endregion Helpers

await describe({
  name: '',
  // Every case mutates shared fixture directories, the process working
  // directory, or a console method, so the cases run one at a time.
  concurrency: 1,
  children: [
    describe({
      name: pruneResponseTempDirs.name,
      children: [
        //region Age policy

        it({
          name: 'keeps every response dir when the count stays under the cap',
          fn: async () => {
            /**
             Local value for parentDir.
             */
            const parentDir = await makeFixture({ label: 'age-under', },);
            /**
             Local value for seeded.
             */
            const seeded = await seedAgedDirs({
              parentDir,
              count: 3,
              ageOffsetMs: 2 * SEED_AGE_STEP_MILLISECONDS,
            },);

            await pruneResponseTempDirs({
              parentDir,
              keepDir: seededAt({
                seeded,
                index: 1,
              },),
            },);

            expect(await ownedEntryNames({ parentDir, }),).toHaveLength(3,);
          },
        },),
        it({
          name: 'deletes response dirs just over 7 days old and keeps those just under',
          fn: async () => {
            /**
             Local value for parentDir.
             */
            const parentDir = await makeFixture({ label: 'age-boundary', },);
            /**
             Local value for overAge.
             */
            const overAge = await seedResponseDir({
              parentDir,
              name: `${RESPONSE_TEMP_DIR_PREFIX}over-age`,
              ageMs: JUST_OVER_MAX_AGE_MILLISECONDS,
            },);
            /**
             Local value for underAge.
             */
            const underAge = await seedResponseDir({
              parentDir,
              name: `${RESPONSE_TEMP_DIR_PREFIX}under-age`,
              ageMs: JUST_UNDER_MAX_AGE_MILLISECONDS,
            },);
            /**
             Local value for keepDir.
             */
            const keepDir = await seedResponseDir({
              parentDir,
              name: `${RESPONSE_TEMP_DIR_PREFIX}just-written`,
              ageMs: 0,
            },);

            await pruneResponseTempDirs({
              parentDir,
              keepDir,
            },);

            /**
             Local value for remaining.
             */
            const remaining = await ownedEntryNames({ parentDir, });
            expect(remaining,).toHaveLength(2,);
            expect(remaining,).toContain(entryName({
              parentDir,
              dirPath: underAge,
            },),);
            expect(remaining,).toContain(entryName({
              parentDir,
              dirPath: keepDir,
            },),);
            expect(remaining,).not.toContain(entryName({
              parentDir,
              dirPath: overAge,
            },),);
          },
        },),

        //endregion Age policy

        //region Count policy

        it({
          name: 'keeps exactly the newest 64 response dirs counting the just-written one',
          fn: async () => {
            /**
             Local value for parentDir.
             */
            const parentDir = await makeFixture({ label: 'count-under', },);
            await seedAgedDirs({
              parentDir,
              count: RESPONSE_MAX_DIR_COUNT - 1,
              ageOffsetMs: (RESPONSE_MAX_DIR_COUNT - 1) * SEED_AGE_STEP_MILLISECONDS,
            },);
            /**
             Local value for keepDir.
             */
            const keepDir = await seedResponseDir({
              parentDir,
              name: `${RESPONSE_TEMP_DIR_PREFIX}just-written`,
              ageMs: 0,
            },);

            await pruneResponseTempDirs({
              parentDir,
              keepDir,
            },);

            expect(await ownedEntryNames({ parentDir, }),).toHaveLength(RESPONSE_MAX_DIR_COUNT,);
          },
        },),
        it({
          name: 'deletes everything beyond the newest 64 when 65 response dirs exist',
          fn: async () => {
            /**
             Local value for parentDir.
             */
            const parentDir = await makeFixture({ label: 'count-over', },);
            /**
             Local value for seeded.
             */
            const seeded = await seedAgedDirs({
              parentDir,
              count: RESPONSE_MAX_DIR_COUNT,
              ageOffsetMs: RESPONSE_MAX_DIR_COUNT * SEED_AGE_STEP_MILLISECONDS,
            },);
            /**
             Oldest seeded directory, trimmed first once the cap applies.
             */
            const oldestSeeded = seededAt({
              seeded,
              index: 0,
            },);
            /**
             Local value for keepDir.
             */
            const keepDir = await seedResponseDir({
              parentDir,
              name: `${RESPONSE_TEMP_DIR_PREFIX}just-written`,
              ageMs: 0,
            },);

            await pruneResponseTempDirs({
              parentDir,
              keepDir,
            },);

            /**
             Local value for remaining.
             */
            const remaining = await ownedEntryNames({ parentDir, });
            expect(remaining,).toHaveLength(RESPONSE_MAX_DIR_COUNT,);
            expect(remaining,).not.toContain(entryName({
              parentDir,
              dirPath: oldestSeeded,
            },),);
            expect(remaining,).toContain(entryName({
              parentDir,
              dirPath: keepDir,
            },),);
          },
        },),
        it({
          name: 'applies the age and count policies together',
          fn: async () => {
            /**
             Local value for parentDir.
             */
            const parentDir = await makeFixture({ label: 'combined', },);
            /**
             Seeding promises for over-age directories deleted by the age policy.
             */
            const overAgeSeeding: Promise<string>[] = [];
            for (let index = 0; index < 5; index += 1) {
              overAgeSeeding.push(seedResponseDir({
                parentDir,
                name: `${RESPONSE_TEMP_DIR_PREFIX}over-age-${String(index,)}`,
                ageMs: JUST_OVER_MAX_AGE_MILLISECONDS + (index * SEED_AGE_STEP_MILLISECONDS),
              },),);
            }
            /**
             Over-age directories deleted by the age policy.
             */
            const overAge = await Promise.all(overAgeSeeding,);
            /**
             Local value for seeded.
             */
            const seeded = await seedAgedDirs({
              parentDir,
              count: 70,
              ageOffsetMs: 70 * SEED_AGE_STEP_MILLISECONDS,
            },);
            /**
             Oldest in-window directory, trimmed by the count policy.
             */
            const oldestInWindow = seededAt({
              seeded,
              index: 0,
            },);
            /**
             Local value for keepDir.
             */
            const keepDir = await seedResponseDir({
              parentDir,
              name: `${RESPONSE_TEMP_DIR_PREFIX}just-written`,
              ageMs: 0,
            },);

            await pruneResponseTempDirs({
              parentDir,
              keepDir,
            },);

            /**
             Local value for remaining.
             */
            const remaining = await ownedEntryNames({ parentDir, });
            expect(remaining,).toHaveLength(RESPONSE_MAX_DIR_COUNT,);
            for (const overAgeDir of overAge) {
              expect(remaining,).not.toContain(entryName({
                parentDir,
                dirPath: overAgeDir,
              },),);
            }
            expect(remaining,).not.toContain(entryName({
              parentDir,
              dirPath: oldestInWindow,
            },),);
            expect(remaining,).toContain(entryName({
              parentDir,
              dirPath: keepDir,
            },),);
          },
        },),
        it({
          name: 'never deletes the directory just returned even when it sorts oldest',
          fn: async () => {
            /**
             Local value for parentDir.
             */
            const parentDir = await makeFixture({ label: 'keep-oldest', },);
            await seedAgedDirs({
              parentDir,
              count: RESPONSE_MAX_DIR_COUNT,
              ageOffsetMs: RESPONSE_MAX_DIR_COUNT * SEED_AGE_STEP_MILLISECONDS,
            },);
            /**
             Oldest entry overall, standing in for a just-returned directory
             whose timestamp sorts beyond the newest 64.
             */
            const keepDir = await seedResponseDir({
              parentDir,
              name: `${RESPONSE_TEMP_DIR_PREFIX}just-written`,
              ageMs: SIX_DAY_AGE_MILLISECONDS,
            },);

            await pruneResponseTempDirs({
              parentDir,
              keepDir,
            },);

            /**
             Local value for remaining.
             */
            const remaining = await ownedEntryNames({ parentDir, });
            expect(remaining,).toHaveLength(RESPONSE_MAX_DIR_COUNT,);
            expect(remaining,).toContain(entryName({
              parentDir,
              dirPath: keepDir,
            },),);
          },
        },),

        //endregion Count policy

        //region Scope and tolerance

        it({
          name: 'never touches entries outside the owned response prefix',
          fn: async () => {
            /**
             Local value for parentDir.
             */
            const parentDir = await makeFixture({ label: 'scope', },);
            /**
             Foreign directory without the owned prefix.
             */
            const foreignDir = join(parentDir, 'other-tool-dir',);
            await mkdir(foreignDir, { recursive: true, },);
            /**
             Near-miss directory named like the owned prefix without its dash.
             */
            const nearMissDir = join(parentDir, 'pi-search-fetch-response',);
            await mkdir(nearMissDir, { recursive: true, },);
            /**
             Foreign file without the owned prefix.
             */
            const foreignFile = await seedFile({ path: join(parentDir, 'notes.txt',), },);
            /**
             Prefix-matching file, not a response directory the policy manages.
             */
            const ownedPrefixFile = await seedFile({
              path: join(parentDir, `${RESPONSE_TEMP_DIR_PREFIX}stray.txt`,),
            },);
            await seedAgedDirs({
              parentDir,
              count: 70,
              ageOffsetMs: 70 * SEED_AGE_STEP_MILLISECONDS,
            },);
            /**
             Local value for keepDir.
             */
            const keepDir = await seedResponseDir({
              parentDir,
              name: `${RESPONSE_TEMP_DIR_PREFIX}just-written`,
              ageMs: 0,
            },);

            await pruneResponseTempDirs({
              parentDir,
              keepDir,
            },);

            /**
             Local value for remaining, carrying the 64 retained response dirs
             plus the prefix-matching file the policy does not manage.
             */
            const remaining = await ownedEntryNames({ parentDir, });
            expect(remaining,).toHaveLength(RESPONSE_MAX_DIR_COUNT + 1,);
            expect(remaining,).toContain('pi-search-fetch-response-stray.txt',);
            expect(remaining,).toContain(entryName({
              parentDir,
              dirPath: keepDir,
            },),);
            expect((await readdir(parentDir,)),).toContain('other-tool-dir',);
            expect((await readdir(parentDir,)),).toContain('pi-search-fetch-response',);
            expect((await readFile(
              foreignFile,
              'utf8',
            )),).toBe(FOREIGN_TEXT,);
            expect((await readFile(
              ownedPrefixFile,
              'utf8',
            )),).toBe(FOREIGN_TEXT,);
          },
        },),
        it({
          name: 'tolerates a response entry that cannot be statted',
          fn: async () => {
            /**
             Local value for parentDir.
             */
            const parentDir = await makeFixture({ label: 'dangling', },);
            /**
             Local value for seeded.
             */
            const seeded = await seedAgedDirs({
              parentDir,
              count: 2,
              ageOffsetMs: SEED_AGE_STEP_MILLISECONDS,
            },);
            /**
             Dangling symlink carrying the owned prefix, statted like a
             concurrent writer's entry that already vanished.
             */
            await symlink(
              join(parentDir, 'missing-target',),
              join(parentDir, `${RESPONSE_TEMP_DIR_PREFIX}dangling-`,),
            );

            await pruneResponseTempDirs({
              parentDir,
              keepDir: seededAt({
                seeded,
                index: 0,
              },),
            },);

            expect(await ownedEntryNames({ parentDir, }),).toHaveLength(3,);
          },
        },),
        it({
          name: 'never throws when the scratch parent cannot be listed',
          fn: async () => {
            /**
             Local value for parentDir.
             */
            const parentDir = await makeFixture({ label: 'missing-parent', },);

            await pruneResponseTempDirs({
              parentDir: join(parentDir, 'not-created',),
              keepDir: join(
                parentDir,
                'not-created',
                `${RESPONSE_TEMP_DIR_PREFIX}just-written`,
              ),
            },);

            expect((await readdir(parentDir,)),).toHaveLength(0,);
          },
        },),
        it({
          name: 'logs deletion failures without failing the call',
          fn: async ctx => {
            /**
             Local value for parentDir.
             */
            const parentDir = await makeFixture({ label: 'deletion-failure', },);
            /**
             Local value for seeded.
             */
            const seeded = await seedAgedDirs({
              parentDir,
              count: RESPONSE_MAX_DIR_COUNT + 1,
              ageOffsetMs: (RESPONSE_MAX_DIR_COUNT + 1) * SEED_AGE_STEP_MILLISECONDS,
            },);
            /**
             Oldest seeded directory, whose deletion fails below.
             */
            const doomed = seededAt({
              seeded,
              index: 0,
            },);
            /**
             Local value for keepDir.
             */
            const keepDir = await seedResponseDir({
              parentDir,
              name: `${RESPONSE_TEMP_DIR_PREFIX}just-written`,
              ageMs: 0,
            },);
            /**
             Local value for recorder.
             */
            const recorder = warnRecorder();
            ctx.sinon.stub(
              console,
              'warn',
            ).callsFake(recorder.note,);
            await chmod(parentDir, 0o555,);
            await using restoredWritable = restoringWritable({ dirPath: parentDir, });
            await pruneResponseTempDirs({
              parentDir,
              keepDir,
            },);
            await Promise.race([
              recorder.firstLine,
              delay(LOG_SETTLE_MILLISECONDS,),
            ],);

            expect(recorder
              .lines
              .some(function carriesDoomed(line,) {
                return line.includes(doomed,);
              },),).toBe(true,);
            expect(await ownedEntryNames({ parentDir, }),).toHaveLength(RESPONSE_MAX_DIR_COUNT + 2,);
          },
        },),
        it({
          name: 'tolerates concurrent pruning passes keeping both just-returned directories',
          fn: async () => {
            /**
             Local value for parentDir.
             */
            const parentDir = await makeFixture({ label: 'concurrent', },);
            await seedAgedDirs({
              parentDir,
              count: 70,
              ageOffsetMs: 70 * SEED_AGE_STEP_MILLISECONDS,
            },);
            /**
             Local value for keepA.
             */
            const keepA = await seedResponseDir({
              parentDir,
              name: `${RESPONSE_TEMP_DIR_PREFIX}writer-a`,
              ageMs: 0,
            },);
            /**
             Local value for keepB.
             */
            const keepB = await seedResponseDir({
              parentDir,
              name: `${RESPONSE_TEMP_DIR_PREFIX}writer-b`,
              ageMs: 0,
            },);

            await Promise.all([
              pruneResponseTempDirs({
                parentDir,
                keepDir: keepA,
              },),
              pruneResponseTempDirs({
                parentDir,
                keepDir: keepB,
              },),
            ],);

            /**
             Local value for remaining.
             */
            const remaining = await ownedEntryNames({ parentDir, });
            expect(remaining,).toHaveLength(RESPONSE_MAX_DIR_COUNT,);
            expect(remaining,).toContain(entryName({
              parentDir,
              dirPath: keepA,
            },),);
            expect(remaining,).toContain(entryName({
              parentDir,
              dirPath: keepB,
            },),);
          },
        },),

        //endregion Scope and tolerance
      ],
    },),
    describe({
      name: 'response temp retention after a full-response write',
      children: [
        it({
          name: 'prunes the scratch parent after a truncated write and keeps the file just returned',
          fn: async () => {
            /**
             Local value for project.
             */
            const project = await makeProject({ label: 'write-prune', },);
            await seedAgedDirs({
              parentDir: project.parentDir,
              count: 70,
              ageOffsetMs: 70 * SEED_AGE_STEP_MILLISECONDS,
            },);
            /**
             Previous working directory restored when the case ends.
             */
            using restoredCwd = scopedCwd({ dir: project.projectDir, });
            /**
             Full response result returned by the truncated write.
             */
            const written = await createJsonContent({ value: OVERSIZED_RESPONSE, },);
            if (written.fullJsonPath === undefined)
              throw new Error('expected a full response path',);

            expect((await readFile(
              written.fullJsonPath,
              'utf8',
            )),).toBe(OVERSIZED_RESPONSE.markdown,);
            /**
             Local value for remaining.
             */
            const remaining = await ownedEntryNames({ parentDir: project.parentDir, });
            expect(remaining,).toHaveLength(RESPONSE_MAX_DIR_COUNT,);
            expect(remaining,).toContain(entryName({
              parentDir: project.parentDir,
              dirPath: dirname(written.fullJsonPath,),
            },),);
          },
        },),
        it({
          name: 'caps each scratch parent separately from two different working directories',
          fn: async () => {
            /**
             Local value for projectA.
             */
            const projectA = await makeProject({ label: 'two-a', },);
            /**
             Local value for projectB.
             */
            const projectB = await makeProject({ label: 'two-b', },);
            await seedAgedDirs({
              parentDir: projectA.parentDir,
              count: 70,
              ageOffsetMs: 70 * SEED_AGE_STEP_MILLISECONDS,
            },);
            await seedAgedDirs({
              parentDir: projectB.parentDir,
              count: 70,
              ageOffsetMs: 70 * SEED_AGE_STEP_MILLISECONDS,
            },);
            /**
             Previous working directory restored when the case ends.
             */
            using restoredCwd = scopedCwd({ dir: projectA.projectDir, });
            /**
             Full response result from the first working directory.
             */
            const writeA = await createJsonContent({ value: OVERSIZED_RESPONSE, },);
            process.chdir(projectB.projectDir,);
            /**
             Full response result from the second working directory.
             */
            const writeB = await createJsonContent({ value: OVERSIZED_RESPONSE, },);
            if (writeA.fullJsonPath === undefined)
              throw new Error('expected a full response path from the first working directory',);
            if (writeB.fullJsonPath === undefined)
              throw new Error('expected a full response path from the second working directory',);

            /**
             Local value for remainingA.
             */
            const remainingA = await ownedEntryNames({ parentDir: projectA.parentDir, });
            /**
             Local value for remainingB.
             */
            const remainingB = await ownedEntryNames({ parentDir: projectB.parentDir, });
            expect(remainingA,).toHaveLength(RESPONSE_MAX_DIR_COUNT,);
            expect(remainingB,).toHaveLength(RESPONSE_MAX_DIR_COUNT,);
            expect(remainingA,).toContain(entryName({
              parentDir: projectA.parentDir,
              dirPath: dirname(writeA.fullJsonPath,),
            },),);
            expect(remainingB,).toContain(entryName({
              parentDir: projectB.parentDir,
              dirPath: dirname(writeB.fullJsonPath,),
            },),);
          },
        },),
      ],
    },),
  ],
},);
