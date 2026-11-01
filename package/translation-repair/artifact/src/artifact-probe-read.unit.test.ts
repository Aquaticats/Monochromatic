/**
 Tests for lifting probe telemetry back out of a settled artifact, where
 absence and malformation must not be treated alike.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ArtifactDeliveryRow,
  ArtifactParseError,
  compareLanes,
} from '@monochromatic-dev/translation-repair-artifact-pool';
import { readArtifactProbe, } from '../dist/final/node/index.mjs';

/**
 Builds one region tally as an artifact carries it.

 @param envelopeId - envelope the region replaced

 @param corroborated - upheld claims of added damage

 @param issueIds - every issue this region serves, which is more than one
 whenever a single replacement covered several accepted issues

 @returns Tally object for a fixture artifact

 @example
 ```ts
 const tally = catTally({ envelopeId: 'envelope/nap', },);
 ```
 */
function catTally(
  {
    envelopeId,
    corroborated = 0,
    issueIds = ['adjudicated/nap',],
    claims = [],
  }: {
    readonly envelopeId: string;
    readonly corroborated?: number;
    readonly issueIds?: readonly string[];
    readonly claims?: readonly unknown[];
  },
): Record<string, unknown> {
  return {
    envelopeId,
    issueIds,
    corroborated,
    removalCorroborated: 0,
    contradicted: 0,
    unanchored: 0,
    preExisting: 0,
    noneFound: 3 - corroborated,
    uncertain: 0,
    claims,
  };
}

/**
 Builds one screened claim as an artifact carries it, quotes included.

 The quote fields are present here precisely so a case can assert the reader
 DROPS them.

 @param modelId - prober that made the claim

 @param admissibility - what the screen made of the quote

 @returns Claim object for a fixture artifact

 @example
 ```ts
 const claim = catClaim({ modelId: 'cat/one', },);
 ```
 */
function catClaim(
  {
    modelId,
    admissibility = 'corroborated',
  }: {
    readonly modelId: string;
    readonly admissibility?: string;
  },
): Record<string, unknown> {
  return {
    modelId,
    admissibility,
    category: 'meaning',
    severity: 'major',
    evidence: 'the cat naps on the sunlit sill',
    omittedText: 'the cat had not napped',
    reason: 'this wording is new in the replacement',
  };
}

/**
 Builds one issue record as an artifact carries it.

 @param repairDisposition - what became of this issue's repair

 @param introducedDefects - probe reading, or absent when never probed

 @param issueId - adjudicated issue this record is about; defaults to one id
 because most cases have a single record and only ownership cases need to
 tell two apart

 @returns Record object for a fixture artifact

 @example
 ```ts
 const record = catRecord({ repairDisposition: 'shipped', },);
 ```
 */
function catRecord(
  {
    repairDisposition,
    introducedDefects,
    issueId = 'adjudicated/nap',
    refined = false,
  }: {
    readonly repairDisposition: string;
    readonly introducedDefects?: unknown;
    readonly issueId?: string;
    readonly refined?: boolean;
  },
): Record<string, unknown> {
  return {
    sliceIndex: 0,
    repairDisposition,
    resolved: true,
    refined,
    issue: { issueId, },
    ...(introducedDefects === undefined ? {} : { introducedDefects, }),
  };
}

/**
 Original of the slice the lanes work on.
 */
const SOURCE_NAP = '猫猫在窗台上睡觉。';

/**
 Archive's own English for it.
 */
const ARCHIVE_NAP = 'The cat sleeps on the sill.';

/**
 Wording the translate lane decided for it.
 */
const FRESH_NAP = 'The cat naps on the windowsill.';

/**
 Identity a preparation gives itself, checked for SYNTAX only.
 */
const PREPARATION_IDENTITY = `sha256-preparation-v1:${'a7'.repeat(32,)}`;

/**
 One lane's delivery ledger over the single slice these fixtures carry.

 @param shippedText - wording this lane delivered

 @param delivery - what it did to get there

 @returns One row, which is the whole ledger here

 @example
 ```ts
 const rows = catLedger({ shippedText: ARCHIVE_NAP, delivery: 'incumbent-retained', },);
 ```
 */
function catLedger(
  {
    shippedText,
    delivery,
  }: {
    readonly shippedText: string;
    readonly delivery: string;
  },
): readonly ArtifactDeliveryRow[] {
  return [
    {
      sliceIndex: 0,
      sourceText: SOURCE_NAP,
      incumbentKind: 'present',
      incumbentText: ARCHIVE_NAP,
      outcome: {
        kind: 'decided',
        acceptedText: shippedText,
      },
      shippedText,
      delivery: { kind: delivery, },
    },
  ] as readonly ArtifactDeliveryRow[];
}

