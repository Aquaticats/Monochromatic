/**
 Tests for the control cases the probe gathers beside the damaged ones.

 THE CONTROL PICKS AT MOST TWO UNFLAGGED REGIONS PER DRAWN ENTRY, closest
 in replaced length to the entry's damaged region first, and it must pick
 ones the damage never touched: a region under an envelope already probed as
 damaged would let the same wording answer as its own control. The flagged
 set excludes those envelopes, the taken set keeps one region per envelope so
 one edit is probed once, and the holder search keeps a control whose
 replaced text the entry's own translation actually carries.

 FIXTURES ARE INVENTED AND CAT-THEMED, in Simplified Chinese against English,
 and the corpus pages live in a throwaway git clone the pin points at, since
 the real inputs are unlicensed corpus pages. Each case hands the gatherer its
 own throwaway runs directory.

 @module
 */

import { mkdir, writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { buildSampleManifest, } from '@monochromatic-dev/translation-repair-lane-contest';
import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  gatherControlCases,
  type RelabelCase,
} from '../../dist/final/node/index.mjs';
import { namingFixtureGit, } from '@monochromatic-dev/translation-repair-corpus-pass/ts/archive-naming.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

//region Control gathering tests
// What the GATHERER does with regions it must not pick and regions it cannot.

/**
 Corpus page of the fixture entries, as the corpus writes one.
 */
const SOURCE_TEXT = '猫坐在垫子上。\n';

/**
 Translation of that page.
 */
const TARGET_TEXT = 'The cat sits on the mat.\n';

/**
 Source page of the entry whose two sections carry one wording.
 */
const REPEATED_SOURCE_TEXT = '## 第一节\n\n猫坐在垫子上。\n\n## 第二节\n\n猫坐在垫子上。\n';

/**
 Translation of that page, both sections carrying the wording of {@link TARGET_TEXT}.
 */
const REPEATED_TARGET_TEXT = '## Section One\n\nThe cat sits on the mat.\n\n## Section Two\n\nThe cat sits on the mat.\n';

/**
 Wording the entries' translations carry, so a control region naming it has a
 holder.
 */
const HELD_BEFORE = 'The cat sits on the mat.';

/**
 Wording no slice of the entries carries.
 */
const UNHELD_BEFORE = 'A cat yowls loudly at the moon.';

/**
 The one damaged case of an entry, flagging its own envelope.

 @param entryId - fixture entry the damage came from

 @returns Damaged cases as the gatherer reads them

 @example
 ```ts
 const damaged = damagedCases({ entryId: 'Kitten', },);
 ```
 */
function damagedCases({ entryId, }: { readonly entryId: string; }): readonly RelabelCase[] {
  return [{
    entryId,
    positions: [2,],
    region: {
      envelopeId: 'env-flagged',
      issueIds: ['adjudicated/naps',],
      before: HELD_BEFORE,
      editorAfter: 'The cat sat on the mat.',
    },
    issues: [],
    sourceText: SOURCE_TEXT,
    baselineText: TARGET_TEXT,
    recorded: 'The cat sat on the mat.',
  },];
}

/**
 Builds a corpus clone whose two fixture entries carry the fixture pages
 inside the caller's scratch directory, and returns the pin pointing at its
 one commit.

 @param cloneDir - scratch directory to build the clone in

 @returns Pin of the clone

 @throws Whatever the fixture git refuses with

 @example
 ```ts
 const pin = await fixturePin({ cloneDir: runs.path, },);
 ```
 */
async function fixturePin(
  { cloneDir, }: { readonly cloneDir: string; },
): Promise<{
  readonly cloneDir: string;
  readonly commitSha: string;
}> {
  await mkdir(join(cloneDir, 'people', 'Kitten',), { recursive: true, },);
  await writeFile(join(cloneDir, 'people', 'Kitten', 'page.md',), SOURCE_TEXT, 'utf8',);
  await writeFile(join(cloneDir, 'people', 'Kitten', 'page.en.md',), TARGET_TEXT, 'utf8',);
  await mkdir(join(cloneDir, 'people', 'Tabby',), { recursive: true, },);
  await writeFile(join(cloneDir, 'people', 'Tabby', 'page.md',), SOURCE_TEXT, 'utf8',);
  await writeFile(join(cloneDir, 'people', 'Tabby', 'page.en.md',), TARGET_TEXT, 'utf8',);
  await mkdir(join(cloneDir, 'people', 'Calico',), { recursive: true, },);
  await writeFile(join(cloneDir, 'people', 'Calico', 'page.md',), REPEATED_SOURCE_TEXT, 'utf8',);
  await writeFile(join(cloneDir, 'people', 'Calico', 'page.en.md',), REPEATED_TARGET_TEXT, 'utf8',);
  await namingFixtureGit({ cloneDir, args: ['init', '--quiet',], },);
  await namingFixtureGit({ cloneDir, args: ['add', 'people',], },);
  await namingFixtureGit({
    cloneDir,
    args: ['commit', '--quiet', '--message', 'fixture pages', '--no-gpg-sign',],
  },);
  const commitSha = await namingFixtureGit({ cloneDir, args: ['rev-parse', 'HEAD',], },);
  return {
    cloneDir,
    commitSha: commitSha.trim(),
  };
}

