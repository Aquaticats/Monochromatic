/**
 Tests for the two gatherers that rebuild prober inputs from a settled run.

 THEY ARE TESTED TOGETHER BECAUSE THEY ARE ONE INSTRUMENT. `gatherRelabelCases`
 builds the arm asking about regions a human read as damaged, and
 `gatherControlCases` builds the arm that says whether the first one means
 anything. Neither reading survives on its own, and both read the same
 artifact, the same manifest and the same corpus pages, so a fixture that
 serves one serves the other and a divergence between them shows up here
 rather than in a run.

 THE CONTROL'S ORDERING IS THE CASE WORTH THE FILE. `byLengthDistance` exists
 because taking whichever unflagged regions came first made the arm answer a
 different question: measured on the first control run, the regions that
 happened to come first replaced 12 to 63 characters while the damaged regions
 replaced 60 to 268, so a quiet control would have been partly a statement
 about how much text there was to damage. This file's fixture puts the SHORTEST
 unflagged region first in document order and requires it to be dropped, so an
 implementation that took document order would return it and fail.

 THE PINS ARE INJECTED, which is why any of this runs. Both functions read
 `RUN_CORPUS_PIN` directly until this landed, so exercising either needed the
 unlicensed corpus clone on disk. They now take the pin the way `censusEntry`
 does, and the cases point them at a throwaway git repository.

 FIXTURE CONTENT IS CAT-THEMED INVENTION mirroring corpus structure only:
 Simplified Chinese against English, one entry, committed once. The real
 inputs are memorial pages nobody licensed us to copy.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { devNull, } from 'node:os';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { spawnKeyless, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import {
  type ArtifactDeliveryRow,
  compareLanes,
} from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  gatherControlCases,
  gatherRelabelCases,
  type RelabelCase,
} from '../../dist/final/node/index.mjs';
import { fixtureGit, REAL_GIT, } from '@monochromatic-dev/translation-repair-adjudication/ts/hermetic-git-run.test-fixture.ts';
import { scratchDirWith, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

//region Probe relabel gathering tests

/**
 Entry every fixture here describes.
 */
const ENTRY_ID = 'whiskers';

/**
 Sheet position the round-three repair sheet marked damaged.

 TAKEN FROM `DAMAGED_CASES` RATHER THAN CHOSEN. `gatherRelabelCases` filters
 the manifest against that constant, so a position invented here would be
 filtered out and every case would pass against an empty result.
 */
const DAMAGED_POSITION = 2;

/**
 Sheet position the reader did NOT mark, so the gatherer must skip it.
 */
const UNDAMAGED_POSITION = 1;

/**
 Sheet position `DAMAGED_CASES` records as the same edit drawn again under a
 second accepted issue, which the gatherer merges into the first draw's case.
 */
const DRAWN_AGAIN_POSITION = 7;

/**
 Wording an edit replaced in the section the reader read as damaged.
 */
const DAMAGED_BEFORE = 'The cat sat on the warm windowsill mat.';

/**
 Unflagged wording FURTHEST in length from the damaged one, and first in the
 page, so document order and length order disagree.
 */
const FAR_BEFORE = 'She purred.';

/**
 Unflagged wording closest in length to the damaged one.
 */
const NEAR_BEFORE = 'The kitten watched from the top stair.';

/**
 Unflagged wording second-closest in length to the damaged one.
 */
const NEXT_BEFORE = 'The old tabby dozed beside the stove.';

/**
 Original page, four sections, in the Simplified Chinese the corpus uses.
 */
const SOURCE_PAGE = [
  '---',
  'name: 小猫-whiskers',
  '---',
  '',
  '## 简介',
  '',
  '猫坐在温暖的窗台垫子上。',
  '',
  '## 呼噜',
  '',
  '她打起了呼噜。',
  '',
  '## 楼梯',
  '',
  '小猫从最高一级台阶上看着。',
  '',
  '## 炉火',
  '',
  '老猫在炉子旁边打盹。',
  '',
].join('\n',);

/**
 Translation of that page, section for section, carrying every replaced
 wording exactly once so a lookup by text can only land in one slice.
 */