/**
 One whole version 2 artifact carrying this case's issue records.

 WHY EVERY FIXTURE HERE IS A WHOLE ARTIFACT and not the bare
 `{ id, issues }` these cases used to pass: the records moved into the repair
 lane at version 2, and `readArtifactProbe` now reaches them through the
 version 2 parser rather than by naming a root key that no longer exists. That
 parser enforces exact top-level keys, so a fixture cannot be patched into
 shape one field at a time; it is a version 2 artifact or it is refused.

 The envelope around `issues` is inert for every case here. Version 2 fixes
 the shape of a lane, not the shape of a result, so the records inside
 participate in no cross-check and each case still varies only its own records.

 @param id - entry this artifact is about

 @param issues - repair lane's issue records, empty when the lane filed none

 @param findings - what the repair lane's stages reported, empty when none spoke

 @returns Artifact as JSON

 @example
 ```ts
 const artifact = probeArtifact({ id: 'Kitten', issues: [], },);
 ```
 */
function probeArtifact(
  {
    id,
    issues = [],
    findings = [],
  }: {
    readonly id: string;
    readonly issues?: readonly unknown[];
    readonly findings?: readonly unknown[];
  },
): Record<string, unknown> {
  /**
   Repair lane's rows, named so the comparison is derived from the same
   ledger the lane carries rather than from a second copy of it.
   */
  const repairDelivery = catLedger({
    shippedText: ARCHIVE_NAP,
    delivery: 'incumbent-retained',
  },);

  /**
   Translate lane's rows, on the same footing.
   */
  const translateDelivery = catLedger({
    shippedText: FRESH_NAP,
    delivery: 'replacement-shipped',
  },);
  return {
    artifactSchemaVersion: 4,
    id,
    tip: 'a'.repeat(40,),
    pipelineDigest: `sha256-tree-v1:${'c'.repeat(64,)}`,
    corpusSha: 'b'.repeat(40,),
    callConfig: {
      roster: ['Tabby',],
      retries: 2,
    },
    durationMs: 40,
    timestamp: '2026-08-16T21:00:00.000Z',
    preparation: {
      identity: PREPARATION_IDENTITY,
      sliceCount: 1,
      sourceChars: 20,
      targetChars: 30,
      sourceBytes: 45,
      alignmentPairCount: 1,
      alignmentFindings: [],
    },
    lanes: {
      repair: {
        result: {
          repairedText: ARCHIVE_NAP,
          status: 'unchanged',
          issues,
          findings,
          sliceCritics: [],
          sliceCount: 1,
          changedSliceIndices: [],
          withdrawnSliceIndices: [],
          sliceTexts: [
            {
              sliceIndex: 0,
              incumbentKind: 'present',
              incumbentText: ARCHIVE_NAP,
              outcome: {
                kind: 'decided',
                acceptedText: ARCHIVE_NAP,
              },
            },
          ],
        },
        delivery: repairDelivery,
      },
      translate: {
        result: {
          translatedText: FRESH_NAP,
          sliceCount: 1,
          changedSliceCount: 1,
          refusedSliceCount: 0,
          withdrawnSliceCount: 0,
          changedSliceIndices: [0,],
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
              incumbentText: ARCHIVE_NAP,
              outcome: {
                kind: 'decided',
                acceptedText: FRESH_NAP,
              },
            },
          ],
        },
        delivery: translateDelivery,
      },
    },
    comparison: compareLanes({
      repair: repairDelivery,
      translate: translateDelivery,
    },),
    laneSelection: { kind: 'pending-human-decision', },
  };
}