/**
 Gathers the control cases of one entry over a throwaway run and manifest of
 its own.

 Everything the gatherer reads is built inside this call, so the case calling
 it writes no process global itself and runs the same either order.

 @param entryId - fixture entry whose controls to gather

 @param repairRegions - regions the artifact records for that entry

 @returns Control cases the gatherer produced

 @throws Whatever the reads refuse with

 @example
 ```ts
 const controls = await controlsFor({ entryId: 'Kitten', repairRegions: [], },);
 ```
 */
async function controlsFor(
  {
    entryId,
    repairRegions,
  }: {
    readonly entryId: string;
    readonly repairRegions: readonly {
      readonly envelopeId: string;
      readonly issueIds: readonly string[];
      readonly before: string;
      readonly editorAfter: string;
    }[];
  },
): Promise<readonly RelabelCase[]> {
  await using corpus = await scratchDir({ prefix: 'whiskers-control-corpus-', },);
  const pin = await fixturePin({ cloneDir: corpus.path, },);
  await using runs = await scratchDir({ prefix: 'whiskers-relabel-control-', },);
  const manifestPath = join(runs.path, 'manifest.json',);
  await writeFile(
    manifestPath,
    JSON.stringify(buildSampleManifest({
      sample: [{
        entryId,
        band: 'small',
        issueId: 'adjudicated/naps',
        category: 'fluency/grammar',
        severity: 'major',
        summary: 'the tense is wrong',
        sourceQuotes: ['猫坐在垫子上',],
        targetQuotes: [HELD_BEFORE,],
        sourceAnchor: 'quoted',
      },],
      seed: 'cat-seed',
      corpusSha: 'sha/1',
      generation: {
        kind: 'unrecorded',
        reason: 'the fixture draw',
      },
    },),),
  );
  await mkdir(join(runs.path, 'artifacts',), { recursive: true, },);
  await writeFile(
    join(runs.path, 'artifacts', `${entryId}.json`,),
    JSON.stringify({
      id: entryId,
      status: 'settled',
      issues: [{
        sliceIndex: 0,
        resolved: false,
        issue: {
          issueId: 'adjudicated/naps',
          status: 'accepted',
          severity: 'minor',
          claims: [{
            claimId: 'claim/whisker',
            claim: {
              category: 'accuracy/omission',
              severity: 'minor',
              summary: 'A purr is dropped from the greeting.',
              spans: [{
                side: 'source',
                nodeId: 'block/0',
                quotedText: '猫坐在垫子上',
              }, {
                side: 'target',
                nodeId: 'block/0',
                quotedText: '',
              },],
            },
          },],
          tallies: {},
        },
        repairRegions,
        refined: false,
        repairDisposition: 'unchanged',
      },],
    },),
    'utf8',
  );
  return await gatherControlCases({
    manifestPath,
    damaged: damagedCases({ entryId, },),
    pin,
    runsDir: runs.path,
  },);
}

