/**
 Tests for the `editor-standing-read` command as built: which files it counts
 as artifacts, and the whole of what it prints and the code it exits with.

 RUN AS AN OPERATOR RUNS IT: every case writes a throwaway archive, runs the
 built command over it in a child, and reads both streams and the exit code.
 The child's environment carries no provider key and no setting of the
 package (`environmentWithoutKeys` of `child-environment.test-fixture.ts`,
 through `runBuiltCommand`), though the command asks no model. The procedure's
 own cases are in `editor-standing-read-run.unit.test.ts`.

 AN ARTIFACT IS A REGULAR FILE. The census and the scheduler list only regular
 files named `*.json`, and this reader took every such NAME, so a directory
 called `Tabby.json` was read as JSON and reported refused, and a symlink was
 followed to whatever it named and counted a second time (ledger B64). The
 command lists two layouts, an archive's own `artifacts/` directory and an
 archive that is itself the artifacts directory, so each layout has a case.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runBuiltCommand, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { BENCH_ENTRY, } from './built-entry.test-fixture.ts';
import { scratchDirWith, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  carrying,
  JUDGED_SEATED_ROUND,
  JUDGED_STANDING,
  NO_ROUNDS,
  OBSERVATIONAL,
  OFF_ROSTER_ROUND,
  refusedEmpty,
  SEATED_ROUND,
  standingRun,
  summaryOf,
  WITHOUT_ROUNDS,
} from './editor-standing-read.test-fixture.ts';

//region Editor standing read listing tests

/**
 Command under test.
 */
const STANDING_COMMAND = 'editor-standing-read';

/**
 What the command left on its two streams.
 */
type StandingStreams = {
  /**
   Exit code the command left behind, its own verdict on what it read.
   */
  readonly code: number;

  /**
   Everything the command wrote to stdout, where the summary line goes.
   */
  readonly stdout: string;

  /**
   Everything the command wrote to stderr, where refusals go.
   */
  readonly stderr: string;
};

/**
 Writes one throwaway archive holding a real artifact, a directory named like
 one, and a symlink named like one.

 @param nested - whether the artifacts sit in the archive's `artifacts/`
 directory, which is what a pass writes, rather than in the archive itself

 @returns Archive path, removed on dispose

 @example
 ```ts
 await using fixture = await throwawayArchive({ nested: true, },);
 ```
 */
async function throwawayArchive(
  { nested, }: { readonly nested: boolean; },
): Promise<AsyncDisposable & { readonly archive: string; }> {
  // Throwaway archive, never a real runs directory.
  return await scratchDirWith({
    prefix: 'editor-standing-read-',
    setup: async function seeded({ path: archive, },): Promise<{ readonly archive: string; }> {
      /**
       Directory the fixture artifacts go into.
       */
      const artifactsDir = nested
        ? join(
          archive,
          'artifacts',
        )
        : archive;

      await mkdir(
        artifactsDir,
        { recursive: true, },
      );
      await writeFile(
        join(
          artifactsDir,
          'Mittens.json',
        ),
        '{}',
        'utf8',
      );
      await mkdir(join(
        artifactsDir,
        'Tabby.json',
      ),);
      await symlink(
        'Mittens.json',
        join(
          artifactsDir,
          'Siamese.json',
        ),
      );

      return { archive, };
    },
  },);
}

/**
 Runs the command over one archive.

 A NON-ZERO EXIT IS THE COMMAND'S OWN VERDICT HERE: it exits 1 when nothing it
 read recorded a judged round, which this fixture never does. The shared fixture
 reports that exit code rather than rejecting on it.

 @param archive - archive the command reads

 @returns Both streams as the command left them

 @throws Error where the command never started, so no case reads two empty
 streams as a clean report

 @example
 ```ts
 const { stdout, } = await standingOver({ archive: fixture.archive, },);
 ```
 */
async function standingOver(
  { archive, }: { readonly archive: string; },
): Promise<StandingStreams> {
  /**
   Command as it finished, or why it never started.
   */
  const finished = await runBuiltCommand({
    command: STANDING_COMMAND,
    packageEntry: BENCH_ENTRY,
    args: [archive,],
  },);


  return {
    code: finished.code,
    stdout: finished.stdout,
    stderr: finished.stderr,
  };
}

/**
 Writes one throwaway run carrying the artifacts named, and runs the command
 over it.

 @param artifacts - artifact values by the file name each is written as, so
 a case lays out several digests in one run

 @returns Both streams as the command left them

 @throws Error where the command never started

 @example
 ```ts
 const { stdout, } = await standingOverArtifacts({ artifacts: { 'Mittens.json': artifact, }, },);
 ```
 */