const TARGET_PAGE = [
  '---',
  'name: Whiskers',
  '---',
  '',
  '## Introduction',
  '',
  DAMAGED_BEFORE,
  '',
  '## Purring',
  '',
  FAR_BEFORE,
  '',
  '## The stairs',
  '',
  NEAR_BEFORE,
  '',
  '## The stove',
  '',
  NEXT_BEFORE,
  '',
].join('\n',);

/**
 One edit the repair lane recorded, named the way an artifact names one.
 */
type FixtureRegion = {
  /**
   Adjudicated issue this edit served.
   */
  readonly issueId: string;

  /**
   Region the edit replaced.
   */
  readonly envelopeId: string;

  /**
   Wording it replaced.
   */
  readonly before: string;
};

/**
 Every edit the fixture run recorded, in DOCUMENT ORDER.

 The order matters: {@link FAR_BEFORE} sits second, ahead of both closer
 regions, so a control that took whichever unflagged regions came first would
 return it.
 */
const REGIONS: readonly FixtureRegion[] = [
  {
    issueId: 'adjudicated/windowsill',
    envelopeId: 'envelope/windowsill',
    before: DAMAGED_BEFORE,
  },
  {
    issueId: 'adjudicated/purr',
    envelopeId: 'envelope/purr',
    before: FAR_BEFORE,
  },
  {
    issueId: 'adjudicated/stair',
    envelopeId: 'envelope/stair',
    before: NEAR_BEFORE,
  },
  {
    issueId: 'adjudicated/stove',
    envelopeId: 'envelope/stove',
    before: NEXT_BEFORE,
  },
];

/**
 Builds the ledger row the artifact carries for its one slice.

 @returns One-row ledger, enough to satisfy the settled parser

 @example
 ```ts
 const rows = ledger();
 ```
 */
function ledger(): readonly ArtifactDeliveryRow[] {
  return [
    {
      sliceIndex: 0,
      sourceText: '猫坐在温暖的窗台垫子上。',
      incumbentKind: 'present',
      incumbentText: DAMAGED_BEFORE,
      outcome: {
        kind: 'decided',
        acceptedText: DAMAGED_BEFORE,
      },
      shippedText: DAMAGED_BEFORE,
      delivery: { kind: 'incumbent-retained', },
    },
  ];
}

/**
 Builds one repair-lane issue record carrying one replaced region.

 @param region - edit this record owns

 @returns Record as the lane stores one

 @example
 ```ts
 const record = issueRecord({ region: REGIONS[0], },);
 ```
 */
function issueRecord({ region, }: { readonly region: FixtureRegion; },): unknown {
  return {
    sliceIndex: 0,
    resolved: false,
    issue: {
      issueId: region.issueId,
      status: 'accepted',
      severity: 'minor',
      claims: [],
      tallies: {},
    },
    repairRegions: [
      {
        envelopeId: region.envelopeId,
        issueIds: [region.issueId,],
        before: region.before,
        editorAfter: `${region.before} Then she yawned.`,
      },
    ],
  };
}

/**
 Builds the settled version 2 artifact the gatherers read their records from.

 @returns Whole artifact value

 @example
 ```ts
 const artifact = settledArtifact();
 ```
 */