await describe({
  name: gatherControlCases.name,
  concurrency: 1,
  children: [
    it({
      name: 'TAKES ONE REGION PER ENVELOPE, the first of it, into a control carrying the slice that holds '
        + 'its replaced text and the issue it served, since a second region of the same envelope would answer '
        + 'as its own control',
      fn: async () => {
        const controls = await controlsFor({
          entryId: 'Kitten',
          repairRegions: [{
            envelopeId: 'env-one',
            issueIds: ['adjudicated/naps',],
            before: HELD_BEFORE,
            editorAfter: 'The cat sat on the mat.',
          }, {
            envelopeId: 'env-one',
            issueIds: ['adjudicated/naps',],
            before: HELD_BEFORE,
            editorAfter: 'The cat napped on the mat.',
          },],
        },);
        expect(controls,).toEqual([{
          entryId: 'Kitten',
          positions: [],
          region: {
            envelopeId: 'env-one',
            issueIds: ['adjudicated/naps',],
            before: HELD_BEFORE,
            editorAfter: 'The cat sat on the mat.',
          },
          issues: [{
            issueId: 'adjudicated/naps',
            status: 'accepted',
            severity: 'minor',
            claims: [{
              claimId: 'claim/whisker',
              claim: {
                category: 'accuracy/omission',
                severity: 'minor',
                summary: 'A purr is dropped from the greeting.',
                spans: [],
              },
            },],
            tallies: {},
          },],
          sourceText: '猫坐在垫子上。',
          baselineText: HELD_BEFORE,
          recorded: 'not probed',
        },],);
      },
    },),

    it({
      name: 'SKIPS A REGION the entry translation never held, since a control over text no slice carries '
        + 'would compare against nothing',
      fn: async () => {
        const controls = await controlsFor({
          entryId: 'Tabby',
          repairRegions: [{
            envelopeId: 'env-two',
            issueIds: ['adjudicated/naps',],
            before: UNHELD_BEFORE,
            editorAfter: 'A cat yowled loudly at the moon.',
          },],
        },);
        expect(controls,).toEqual([],);
      },
    },),

    it({
      name: 'REFUSES A REGION whose replaced text more than one slice of the entry carries, since the control '
        + 'would compare against whichever slice came first',
      fn: async () => {
        /**
         What the gather raised over an entry with two sections carrying the wording.
         */
        const refusal = await rejectionOf(async function gathersAmbiguousControl(): Promise<unknown> {
          return controlsFor({
            entryId: 'Calico',
            repairRegions: [{
              envelopeId: 'env-three',
              issueIds: ['adjudicated/naps',],
              before: HELD_BEFORE,
              editorAfter: 'The cat sat on the mat.',
            },],
          },);
        },);
        expect(refusal,).toBeInstanceOf(ArtifactParseError,);
        expect(String(refusal,),).toBe(
          `ArtifactParseError: artifact parse failed at slice holding the replaced text of ${
            String(HELD_BEFORE.length,)
          } characters: expected present in one slice; more than one slice carries it, so which of them `
            + 'production sent cannot be read.',
        );
      },
    },),

    it({
      name: 'LEAVES OUT A REGION under the envelope the entry\'s damaged case probed and TAKES the one beside it, '
        + 'since the damaged wording would otherwise answer as its own control',
      fn: async () => {
        const controls = await controlsFor({
          entryId: 'Kitten',
          repairRegions: [{
            envelopeId: 'env-flagged',
            issueIds: ['adjudicated/naps',],
            before: HELD_BEFORE,
            editorAfter: 'The cat sat on the mat.',
          }, {
            envelopeId: 'env-one',
            issueIds: ['adjudicated/naps',],
            before: HELD_BEFORE,
            editorAfter: 'The cat napped on the mat.',
          },],
        },);
        expect(controls.map(function envelopeOf(control,): string {
          return control.region.envelopeId;
        },),).toEqual(['env-one',],);
      },
    },),

    it({
      name: 'TAKES TWO REGIONS AN ENTRY, closest in replaced length to its damaged region first, so the '
        + 'control differs from the damage in the human verdict rather than in length',
      fn: async () => {
        // Replaced lengths 7, 15 and 24 against the damaged region's 24, in
        // the order the artifact records them.
        const controls = await controlsFor({
          entryId: 'Kitten',
          repairRegions: [{
            envelopeId: 'env-a',
            issueIds: ['adjudicated/naps',],
            before: 'The cat',
            editorAfter: 'A cat',
          }, {
            envelopeId: 'env-b',
            issueIds: ['adjudicated/naps',],
            before: 'sits on the mat',
            editorAfter: 'naps on the mat',
          }, {
            envelopeId: 'env-c',
            issueIds: ['adjudicated/naps',],
            before: HELD_BEFORE,
            editorAfter: 'The cat sat on the mat.',
          },],
        },);
        expect(controls.map(function envelopeOf(control,): string {
          return control.region.envelopeId;
        },),).toEqual(['env-c', 'env-b',],);
      },
    },),
  ],
},);

//endregion Control gathering tests
