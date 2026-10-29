/**
 Tests for the parser that reads attribution out of settled artifacts.

 These exist because the report's own tests hand `sliceCritics` in by hand, so
 they exercise the FOLD and never the WIRING. The eligible-versus-ineligible
 decision the whole report rests on is not made there at all: it is made in
 `attributionEntryOf`, by OMITTING the key for an artifact that carries no
 attribution.

 The distinction these guard is ABSENT versus MALFORMED. Only an absent key
 means the entry predates attribution. A key that is present but corrupt must
 fail loudly, because letting it fall through to the same omission would move
 a broken artifact into the pre-feature population on the strength of its own
 breakage, and the population is what every number divides by.

 Fixtures are cat-themed invention.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  attributionEntryOf,
  gatherAttributionEntries,
} from '../../dist/final/node/index.mjs';
import {
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { scratchDirWith, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  NO_POOL_POLICY,
  UNASKED_REPOSITORY,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts/corpus-run/pool-policy.test-fixture.ts';

/**
 Pipeline commit every fixture artifact carries unless its case sets one.

 Invented, and shared, so the fixture directory is one generation and the
 generation guard passes without these parsing cases having to think about it.
 */
const SHARED_TIP = 'f000000000000000000000000000000000000000';

/**
 Built pipeline every fixture artifact carries unless its case sets one.

 Shared for the same reason as {@link SHARED_TIP}, and separate from it
 because this is the field the pool actually partitions by: a commit says
 where code came from, this says which build ran.
 */
const SHARED_GENERATION = `sha256-tree-v1:${'f'.repeat(64,)}`;

/**
 Critic used throughout.
 */
const TABBY = SEAT_SYNTHETIC_TEXT_EVERYWHERE;

/**
 Second critic, used only where a proposer must be a DIFFERENT model from
 whichever one a chunk heard, so a check reading `heardCriticIds` has
 something to actually refuse instead of trivially agreeing with itself.
 */
const QUIET = SEAT_SYNTHETIC_VISION_NO_OPENROUTER;

/**
 Claim the fixtures attribute.
 */
const NAP = 'issue/nap';

/**
 Writes artifacts into a fresh throwaway directory that removes itself.

 @param artifacts - file name to artifact body

 @returns Directory holding them, disposable

 @example
 ```ts
 await using scratch = await writeArtifacts({ artifacts: { 'a.json': {}, }, },);
 ```
 */
async function writeArtifacts(
  {
    artifacts,
  }: {
    readonly artifacts: Record<string, unknown>;
  },
): Promise<{ readonly dir: string; } & AsyncDisposable> {
  // Throwaway directory, never a real runs directory.
  return await scratchDirWith({
    prefix: 'attribution-read-',
    setup: async function seeded({ path: dir, },): Promise<{ readonly dir: string; }> {
      await Promise.all(Object
        .entries(artifacts,)
        .map(async function writeOne([name, body,],) {
        /**
         Body as written, with a pipeline commit supplied when the case did not
         name one.

         Every settled artifact carries `tip` and `pipelineDigest`, and the
         readers now refuse a pool they cannot partition by `pipelineDigest`. These
         cases are about PARSING rather than about generations, so they get one
         shared pair and stay a single-generation pool; a case that wants to
         exercise the generation guard sets its own. Deliberately not defaulted
         inside the reader: an artifact recording no pipeline is exactly what
         must not be quietly accepted.
         */
          const written = (((typeof body) === 'object') && (body !== null))
            ? {
              tip: SHARED_TIP,
              pipelineDigest: SHARED_GENERATION,
              ...body,
            }
            : body;

          await writeFile(
            join(
              dir,
              name,
            ),
            ((typeof written) === 'string') ? written : JSON.stringify(written,),
            'utf8',
          );
        },),);

      return { dir, };
    },
  },);
}

/**
 Builds an artifact carrying attribution and one accepted issue.

 Deliberately NOT empty. Fixtures whose `claimAttributions` and `issues` are
 both empty are satisfied by parsers that always return nothing, so they
 constrain neither the proposer path nor the issue path.

 @param sliceCritics - calibration to record

 @returns Artifact body

 @example
 ```ts
 const body = artifactWith({ sliceCritics, },);
 ```
 */
function artifactWith(
  {
    sliceCritics,
  }: {
    readonly sliceCritics: unknown;
  },
): Record<string, unknown> {
  return {
    // GENERATION 1, stated, because that is the generation that kept these
    // records at the artifact root and spelled the key `chunkCritics`, and each
    // record's index `chunkIndex`, which is how every caller here spells the
    // records it hands in. An unversioned body would read the same way and say
    // less.
    artifactSchemaVersion: 1,
    id: 'Whiskers',
    chunkCritics: sliceCritics,
    issues: [
      {
        chunkIndex: 0,
        issue: {
          status: 'accepted',
          claims: [{ claimId: NAP, },],
        },
      },
    ],
  };
}

/**
 Gathers a directory and keys its refusals by the file each one names.

 @param artifactsDir - directory the case wrote its artifacts into

 @returns Refusal reason per file, absent for a file the gather accepted, so
 a shape a check let through reads as a missing key

 @example
 ```ts
 const reasons = await refusalsByFile({ artifactsDir: scratch.dir, },);
 ```
 */
