/**
 Tests for finding the slice a recorded replacement was made in.

 THE PROBE RE-CARVES WHAT THE RUN CARVED, which is the only reason this
 function exists: a relabelling asks a fresh reader about a region production
 already edited, and if the re-carve landed on different text the reader would
 be answering about a passage production never sent. So the lookup is by TEXT
 rather than by index, and the two returned halves must come from ONE slice.
 A function that returned the whole document for both would satisfy a
 careless test and quietly widen every prompt the probe sends.

 THE REFUSAL IS A SECURITY BOUNDARY, not just a diagnostic.
 `ArtifactParseError` carries `messageNamesOnly`, which `reportingRefusals`
 reads as permission to print the whole message, and the marker's justification
 is that the class "names the artifact path and the shape the value failed to
 satisfy, and quotes neither the value nor the file". Until 2026-08-25 this
 site put 80 characters of the replaced TRANSLATION into that path, so a probe
 that could not find its slice printed a memorial page's wording to a terminal
 and into the run's log. Sweeping the package found it was the only one of 47
 interpolating paths that quoted text rather than a structural position. The
 "REFUSES to quote the text it could not find" case is that guard, and it is the reason this file is worth its length.

 FIXTURES ARE INVENTED AND CAT-THEMED, in Simplified Chinese against English,
 because the real inputs are unlicensed corpus pages. Each case hands the
 gatherer its own throwaway runs directory.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { mkdir, writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import { buildSampleManifest, } from '@monochromatic-dev/translation-repair-lane-contest';
import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  gatherRelabelCases,
  locateSlice,
} from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

//region Relabel case location tests

/**
 Source page, two sections, as the corpus writes one.
 */
const SOURCE_TEXT = [
  '## 第一节',
  '',
  '猫坐在垫子上。',
  '',
  '## 第二节',
  '',
  '小猫在楼梯上看着。',
  '',
].join('\n',);

/**
 Translation of that page, section for section.
 */
const TARGET_TEXT = [
  '## Section One',
  '',
  'The cat sat on the mat.',
  '',
  '## Section Two',
  '',
  'The kitten watched from the stairs.',
  '',
].join('\n',);

/**
 Wording the first section carries, which an edit replaced.
 */
const FIRST_WORDING = 'The cat sat on the mat.';

/**
 Wording the second section carries, so a lookup has to choose.
 */
const SECOND_WORDING = 'The kitten watched from the stairs.';

/**
 Source of the section holding {@link FIRST_WORDING}.
 */
const FIRST_SOURCE = '## 第一节\n\n猫坐在垫子上。';

/**
 Source of the section holding {@link SECOND_WORDING}.
 */
const SECOND_SOURCE = '## 第二节\n\n小猫在楼梯上看着。';

/**
 Source page whose two sections carry one wording.
 */
const REPEATED_SOURCE_TEXT = '## 第一节\n\n猫坐在垫子上。\n\n## 第二节\n\n猫坐在垫子上。\n';

/**
 Translation of that page, both sections carrying {@link FIRST_WORDING}.
 */
const REPEATED_TARGET_TEXT = `## Section One\n\n${FIRST_WORDING}\n\n## Section Two\n\n${FIRST_WORDING}\n`;

/**
 Wording no slice carries, standing in for a re-carve that drifted.

 Written as a sentence a search can find whole, so a message that quoted it
 back could not be mistaken for a coincidence.
 */
const ABSENT_WORDING = 'the dog barked once at the gate and then lay down again';

/**
 Attempts to locate the slice holding {@link ABSENT_WORDING}, which no slice
 carries, standing in for a re-carve that drifted from the run it is relabelling.

 @throws ArtifactParseError naming only the absent wording's length

 @example
 ```ts
 expect(refuseMissingSlice,).toThrow(ArtifactParseError,);
 ```
 */
function refuseMissingSlice(): void {
  locateSlice({
    sourceText: SOURCE_TEXT,
    targetText: TARGET_TEXT,
    before: ABSENT_WORDING,
  },);
}

