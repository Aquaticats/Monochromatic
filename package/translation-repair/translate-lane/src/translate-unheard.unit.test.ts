/**
 Tests for what a settled record means when no translator was heard.

 WHAT THESE PIN is a property three other places already rest on: the driver
 refusing to cache such a slice, the wording builder reporting it as the
 archive standing by default, and the artifact reader that will read both. All
 three assume the record left the archive alone, and nothing checked it.

 A stage that heard nobody and returned a change would travel: the lane's
 shipped set is built from `changed`, so the document would carry a
 replacement while the wording ledger reported that nobody produced one.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  assertUnheardKeptIncumbent,
  heardNobody,
  type TranslateSliceRecord,
  TranslateUnheardError,
} from '../dist/final/node/index.mjs';

/**
 Archive wording of the slice every case here uses.
 */
const ARCHIVE_NAP = 'The cat sleeps on the sill.';

/**
 Builds a settled record with the three fields this rule relates.

 @param heardTranslators - voices the producing stage heard

 @param outputText - text the driver accepted for assembly

 @param changed - whether that text differs from the archive's, as the record
 claims rather than as it is

 @returns Record shaped as the driver settles one

 @example
 ```ts
 const record = recordFor({ heardTranslators: 0, outputText: ARCHIVE_NAP, changed: false, },);
 ```
 */
function recordFor(
  {
    heardTranslators,
    outputText,
    changed,
  }: {
    readonly heardTranslators: number;
    readonly outputText: string;
    readonly changed: boolean;
  },
): TranslateSliceRecord {
  return {
    kind: 'translate-slice',
    schemaVersion: 1,
    sliceIndex: 4,
    outputText,
    changed,
    disposition: 'stage-result',
    findings: [],
    stageResult: {
      text: outputText,
      origin: 'fresh',
      decision: 'judged',
      voteWeight: 1,
      ballots: [],
      heardTranslators,
      candidateCount: heardTranslators,
      slate: [],
      perCandidate: [],
      findings: [],
    },
  } as unknown as TranslateSliceRecord;
}

await describe({
  name: assertUnheardKeptIncumbent.name,
  children: [
    it({
      name:
        'accepts the only shape hearing nobody can take: the archive`s own wording, carried forward, '
          + 'claiming no change',
      fn: async () => {
        assertUnheardKeptIncumbent({
          sliceIndex: 4,
          record: recordFor({
            heardTranslators: 0,
            outputText: ARCHIVE_NAP,
            changed: false,
          },),
          incumbentText: ARCHIVE_NAP,
        },);
      },
    },),
    it({
      name:
        'REFUSES a record that heard nobody and carries wording of its own, since a rendering nobody '
          + 'produced would be written into the document',
      fn: async () => {
        expect(function wroteWhatNobodySaid() {
          assertUnheardKeptIncumbent({
            sliceIndex: 4,
            record: recordFor({
              heardTranslators: 0,
              outputText: 'The cat is asleep on the windowsill.',
              changed: false,
            },),
            incumbentText: ARCHIVE_NAP,
          },);
        },).toThrow(TranslateUnheardError,);
      },
    },),
    it({
      name:
        'REFUSES a record that heard nobody and claims a change, because the shipped set is built from '
          + 'that flag: the document would carry a replacement while the wording ledger reported that '
          + 'nobody produced one, which is the contradiction the two axes exist to make impossible',
      fn: async () => {
        /**
         What claimedAChange raised, read for its class as well as its wording.
         */
        const refusalOfClaimedAChange = caught(function claimedAChange() {
          assertUnheardKeptIncumbent({
            sliceIndex: 4,
            record: recordFor({
              heardTranslators: 0,
              outputText: ARCHIVE_NAP,
              changed: true,
            },),
            incumbentText: ARCHIVE_NAP,
          },);
        },);

        expect(refusalOfClaimedAChange,).toBeInstanceOf(TranslateUnheardError,);
        expect((refusalOfClaimedAChange as Error).message,).toContain('reports a change',);
      },
    },),
    it({
      name:
        'says nothing about a record that DID hear somebody, whatever it decided, since this rule is '
          + 'about silence and a lane that was answered may legitimately replace the archive wording',
      fn: async () => {
        assertUnheardKeptIncumbent({
          sliceIndex: 4,
          record: recordFor({
            heardTranslators: 2,
            outputText: 'The cat is asleep on the windowsill.',
            changed: true,
          },),
          incumbentText: ARCHIVE_NAP,
        },);
      },
    },),
    it({
      name:
        'names hearing nobody in one place, because the driver, the wording builder and this rule all '
          + 'ask it, and two spellings would eventually disagree about which slice fell between them',
      fn: async () => {
        expect(heardNobody({
          record: recordFor({
            heardTranslators: 0,
            outputText: ARCHIVE_NAP,
            changed: false,
          },),
        },),).toBe(true,);
        expect(heardNobody({
          record: recordFor({
            heardTranslators: 1,
            outputText: ARCHIVE_NAP,
            changed: false,
          },),
        },),).toBe(false,);
      },
    },),
  ],
},);
