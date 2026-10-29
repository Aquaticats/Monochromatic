/**
 Tests for the repeat pairings and the band the settled audit reads off them.

 The band exists because the headline of the settled audit is a comparison, and a
 comparison resolves nothing narrower than the spread the instrument moves
 through on unchanged input. The sharpest cases here are the REFUSALS, since
 a pairing that should not have happened reports a band narrower than the
 truth and makes every comparison look better resolved than it is: two rows
 that merely share a slot, two rows whose text moved, and two rows that
 predate the recorded identity and would otherwise pair through their shared
 absence.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  auditRepeatsAcross,
  auditRepeatsWithin,
  digestAuditedText,
  repeatBandOf,
  sameAuditedText,
  type SettledAuditRow,
  textIdentityOf,
} from '../../dist/final/node/index.mjs';
import { statedRefusalMessage, } from '@monochromatic-dev/translation-repair-glossary/ts/stated-refusal-message.test-fixture.ts';

/**
 Builds one audited slice carrying a stated number of anchored claims.

 @param runSet - archive subdirectory

 @param entryId - corpus entry

 @param sliceIndex - slice index

 @param claims - how many claims anchored, over one voice

 @param texts - what the audit was shown, omitted to leave it unrecorded

 @returns Row shaped as the probe persists it

 @example
 ```ts
 const row = rowFor({ runSet: 'first', entryId: 'mittens', sliceIndex: 0, claims: 1, },);
 ```
 */
function rowFor(
  {
    runSet,
    entryId,
    sliceIndex,
    claims,
    texts,
  }: {
    readonly runSet: string;
    readonly entryId: string;
    readonly sliceIndex: number;
    readonly claims: number;
    readonly texts?: {
      readonly sourceText: string;
      readonly candidateText: string;
      readonly referenceContext: string;
    };
  },
): SettledAuditRow {
  return {
    runSet,
    entryId,
    sliceIndex,
    deliveryKind: 'replacement-shipped',
    auditsArchiveText: false,
    artifactDigest: 'sha256-tree-v1:cafef00d',
    corpusSha: 'b'.repeat(40,),
    identityKind: 'declared',
    ...((texts === undefined) ? {} : { textIdentity: digestAuditedText(texts,), }),
    report: {
      corroborated: [],
      agreed: [],
      near: [],
      findings: [],
      rows: [{
        modelId: 'hf:cat/Tabby-1',
        verdict: (claims === 0) ? 'no-defect-found' : 'defects-found',
        findings: Array.from(
          { length: claims, },
          function asFinding(): Record<string, unknown> {
            return {
              category: 'omission',
              source: { kind: 'unused', },
              candidate: { kind: 'unused', },
              reason: 'the cat left',
            };
          },
        ),
        dropped: [],
      },],
    },
  } as unknown as SettledAuditRow;
}

/**
 One pair of texts, used wherever two rows are meant to match.
 */
const SAME_TEXTS = {
  sourceText: '毛毛跳上窗台。',
  candidateText: 'Mittens jumped onto the windowsill.',
  referenceContext: '',
} as const;

/**
 The same pair, shown with what the page cites.
 */
const SAME_TEXTS_CITED = {
  ...SAME_TEXTS,
  referenceContext: 'Reference 1 (https://example.org/mittens): Mittens naps on sills.',
} as const;

/**
 A different rendering of the same original.
 */