function settledArtifact(): Record<string, unknown> {
  /**
   Ledger both lanes carry, which the comparison is computed over.
   */
  const delivery = ledger();

  return {
    artifactSchemaVersion: 4,
    id: ENTRY_ID,
    tip: 'a'.repeat(40,),
    pipelineDigest: `sha256-tree-v1:${'c'.repeat(64,)}`,
    corpusSha: 'b'.repeat(40,),
    callConfig: {
      roster: ['cat-house/tabbyscribe-2',],
      retries: 2,
    },
    durationMs: 40,
    timestamp: '2026-08-25T12:00:00.000Z',
    preparation: {
      identity: `sha256-preparation-v1:${'a7'.repeat(32,)}`,
      sliceCount: 1,
      sourceChars: 40,
      targetChars: 60,
      sourceBytes: 90,
      alignmentPairCount: 1,
      alignmentFindings: [],
    },
    lanes: {
      repair: {
        result: {
          repairedText: DAMAGED_BEFORE,
          status: 'unchanged',
          issues: REGIONS.map(function toRecord(region,) {
            return issueRecord({ region, },);
          },),
          findings: [],
          sliceCritics: [
            {
              sliceIndex: 0,
              heardCriticIds: [],
              claimAttributions: [],
            },
          ],
          sliceCount: 1,
          changedSliceIndices: [],
          withdrawnSliceIndices: [],
          sliceTexts: [
            {
              sliceIndex: 0,
              incumbentKind: 'present',
              incumbentText: DAMAGED_BEFORE,
              outcome: {
                kind: 'decided',
                acceptedText: DAMAGED_BEFORE,
              },
            },
          ],
        },
        delivery,
      },
      translate: {
        result: {
          translatedText: DAMAGED_BEFORE,
          sliceCount: 1,
          changedSliceCount: 0,
          refusedSliceCount: 0,
          withdrawnSliceCount: 0,
          changedSliceIndices: [],
          withdrawnSliceIndices: [],
          resumedSliceCount: 0,
          status: 'complete',
          unfilled: [],
          slices: [],
          sliceSelections: [],
          findings: [],
          sliceTexts: [
            {
              sliceIndex: 0,
              incumbentKind: 'present',
              incumbentText: DAMAGED_BEFORE,
              outcome: {
                kind: 'decided',
                acceptedText: DAMAGED_BEFORE,
              },
            },
          ],
        },
        delivery,
      },
    },
    comparison: compareLanes({
      repair: delivery,
      translate: delivery,
    },),
    laneSelection: { kind: 'pending-human-decision', },
  };
}

/**
 Builds the drawn manifest the gatherers index positions into.

 The undamaged item sits FIRST so the damaged one lands at position
 {@link DAMAGED_POSITION}, which is what the sheet marked and what the parser
 requires to match where the item sits.

 @param alsoDrawn - issues drawn after those two, at the positions that
 follow, so a case can draw one edit again

 @returns Manifest value, as a draw writes one

 @example
 ```ts
 const manifest = drawnManifest({ alsoDrawn: [], },);
 ```
 */
function drawnManifest({ alsoDrawn, }: { readonly alsoDrawn: readonly string[]; },): Record<string, unknown> {
  return {
    seed: 'whiskers-seed',
    corpusSha: 'b'.repeat(40,),
    items: [
      {
        position: UNDAMAGED_POSITION,
        entryId: ENTRY_ID,
        issueId: 'adjudicated/stove',
      },
      {
        position: DAMAGED_POSITION,
        entryId: ENTRY_ID,
        issueId: 'adjudicated/windowsill',
      },
      ...alsoDrawn.map(function toItem(
        issueId,
        index,
      ) {
        return {
          position: DAMAGED_POSITION + index + 1,
          entryId: ENTRY_ID,
          issueId,
        };
      },),
    ],
  };
}

/**
 Everything one case needs on disk, disposed together.
 */
type Rig = AsyncDisposable & {
  /**
   Pin naming the throwaway corpus clone and its one commit.
   */
  readonly pin: {
    readonly cloneDir: string;
    readonly commitSha: string;
  };

  /**
   Path the drawn manifest was written to.
   */
  readonly manifestPath: string;

  /**
   Throwaway runs directory holding the settled artifact.
   */
  readonly runsDir: string;
};

/**
 Stands up a throwaway corpus clone, a throwaway runs directory holding one
 settled artifact, and a manifest naming two drawn items.

 The runs directory is handed to each gatherer as the rig's `runsDir`.

 @param alsoDrawn - issues the manifest draws after its two items, none by
 default

 @returns Rig carrying the pin and the manifest path

 @example
 ```ts
 await using rig = await gatheringRig();
 ```
 */
