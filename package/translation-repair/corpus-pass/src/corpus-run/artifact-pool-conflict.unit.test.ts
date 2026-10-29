/**
 Tests that the artifact pool reads its policy as the value an entry hands
 it, and REFUSES two pools asked for at once.

 WHAT THE TWO REQUESTS ARE. `TRANSLATION_REPAIR_REQUIRED_COMMIT` filters the
 pool to entries whose recorded pipeline contains a commit; setting
 `TRANSLATION_REPAIR_POOL_ALL` to its one accepted value takes every
 generation instead. Preferring either silently would record a policy nobody
 chose, and the report printed above the resulting number would name that
 policy as though it had been requested.

 THE ENTRY READS THE ENVIRONMENT, AND THE POOL READS A VALUE. `readPoolPolicy`
 is what an entry file calls with the environment it runs in, and
 `resolvePool` takes what it returned. Until 2026-10-10 `resolvePool` read
 the environment itself, so every library caller pooled under whatever the
 shell exported and these cases had to write the process's environment.

 WHAT WAS MEASURED. On 2026-08-25, inverting the comparison that reads the
 pool-all variable failed no test in this package. A reader that mistook
 PRESENCE for the accepted VALUE would refuse ordinary invocations and admit
 the contradictory one, which is why the reader is pinned on the accepted
 value, on another wording and on an exported-but-empty variable.

 NO NETWORK, and no shared state: no case writes the process's environment,
 and the refused pool names a directory nothing reads.

 THE REQUIRED COMMIT IS RESOLVED IN THE REPOSITORY THE POOL IS HANDED, and
 every recorded commit asked there whether it contains it. The case builds
 that repository, two invented commits in a throwaway directory
 (`makeNamingArchive`), and writes throwaway artifacts recording its head and
 the head's parent, so it reads no history of the checkout it runs in. Until
 2026-10-10 the pool took no repository and asked this checkout, so the case
 read this checkout's head and parent and failed on a checkout of one commit.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  readPoolPolicy,
  resolvePool,
} from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  digestPipeline,
  StatedRefusalError,
} from '@monochromatic-dev/translation-repair-glossary';
import { makeNamingArchive, } from '../archive-naming.test-fixture.ts';
import { builtDirectoryOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/built-package-directory.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import {
  CONFLICTING_POOL_SAYS,
  UNASKED_REPOSITORY,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts/corpus-run/pool-policy.test-fixture.ts';

//region Fixtures

/**
 Variable naming a commit the pooled entries must contain.
 */
const REQUIRED_COMMIT_VAR = 'TRANSLATION_REPAIR_REQUIRED_COMMIT';

/**
 Variable asking for every generation at once.
 */
const POOL_ALL_VAR = 'TRANSLATION_REPAIR_POOL_ALL';

//endregion Fixtures