await describe({
  name: locateSlice.name,
  concurrency: 1,
  children: [
    it({
      name: 'RETURNS the slice carrying the replaced text, not the whole page',
      fn: async () => {
        expect(locateSlice({
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
          before: FIRST_WORDING,
        },),).toEqual({
          sourceText: FIRST_SOURCE,
          baselineText: `## Section One\n\n${FIRST_WORDING}`,
        },);
      },
    },),
    it({
      name: 'CHOOSES the slice by its text, so a later section is found as readily',
      fn: async () => {
        // The positive control for the "RETURNS the slice carrying the replaced
        // text" case: a lookup that always returned the first slice would pass
        // that one and fail this.
        expect(locateSlice({
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
          before: SECOND_WORDING,
        },),).toEqual({
          sourceText: SECOND_SOURCE,
          baselineText: `## Section Two\n\n${SECOND_WORDING}`,
        },);
      },
    },),
    it({
      name: 'PAIRS the source and the translation of ONE slice, never across two',
      fn: async () => {
        /**
         Second section's slice, found by its own wording.
         */
        const found = locateSlice({
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
          before: SECOND_WORDING,
        },);

        // A prompt built from the first section's source and the second's
        // translation would ask a reader to compare unrelated passages, and
        // every claim it drew would be noise.
        expect(found.sourceText.includes('第一节',),).toBe(false,);
        expect(found.baselineText.includes('Section One',),).toBe(false,);
      },
    },),
    it({
      name: 'REFUSES when no slice carries the replaced text, since the re-carve drifted',
      fn: async () => {
        expect(refuseMissingSlice,).toThrow(ArtifactParseError,);
      },
    },),
    it({
      name: 'REFUSES to quote the text it could not find, naming only its length',
      fn: async () => {
        /**
         Refusal the missing lookup raised, or nothing when it did not raise.
         */
        let said = '';

        try {
          locateSlice({
            sourceText: SOURCE_TEXT,
            targetText: TARGET_TEXT,
            before: ABSENT_WORDING,
          },);
        }
        catch (refused) {
          said = String(refused,);
        }

        // The class is marked quote-free, so `reportingRefusals` prints this
        // message whole. Anything of the corpus that reaches it reaches a
        // terminal and a log.
        expect(said.includes(ABSENT_WORDING,),).toBe(false,);
        expect(said.includes('the dog barked',),).toBe(false,);
        expect(said.includes(`${String(ABSENT_WORDING.length,)} characters`,),).toBe(true,);
      },
    },),
    it({
      name: 'SAYS what a missing slice means, so the refusal is actionable',
      fn: async () => {
        expect(refuseMissingSlice,).toThrow('slicing no longer reproduces the run',);
      },
    },),
    it({
      name: 'REFUSES when more than one slice carries the replaced text, since the prompt would be rebuilt '
        + 'from whichever came first',
      fn: async () => {
        /**
         What the lookup raised over two sections that both carry the wording.
         */
        const refusal = caught(function locatesAmbiguously(): unknown {
          return locateSlice({
            sourceText: REPEATED_SOURCE_TEXT,
            targetText: REPEATED_TARGET_TEXT,
            before: FIRST_WORDING,
          },);
        },);
        expect(refusal,).toBeInstanceOf(ArtifactParseError,);
        expect(String(refusal,),).toBe(
          `ArtifactParseError: artifact parse failed at slice holding the replaced text of ${
            String(FIRST_WORDING.length,)
          } characters: expected present in one slice; more than one slice carries it, so which of them `
            + 'production sent cannot be read.',
        );
      },
    },),
    //region Relabel case gathering cases
    // What the GATHERER does with a manifest whose draw names nothing it can rebuild.
    describe({
      name: gatherRelabelCases.name,
      children: [
        it({
          name: 'SKIPS A DRAWN ITEM whose issue id matches no damaged candidate, since the manifest names '
            + 'positions production already damaged but nothing says the draw kept the same issues',
          fn: async () => {
            /**
             Throwaway run holding the manifest and the entry artifact.
             */
            await using runs = await scratchDir({ prefix: 'whiskers-relabel-case-', },);
            const manifestPath = join(runs.path, 'manifest.json',);
            await writeFile(
              manifestPath,
              JSON.stringify(buildSampleManifest({
                sample: [{
                  entryId: 'Kitten',
                  band: 'small',
                  issueId: 'adjudicated/naps',
                  category: 'fluency/grammar',
                  severity: 'major',
                  summary: 'the tense is wrong',
                  sourceQuotes: ['猫坐在垫子上',],
                  targetQuotes: ['The cat sits on the mat.',],
                  sourceAnchor: 'quoted',
                }, {
                  entryId: 'Kitten',
                  band: 'small',
                  issueId: 'adjudicated/never',
                  category: 'fluency/grammar',
                  severity: 'major',
                  summary: 'the tense is wrong',
                  sourceQuotes: ['猫坐在垫子上',],
                  targetQuotes: ['The cat sits on the mat.',],
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
            /**
             Where the artifact of the drawn entry sits.
             */
            const artifactPath = join(runs.path, 'artifacts', 'Kitten.json',);
            /**
             Pin naming a clone that does not exist, so a case that reached
             the corpus read would be refused there and read no page.
             */
            const pin = {
              cloneDir: join(runs.path, 'no-clone-here',),
              commitSha: 'deadbeef',
            };
            // THE DAMAGED POSITION DRAWS `adjudicated/never`, and the entry
            // settled a record under the other id only, so the gatherer
            // compares the two ids and finds no match.
            await writeFile(
              artifactPath,
              JSON.stringify({
                id: 'Kitten',
                status: 'settled',
                issues: [{
                  issue: {
                    issueId: 'adjudicated/naps',
                    status: 'accepted',
                    severity: 'major',
                    claims: [],
                  },
                },],
              },),
              'utf8',
            );
            expect(await gatherRelabelCases({
              manifestPath,
              pin,
              runsDir: runs.path,
            },),).toEqual([],);
            // THE CONTROL: with the drawn id settled, the gatherer goes on to
            // the page, which the absent clone refuses. So the empty answer
            // came from the id comparison and from nothing before it.
            await writeFile(
              artifactPath,
              JSON.stringify({
                id: 'Kitten',
                status: 'settled',
                issues: [{
                  issue: {
                    issueId: 'adjudicated/never',
                    status: 'accepted',
                    severity: 'major',
                    claims: [],
                  },
                },],
              },),
              'utf8',
            );
            await expect(gatherRelabelCases({
              manifestPath,
              pin,
              runsDir: runs.path,
            },),).rejects
              .toThrow(
              'corpus read failed for deadbeef:people/Kitten/page.md (unreadable-clone); git could not open the '
                + 'clone: check that the directory exists, is the top of a git repository, and is one git may '
                + 'read.',
              );
          },
        },),
      ],
    },),
    //endregion Relabel case gathering cases
  ],
},);

//endregion Relabel case location tests
