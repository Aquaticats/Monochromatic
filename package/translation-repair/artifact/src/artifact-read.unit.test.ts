import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool';
import { parseSettledArtifact, } from '../dist/final/node/index.mjs';

/**
 Builds one issue record wrapper with the chosen fate and claim fields; extra
 artifact fields the parser ignores are included to mirror real artifacts.
 */
function catRecord(
  {
    status,
    issueId,
    category,
    summary,
    spans,
  }: {
    readonly status: string;
    readonly issueId: unknown;
    readonly category: string;
    readonly summary: string;
    readonly spans: readonly unknown[];
  },
): unknown {
  return {
    sliceIndex: 0,
    resolved: false,
    issue: {
      issueId,
      status,
      severity: 'minor',
      claims: [
        {
          claimId: 'claim/whisker',
          claim: {
            category,
            severity: 'minor',
            summary,
            spans,
          },
        },
      ],
      tallies: {},
    },
  };
}

/**
 A well-formed accepted purring-omission record.
 */
function catAcceptedRecord(): unknown {
  return catRecord({
    status: 'accepted',
    issueId: 'adjudicated/purr',
    category: 'accuracy/omission',
    summary: 'A purr is dropped from the greeting.',
    spans: [
      { side: 'source', nodeId: 'block/0', quotedText: '呼噜', },
      { side: 'target', nodeId: 'block/0', quotedText: '', },
    ],
  },);
}

/**
 Wraps issue records in an artifact envelope with filler metadata.
 */
function catArtifact(
  { issues, }: { readonly issues: readonly unknown[]; },
): unknown {
  return {
    id: 'Kitten',
    tip: 'tip/1',
    corpusSha: 'sha/1',
    status: 'repaired',
    durationMs: 1,
    issues,
  };
}

await describe({
  name: parseSettledArtifact.name,
  children: [
    //region Accepted extraction

    it({
      name: 'returns id, status, and the accepted issues',
      fn: async () => {
        const parsed = parseSettledArtifact({
          value: catArtifact({ issues: [catAcceptedRecord(),], },),
        },);
        expect(parsed.id,).toBe('Kitten',);
        expect(parsed.status,).toBe('repaired',);
        expect(parsed.acceptedIssues,).toHaveLength(1,);
        expect(parsed.acceptedIssues[0]?.issue.issueId,).toBe('adjudicated/purr',);
        expect(parsed.acceptedIssues[0]?.issue.claims[0]?.claim.category,)
          .toBe('accuracy/omission',);
        expect(parsed.acceptedIssues[0]?.issue.claims[0]?.claim.spans,).toHaveLength(2,);
      },
    },),

    it({
      name:
        'hands back the repair READING rather than an absent field, so a caller can tell an artifact '
          + 'written before repair recording from a run that recorded one. This used to collapse into an '
          + 'optional property, which handed the caller back the question the reading had just answered',
      fn: async () => {
        /**
         An artifact from before repair recording existed, whose records carry
         no repair fields at all.
         */
        const older = parseSettledArtifact({
          value: catArtifact({ issues: [catAcceptedRecord(),], },),
        },);
        expect(older.acceptedIssues[0]?.repair
          .kind,).toBe('unrecorded',);

        /**
         The same issue in a run that recorded what it shipped.
         */
        const recorded = parseSettledArtifact({
          value: catArtifact({
            issues: [
              {
                ...(catAcceptedRecord() as Record<string, unknown>),
                resolved: true,
                repairDisposition: 'shipped',
                refined: false,
                repairRegions: [
                  {
                    envelopeId: 'envelope/purr',
                    issueIds: ['adjudicated/purr',],
                    before: 'The cat greets you.',
                    editorAfter: 'The cat greets you with a purr.',
                  },
                ],
              },
            ],
          },),
        },);

        /**
         What that run recorded about the one accepted issue.
         */
        const reading = recorded.acceptedIssues[0]?.repair;
        if (reading?.kind !== 'recorded')
          throw new Error('expected a recorded reading',);

        expect(reading.repair
          .disposition,).toBe('shipped',);
        expect(reading.repair
          .regions[0]?.editorAfter,).toBe('The cat greets you with a purr.',);
      },
    },),

    it({
      name: 'excludes rejected and needs-human issues from the denominator',
      fn: async () => {
        const parsed = parseSettledArtifact({
          value: catArtifact({
            issues: [
              catAcceptedRecord(),
              catRecord({
                status: 'rejected',
                issueId: 'adjudicated/hiss',
                category: 'accuracy/addition',
                summary: 'A hiss is fabricated.',
                spans: [{ side: 'target', quotedText: 'hiss', },],
              },),
              catRecord({
                status: 'needs-human',
                issueId: 'adjudicated/maybe',
                category: 'style/awkward-phrasing',
                summary: 'A meow reads oddly.',
                spans: [{ side: 'target', quotedText: 'meow', },],
              },),
            ],
          },),
        },);
        expect(parsed.acceptedIssues,).toHaveLength(1,);
        expect(parsed.acceptedIssues[0]?.issue.issueId,).toBe('adjudicated/purr',);
      },
    },),

    it({
      name: 'tolerates a malformed NON-accepted issue since it is not counted',
      fn: async () => {
        const parsed = parseSettledArtifact({
          value: catArtifact({
            issues: [
              catAcceptedRecord(),
              catRecord({
                status: 'rejected',
                issueId: 42,
                category: 'accuracy/addition',
                summary: 'Malformed id but rejected, so skipped before reading it.',
                spans: [],
              },),
            ],
          },),
        },);
        expect(parsed.acceptedIssues,).toHaveLength(1,);
      },
    },),

    //endregion Accepted extraction

    //region Loud aborts

    it({
      name: 'throws on a malformed ACCEPTED issue rather than dropping it',
      fn: async () => {
        let caught: unknown;
        try {
          parseSettledArtifact({
            value: catArtifact({
              issues: [
                catRecord({
                  status: 'accepted',
                  issueId: 42,
                  category: 'accuracy/omission',
                  summary: 'Accepted but with a non-string id.',
                  spans: [{ side: 'source', quotedText: '呼噜', },],
                },),
              ],
            },),
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(ArtifactParseError,);
        expect((caught as Error).message,).toContain('issueId',);
      },
    },),

    it({
      name: 'throws on an accepted issue span with an unknown side',
      fn: async () => {
        let caught: unknown;
        try {
          parseSettledArtifact({
            value: catArtifact({
              issues: [
                catRecord({
                  status: 'accepted',
                  issueId: 'adjudicated/purr',
                  category: 'accuracy/omission',
                  summary: 'Accepted but the span side is bogus.',
                  spans: [{ side: 'middle', quotedText: '呼噜', },],
                },),
              ],
            },),
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(ArtifactParseError,);
        expect((caught as Error).message,).toContain('side',);
      },
    },),

    it({
      name: 'throws when the issue fate cannot even be read',
      fn: async () => {
        let caught: unknown;
        try {
          parseSettledArtifact({
            value: catArtifact({ issues: [{ sliceIndex: 0, issue: {}, resolved: false, },], },),
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(ArtifactParseError,);
        expect((caught as Error).message,).toContain('status',);
      },
    },),

    it({
      name: 'throws when the top-level shape is not an artifact',
      fn: async () => {
        let caught: unknown;
        try {
          parseSettledArtifact({ value: 'not an artifact', },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(ArtifactParseError,);
      },
    },),

    //endregion Loud aborts
  ],
},);