await describe({
  name: readArtifactProbe.name,
  children: [
    it({
      name: 'reads only shipped records, since the repair sheet grades only '
        + 'those and counting the rest would put regions nobody judged into a '
        + 'rate about judged ones',
      fn: async () => {
        const reading = readArtifactProbe({
          value: probeArtifact({
            id: 'Kitten',
            issues: [
              catRecord({
                repairDisposition: 'shipped',
                introducedDefects: {
                  heardProbers: 3,
                  configuredProbers: 3,
                  regions: [catTally({ envelopeId: 'envelope/nap', },),],
                },
              },),
              catRecord({
                repairDisposition: 'not-selected',
                introducedDefects: {
                  heardProbers: 3,
                  configuredProbers: 3,
                  regions: [catTally({ envelopeId: 'envelope/chase', },),],
                },
              },),
            ],
          },),
          path: 'Kitten',
        },);
        expect(reading.readings,).toHaveLength(1,);
        expect(reading.repairShippedRecords,).toBe(1,);
        expect(reading.readings[0]
          ?.regions[0]
          ?.envelopeId,).toBe('envelope/nap',);
      },
    },),

    it({
      name: 'pairs each reading with the issue whose RECORD carried it, not '
        + 'with the issues its regions name. One replacement can serve several '
        + 'accepted issues, so a shared envelope appears in every one of their '
        + 'readings and names all of them; deciding ownership from those lists '
        + 'attaches whichever record was indexed last, which is how a graded '
        + 'sheet position would receive another record\'s probe verdict while '
        + 'the counts looked entirely normal',
      fn: async () => {
        /**
         Envelope serving both issues, exactly as a merged replacement does.
         */
        const shared = catTally({
          envelopeId: 'envelope/shared',
          issueIds: [
            'adjudicated/nap',
            'adjudicated/chase',
          ],
        },);
        const reading = readArtifactProbe({
          value: probeArtifact({
            id: 'Kitten',
            issues: [
              catRecord({
                repairDisposition: 'shipped',
                issueId: 'adjudicated/nap',
                introducedDefects: {
                  heardProbers: 3,
                  configuredProbers: 3,
                  regions: [shared,],
                },
              },),
              catRecord({
                repairDisposition: 'shipped',
                issueId: 'adjudicated/chase',
                introducedDefects: {
                  heardProbers: 3,
                  configuredProbers: 3,
                  regions: [shared,],
                },
              },),
            ],
          },),
          path: 'Kitten',
        },);

        expect(reading.owned
          .map(function toIssueId(entry,) {
            return entry.issueId;
          },),).toEqual([
            'adjudicated/nap',
            'adjudicated/chase',
          ],);
        expect(reading.owned,).toHaveLength(2,);
      },
    },),

    it({
      name: 'keeps claim IDENTITY and drops the quotes. The majority rule '
        + 'counts distinct probers, so modelId and admissibility have to '
        + 'survive parsing or every region reads as uncorroborated; the quote '
        + 'fields carry unlicensed corpus text into a summary meant to be '
        + 'pasteable, so they must not',
      fn: async () => {
        const reading = readArtifactProbe({
          value: probeArtifact({
            id: 'Kitten',
            issues: [
              catRecord({
                repairDisposition: 'shipped',
                introducedDefects: {
                  heardProbers: 3,
                  configuredProbers: 3,
                  regions: [
                    catTally({
                      envelopeId: 'envelope/nap',
                      corroborated: 1,
                      claims: [catClaim({ modelId: 'cat/one', },),],
                    },),
                  ],
                },
              },),
            ],
          },),
          path: 'Kitten',
        },);

        /**
         Sole parsed claim.
         */
        const claim = reading.readings[0]
          ?.regions[0]
          ?.claims[0];

        expect(claim?.modelId,).toBe('cat/one',);
        expect(claim?.admissibility,).toBe('corroborated',);

        // ABSENT, not empty. The reader used to write '' into every quote
        // field so the result satisfied a full claim type, which made "this
        // reader does not parse quotes" indistinguishable from "the prober
        // quoted nothing" to anyone reading the value. Only the first is ever
        // true here, since the screen cannot admit an unanchored claim as
        // corroborated.
        expect(Object.keys(claim ?? {},)
          .toSorted(),).toEqual([
            'admissibility',
            'modelId',
          ],);
      },
    },),

    it({
      name: 'THROWS on an admissibility the screen never emits, rather than '
        + 'treating it as non-upholding. An unrecognized verdict name would '
        + 'silently zero the corroboration of every region and read as a clean '
        + 'run, which is the failure shape that looks most like success',
      fn: async () => {
        /**
         Reads a claim under an admissibility the screen cannot record.
         */
        function readsUnknownAdmissibility() {
          readArtifactProbe({
            value: probeArtifact({
              id: 'Kitten',
              issues: [
                catRecord({
                  repairDisposition: 'shipped',
                  introducedDefects: {
                    heardProbers: 3,
                    configuredProbers: 3,
                    regions: [
                      catTally({
                        envelopeId: 'envelope/nap',
                        corroborated: 1,
                        claims: [
                          catClaim({
                            modelId: 'cat/one',
                            admissibility: 'probably-fine',
                          },),
                        ],
                      },),
                    ],
                  },
                },),
              ],
            },),
            path: 'Kitten',
          },);
        }

        expect(readsUnknownAdmissibility,).toThrow(ArtifactParseError,);
        expect(readsUnknownAdmissibility,).toThrow('.claims[0].admissibility',);
      },
    },),

    it({
      name: 'carries the REFINED flag with each reading, because it decides '
        + 'whether the reading is about the text that shipped. The probe runs '
        + 'inside the accuracy stage and the naturalness lane runs after it, so '
        + 'on a refined slice the probe judged wording the lane replaced, while '
        + 'the repair sheet asks the human to grade the returned wording. '
        + 'Without this flag that mismatch joins silently',
      fn: async () => {
        const reading = readArtifactProbe({
          value: probeArtifact({
            id: 'Kitten',
            issues: [
              catRecord({
                repairDisposition: 'shipped',
                issueId: 'adjudicated/nap',
                refined: true,
                introducedDefects: {
                  heardProbers: 3,
                  configuredProbers: 3,
                  regions: [],
                },
              },),
              catRecord({
                repairDisposition: 'shipped',
                issueId: 'adjudicated/chase',
                introducedDefects: {
                  heardProbers: 3,
                  configuredProbers: 3,
                  regions: [],
                },
              },),
            ],
          },),
          path: 'Kitten',
        },);

        expect(reading.owned
          .map(function toRefined(entry,) {
            return entry.refined;
          },),).toEqual([
            true,
            false,
          ],);
      },
    },),

    it({
      name: 'THROWS when a declared count disagrees with the claim list it is '
        + 'the tally of. The screen derives one from the other, so they are a '
        + 'single fact written twice, but they are read by different consumers: '
        + 'the CLAIMS report sums the counts while the majority rule reads the '
        + 'claims. A disagreement makes a region report one corroboration and '
        + 'flag nothing, or flag a majority while reporting none, and both look '
        + 'like ordinary output',
      fn: async () => {
        /**
         Reads a region whose declared count and claim list disagree.
         */
        function readsDisagreeingCount() {
          readArtifactProbe({
            value: probeArtifact({
              id: 'Kitten',
              issues: [
                catRecord({
                  repairDisposition: 'shipped',
                  introducedDefects: {
                    heardProbers: 3,
                    configuredProbers: 3,
                    regions: [
                      catTally({
                        envelopeId: 'envelope/nap',
                        // Says two were upheld while listing one.
                        corroborated: 2,
                        claims: [catClaim({ modelId: 'cat/one', },),],
                      },),
                    ],
                  },
                },),
              ],
            },),
            path: 'Kitten',
          },);
        }

        expect(readsDisagreeingCount,).toThrow(ArtifactParseError,);
        expect(readsDisagreeingCount,).toThrow('.regions[0]',);
      },
    },),

    it({
      name: 'READS AN ABSENT preExisting AS ZERO, which is what every artifact '
        + 'settled before that outcome existed carries. Every other count here '
        + 'is required, and requiring this one would make the whole earlier '
        + 'population unreadable by the same reader that still has to score it. '
        + 'Zero is what those runs meant rather than a convenience: they '
        + 'rendered the accepted issues into the prompt, so no claim of theirs '
        + 'could be dismissed as restating one',
      fn: async () => {
        /**
         Tally as this fixture builds it, with every count present.
         */
        const declared = catTally({ envelopeId: 'envelope/nap', },);

        /**
         The same tally with the key gone rather than present and zero, which
         is the difference between an older artifact and a current one.
         */
        const beforePreExisting = Object.fromEntries(Object
          .entries(declared,)
          .filter(function isNotPreExisting([field,],): boolean {
            return field !== 'preExisting';
          },),);

        /**
         What the reader made of it.
         */
        const reading = readArtifactProbe({
          value: probeArtifact({
            id: 'Kitten',
            issues: [
              catRecord({
                repairDisposition: 'shipped',
                introducedDefects: {
                  heardProbers: 3,
                  configuredProbers: 3,
                  regions: [beforePreExisting,],
                },
              },),
            ],
          },),
          path: 'Kitten',
        },);

        expect(reading.readings[0]
          ?.regions[0]
          ?.preExisting,).toBe(0,);
      },
    },),

    it({
      name: 'THROWS on a disposition the pipeline never writes rather than '
        + 'filing it under not-shipped. A typo is not another disposition, it '
        + 'means writer and reader disagree, and quietly excluding the record '
        + 'shrinks the denominator of a rate with nothing recording that it '
        + 'happened',
      fn: async () => {
        /**
         Reads a record under a disposition the pipeline never writes.
         */
        function readsUnknownDisposition() {
          readArtifactProbe({
            value: probeArtifact({
              id: 'Kitten',
              issues: [catRecord({ repairDisposition: 'shippped', },),],
            },),
            path: 'Kitten',
          },);
        }

        expect(readsUnknownDisposition,).toThrow(ArtifactParseError,);
        expect(readsUnknownDisposition,).toThrow('.repairDisposition',);
      },
    },),

    it({
      name: 'treats an absent probe field as ordinary and counts it, because '
        + 'an artifact predating the probe is not a fault while a run whose '
        + 'probe never fired still has to be visible',
      fn: async () => {
        const reading = readArtifactProbe({
          value: probeArtifact({
            id: 'Kitten',
            issues: [catRecord({ repairDisposition: 'shipped', },),],
          },),
          path: 'Kitten',
        },);
        expect(reading.readings,).toHaveLength(0,);
        expect(reading.repairShippedRecords,).toBe(1,);
        expect(reading.repairUnprobedRecords,).toBe(1,);
      },
    },),

    it({
      name: 'throws when a PRESENT probe field is malformed, since that means '
        + 'writer and reader disagree and every count downstream is unsound',
      fn: async () => {
        /**
         Reads a probe field that is present and malformed.
         */
        function readsMalformed() {
          readArtifactProbe({
            value: probeArtifact({
              id: 'Kitten',
              issues: [
                catRecord({
                  repairDisposition: 'shipped',
                  introducedDefects: {
                    heardProbers: 3,
                    configuredProbers: 'three',
                    regions: [],
                  },
                },),
              ],
            },),
            path: 'Kitten',
          },);
        }

        expect(readsMalformed,).toThrow(ArtifactParseError,);
        expect(readsMalformed,).toThrow('.configuredProbers',);
      },
    },),

    it({
      name: 'refuses a fractional count rather than rounding it, because a '
        + 'fraction means the field is not the tally the reader thinks it is',
      fn: async () => {
        /**
         Reads a prober count written as a fraction.
         */
        function readsFraction() {
          readArtifactProbe({
            value: probeArtifact({
              id: 'Kitten',
              issues: [
                catRecord({
                  repairDisposition: 'shipped',
                  introducedDefects: {
                    heardProbers: 2.5,
                    configuredProbers: 3,
                    regions: [],
                  },
                },),
              ],
            },),
            path: 'Kitten',
          },);
        }

        expect(readsFraction,).toThrow(ArtifactParseError,);
        expect(readsFraction,).toThrow('.heardProbers',);
      },
    },),

    it({
      name: 'reads an entry whose repair lane FILED NOTHING as a zero rather than a refusal, '
        + 'because a lane that found no issue is an ordinary outcome and a rate over no records '
        + 'is honestly zero. NOT the same as an absent list, which is a writer this reader does '
        + 'not know and is refused; that refusal is pinned beside the reader that draws these '
        + 'records. The distinction only became expressible at version 2, where a lane always '
        + 'writes a result and so CAN say it filed nothing',
      fn: async () => {
        const reading = readArtifactProbe({
          value: probeArtifact({ id: 'Kitten', },),
          path: 'Kitten',
        },);
        expect(reading.readings,).toHaveLength(0,);
        expect(reading.repairShippedRecords,).toBe(0,);
        expect(reading.repairUnprobedRecords,).toBe(0,);
      },
    },),

    it({
      name: 'READS a shipped record\'s naturalness-rewrite probe beside the accuracy readings, and KEEPS the lane\'s '
        + 'findings that are text while passing over one that is not, since that count must survive a run whose '
        + 'findings drifted',
      fn: async () => {
        /**
         Rewrite probe the refined record carries.
         */
        const refinementDefects = {
          heardProbers: 3,
          configuredProbers: 3,
          regions: [catTally({ envelopeId: 'envelope/refined', },),],
        };
        const reading = readArtifactProbe({
          value: probeArtifact({
            id: 'Kitten',
            issues: [
              {
                ...catRecord({
                  repairDisposition: 'shipped',
                  refined: true,
                },),
                refinementDefects,
              },
            ],
            findings: ['stage-quorum-unmet (critic 0/6)', 7, 'whiskers-checked',],
          },),
          path: 'Kitten',
        },);
        expect(reading.refinementReadings,).toEqual([refinementDefects,],);
        expect(reading.findings,).toEqual(['stage-quorum-unmet (critic 0/6)', 'whiskers-checked',],);
        expect(reading.readings,).toHaveLength(0,);
        expect(reading.hasRewrites,).toBe(true,);
      },
    },),
  ],
},);