async function standingOverArtifacts(
  { artifacts, }: { readonly artifacts: Readonly<Record<string, unknown>>; },
): Promise<StandingStreams> {
  /**
   The run directory, removed when this returns.
   */
  await using run = await standingRun({ artifacts, },);
  return await standingOver({ archive: run.dir, },);
}

/**
 Writes one throwaway run carrying exactly one artifact, and runs the command
 over it.

 @param artifact - artifact value to write as `Mittens.json`

 @returns Both streams as the command left them

 @throws Error where the command never started

 @example
 ```ts
 const { stdout, } = await standingOverArtifact({ artifact, },);
 ```
 */
async function standingOverArtifact(
  { artifact, }: { readonly artifact: unknown; },
): Promise<StandingStreams> {
  return await standingOverArtifacts({ artifacts: { 'Mittens.json': artifact, }, },);
}

await describe({
  name: `${STANDING_COMMAND} as built`,
  children: [
    it({
      name: 'COUNTS only the regular file in an archive\'s artifacts directory, skipping a directory and a '
        + 'symlink named like an artifact (ledger B64), names the one file it refused, and exits 1 with no round '
        + 'read',
      fn: async () => {
        await using fixture = await throwawayArchive({ nested: true, },);
        expect(await standingOver({ archive: fixture.archive, },),).toEqual({
          code: 1,
          stdout: `${summaryOf({ archives: 1, artifacts: 1, },)}${OBSERVATIONAL}${NO_ROUNDS}`,
          stderr: refusedEmpty({ path: join(fixture.archive, 'artifacts', 'Mittens.json',), },),
        },);
      },
    },),

    it({
      name: 'COUNTS only the regular file in an archive that is itself the artifacts directory (ledger B64), '
        + 'names the one file it refused, and exits 1 with no round read',
      fn: async () => {
        await using fixture = await throwawayArchive({ nested: false, },);
        expect(await standingOver({ archive: fixture.archive, },),).toEqual({
          code: 1,
          stdout: `${summaryOf({ archives: 1, artifacts: 1, },)}${OBSERVATIONAL}${NO_ROUNDS}`,
          stderr: refusedEmpty({ path: join(fixture.archive, 'Mittens.json',), },),
        },);
      },
    },),

    it({
      name: 'SAYS NOTHING WAS FOUND where the run directory will not read at all, naming why for both layouts, '
        + 'rather than reading an empty standing, and exits 1',
      fn: async () => {
        await using fixture = await scratchDirWith({
          prefix: 'editor-standing-read-',
          setup: async function empty({ path, },): Promise<{ readonly archive: string; }> {
            return { archive: join(path, 'never-written',), };
          },
        },);
        expect(await standingOver({ archive: fixture.archive, },),).toEqual({
          code: 1,
          stdout: `${summaryOf({ archives: 1, artifacts: 0, },)}${OBSERVATIONAL}${NO_ROUNDS}`,
          stderr: `editor-standing-read: no artifacts under ${fixture.archive} (ENOENT on artifacts/, ENOENT on the `
            + 'directory itself)\n',
        },);
      },
    },),

    it({
      name: 'READS EVERY ARCHIVE NAMED, counting two archives and the artifact of each, and refuses each '
        + 'unreadable file by its own path, in the order the archives were named',
      fn: async () => {
        await using first = await throwawayArchive({ nested: true, },);
        await using second = await throwawayArchive({ nested: false, },);

        /**
         What the command left over both archives at once.
         */
        const finished = await runBuiltCommand({
          command: STANDING_COMMAND,
          packageEntry: BENCH_ENTRY,
          args: [
            first.archive,
            second.archive,
          ],
        },);

        expect({
          code: finished.code,
          stdout: finished.stdout,
          stderr: finished.stderr,
        },).toEqual({
          code: 1,
          stdout: `${summaryOf({ archives: 2, artifacts: 2, },)}${OBSERVATIONAL}${NO_ROUNDS}`,
          stderr: refusedEmpty({ path: join(first.archive, 'artifacts', 'Mittens.json',), },)
            + refusedEmpty({ path: join(second.archive, 'Mittens.json',), },),
        },);
      },
    },),

    it({
      name: 'COUNTS an artifact whose chunks the schema predates as earlier-schema, one naming a '
        + 'model the roster dropped as off-roster, and one naming a seated model as read, rendering its '
        + 'standing over the round a ballot was cast on, and says no round was judged, at exit 1, where '
        + 'the seated model\'s only round drew no ballot, and beside a judged digest counts a digest whose only '
        + 'round drew no ballot, and that round, on the summary line rather than dropping both, and names one '
        + 'off-roster artifact with the verb in the singular',
      fn: async () => {
        /**
         Runs whose artifact records no chunks and one whose round names a
         model the roster dropped.
         */
        const earlier = await standingOverArtifact({ artifact: WITHOUT_ROUNDS, });
        expect(earlier.stdout.includes('earlierSchema=1',),).toBe(true,);
        const offRoster = await standingOverArtifact({ artifact: carrying({ rounds: [OFF_ROSTER_ROUND,], },), },);
        expect(offRoster.stderr.includes('not-on-roster',),).toBe(true,);
        expect(offRoster.stdout.includes('earlierRoster=1',),).toBe(true,);

        /**
         Opening lines every report over the one seated artifact prints.
         */
        const opening = 'editor-standing-read: archives=1 artifacts=1 read=1 earlierRoster=0 earlierSchema=0 ';

        /**
         The artifact whose one round a ballot was cast on.
         */
        const judged = carrying({ rounds: [JUDGED_SEATED_ROUND,], },);

        /**
         The report of that artifact's digest, which every run carrying it
         prints.
         */
        const judgedReport = `sha256-tree-v1:${'0'.repeat(64,)} over 1 entry\n`
          + `  EDITOR : 1 judged round from 1 of 1 chunk\n${JUDGED_STANDING}`
          + '  REFINER: 0 judged rounds from 0 of 1 chunk\n';

        /**
         A round a ballot was cast on: the standing and its report render.
         */
        const judgedAlone = await standingOverArtifact({ artifact: judged, },);

        /**
         That artifact beside one built by another pipeline, whose one round
         drew no ballot: the second digest has no standing to print, and the
         summary counts it and its round rather than dropping both.
         */
        const judgedBesideUnjudged = await standingOverArtifacts({
          artifacts: {
            'Mittens.json': judged,
            'Tabby.json': {
              ...carrying({ rounds: [SEATED_ROUND,], },),
              id: 'Tabby',
              pipelineDigest: `sha256-tree-v1:${'1'.repeat(64,)}`,
            },
          },
        },);

        /**
         The same round with no ballot alone: no round was judged, and the
         report says so rather than rendering a standing over it.
         */
        const unjudgedAlone = await standingOverArtifact({ artifact: carrying({ rounds: [SEATED_ROUND,], },), },);

        // Every run's streams compared at once and side by side, so a
        // difference in one never hides what the others printed; the run whose
        // one artifact names a model the roster dropped names its absence with
        // the count of one in the singular.
        expect({
          codes: [
            judgedAlone.code,
            judgedBesideUnjudged.code,
            unjudgedAlone.code,
          ],
          stderrs: [
            judgedAlone.stderr,
            judgedBesideUnjudged.stderr,
            unjudgedAlone.stderr,
          ],
          judgedAlone: judgedAlone.stdout,
          judgedBesideUnjudged: judgedBesideUnjudged.stdout,
          unjudgedAlone: unjudgedAlone.stdout,
          offRoster: offRoster.stdout,
        },).toEqual({
          codes: [
            0,
            0,
            1,
          ],
          stderrs: [
            '',
            '',
            '',
          ],
          judgedAlone: `${opening}digestsJudged=1 digestsUnjudged=0 unjudgedDigestRounds=0\n`
            + `${OBSERVATIONAL}\n${judgedReport}`,
          judgedBesideUnjudged: 'editor-standing-read: archives=1 artifacts=2 read=2 earlierRoster=0 earlierSchema=0 '
            + `digestsJudged=1 digestsUnjudged=1 unjudgedDigestRounds=1\n${OBSERVATIONAL}\n${judgedReport}`,
          unjudgedAlone: `${opening}digestsJudged=0 digestsUnjudged=1 unjudgedDigestRounds=1\n${OBSERVATIONAL}`
            + '  NO JUDGED ROUNDS. 1 round was recorded here and drew no ballot: a slate of one candidate, which '
            + 'is what every producer proposing the same wording leaves, needs no vote, and a panel whose every '
            + 'judge abstained or failed casts none.\n',
          offRoster: 'editor-standing-read: archives=1 artifacts=1 read=0 earlierRoster=1 earlierSchema=0 '
            + `digestsJudged=0 digestsUnjudged=0 unjudgedDigestRounds=0\n${OBSERVATIONAL}`
            + '  NO ROUNDS UNDER THE CURRENT ROSTER. 1 of these artifacts names a model the roster no longer seats, '
            + 'so it was settled under an earlier one and is not evidence about the models seated now. This is an '
            + 'absent measurement, not a poor one.\n',
        },);
      },
    },),

    it({
      name: 'COUNTS BESIDE A SEAT\'S JUDGED ROUND THE ROUNDS OF THE SAME DIGEST THAT DREW NO BALLOT, on the seat\'s '
        + 'own line, one in the singular and two in the plural, while the standing reads the judged round alone',
      fn: async () => {
        /**
         The judged artifact beside one of the same pipeline whose chunk
         recorded the rounds given, none of them judged.

         @param unjudged - rounds the second artifact's one chunk records

         @returns Both streams as the command left them

         @example
         ```ts
         const run = await besideUnjudged({ unjudged: [SEATED_ROUND,], },);
         ```
         */
        async function besideUnjudged({ unjudged, }: { readonly unjudged: readonly unknown[]; },): Promise<StandingStreams> {
          return await standingOverArtifacts({
            artifacts: {
              'Mittens.json': carrying({ rounds: [JUDGED_SEATED_ROUND,], },),
              'Tabby.json': {
                ...carrying({ rounds: unjudged, },),
                id: 'Tabby',
              },
            },
          },);
        }

        /**
         What the command printed over one digest of two entries whose editor
         seat drew the count of unballoted rounds given.

         @param drewNoBallot - how the seat's line counts its unballoted rounds

         @returns Command's whole standard output

         @example
         ```ts
         const printed = reportOver({ drewNoBallot: '1 round', },);
         ```
         */
        function reportOver({ drewNoBallot, }: { readonly drewNoBallot: string; },): string {
          return 'editor-standing-read: archives=1 artifacts=2 read=2 earlierRoster=0 earlierSchema=0 '
            + `digestsJudged=1 digestsUnjudged=0 unjudgedDigestRounds=0\n${OBSERVATIONAL}\n`
            + `sha256-tree-v1:${'0'.repeat(64,)} over 2 entries\n`
            + `  EDITOR : 1 judged round from 1 of 2 chunks; ${drewNoBallot} drew no ballot\n${JUDGED_STANDING}`
            + '  REFINER: 0 judged rounds from 0 of 2 chunks\n';
        }

        /**
         One unballoted round beside the judged one, and two.
         */
        const [
          one,
          two,
        ] = await Promise.all([
          besideUnjudged({ unjudged: [SEATED_ROUND,], },),
          besideUnjudged({
            unjudged: [
              SEATED_ROUND,
              {
                ...SEATED_ROUND,
                envelopeId: 'env/2',
              },
            ],
          },),
        ],);

        expect({
          one,
          two,
        },).toEqual({
          one: {
            code: 0,
            stdout: reportOver({ drewNoBallot: '1 round', },),
            stderr: '',
          },
          two: {
            code: 0,
            stdout: reportOver({ drewNoBallot: '2 rounds', },),
            stderr: '',
          },
        },);
      },
    },),

    it({
      name: 'READS THE RUNS THE VARIABLE NAMES when the command line names no directory',
      fn: async () => {
        await using fixture = await scratchDirWith({
          prefix: 'editor-standing-read-',
          setup: async function seeded({ path, },): Promise<{ readonly archive: string; }> {
            await mkdir(join(path, 'artifacts',), { recursive: true, },);
            await writeFile(join(path, 'artifacts', 'Mittens.json',), '{}', 'utf8',);
            return { archive: path, };
          },
        },);
        const finished = await runBuiltCommand({
          command: STANDING_COMMAND,
          packageEntry: BENCH_ENTRY,
          env: { TRANSLATION_REPAIR_RUNS_DIR: fixture.archive, },
        },);
        // The summary names one artifact and the refusal names the fixture's own file, which a
        // read of the default runs directory could not.
        expect(finished.stdout.split('\n',).at(0,),).toBe(
          'editor-standing-read: archives=0 artifacts=1 read=0 earlierRoster=0 earlierSchema=0 digestsJudged=0 '
            + 'digestsUnjudged=0 unjudgedDigestRounds=0',
        );
        expect(finished.stderr,).toBe(
          `editor-standing-read: ${join(fixture.archive, 'artifacts', 'Mittens.json',)} refused, `
            + 'artifact parse failed at artifact.artifactSchemaVersion: expected a number.\n',
        );
      },
    },),
  ],
},);

//endregion Editor standing read listing tests