const OTHER_TEXTS = {
  sourceText: '毛毛跳上窗台。',
  candidateText: 'Mittens hopped up on the sill.',
  referenceContext: '',
} as const;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: digestAuditedText.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'DIGESTS BOTH SIDES SEPARATELY, so a pair whose original matches and whose rendering '
            + 'does not is a comparison of two renderings rather than two readings of one text',
          fn: async () => {
            /**
             Same original, different English.
             */
            const mine = digestAuditedText(SAME_TEXTS,);

            /**
             The other rendering.
             */
            const theirs = digestAuditedText(OTHER_TEXTS,);

            expect(mine.kind,).toBe('digested',);
            if ((mine.kind !== 'digested') || (theirs.kind !== 'digested'))
              throw new Error('both fixtures are digested by construction',);
            expect(mine.source,).toBe(theirs.source,);
            expect(mine.candidate,).not.toBe(theirs.candidate,);
          },
        },),

        it({
          name: 'CARRIES NO TEXT, since a run file is read, grepped and quoted into docs while the '
            + 'corpus itself goes only to the production provider',
          fn: async () => {
            /**
             What lands on the row.
             */
            const identity = digestAuditedText(SAME_TEXTS,);
            if (identity.kind !== 'digested')
              throw new Error('digested by construction',);

            expect(identity.source.includes('毛毛',),).toBe(false,);
            expect(identity.candidate.includes('Mittens',),).toBe(false,);
            expect(identity.source.startsWith('sha256-audited-v1:',),).toBe(true,);
          },
        },),

        // LEDGER B29: the audit now shows what the page cites, and an auditor
        // shown references answers a different question from one shown none.
        it({
          name: 'DIGESTS THE REFERENCES WHERE SOME WERE SHOWN and records nothing for them where none were, so a '
            + 'row persisted before the audit showed references keys exactly as one shown none today',
          fn: async () => {
            /**
             Shown none.
             */
            const none = digestAuditedText(SAME_TEXTS,);

            /**
             Shown what the page cites.
             */
            const cited = digestAuditedText(SAME_TEXTS_CITED,);

            expect(Object.keys(none,),).toStrictEqual(['kind', 'source', 'candidate',],);
            if (cited.kind !== 'digested')
              throw new Error('digested by construction',);
            expect(cited.references?.startsWith('sha256-audited-v1:',),).toBe(true,);
            expect(cited.references?.includes('Mittens',),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: textIdentityOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REPORTS AN OLDER ROW AS UNRECORDED rather than throwing, because a run persisted '
            + 'before this field existed still answers every other reading',
          fn: async () => {
            expect(textIdentityOf({
              row: rowFor({
                runSet: 'first',
                entryId: 'mittens',
                sliceIndex: 0,
                claims: 1,
              },),
            },).kind,).toBe('unrecorded',);
          },
        },),
      ],
    },),

    describe({
      name: sameAuditedText.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES TO MATCH TWO UNRECORDED ROWS, which is the whole reason the field is a tagged '
            + 'union: comparing two absences for equality would pair rows by their shared lack of '
            + 'evidence and read every such pair as one text audited twice',
          fn: async () => {
            /**
             Two rows from a run that predates the field.
             */
            const left = rowFor({
              runSet: 'first',
              entryId: 'mittens',
              sliceIndex: 0,
              claims: 1,
            },);

            /**
             Its counterpart.
             */
            const right = rowFor({
              runSet: 'second',
              entryId: 'mittens',
              sliceIndex: 0,
              claims: 4,
            },);

            expect(sameAuditedText({
              left,
              right,
            },),).toBe(false,);
          },
        },),
        it({
          name: 'REFUSES TO MATCH A ROW SHOWN CITED REFERENCES with one shown none over the same pair, and matches '
            + 'two shown the same references',
          fn: async () => {
            /**
             Row shown the references.
             */
            const cited = rowFor({ runSet: 'first', entryId: 'mittens', sliceIndex: 0, claims: 0, texts: SAME_TEXTS_CITED, },);

            /**
             Same pair shown none.
             */
            const none = rowFor({ runSet: 'second', entryId: 'mittens', sliceIndex: 0, claims: 1, texts: SAME_TEXTS, },);

            /**
             Same pair shown the same references again.
             */
            const again = rowFor({ runSet: 'third', entryId: 'mittens', sliceIndex: 0, claims: 2, texts: SAME_TEXTS_CITED, },);

            expect(sameAuditedText({ left: cited, right: none, },),).toBe(false,);
            expect(sameAuditedText({ left: none, right: cited, },),).toBe(false,);
            expect(sameAuditedText({ left: cited, right: again, },),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: auditRepeatsWithin.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PAIRS ONE TEXT AUDITED TWICE across two run sets of one entry, which is the free '
            + 'repeat two artifacts of one entry already contain',
          fn: async () => {
            /**
             Same slice of one entry, settled twice, carrying the same characters.
             */
            const pairs = auditRepeatsWithin({
              rows: [
                rowFor({
                  runSet: 'first',
                  entryId: 'mittens',
                  sliceIndex: 0,
                  claims: 1,
                  texts: SAME_TEXTS,
                },),
                rowFor({
                  runSet: 'second',
                  entryId: 'mittens',
                  sliceIndex: 0,
                  claims: 4,
                  texts: SAME_TEXTS,
                },),
              ],
            },);

            expect(pairs.length,).toBe(1,);
            expect(pairs[0]?.entryId,).toBe('mittens',);
            expect(pairs[0]?.left.claimed,).toBe(1,);
            expect(pairs[0]?.right.claimed,).toBe(4,);
          },
        },),

        it({
          name: 'REFUSES TO PAIR A SLOT WHOSE TEXT DIFFERS, because two artifacts of one entry can '
            + 'agree on the original and ship different English there, and reading that as a repeat '
            + 'would report a spread the instrument never showed',
          fn: async () => {
            expect(auditRepeatsWithin({
              rows: [
                rowFor({
                  runSet: 'first',
                  entryId: 'mittens',
                  sliceIndex: 0,
                  claims: 1,
                  texts: SAME_TEXTS,
                },),
                rowFor({
                  runSet: 'second',
                  entryId: 'mittens',
                  sliceIndex: 0,
                  claims: 4,
                  texts: OTHER_TEXTS,
                },),
              ],
            },).length,).toBe(0,);
          },
        },),

        it({
          name: 'REFUSES TO PAIR TWO ROWS OF ONE RUN SET, since a run set holds each slice once and a '
            + 'pair inside it would have to be the same row against itself',
          fn: async () => {
            expect(auditRepeatsWithin({
              rows: [
                rowFor({
                  runSet: 'first',
                  entryId: 'mittens',
                  sliceIndex: 0,
                  claims: 1,
                  texts: SAME_TEXTS,
                },),
                rowFor({
                  runSet: 'first',
                  entryId: 'mittens',
                  sliceIndex: 0,
                  claims: 4,
                  texts: SAME_TEXTS,
                },),
              ],
            },).length,).toBe(0,);
          },
        },),

        it({
          name: 'REFUSES EVERY PAIR IN A RUN THAT RECORDED NO TEXT IDENTITY, so an older run reports '
            + 'no band at all rather than a band built from slot equality',
          fn: async () => {
            expect(auditRepeatsWithin({
              rows: [
                rowFor({
                  runSet: 'first',
                  entryId: 'mittens',
                  sliceIndex: 0,
                  claims: 1,
                },),
                rowFor({
                  runSet: 'second',
                  entryId: 'mittens',
                  sliceIndex: 0,
                  claims: 4,
                },),
              ],
            },).length,).toBe(0,);
          },
        },),
      ],
    },),

    describe({
      name: auditRepeatsAcross.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PAIRS TWO RUNS SUBJECT BY SUBJECT, keyed on the run set as well as the entry and the '
            + 'slice so two artifacts of one entry are never crossed with each other',
          fn: async () => {
            /**
             One subject, bought by two runs.
             */
            const {
              paired,
              textMoved,
            } = auditRepeatsAcross({
              first: [rowFor({
                runSet: 'first',
                entryId: 'mittens',
                sliceIndex: 0,
                claims: 1,
                texts: SAME_TEXTS,
              },),],
              second: [rowFor({
                runSet: 'first',
                entryId: 'mittens',
                sliceIndex: 0,
                claims: 5,
                texts: SAME_TEXTS,
              },),],
            },);

            expect(paired.length,).toBe(1,);
            expect(textMoved.length,).toBe(0,);
            expect(paired[0]?.left.claimed,).toBe(1,);
            expect(paired[0]?.right.claimed,).toBe(5,);
          },
        },),

        it({
          name: 'NAMES A SLOT WHOSE TEXT MOVED instead of pairing it, because the archive changing '
            + 'between two runs invalidates that subject as a band measurement and is worth knowing',
          fn: async () => {
            /**
             Same slot, different rendering, which means the archive moved.
             */
            const {
              paired,
              textMoved,
            } = auditRepeatsAcross({
              first: [rowFor({
                runSet: 'first',
                entryId: 'mittens',
                sliceIndex: 3,
                claims: 1,
                texts: SAME_TEXTS,
              },),],
              second: [rowFor({
                runSet: 'first',
                entryId: 'mittens',
                sliceIndex: 3,
                claims: 5,
                texts: OTHER_TEXTS,
              },),],
            },);

            expect(paired.length,).toBe(0,);
            expect(textMoved,).toEqual(['first/mittens#3',],);
          },
        },),

        it({
          name: 'SEPARATES "the archive moved" FROM "nobody recorded what this saw", because a run '
            + 'that predates the recorded identity would otherwise be reported as forty slots whose '
            + 'text changed under it, which is a confident statement about the archive made from the '
            + 'absence of evidence about the run',
          fn: async () => {
            /**
             An older run against a current one: same subject, no recorded identity on one side.
             */
            const {
              paired,
              textMoved,
              unverifiable,
            } = auditRepeatsAcross({
              first: [rowFor({
                runSet: 'first',
                entryId: 'mittens',
                sliceIndex: 2,
                claims: 1,
              },),],
              second: [rowFor({
                runSet: 'first',
                entryId: 'mittens',
                sliceIndex: 2,
                claims: 5,
                texts: SAME_TEXTS,
              },),],
            },);

            expect(paired.length,).toBe(0,);
            expect(textMoved.length,).toBe(0,);
            expect(unverifiable,).toEqual(['first/mittens#2',],);
          },
        },),

        it({
          name: 'DROPS A SUBJECT ONLY ONE RUN BOUGHT, since a capped run holds a prefix of the other '
            + 'and inventing a side for the rest would report a gap nobody measured',
          fn: async () => {
            expect(auditRepeatsAcross({
              first: [
                rowFor({
                  runSet: 'first',
                  entryId: 'mittens',
                  sliceIndex: 0,
                  claims: 1,
                  texts: SAME_TEXTS,
                },),
                rowFor({
                  runSet: 'first',
                  entryId: 'mittens',
                  sliceIndex: 1,
                  claims: 2,
                  texts: SAME_TEXTS,
                },),
              ],
              second: [rowFor({
                runSet: 'first',
                entryId: 'mittens',
                sliceIndex: 0,
                claims: 3,
                texts: SAME_TEXTS,
              },),],
            },).paired.length,).toBe(1,);
          },
        },),

        it({
          name: 'REFUSES A LATER RUN THAT NAMES ONE SUBJECT ON TWO ROWS, naming the rows by position and no '
            + 'entry, since the later row would replace the first in the lookup the pairing reads',
          fn: async () => {
            expect(statedRefusalMessage({
              read: function readsRepeatedLater(): unknown {
                return auditRepeatsAcross({
                  first: [rowFor({
                    runSet: 'first',
                    entryId: 'mittens',
                    sliceIndex: 0,
                    claims: 1,
                    texts: SAME_TEXTS,
                  },),],
                  second: [
                    rowFor({
                      runSet: 'first',
                      entryId: 'mittens',
                      sliceIndex: 0,
                      claims: 2,
                      texts: SAME_TEXTS,
                    },),
                    rowFor({
                      runSet: 'first',
                      entryId: 'whiskers',
                      sliceIndex: 0,
                      claims: 3,
                      texts: SAME_TEXTS,
                    },),
                    rowFor({
                      runSet: 'first',
                      entryId: 'mittens',
                      sliceIndex: 0,
                      claims: 4,
                      texts: SAME_TEXTS,
                    },),
                  ],
                },);
              },
            },),).toBe(
              'the later run names one subject on two rows, row 0 and row 2, so which of them the pairing '
                + 'should read cannot be told',
            );
          },
        },),

        it({
          name: 'REFUSES AN EARLIER RUN THAT NAMES ONE SUBJECT ON TWO ROWS, since it would pair the later '
            + 'row with both and count one audit twice in the band',
          fn: async () => {
            expect(statedRefusalMessage({
              read: function readsRepeatedEarlier(): unknown {
                return auditRepeatsAcross({
                  first: [
                    rowFor({
                      runSet: 'first',
                      entryId: 'mittens',
                      sliceIndex: 0,
                      claims: 1,
                      texts: SAME_TEXTS,
                    },),
                    rowFor({
                      runSet: 'first',
                      entryId: 'mittens',
                      sliceIndex: 0,
                      claims: 2,
                      texts: SAME_TEXTS,
                    },),
                  ],
                  second: [rowFor({
                    runSet: 'first',
                    entryId: 'mittens',
                    sliceIndex: 0,
                    claims: 3,
                    texts: SAME_TEXTS,
                  },),],
                },);
              },
            },),).toBe(
              'the earlier run names one subject on two rows, row 0 and row 1, so which of them the pairing '
                + 'should read cannot be told',
            );
          },
        },),

        it({
          name: 'PAIRS A RUN WRITTEN BEFORE THE RENAME OF `chunkIndex` TO `sliceIndex` with a run of this '
            + 'generation by that value, since the rename changed the field\'s name and not what it holds',
          fn: async () => {
            /**
             Row as the earlier generation wrote it: its slice index under the old name.
             */
            const { sliceIndex: renamed, ...withoutSliceIndex } = rowFor({
              runSet: 'first',
              entryId: 'mittens',
              sliceIndex: 4,
              claims: 1,
              texts: SAME_TEXTS,
            },);
            const old = [{ ...withoutSliceIndex, chunkIndex: renamed, },] as unknown as readonly SettledAuditRow[];
            const { paired, } = auditRepeatsAcross({
              first: old,
              second: [rowFor({
                runSet: 'first',
                entryId: 'mittens',
                sliceIndex: 4,
                claims: 5,
                texts: SAME_TEXTS,
              },),],
            },);
            expect(paired.map(function sliceOf(pair,): number {
              return pair.sliceIndex;
            },),).toEqual([4,],);
            expect(paired[0]?.left.claimed,).toBe(1,);
            expect(paired[0]?.right.claimed,).toBe(5,);
          },
        },),

        it({
          name: 'REFUSES A ROW OF A RUN FILE WITH NO NAMEABLE SUBJECT, since a key built from a missing '
            + 'entry or a non-numeric slice would join rows that name nothing',
          fn: async () => {
            /**
             Rows as a run file parses to them: one with its entry id written as a number.
             */
            const parsed = [{
              ...rowFor({
                runSet: 'first',
                entryId: 'mittens',
                sliceIndex: 0,
                claims: 1,
                texts: SAME_TEXTS,
              },),
              entryId: 7,
            },] as unknown as readonly SettledAuditRow[];
            expect(statedRefusalMessage({
              read: function readsUnnameable(): unknown {
                return auditRepeatsAcross({
                  first: [],
                  second: parsed,
                },);
              },
            },),).toBe(
              'row 0 of the later run does not name its subject by a text run set, a text entry id and a '
                + 'numeric slice index',
            );
          },
        },),
      ],
    },),

    describe({
      name: repeatBandOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REPORTS ZERO PAIRS AS ZERO PAIRS, which the printer turns into a refusal to quote a '
            + 'band rather than into a row of zeroes that reads as perfect agreement',
          fn: async () => {
            expect(repeatBandOf({ pairs: [], },).pairs,).toBe(0,);
            expect(repeatBandOf({ pairs: [], },).widest,).toBe(0,);
          },
        },),

        it({
          name: 'MEASURES THE GAP IN BOTH DIRECTIONS, because neither side of a repeat is the '
            + 'reference: both are single readings of one text and calling either correct is the '
            + 'assumption the measurement exists to avoid',
          fn: async () => {
            /**
             Two repeats, one where the later run claimed more and one where it claimed less.
             */
            const band = repeatBandOf({
              pairs: auditRepeatsWithin({
                rows: [
                  rowFor({
                    runSet: 'first',
                    entryId: 'mittens',
                    sliceIndex: 0,
                    claims: 1,
                    texts: SAME_TEXTS,
                  },),
                  rowFor({
                    runSet: 'second',
                    entryId: 'mittens',
                    sliceIndex: 0,
                    claims: 5,
                    texts: SAME_TEXTS,
                  },),
                  rowFor({
                    runSet: 'first',
                    entryId: 'mittens',
                    sliceIndex: 1,
                    claims: 3,
                    texts: OTHER_TEXTS,
                  },),
                  rowFor({
                    runSet: 'second',
                    entryId: 'mittens',
                    sliceIndex: 1,
                    claims: 1,
                    texts: OTHER_TEXTS,
                  },),
                ],
              },),
            },);

            expect(band.pairs,).toBe(2,);
            expect(band.widest,).toBe(4,);
            expect(band.totalGap,).toBe(6,);
            expect(band.agreedExactly,).toBe(0,);
          },
        },),

        it({
          name: 'COUNTS THE PAIRS WHERE ONE AUDIT WAS SILENT AND THE OTHER WAS NOT, which is the '
            + 'sharpest form of the spread: a gate reading "claimed at least one" would flip on those '
            + 'subjects for no reason in the text',
          fn: async () => {
            /**
             One repeat where a voice went quiet, one where both spoke.
             */
            const band = repeatBandOf({
              pairs: auditRepeatsWithin({
                rows: [
                  rowFor({
                    runSet: 'first',
                    entryId: 'mittens',
                    sliceIndex: 0,
                    claims: 0,
                    texts: SAME_TEXTS,
                  },),
                  rowFor({
                    runSet: 'second',
                    entryId: 'mittens',
                    sliceIndex: 0,
                    claims: 2,
                    texts: SAME_TEXTS,
                  },),
                  rowFor({
                    runSet: 'first',
                    entryId: 'mittens',
                    sliceIndex: 1,
                    claims: 2,
                    texts: OTHER_TEXTS,
                  },),
                  rowFor({
                    runSet: 'second',
                    entryId: 'mittens',
                    sliceIndex: 1,
                    claims: 2,
                    texts: OTHER_TEXTS,
                  },),
                ],
              },),
            },);

            expect(band.silentOnOneSide,).toBe(1,);
            expect(band.agreedExactly,).toBe(1,);
          },
        },),
      ],
    },),
  ],
},);