async function gatheringRig(
  { alsoDrawn = [], }: { readonly alsoDrawn?: readonly string[]; } = {},
): Promise<Rig> {
  /**
   Throwaway corpus clone, built and guarded on its own: a throw anywhere in
   its own setup already removes it, through scratchDirWith's own `catch`.
   */
  const corpus = await scratchDirWith({
    prefix: 'whiskers-relabel-corpus-',
    setup: async function cloned({ path: cloneDir, },): Promise<{ readonly commitSha: string; }> {
      await spawnKeyless({
        file: REAL_GIT,
        args: [
          'init',
          cloneDir,
        ],
        extra: {
          GIT_CONFIG_GLOBAL: devNull,
          GIT_CONFIG_SYSTEM: devNull,
        },
      },);
      await mkdir(
        join(
          cloneDir,
          'people',
          ENTRY_ID,
        ),
        { recursive: true, },
      );
      await writeFile(
        join(
          cloneDir,
          'people',
          ENTRY_ID,
          'page.md',
        ),
        SOURCE_PAGE,
        'utf8',
      );
      await writeFile(
        join(
          cloneDir,
          'people',
          ENTRY_ID,
          'page.en.md',
        ),
        TARGET_PAGE,
        'utf8',
      );
      await fixtureGit({
        cloneDir,
        args: [
          'add',
          `people/${ENTRY_ID}/page.md`,
          `people/${ENTRY_ID}/page.en.md`,
        ],
      },);
      await fixtureGit({
        cloneDir,
        args: [
          '-c',
          'user.name=cat',
          '-c',
          'user.email=cat@example.org',
          'commit',
          '--message',
          'add whiskers',
          '--no-gpg-sign',
        ],
      },);

      /**
       Commit every read pins to.
       */
      const commitSha = (await fixtureGit({
        cloneDir,
        args: [
          'rev-parse',
          'HEAD',
        ],
      },))
        .trim();

      return { commitSha, };
    },
  },);

  // THE OUTER OWNS THE INNER FROM HERE: `corpus` already disposes itself on
  // a throw inside its own setup, but once it has returned successfully
  // nothing disposes it automatically until the caller's `await using`
  // fires, so a throw from the runs directory's own setup (guarded the same
  // way, by its own scratchDirWith) must still remove `corpus` here.
  try {
    /**
     Throwaway runs directory holding one settled artifact and the drawn
     manifest.
     */
    const runs = await scratchDirWith({
      prefix: 'whiskers-relabel-runs-',
      setup: async function seeded({ path: runsDir, },): Promise<{ readonly manifestPath: string; }> {
        await mkdir(
          join(
            runsDir,
            'artifacts',
          ),
          { recursive: true, },
        );
        await writeFile(
          join(
            runsDir,
            'artifacts',
            `${ENTRY_ID}.json`,
          ),
          JSON.stringify(settledArtifact(),),
          'utf8',
        );

        /**
         Path the manifest was written to, outside the runs directory
         because the reader takes whatever path it is handed.
         */
        const manifestPath = join(
          runsDir,
          'manifest.json',
        );

        await writeFile(
          manifestPath,
          JSON.stringify(drawnManifest({ alsoDrawn, },),),
          'utf8',
        );

        return { manifestPath, };
      },
    },);

    return {
      pin: {
        cloneDir: corpus.path,
        commitSha: corpus.commitSha,
      },
      manifestPath: runs.manifestPath,
      runsDir: runs.path,
      [Symbol.asyncDispose]: async function removeRig() {
        await runs[Symbol.asyncDispose]();
        await corpus[Symbol.asyncDispose]();
      },
    };
  }
  catch (error) {
    await corpus[Symbol.asyncDispose]();
    throw error;
  }
}

/**
 Reads the envelope ids off a gathered arm, in the order it returned them.

 @param cases - what a gatherer returned

 @returns Envelope ids, order preserved

 @example
 ```ts
 const ids = envelopesOf({ cases, },);
 ```
 */