await describe({
  name: 'artifact pool policy',
  concurrency: 1,
  children: [
    describe({
      name: readPoolPolicy.name,
      concurrency: 1,
      children: [
        it({
          name: 'READS NEITHER REQUEST from an environment that sets neither variable',
          fn: async () => {
            expect(readPoolPolicy({ env: { PURR_LEVEL: 'loud', }, },),).toEqual({
              requiredCommit: '',
              poolAll: false,
            },);
          },
        },),

        it({
          name: 'READS THE REQUIRED COMMIT as the invoker wrote it and the pool-all request by its one accepted '
            + 'value',
          fn: async () => {
            expect(readPoolPolicy({
              env: {
                [REQUIRED_COMMIT_VAR]: 'tabby-tip',
                [POOL_ALL_VAR]: 'yes',
              },
            },),).toEqual({
              requiredCommit: 'tabby-tip',
              poolAll: true,
            },);
          },
        },),

        it({
          name: 'READS A POOL-ALL VARIABLE EXPORTED WITH ANY OTHER WORDING AS NO REQUEST, since a reader noticing '
            + 'the NAME rather than the value it accepts would refuse ordinary invocations',
          fn: async () => {
            expect(readPoolPolicy({
              env: {
                [REQUIRED_COMMIT_VAR]: 'tabby-tip',
                [POOL_ALL_VAR]: 'no',
              },
            },),).toEqual({
              requiredCommit: 'tabby-tip',
              poolAll: false,
            },);
          },
        },),

        it({
          name: 'READS AN EXPORTED-BUT-EMPTY REQUIRED COMMIT AS NONE, an ordinary shell accident',
          fn: async () => {
            expect(readPoolPolicy({
              env: {
                [REQUIRED_COMMIT_VAR]: '',
                [POOL_ALL_VAR]: '',
              },
            },),).toEqual({
              requiredCommit: '',
              poolAll: false,
            },);
          },
        },),
      ],
    },),

    describe({
      name: resolvePool.name,
      concurrency: 1,
      children: [
        it({
          name: 'REFUSES AS STATED a filtered pool and an unfiltered one asked for together in the policy it is '
            + 'handed, rather than picking one and printing its name above a number nobody requested',
          fn: async () => {
            /**
             What the reader said about the pair.
             */
            const refusal = await rejectionOf(async function overBothPools() {
              await resolvePool({
                artifactsDir: join(
                  scratchRoot(),
                  'translation-repair-pool-conflict-unread',
                ),
                policy: {
                  requiredCommit: 'tabby-tip',
                  poolAll: true,
                },
                repository: UNASKED_REPOSITORY,
              },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(`StatedRefusalError: ${CONFLICTING_POOL_SAYS}`,);
          },
        },),

        it({
          name: 'RESOLVES A REQUIRED COMMIT NAMED BY A REVISION TO ITS FULL ID IN THE REPOSITORY IT IS HANDED, says so '
            + 'first, and pools the entries whose recorded commit contains it there, censusing the whole directory '
            + 'where the caller hands no listing; one named by its full id is pooled the same without that line',
          fn: async (ctx,) => {
            await using scratch = await scratchDir({ prefix: 'pool-required-', },);
            const artifactsDir = scratch.path;
            /**
             The repository the pool is handed: two invented commits in a
             throwaway directory, the second the head.
             */
            await using repository = await makeNamingArchive({},);
            /**
             The head's full id.
             */
            const head = repository.pin.commitSha;
            /**
             The head's parent's full id, which does not contain the head.
             */
            const parent = repository.parentCommit;
            await Promise.all([
              ['mittens', head,],
              ['whiskers', parent,],
            ].map(async function placed([entryId, tip,],): Promise<void> {
              await writeFile(
                join(artifactsDir, `${entryId ?? ''}.json`,),
                JSON.stringify({
                  id: entryId,
                  tip,
                  pipelineDigest: `sha256-tree-v1:${'0123456789abcdef'.repeat(4,)}`,
                },),
                'utf8',
              );
            },),);

            /**
             What one pool required the commit given and what it printed first.

             @param requiredCommit - commit as the invoker wrote it

             @returns Pool's selection and partition, and its first printed line

             @example
             ```ts
             const pooled = await pooledRequiring({ requiredCommit: 'HEAD', },);
             ```
             */
            async function pooledRequiring(
              { requiredCommit, }: { readonly requiredCommit: string; },
            ): Promise<Readonly<Record<string, unknown>>> {
              using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
              /**
               The pool resolved under the requirement.
               */
              const eligible = await resolvePool({
                artifactsDir,
                policy: {
                  requiredCommit,
                  poolAll: false,
                },
                repository: repository.pin.cloneDir,
              },);
              return {
                selection: eligible.selection,
                entryIds: eligible.entryIds,
                excludedIds: eligible.excludedIds,
                firstLine: printed.lines.at(0,),
              };
            }

            /**
             Pipeline the built pool reports reading by: the build it stands in, which is the artifact-pool package's.
             */
            const { digest, } = await digestPipeline({
              dir: builtDirectoryOf({
                packageEntry: import.meta.resolve('@monochromatic-dev/translation-repair-artifact-pool',),
              },),
            },);
            expect({
              byRevision: await pooledRequiring({ requiredCommit: 'HEAD', },),
              byFullId: await pooledRequiring({ requiredCommit: head, },),
            },).toEqual({
              byRevision: {
                selection: { kind: 'required-commit', commit: head, },
                entryIds: ['mittens',],
                excludedIds: ['whiskers',],
                firstLine: `POOL required commit HEAD resolves to ${head}`,
              },
              byFullId: {
                selection: { kind: 'required-commit', commit: head, },
                entryIds: ['mittens',],
                excludedIds: ['whiskers',],
                firstLine: `POOL read by pipeline ${digest}`,
              },
            },);
          },
        },),
      ],
    },),
  ],
},);