async function refusalsByFile(
  {
    artifactsDir,
  }: {
    readonly artifactsDir: string;
  },
): Promise<ReadonlyMap<string, string>> {
  /**
   What the directory yielded.
   */
  const { malformed, } = await gatherAttributionEntries({ artifactsDir, policy: NO_POOL_POLICY, repository: UNASKED_REPOSITORY, },);

  return new Map(malformed.map(function toNameAndReason({ name, reason, },): [string, string,] {
    return [
      name,
      reason,
    ];
  },),);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: gatherAttributionEntries.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'parses attribution and issues DOWN TO their contents, so a parser '
            + 'that returned empty proposers or empty claim ids would fail here '
            + 'rather than passing on fixtures that carry neither',
          fn: async () => {
            /**
             Artifact with one attributed claim and one accepted issue naming it.
             */
            await using scratch = await writeArtifacts({
              artifacts: {
                'Whiskers.json': artifactWith({
                  sliceCritics: [{
                    chunkIndex: 3,
                    heardCriticIds: [TABBY,],
                    claimAttributions: [{
                      claimId: NAP,
                      proposers: [{ modelId: TABBY, emissionCount: 2, },],
                    },],
                  },],
                },),
              },
            },);

            /**
             Entries as the CLI would gather them.
             */
            const { entries, } = await gatherAttributionEntries({ artifactsDir: scratch.dir, policy: NO_POOL_POLICY, repository: UNASKED_REPOSITORY, },);

            /**
             Chunk record the artifact carried.
             */
            const record = entries[0]?.sliceCritics?.[0];

            expect(record?.sliceIndex,).toBe(3,);
            expect(record?.heardCriticIds,).toStrictEqual([TABBY,],);
            expect(record?.claimAttributions[0]?.claimId,).toBe(NAP,);
            expect(record?.claimAttributions[0]?.proposers,)
              .toStrictEqual([{ modelId: TABBY, emissionCount: 2, },],);
            expect(entries[0]?.issues,)
              .toStrictEqual([{ status: 'accepted', claimIds: [NAP,], },],);
          },
        },),

        it({
          name: 'treats an ABSENT sliceCritics key as an entry that predates '
            + 'attribution, which is the decision the whole report rests on and the '
            + 'one its own tests cannot reach, since they supply sliceCritics by '
            + 'hand and so make every entry eligible by construction',
          fn: async () => {
            /**
             Artifact written before attribution existed.
             */
            await using scratch = await writeArtifacts({
              artifacts: { 'Mittens.json': { id: 'Mittens', issues: [], }, },
            },);

            /**
             Entries as the CLI would gather them.
             */
            const { entries, } = await gatherAttributionEntries({ artifactsDir: scratch.dir, policy: NO_POOL_POLICY, repository: UNASKED_REPOSITORY, },);

            // Undefined, NOT an empty array. An empty array would read as an entry
            // whose critics raised nothing, which is the exact conflation the
            // eligible population exists to prevent.
            expect(entries[0]?.sliceCritics,).toBeUndefined();
          },
        },),

        it({
          name: 'REFUSES, NAMING issues[0].issue, an issue record whose `issue` field is an array, '
            + 'superseding the earlier reading that let it CONTRIBUTE NOTHING and produce the spurious '
            + 'accepted-but-silent view {status: "", claimIds: []}: `status` and `claims` are written by '
            + 'every generation, so a present-but-malformed `issue` is corruption rather than an entry '
            + 'this reader may quietly read as empty (ledger B92, B106)',
          fn: async () => {
            /**
             One sound artifact beside one whose issue record names an array
             rather than a record under `issue`. VERSION 1, root-level `issues`,
             the shape the "parses attribution and issues DOWN TO their contents"
             case already proves survives the pool and reaches `attributionEntryOf`, so
             this entry's absence from `entries` cannot be mistaken for the refusal
             under test.
             */
            await using scratch = await writeArtifacts({
              artifacts: {
                'Whiskers.json': artifactWith({
                  sliceCritics: [{
                    chunkIndex: 0,
                    heardCriticIds: [TABBY,],
                    claimAttributions: [],
                  },],
                },),
                'Mittens.json': {
                  artifactSchemaVersion: 1,
                  id: 'Mittens',
                  issues: [{ chunkIndex: 0, issue: ['stray',], },],
                },
              },
            },);

            /**
             What the directory yielded.
             */
            const { entries, malformed, } = await gatherAttributionEntries({ artifactsDir: scratch.dir, policy: NO_POOL_POLICY, repository: UNASKED_REPOSITORY, },);

            // The sound sibling still produces its own entry, so Mittens' absence
            // from `entries` is the refusal under test and not a pool-wide failure.
            expect(entries,).toHaveLength(1,);
            expect(entries[0]?.id,).toBe('Whiskers',);

            expect(malformed,).toHaveLength(1,);
            expect(malformed[0]?.name,).toBe('Mittens.json',);
            expect(malformed[0]?.reason,).toContain('issues[0].issue',);
          },
        },),

        it({
          name: 'NAMES THE FIELD rather than reading a fallback at every level `attributionEntryOf` and '
            + '`recordsHolderOf` themselves can break, mirroring the `decodeSliceCritics` table that '
            + 'pins the same discipline inside the decoder. Every field here is written by every '
            + 'generation that has ever settled an artifact, so for each of them BOTH absence and '
            + 'malformation are corruption: `issues` missing is not a legacy entry, since `issues` '
            + 'predates attribution itself, and a `lanes` key appearing on a generation that keeps its '
            + 'records at the root is not a two-lane artifact read early',
          fn: async () => {
            /**
             One artifact per shape `attributionEntryOf` or `recordsHolderOf` must refuse,
             each paired with a substring its refusal has to carry. Carrying BOTH
             the path and the reason, as the `decodeSliceCritics` table does, since a path
             alone is carried by every refusal raised on that path, so it cannot tell
             which check refused.
             */
            const broken: readonly {
              readonly id: string;
              readonly expects: string;
              readonly artifact: Record<string, unknown>;
            }[] = [
              {
                id: 'IssuesAbsent',
                expects: '.issues: expected an array',
                artifact: { artifactSchemaVersion: 1, },
              },
              {
                id: 'IssuesNotArray',
                expects: '.issues: expected an array',
                artifact: { artifactSchemaVersion: 1, issues: 'not an array', },
              },
              {
                id: 'IssueRecordNotRecord',
                expects: 'issues[0]: expected a record',
                artifact: { artifactSchemaVersion: 1, issues: ['stray',], },
              },
              {
                id: 'StatusAbsent',
                expects: 'issues[0].issue.status: expected a string',
                artifact: {
                  artifactSchemaVersion: 1,
                  issues: [{ chunkIndex: 0, issue: { claims: [{ claimId: NAP, },], }, },],
                },
              },
              {
                id: 'StatusNotString',
                expects: 'issues[0].issue.status: expected a string',
                artifact: {
                  artifactSchemaVersion: 1,
                  issues: [{
                    chunkIndex: 0,
                    issue: { status: 7, claims: [{ claimId: NAP, },], },
                  },],
                },
              },
              {
                id: 'ClaimsAbsent',
                expects: 'issues[0].issue.claims: expected an array',
                artifact: {
                  artifactSchemaVersion: 1,
                  issues: [{ chunkIndex: 0, issue: { status: 'accepted', }, },],
                },
              },
              {
                id: 'ClaimsNotArray',
                expects: 'issues[0].issue.claims: expected an array',
                artifact: {
                  artifactSchemaVersion: 1,
                  issues: [{ chunkIndex: 0, issue: { status: 'accepted', claims: 'not an array', }, },],
                },
              },
              {
                id: 'ClaimMemberNotRecord',
                expects: 'issues[0].issue.claims[0]: expected a record',
                artifact: {
                  artifactSchemaVersion: 1,
                  issues: [{ chunkIndex: 0, issue: { status: 'accepted', claims: ['stray',], }, },],
                },
              },
              {
                id: 'ClaimIdAbsent',
                expects: 'issues[0].issue.claims[0].claimId: expected a string',
                artifact: {
                  artifactSchemaVersion: 1,
                  issues: [{ chunkIndex: 0, issue: { status: 'accepted', claims: [{},], }, },],
                },
              },
              {
                id: 'ClaimIdNotString',
                expects: 'issues[0].issue.claims[0].claimId: expected a string',
                artifact: {
                  artifactSchemaVersion: 1,
                  issues: [{
                    chunkIndex: 0,
                    issue: { status: 'accepted', claims: [{ claimId: 7, },], },
                  },],
                },
              },
              {
                id: 'LanesPresentOnVersion1',
                expects: '.lanes: expected no lanes, since this generation keeps its records at the root',
                artifact: { artifactSchemaVersion: 1, lanes: { repair: { result: {}, }, }, },
              },
              {
                id: 'LanesPresentUnversioned',
                expects: '.lanes: expected no lanes, since this generation keeps its records at the root',
                artifact: { lanes: {}, },
              },
              {
                id: 'LanesNotRecord',
                expects: '.lanes: expected a record',
                artifact: { artifactSchemaVersion: 2, lanes: 'not a record', },
              },
              {
                id: 'RepairAbsent',
                expects: '.lanes.repair: expected a record',
                artifact: { artifactSchemaVersion: 2, lanes: {}, },
              },
              {
                id: 'ResultNotRecord',
                expects: '.lanes.repair.result: expected a record',
                artifact: { artifactSchemaVersion: 2, lanes: { repair: { result: 'not a record', }, }, },
              },
              {
                // A VERSION 2+ CRITIC RECORD WHOSE CLAIM NAMES AN UNHEARD PROPOSER.
                // QUIET proposes while only TABBY was recorded as heard on this
                // chunk, which `decodeSliceCritics` must refuse rather than credit
                // a critic the record itself says never answered.
                id: 'ProposerNotHeard',
                expects: '.modelId: expected a critic named in heardCriticIds',
                artifact: {
                  artifactSchemaVersion: 2,
                  lanes: {
                    repair: {
                      result: {
                        chunkCritics: [{
                          chunkIndex: 0,
                          heardCriticIds: [TABBY,],
                          claimAttributions: [{
                            claimId: NAP,
                            proposers: [{ modelId: QUIET, emissionCount: 1, },],
                          },],
                        },],
                      },
                    },
                  },
                },
              },
            ];

            /**
             All of them written into one directory beside a readable sibling, so
             a shape that was quietly accepted shows up as a missing row rather
             than as a passing case.
             */
            await using scratch = await writeArtifacts({
              artifacts: {
                'Readable.json': {
                  ...artifactWith({
                    sliceCritics: [{
                      chunkIndex: 0,
                      heardCriticIds: [TABBY,],
                      claimAttributions: [{
                        claimId: NAP,
                        proposers: [{ modelId: TABBY, emissionCount: 1, },],
                      },],
                    },],
                  },),
                  id: 'Readable',
                },
                ...Object.fromEntries(broken.map(function toFile(one,): [string, unknown,] {
                  return [
                    `${one.id}.json`,
                    { ...one.artifact, id: one.id, },
                  ];
                },),),
              },
            },);

            /**
             Why each file failed, keyed by the file that failed.
             */
            const reasons = await refusalsByFile({ artifactsDir: scratch.dir, },);

            expect(reasons.size,).toBe(broken.length,);
            for (const one of broken) {
              /**
               Why this file failed, absent where the check under test accepted a
               shape it was supposed to refuse.
               */
              const reason = reasons.get(`${one.id}.json`,);

              expect(String(reason,),).toContain(one.expects,);
            }
          },
        },),

        it({
          name: 'ISOLATES a failure to the artifact that caused it, so one corrupt '
            + 'or half-written file costs its own row and not the whole run. A pass '
            + 'killed at its hard cap can leave a truncated artifact, and a bare '
            + 'Promise.all over the directory would turn that single file into no '
            + 'calibration at all for every other entry',
          fn: async () => {
            /**
             One sound artifact beside one truncated mid-write.
             */
            await using scratch = await writeArtifacts({
              artifacts: {
                'Whiskers.json': artifactWith({
                  sliceCritics: [{
                    chunkIndex: 0,
                    heardCriticIds: [TABBY,],
                    claimAttributions: [],
                  },],
                },),
                'truncated.json': '{"id":"Mittens","chunkCri',
              },
            },);

            /**
             What the directory yielded.
             */
            const { entries, malformed, } = await gatherAttributionEntries({
              policy: NO_POOL_POLICY,
              repository: UNASKED_REPOSITORY,
              artifactsDir: scratch.dir,
            },);

            // The sound artifact still produces its entry.
            expect(entries,).toHaveLength(1,);
            expect(entries[0]?.id,).toBe('Whiskers',);

            // The broken one is NAMED rather than silently absent, since an
            // artifact missing from both populations changes every count built on them.
            expect(malformed,).toHaveLength(1,);
            expect(malformed[0]?.name,).toBe('truncated.json',);
            expect(malformed[0]?.reason.length,).toBeGreaterThan(0,);
          },
        },),

        it({
          name: 'THROWS on a critic key that is present but not an array, '
            + 'rather than reading it as an entry that predates attribution. Only '
            + 'absence means legacy; tolerating null or a string here would let '
            + 'corruption quietly shrink the eligible population instead. The '
            + 'refusal names the key AS THE FILE SPELLS IT, which on this '
            + 'generation 1 fixture is chunkCritics',
          fn: async () => {
            await Promise.all([null, {}, 'corrupt', 7,].map(async function rejectsIt(corrupt,) {
              /**
               Artifact whose attribution key is present and unusable.
               */
              await using scratch = await writeArtifacts({
                artifacts: { 'Whiskers.json': artifactWith({ sliceCritics: corrupt, },), },
              },);

              expect(
                (await gatherAttributionEntries({ artifactsDir: scratch.dir, policy: NO_POOL_POLICY, repository: UNASKED_REPOSITORY, },)).malformed[0]
                  ?.reason,
              ).toContain('chunkCritics',);
            },),);
          },
        },),

        it({
          name: 'THROWS on a chunk index that is missing, negative or fractional, '
            + 'rather than dropping the record. Dropping does not protect the chunk '
            + 'count, it silently shrinks it, and a smaller denominator raises every '
            + 'rate divided by it while looking entirely ordinary',
          fn: async () => {
            await Promise.all(['one', -1, 1.5, undefined,].map(async function rejectsIt(chunkIndex,) {
              /**
               Artifact carrying one unusable chunk index.
               */
              await using scratch = await writeArtifacts({
                artifacts: {
                  'Whiskers.json': artifactWith({
                    sliceCritics: [{
                      chunkIndex,
                      heardCriticIds: [TABBY,],
                      claimAttributions: [],
                    },],
                  },),
                },
              },);

              expect(
                (await gatherAttributionEntries({ artifactsDir: scratch.dir, policy: NO_POOL_POLICY, repository: UNASKED_REPOSITORY, },)).malformed[0]
                  ?.reason,
              ).toContain('chunkIndex',);
            },),);
          },
        },),

        it({
          name: 'THROWS on repeated heard critics, repeated chunk indices and '
            + 'repeated proposers, each of which is a SET written as an array and '
            + 'each of which would inflate a count silently: a critic heard twice '
            + 'on one chunk doubles its own denominator',
          fn: async () => {
            /**
             One chunk naming the same critic twice as having answered.
             */
            await using heard = await writeArtifacts({
              artifacts: {
                'Whiskers.json': artifactWith({
                  sliceCritics: [{
                    chunkIndex: 0,
                    heardCriticIds: [TABBY, TABBY,],
                    claimAttributions: [],
                  },],
                },),
              },
            },);
            expect(
              (await gatherAttributionEntries({ artifactsDir: heard.dir, policy: NO_POOL_POLICY, repository: UNASKED_REPOSITORY, },)).malformed[0]
                ?.reason,
            ).toContain('distinct',);

            /**
             Two records claiming to describe the same chunk.
             */
            await using chunks = await writeArtifacts({
              artifacts: {
                'Whiskers.json': artifactWith({
                  sliceCritics: [
                    { chunkIndex: 0, heardCriticIds: [TABBY,], claimAttributions: [], },
                    { chunkIndex: 0, heardCriticIds: [TABBY,], claimAttributions: [], },
                  ],
                },),
              },
            },);
            expect(
              (await gatherAttributionEntries({ artifactsDir: chunks.dir, policy: NO_POOL_POLICY, repository: UNASKED_REPOSITORY, },)).malformed[0]
                ?.reason,
            ).toContain('one record per chunk',);

            /**
             One claim crediting the same critic twice.
             */
            await using proposers = await writeArtifacts({
              artifacts: {
                'Whiskers.json': artifactWith({
                  sliceCritics: [{
                    chunkIndex: 0,
                    heardCriticIds: [TABBY,],
                    claimAttributions: [{
                      claimId: NAP,
                      proposers: [
                        { modelId: TABBY, emissionCount: 1, },
                        { modelId: TABBY, emissionCount: 1, },
                      ],
                    },],
                  },],
                },),
              },
            },);
            expect(
              (await gatherAttributionEntries({ artifactsDir: proposers.dir, policy: NO_POOL_POLICY, repository: UNASKED_REPOSITORY, },)).malformed[0]
                ?.reason,
            ).toContain('one entry per critic',);
          },
        },),

        it({
          name: 'THROWS on a chunk crediting the SAME CLAIM twice, which is the '
            + 'fourth set written as an array here and the one the case beside this '
            + 'one misses: the writer keys attributions by claim id, so a repeat '
            + 'counts one claim as two and lifts every per-claim rate on its own',
          fn: async () => {
            /**
             One chunk carrying the same claim id under two attributions.
             */
            await using claims = await writeArtifacts({
              artifacts: {
                'Whiskers.json': artifactWith({
                  sliceCritics: [{
                    chunkIndex: 0,
                    heardCriticIds: [TABBY,],
                    claimAttributions: [
                      { claimId: NAP, proposers: [{ modelId: TABBY, emissionCount: 1, },], },
                      { claimId: NAP, proposers: [{ modelId: TABBY, emissionCount: 1, },], },
                    ],
                  },],
                },),
              },
            },);
            expect(
              (await gatherAttributionEntries({ artifactsDir: claims.dir, policy: NO_POOL_POLICY, repository: UNASKED_REPOSITORY, },)).malformed[0]
                ?.reason,
            ).toContain('one entry per claim',);
          },
        },),

        it({
          name: 'NAMES THE FIELD rather than the file at every level a record can '
            + 'break, which is what makes a refusal actionable: a settled artifact '
            + 'runs to thousands of lines, and nine shapes that all reported the '
            + 'file would leave an operator to find the broken field by hand',
          fn: async () => {
            /**
             One artifact per shape a decoder refuses, each paired with the path
             AND the reason its refusal has to carry.

             BOTH HALVES, because a path alone is a prefix of the path the next
             check down would name: with the record checks removed, reading a
             field off a string yields `undefined`, the field's own check refuses that
             instead, and `proposers[0]` is satisfied by a refusal naming
             `proposers[0].modelId`. Three of these passed against three missing
             guards before the reason was pinned beside the path.

             `heardCriticIds` is decoded before `claimAttributions`, since a
             proposer's `modelId` is now checked against it, so the shapes aimed
             at `claimAttributions` carry a valid `heardCriticIds: [TABBY]`: a
             broken one would be refused first and the case would then pin a
             field it was not aiming at.
             */
            const broken: readonly {
              readonly id: string;
              readonly expects: string;
              readonly critics: unknown;
            }[] = [
              {
                id: 'HeardNotArray',
                expects: '.heardCriticIds: expected an array',
                critics: [{ chunkIndex: 0, heardCriticIds: TABBY, claimAttributions: [], },],
              },
              {
                id: 'HeardMemberNotString',
                expects: 'heardCriticIds[0]: expected a string',
                critics: [{ chunkIndex: 0, heardCriticIds: [7,], claimAttributions: [], },],
              },
              {
                id: 'RecordNotRecord',
                expects: 'chunkCritics[0]: expected a record',
                critics: ['a nap',],
              },
              {
                id: 'AttributionsNotArray',
                expects: '.claimAttributions: expected an array',
                critics: [{ chunkIndex: 0, heardCriticIds: [TABBY,], claimAttributions: NAP, },],
              },
              {
                id: 'AttributionNotRecord',
                expects: 'claimAttributions[0]: expected a record',
                critics: [{ chunkIndex: 0, heardCriticIds: [TABBY,], claimAttributions: [NAP,], },],
              },
              {
                id: 'ClaimIdNotString',
                expects: 'claimAttributions[0].claimId: expected a string',
                critics: [{
                  chunkIndex: 0,
                  heardCriticIds: [TABBY,],
                  claimAttributions: [{ claimId: 7, proposers: [{ modelId: TABBY, emissionCount: 1, },], },],
                },],
              },
              {
                id: 'ProposersNotArray',
                expects: '.proposers: expected an array',
                critics: [{
                  chunkIndex: 0,
                  heardCriticIds: [TABBY,],
                  claimAttributions: [{ claimId: NAP, proposers: TABBY, },],
                },],
              },
              {
                id: 'ProposerNotRecord',
                expects: 'proposers[0]: expected a record',
                critics: [{
                  chunkIndex: 0,
                  heardCriticIds: [TABBY,],
                  claimAttributions: [{ claimId: NAP, proposers: [TABBY,], },],
                },],
              },
              {
                id: 'ProposerModelIdNotString',
                expects: 'proposers[0].modelId: expected a string',
                critics: [{
                  chunkIndex: 0,
                  heardCriticIds: [TABBY,],
                  claimAttributions: [{ claimId: NAP, proposers: [{ modelId: 7, emissionCount: 1, },], },],
                },],
              },
            ];

            /**
             All nine written into one directory, so a single gather answers them
             together and a shape that was quietly accepted shows up as a missing
             row rather than as a passing case.

             Each artifact records the id its file is named for, because the pool
             treats a file whose recorded id is not its file name as unplaceable,
             and one readable artifact rides along because a pool with nothing
             placeable in it is refused before any shape is reported.
             */
            await using scratch = await writeArtifacts({
              artifacts: {
                'Readable.json': {
                  ...artifactWith({
                    sliceCritics: [{
                      chunkIndex: 0,
                      heardCriticIds: [TABBY,],
                      claimAttributions: [{
                        claimId: NAP,
                        proposers: [{ modelId: TABBY, emissionCount: 1, },],
                      },],
                    },],
                  },),
                  id: 'Readable',
                },
                ...Object.fromEntries(broken.map(function toFile(one,): [string, unknown,] {
                  return [
                    `${one.id}.json`,
                    {
                      ...artifactWith({ sliceCritics: one.critics, },),
                      id: one.id,
                    },
                  ];
                },),),
              },
            },);

            /**
             Why each file failed, keyed by the file that failed.
             */
            const reasons = await refusalsByFile({ artifactsDir: scratch.dir, },);

            expect(reasons.size,).toBe(broken.length,);
            for (const one of broken) {
              /**
               Why this file failed, absent where the decoders accepted a shape
               they were supposed to refuse.
               */
              const reason = reasons.get(`${one.id}.json`,);

              // STRINGIFIED so an accepted shape fails on the expected path being
              // absent, rather than throwing on an absent row, which reads as a
              // broken test instead of as the missing refusal it is.
              expect(String(reason,),).toContain(one.expects,);
            }
          },
        },),

        it({
          name: 'THROWS on an emission count below one, since a proposer that '
            + 'emitted a claim zero times did not propose it and crediting one '
            + 'would manufacture support from a critic that stayed silent',
          fn: async () => {
            /**
             Proposer credited with no emissions at all.
             */
            await using scratch = await writeArtifacts({
              artifacts: {
                'Whiskers.json': artifactWith({
                  sliceCritics: [{
                    chunkIndex: 0,
                    heardCriticIds: [TABBY,],
                    claimAttributions: [{
                      claimId: NAP,
                      proposers: [{ modelId: TABBY, emissionCount: 0, },],
                    },],
                  },],
                },),
              },
            },);

            expect(
              (await gatherAttributionEntries({ artifactsDir: scratch.dir, policy: NO_POOL_POLICY, repository: UNASKED_REPOSITORY, },)).malformed[0]
                ?.reason,
            ).toContain('emissionCount',);
          },
        },),
        it({
          name: 'READS THE REPAIR LANE, where version 2 keeps these records, rather than the artifact '
            + 'root where version 1 kept them. This is the case every other fixture here could not '
            + 'reach: all of them are version-1-shaped, so all of them passed while the reader was '
            + 'blind to every artifact the current pipeline writes. Measured over the settled '
            + 'population before this held, 0 of 47 artifacts carried sliceCritics at the root and 47 '
            + 'of 47 carried it in the lane, so every one was filed as PREDATING attribution, which is '
            + 'the one wrong answer here that reads like an ordinary census of an older corpus. '
            + 'DECOYS sit at the root, deliberately different from the lane records, so a reader still '
            + 'asking the root is caught rather than agreeing by coincidence',
          fn: async () => {
            /**
             Two-lane artifact of generation 3, the first to spell the critic
             record `sliceCritics`, which still spelled each record's index
             `chunkIndex`, with a decoy planted at the root where generation 1
             kept these records.
             */
            await using scratch = await writeArtifacts({
              artifacts: {
                'Whiskers.json': {
                  artifactSchemaVersion: 3,
                  id: 'Whiskers',
                  sliceCritics: [{
                    chunkIndex: 9,
                    heardCriticIds: [TABBY,],
                    claimAttributions: [{
                      claimId: 'decoy-must-not-be-read',
                      proposers: [{ modelId: TABBY, emissionCount: 7, },],
                    },],
                  },],
                  issues: [],
                  lanes: {
                    repair: {
                      result: {
                        sliceCritics: [{
                          chunkIndex: 3,
                          heardCriticIds: [TABBY,],
                          claimAttributions: [{
                            claimId: NAP,
                            proposers: [{ modelId: TABBY, emissionCount: 2, },],
                          },],
                        },],
                        issues: [
                          {
                            chunkIndex: 0,
                            issue: {
                              status: 'accepted',
                              claims: [{ claimId: NAP, },],
                            },
                          },
                        ],
                      },
                    },
                  },
                },
              },
            },);

            /**
             Entries as the CLI would gather them.
             */
            const { entries, } = await gatherAttributionEntries({ artifactsDir: scratch.dir, policy: NO_POOL_POLICY, repository: UNASKED_REPOSITORY, },);

            // ELIGIBLE, which is the half that decides the population. Reading the
            // root left this undefined and moved the entry into the pre-feature
            // bucket without anything reporting that it had happened.
            expect(entries[0]?.sliceCritics,).toBeDefined();

            /**
             Chunk record the repair lane carried.
             */
            const record = entries[0]?.sliceCritics?.[0];

            expect(record?.sliceIndex,).toBe(3,);
            expect(record?.claimAttributions[0]?.claimId,).toBe(NAP,);
            expect(entries[0]?.issues,)
              .toStrictEqual([{ status: 'accepted', claimIds: [NAP,], },],);
          },
        },),

        it({
          name: 'READS A GENERATION 2 LANE, which spells the critic record chunkCritics and each '
            + 'record\'s index chunkIndex, into the same entry a later generation yields. Without this '
            + 'the renames read as a clean cut and are not: stored generation 2 artifacts carry both '
            + 'older spellings, and a reader asking for the newer index refused every one of them as '
            + 'malformed (ledger B107)',
          fn: async () => {
            /**
             Two-lane artifact of the generation before the rename, spelled the
             way that generation spelled it.
             */
            await using scratch = await writeArtifacts({
              artifacts: {
                'Whiskers.json': {
                  artifactSchemaVersion: 2,
                  id: 'Whiskers',
                  issues: [],
                  lanes: {
                    repair: {
                      result: {
                        chunkCritics: [{
                          chunkIndex: 3,
                          heardCriticIds: [TABBY,],
                          claimAttributions: [{
                            claimId: NAP,
                            proposers: [{ modelId: TABBY, emissionCount: 2, },],
                          },],
                        },],
                        issues: [
                          {
                            chunkIndex: 0,
                            issue: {
                              status: 'accepted',
                              claims: [{ claimId: NAP, },],
                            },
                          },
                        ],
                      },
                    },
                  },
                },
              },
            },);

            /**
             Entries as the CLI would gather them.
             */
            const { entries, } = await gatherAttributionEntries({ artifactsDir: scratch.dir, policy: NO_POOL_POLICY, repository: UNASKED_REPOSITORY, },);

            expect(entries[0]?.sliceCritics,).toBeDefined();

            /**
             Chunk record the repair lane carried, under the older spelling.
             */
            const record = entries[0]?.sliceCritics?.[0];

            expect(record?.sliceIndex,).toBe(3,);
            expect(record?.claimAttributions[0]?.claimId,).toBe(NAP,);
            expect(entries[0]?.issues,)
              .toStrictEqual([{ status: 'accepted', claimIds: [NAP,], },],);
          },
        },),

        it({
          name: 'READS THE GENERATION THE PASS WRITES, which spells both the critic record and its index '
            + 'with slice, as the control for the two older spellings: one reader, three spellings, '
            + 'one entry shape',
          fn: async () => {
            /**
             Two-lane artifact of the newest generation, spelled as it writes.
             */
            await using scratch = await writeArtifacts({
              artifacts: {
                'Whiskers.json': {
                  artifactSchemaVersion: 14,
                  id: 'Whiskers',
                  lanes: {
                    repair: {
                      result: {
                        sliceCritics: [{
                          sliceIndex: 3,
                          heardCriticIds: [TABBY,],
                          claimAttributions: [{
                            claimId: NAP,
                            proposers: [{ modelId: TABBY, emissionCount: 2, },],
                          },],
                        },],
                        issues: [
                          {
                            sliceIndex: 0,
                            issue: {
                              status: 'accepted',
                              claims: [{ claimId: NAP, },],
                            },
                          },
                        ],
                      },
                    },
                  },
                },
              },
            },);

            /**
             Entries as the CLI would gather them.
             */
            const { entries, malformed, } = await gatherAttributionEntries({ artifactsDir: scratch.dir, policy: NO_POOL_POLICY, repository: UNASKED_REPOSITORY, },);

            expect(malformed,).toStrictEqual([],);
            expect(entries[0]?.sliceCritics?.[0]?.sliceIndex,).toBe(3,);
          },
        },),

        it({
          name: 'REFUSES A RECORD INDEX SPELLED AS ANOTHER GENERATION SPELLS IT, naming the key its own '
            + 'generation wrote: a generation 2 record carrying sliceIndex, or a generation 14 record '
            + 'carrying chunkIndex, was written by no writer this package has had, so reading either '
            + 'under the other name would credit a record its own generation never produced',
          fn: async () => {
            /**
             One artifact per generation, each carrying the other spelling's index.
             */
            await using scratch = await writeArtifacts({
              artifacts: {
                'Mittens.json': {
                  artifactSchemaVersion: 2,
                  id: 'Mittens',
                  issues: [],
                  lanes: {
                    repair: {
                      result: {
                        chunkCritics: [{
                          sliceIndex: 3,
                          heardCriticIds: [TABBY,],
                          claimAttributions: [],
                        },],
                        issues: [],
                      },
                    },
                  },
                },
                'Whiskers.json': {
                  artifactSchemaVersion: 14,
                  id: 'Whiskers',
                  lanes: {
                    repair: {
                      result: {
                        sliceCritics: [{
                          chunkIndex: 3,
                          heardCriticIds: [TABBY,],
                          claimAttributions: [],
                        },],
                        issues: [],
                      },
                    },
                  },
                },
              },
            },);

            /**
             Why each file failed, keyed by the file that failed.
             */
            const reasons = await refusalsByFile({ artifactsDir: scratch.dir, },);

            /**
             Refusal of the generation 2 record, absent where it was read.
             */
            const olderRefusal = String(reasons.get('Mittens.json',),);

            /**
             Refusal of the generation 14 record, absent where it was read.
             */
            const newerRefusal = String(reasons.get('Whiskers.json',),);

            expect(olderRefusal,).toContain('Mittens chunkCritics[0].chunkIndex: expected',);
            expect(newerRefusal,).toContain('Whiskers sliceCritics[0].sliceIndex: expected',);
          },
        },),
      ],
    },),

    describe({
      name: attributionEntryOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS a sound artifact whose `id` is its file name into the entry the gather '
            + 'yields for it, so the refusals in this suite are measured against a reader that '
            + 'accepts the same body once it is whole',
          fn: async () => {
            /**
             Entry read straight from the body, under the file name it records.
             */
            const entry = attributionEntryOf({
              name: 'Whiskers.json',
              parsed: artifactWith({
                sliceCritics: [{
                  chunkIndex: 0,
                  heardCriticIds: [TABBY,],
                  claimAttributions: [{
                    claimId: NAP,
                    proposers: [{ modelId: TABBY, emissionCount: 1, },],
                  },],
                },],
              },),
            },);

            expect(entry.id,).toBe('Whiskers',);
            expect(entry.sliceCritics?.[0]?.claimAttributions[0]?.proposers,)
              .toStrictEqual([{ modelId: TABBY, emissionCount: 1, },],);
            expect(entry.issues,).toStrictEqual([{ status: 'accepted', claimIds: [NAP,], },],);
          },
        },),

        it({
          name: 'REFUSES, naming the file, a body that is not a record. Placement turns such a '
            + 'file away before the gather reads it, so this reader meets one only when the file '
            + 'changes between the two reads, and only a direct call can show the refusal',
          fn: async () => {
            for (const parsed of [null, ['stray',], 'Whiskers', 7,]) {
              /**
               Read of this body under a sound file name.
               */
              const read = () =>
                attributionEntryOf({
                  name: 'Whiskers.json',
                  parsed,
                },);

              expect(read,).toThrow(ArtifactParseError,);
              expect(read,).toThrow('Whiskers.json: expected a record.',);
            }
          },
        },),

        it({
          name: 'REFUSES, naming the file and its `id`, a body whose `id` is absent, not a string, '
            + 'empty, or another entry\'s. The pool admitted the artifact under its file name, so '
            + 'an entry read under any other identity would be counted under a name the pool never '
            + 'placed; a file named only `.json` keys no entry at all',
          fn: async () => {
            /**
             Body the reader accepts under `Whiskers.json`, which each row
             changes only in its `id`.
             */
            const sound = artifactWith({ sliceCritics: [], },);

            /**
             File name and body pairs the reader must refuse. The first leaves
             the `id` key out entirely rather than writing it as undefined.
             */
            const broken: readonly {
              readonly name: `${string}.json`;
              readonly body: Readonly<Record<string, unknown>>;
            }[] = [
              {
                name: 'Whiskers.json',
                body: Object.fromEntries(Object.entries(sound,).filter(function isNotId([key,],): boolean {
                  return key !== 'id';
                },),),
              },
              { name: 'Whiskers.json', body: { ...sound, id: 7, }, },
              { name: 'Whiskers.json', body: { ...sound, id: '', }, },
              { name: 'Whiskers.json', body: { ...sound, id: 'Mittens', }, },
              { name: 'Whiskers.json', body: { ...sound, id: 'Whiskers.json', }, },
              { name: '.json', body: { ...sound, id: '', }, },
            ];

            for (const one of broken) {
              /**
               Read of this row's body under this row's file name.
               */
              const read = () =>
                attributionEntryOf({
                  name: one.name,
                  parsed: one.body,
                },);

              expect(read,).toThrow(ArtifactParseError,);
              expect(read,).toThrow(
                `${one.name}.id: expected the entry id the file is named for, since the pool admitted the artifact under that name.`,
              );
            }
          },
        },),
      ],
    },),
  ],
},);