function envelopesOf(
  { cases, }: { readonly cases: readonly RelabelCase[]; },
): readonly string[] {
  return cases.map(function toEnvelopeId(gathered,) {
    return gathered.region
      .envelopeId;
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: gatherRelabelCases.name,
      children: [
        it({
          name: 'GATHERS the drawn item the reader marked damaged, and only that one',
          fn: async () => {
            await using rig = await gatheringRig();

            /**
             Damaged arm rebuilt from the fixture run.
             */
            const cases = await gatherRelabelCases({
              manifestPath: rig.manifestPath,
              pin: rig.pin,
              runsDir: rig.runsDir,
            },);

            // The manifest names two items and the artifact carries four edits.
            // Only the item whose position the round-three sheet marked is a case,
            // so a gatherer that ignored `DAMAGED_CASES` would return two.
            expect(envelopesOf({ cases, },),).toEqual(['envelope/windowsill',],);
          },
        },),
        it({
          name: 'RECORDS the sheet position each case came from, so a case can be traced back',
          fn: async () => {
            await using rig = await gatheringRig();

            expect((await gatherRelabelCases({
              manifestPath: rig.manifestPath,
              pin: rig.pin,
              runsDir: rig.runsDir,
            },))[0]?.positions,).toEqual([DAMAGED_POSITION,],);
          },
        },),
        it({
          name: 'MERGES an edit drawn again at a second damaged position into the first draw\'s case, which carries '
            + 'both positions, so the prober asks about one edit once',
          fn: async () => {
            // Positions 3 to 6 draw undamaged issues, which the gatherer skips,
            // so the windowsill edit's second draw lands at position 7.
            await using rig = await gatheringRig({
              alsoDrawn: [
                'adjudicated/purr',
                'adjudicated/stair',
                'adjudicated/stove',
                'adjudicated/purr',
                'adjudicated/windowsill',
              ],
            },);

            /**
             Damaged arm rebuilt from the run that drew the edit twice.
             */
            const cases = await gatherRelabelCases({
              manifestPath: rig.manifestPath,
              pin: rig.pin,
              runsDir: rig.runsDir,
            },);

            expect(cases.map(function toDraws(gathered,) {
              return {
                envelopeId: gathered.region
                  .envelopeId,
                positions: gathered.positions,
              };
            },),).toEqual([{
              envelopeId: 'envelope/windowsill',
              positions: [
                DAMAGED_POSITION,
                DRAWN_AGAIN_POSITION,
              ],
            },],);
          },
        },),
        it({
          name: 'PAIRS the replaced region with ITS OWN slice, not with the whole page',
          fn: async () => {
            // A prompt built from the whole document would ask the prober about a
            // passage production never sent, and every claim it drew would describe
            // a different prompt than the one under investigation.
            await using rig = await gatheringRig();

            /**
             The one damaged case.
             */
            const [gathered,] = await gatherRelabelCases({
              manifestPath: rig.manifestPath,
              pin: rig.pin,
              runsDir: rig.runsDir,
            },);

            expect(gathered?.baselineText.includes(DAMAGED_BEFORE,),).toBe(true,);
            expect(gathered?.baselineText.includes(FAR_BEFORE,),).toBe(false,);
            expect(gathered?.sourceText.includes('窗台',),).toBe(true,);
            expect(gathered?.sourceText.includes('呼噜',),).toBe(false,);
          },
        },),
        it({
          name: 'CARRIES the issues the region served, and names an unprobed region as unprobed',
          fn: async () => {
            await using rig = await gatheringRig();

            /**
             The one damaged case.
             */
            const [gathered,] = await gatherRelabelCases({
              manifestPath: rig.manifestPath,
              pin: rig.pin,
              runsDir: rig.runsDir,
            },);

            expect(gathered?.issues.map(function toId(issue,) {
              return issue.issueId;
            },),).toEqual(['adjudicated/windowsill',],);
            expect(gathered?.recorded,).toBe('not probed',);
          },
        },),
      ],
      concurrency: 1,
    },),

    describe({
      name: gatherControlCases.name,
      children: [
        it({
          name: 'NEVER re-probes an envelope the damaged arm already probed',
          fn: async () => {
            // The control exists to say whether the damaged arm's claims are about
            // the damage or about the prompt. Handing it the same edit would make
            // the two arms one arm and the comparison vacuous.
            await using rig = await gatheringRig();

            /**
             Damaged arm, whose envelope the control must exclude.
             */
            const damaged = await gatherRelabelCases({
              manifestPath: rig.manifestPath,
              pin: rig.pin,
              runsDir: rig.runsDir,
            },);

            /**
             Control arm drawn from the same entry.
             */
            const controls = await gatherControlCases({
              manifestPath: rig.manifestPath,
              damaged,
              pin: rig.pin,
              runsDir: rig.runsDir,
            },);

            expect(envelopesOf({ cases: controls, },)
              .includes('envelope/windowsill',),).toBe(false,);
          },
        },),
        it({
          name: 'ORDERS unflagged regions by how closely they match the damaged replaced length, '
            + 'so the arm is not partly a statement about how much text there was to damage',
          fn: async () => {
            // THE DEFECT THIS GUARDS. Taking whichever regions came first would
            // return the short one, which sits ahead of both closer regions in the
            // page. The fixture is built so document order and length order
            // disagree, which is the only way this case can fail.
            await using rig = await gatheringRig();

            /**
             Damaged arm, whose replaced length the control matches against.
             */
            const damaged = await gatherRelabelCases({
              manifestPath: rig.manifestPath,
              pin: rig.pin,
              runsDir: rig.runsDir,
            },);

            /**
             Control arm, closest replaced length first.
             */
            const controls = await gatherControlCases({
              manifestPath: rig.manifestPath,
              damaged,
              pin: rig.pin,
              runsDir: rig.runsDir,
            },);

            expect(envelopesOf({ cases: controls, },),).toEqual([
              'envelope/stair',
              'envelope/stove',
            ],);
          },
        },),
        it({
          name: 'DROPS the region furthest in length even though it comes FIRST in the page',
          fn: async () => {
            // The positive control for the ordering case: an implementation that
            // sorted by length but ignored the per-entry cap would return all three
            // and pass that one while failing this.
            await using rig = await gatheringRig();

            /**
             Damaged arm, whose replaced length the control matches against.
             */
            const damaged = await gatherRelabelCases({
              manifestPath: rig.manifestPath,
              pin: rig.pin,
              runsDir: rig.runsDir,
            },);

            /**
             Control arm, capped at two regions per entry.
             */
            const controls = await gatherControlCases({
              manifestPath: rig.manifestPath,
              damaged,
              pin: rig.pin,
              runsDir: rig.runsDir,
            },);

            expect(controls.length,).toBe(2,);
            expect(envelopesOf({ cases: controls, },)
              .includes('envelope/purr',),).toBe(false,);

            // Spelled out so the reader can see WHY that one is dropped rather than
            // taking the fixture's word for it.
            expect(Math.abs(FAR_BEFORE.length - DAMAGED_BEFORE.length,),)
              .toBeGreaterThan(Math.abs(NEXT_BEFORE.length - DAMAGED_BEFORE.length,),);
          },
        },),
        it({
          name: 'PAIRS each control region with the slice that carries it, never across two',
          fn: async () => {
            await using rig = await gatheringRig();

            /**
             Damaged arm, whose replaced length the control matches against.
             */
            const damaged = await gatherRelabelCases({
              manifestPath: rig.manifestPath,
              pin: rig.pin,
              runsDir: rig.runsDir,
            },);

            /**
             Control arm drawn from the same entry.
             */
            const controls = await gatherControlCases({
              manifestPath: rig.manifestPath,
              damaged,
              pin: rig.pin,
              runsDir: rig.runsDir,
            },);

            for (const control of controls) {
              expect(control.baselineText.includes(control.region
                .before,),).toBe(true,);
              expect(control.baselineText.includes(DAMAGED_BEFORE,),).toBe(false,);
              expect(control.entryId,).toBe(ENTRY_ID,);
            }
          },
        },),
        it({
          name: 'RETURNS nothing when the damaged arm named no entry, rather than drawing at large',
          fn: async () => {
            // The control's entries come from the damaged cases, which is what
            // holds prose style, translator and subject matter fixed. A control
            // that fell back to the whole pool would vary those too, and the only
            // thing meant to differ between the arms is whether a human saw damage.
            await using rig = await gatheringRig();

            expect(await gatherControlCases({
              manifestPath: rig.manifestPath,
              damaged: [],
              pin: rig.pin,
              runsDir: rig.runsDir,
            },),).toEqual([],);
          },
        },),
      ],
      concurrency: 1,
    },),
  ],
},);

//endregion Probe relabel gathering tests
