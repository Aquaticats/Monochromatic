/**
 Tests for the reviewed fidelity references: building one from pinned files
 and its manifest entry, selecting manifest entries, expanding references
 into the fixed trial matrix, and the checked-in manifest's own invariants.
 The request check and the pinned corpus read have files of their own.

 Every refusal case names the verification boundary that fired and asserts
 the whole message, so a refusal raised at another boundary fails the case
 that expects this one. Fixtures are cat-themed invention. No corpus content
 appears here; the manifest cases read the checked-in manifest's metadata
 only.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  alterSharedNumber,
  deleteOneSentence,
  type FidelityDamageKind,
  insertBorrowedSentence,
} from '@monochromatic-dev/translation-repair-repair';
import {
  buildReviewedFidelityReference,
  type FidelityReferenceEdit,
  type FidelityReferenceOperation,
  type FidelityReferenceSpec,
  MIN_REVIEWED_REFERENCE_CHARS,
  REVIEWED_FIDELITY_REFERENCES,
  reviewedFidelityTrials,
  selectReviewedFidelitySpecs,
} from '../dist/final/node/index.mjs';

import {
  expectRefusal,
  REVIEW_DONOR,
  REVIEW_REFERENCE,
  REVIEW_SOURCE,
  reviewedFixture,
  summaryOf,
} from './fidelity-reference.test-fixture.ts';

//region Rebound fixtures
// Reviewed inputs rebuilt around a chosen archive text, so a case can vary the
// reference, its edits or where the donor paragraph sits.

/**
 Where the donor paragraph sits in the archive file: after the reviewed range or before it.
 */
type DonorLayout = 'after' | 'before';

/**
 Both donor placements.
 */
const LAYOUTS: readonly DonorLayout[] = [
  'after',
  'before',
];

/**
 Width of the donor span the overlap cases cut from beside the reviewed range.
 */
const DONOR_SPAN_CHARS = 20;

/**
 Rebinds the invented fixture to an archive text, its reference and the edits
 between them, computing every hash and damage identity from the builders the
 production code runs.

 @param original - archive range as the corpus holds it, before folding and edits

 @param reference - reference the edits must produce from `original`

 @param edits - reviewed corrections, none for an unedited reference

 @param layout - where the donor paragraph sits against the range

 @returns Invented files and the spec that binds them

 @example
 ```ts
 const { sourceFile, archiveFile, spec, } = rebound({ original: REVIEW_REFERENCE, reference: REVIEW_REFERENCE, edits: [], layout: 'after', },);
 ```
 */
function rebound(
  {
    original,
    reference,
    edits,
    layout,
  }: {
    readonly original: string;
    readonly reference: string;
    readonly edits: readonly FidelityReferenceEdit[];
    readonly layout: DonorLayout;
  },
): {
  readonly sourceFile: string;
  readonly archiveFile: string;
  readonly spec: FidelityReferenceSpec;
} {
  /**
   The invented source and spec this rebinds.
   */
  const fixture = reviewedFixture();
  /**
   Text between the archive range and the donor.
   */
  const gap = '\n\n';
  /**
   Whether the donor follows the archive range.
   */
  const donorFollows = layout === 'after';
  /**
   Archive range's start in the file.
   */
  const archiveStart = donorFollows ? 0 : REVIEW_DONOR.length + gap.length;
  /**
   Donor's start in the file.
   */
  const donorStart = donorFollows ? original.length + gap.length : 0;
  /**
   Variants the reviewed damage families build from the reference.
   */
  const damages = [
    deleteOneSentence({ cleanText: reference, },),
    insertBorrowedSentence({
      cleanText: reference,
      donorTexts: [REVIEW_DONOR,],
    },),
    alterSharedNumber({
      cleanText: reference,
      sourceText: REVIEW_SOURCE,
    },),
  ];
  return {
    sourceFile: fixture.sourceFile,
    archiveFile: donorFollows ? `${original}${gap}${REVIEW_DONOR}` : `${REVIEW_DONOR}${gap}${original}`,
    spec: {
      ...fixture.spec,
      archive: {
        startOffset: archiveStart,
        endOffset: archiveStart + original.length,
        hash: hashContent({ content: original, },),
      },
      referenceHash: hashContent({ content: reference, },),
      referenceChars: reference.length,
      edits,
      donor: {
        startOffset: donorStart,
        endOffset: donorStart + REVIEW_DONOR.length,
        hash: hashContent({ content: REVIEW_DONOR, },),
      },
      damages: damages.map(function identity(damage,) {
        if (damage.kind !== 'damaged')
          throw new Error('the invented reference must admit every reviewed damage family',);
        return {
          kind: damage.damageKind,
          hash: hashContent({ content: damage.damagedText, },),
          changedChars: damage.changedChars,
        };
      },),
    },
  };
}

/**
 Builds a reference from files and a spec, handing back what it threw.

 @param sourceFile - invented source file

 @param archiveFile - invented archive file

 @param spec - spec the build must refuse

 @returns What the build threw

 @throws {@link Error} when the build returns, since every caller expects a refusal

 @example
 ```ts
 const refusal = refusalOfBuild({ sourceFile, archiveFile, spec, },);
 ```
 */
function refusalOfBuild(
  {
    sourceFile,
    archiveFile,
    spec,
  }: {
    readonly sourceFile: string;
    readonly archiveFile: string;
    readonly spec: FidelityReferenceSpec;
  },
): unknown {
  return caught(function build(): unknown {
    return buildReviewedFidelityReference({
      sourceFile,
      archiveFile,
      spec,
    },);
  },);
}

/**
 The invented inputs with a donor span cut from the file beside the reviewed
 range, sharing the given count of characters with it, and no insertion
 variant, whose hash the cut text would change.

 @param layout - which side of the range the span lies on

 @param overlap - characters the span shares with the range, none where it
 only touches

 @returns Invented files and the spec that binds them

 @example
 ```ts
 const touching = donorSpanSpec({ layout: 'after', overlap: 0, },);
 ```
 */
function donorSpanSpec(
  {
    layout,
    overlap,
  }: {
    readonly layout: DonorLayout;
    readonly overlap: number;
  },
): {
  readonly sourceFile: string;
  readonly archiveFile: string;
  readonly spec: FidelityReferenceSpec;
} {
  /**
   Invented inputs with the donor paragraph beside the range.
   */
  const built = rebound({
    original: REVIEW_REFERENCE,
    reference: REVIEW_REFERENCE,
    edits: [],
    layout,
  },);
  /**
   Where the donor span starts.
   */
  const startOffset = layout === 'after'
    ? built.spec.archive.endOffset - overlap
    : (built.spec.archive.startOffset + overlap) - DONOR_SPAN_CHARS;
  /**
   Where the donor span ends.
   */
  const endOffset = startOffset + DONOR_SPAN_CHARS;
  return {
    ...built,
    spec: {
      ...built.spec,
      damages: built.spec.damages.filter((damage,) => damage.kind !== 'insertion',),
      donor: {
        startOffset,
        endOffset,
        hash: hashContent({
          content: built.archiveFile.slice(
            startOffset,
            endOffset,
          ),
        },),
      },
    },
  };
}

/**
 One way a spec drifts from its reviewed inputs and the boundary that must refuse it.
 */
type DriftRow = {
  /**
   What the drift is, as the case name states it.
   */
  readonly name: string;

  /**
   Boundary whose refusal the drift must raise.
   */
  readonly operation: FidelityReferenceOperation;

  /**
   Identifier the refusal names.
   */
  readonly referenceId: string;

  /**
   The invented spec with the drift applied.
   */
  readonly change: (spec: FidelityReferenceSpec,) => FidelityReferenceSpec;
};

/**
 Identifier of the invented reviewed reference.
 */
const INVENTED_ID = 'invented-reference';

/**
 Every drift of the invented spec, each with the boundary that must refuse it.
 */
const DRIFTS: readonly DriftRow[] = [
  {
    name: 'a source hash that no longer matches',
    operation: 'source',
    referenceId: INVENTED_ID,
    change: (spec,) => ({ ...spec, source: { ...spec.source, hash: 'wrong', }, }),
  },
  {
    name: 'a negative source start',
    operation: 'source',
    referenceId: INVENTED_ID,
    change: (spec,) => ({ ...spec, source: { ...spec.source, startOffset: -1, }, }),
  },
  {
    name: 'a fractional source start whose bytes would otherwise match',
    operation: 'source',
    referenceId: INVENTED_ID,
    change: (spec,) => ({ ...spec, source: { ...spec.source, startOffset: spec.source.startOffset + 0.5, }, }),
  },
  {
    name: 'a fractional source end whose bytes would otherwise match',
    operation: 'source',
    referenceId: INVENTED_ID,
    change: (spec,) => ({ ...spec, source: { ...spec.source, endOffset: spec.source.endOffset + 0.5, }, }),
  },
  {
    name: 'an empty source span whose hash is the empty text\'s',
    operation: 'source',
    referenceId: INVENTED_ID,
    change: (spec,) => ({
      ...spec,
      source: {
        ...spec.source,
        endOffset: spec.source.startOffset,
        hash: hashContent({ content: '', },),
      },
    }),
  },
  {
    name: 'an archive span that runs past the end of the file',
    operation: 'archive',
    referenceId: INVENTED_ID,
    change: (spec,) => ({ ...spec, archive: { ...spec.archive, endOffset: 100_000, }, }),
  },
  {
    name: 'an archive hash that no longer matches',
    operation: 'archive',
    referenceId: INVENTED_ID,
    change: (spec,) => ({ ...spec, archive: { ...spec.archive, hash: 'wrong', }, }),
  },
  {
    name: 'a reference hash that no longer matches',
    operation: 'reference',
    referenceId: INVENTED_ID,
    change: (spec,) => ({ ...spec, referenceHash: 'wrong', }),
  },
  {
    name: 'a reference length one character off',
    operation: 'reference',
    referenceId: INVENTED_ID,
    change: (spec,) => ({ ...spec, referenceChars: spec.referenceChars + 1, }),
  },
  {
    name: 'a donor end one past the file, whose clamped slice would still match the hash',
    operation: 'donor',
    referenceId: INVENTED_ID,
    change: (spec,) => ({ ...spec, donor: { ...spec.donor, endOffset: spec.donor.endOffset + 1, }, }),
  },
  {
    name: 'a donor hash that no longer matches',
    operation: 'donor',
    referenceId: INVENTED_ID,
    change: (spec,) => ({ ...spec, donor: { ...spec.donor, hash: 'wrong', }, }),
  },
  {
    name: 'a donor that is the reference\'s own archive range',
    operation: 'donor',
    referenceId: INVENTED_ID,
    change: (spec,) => ({ ...spec, donor: spec.archive, }),
  },
  {
    name: 'an empty damage list',
    operation: 'damage',
    referenceId: INVENTED_ID,
    change: (spec,) => ({ ...spec, damages: [], }),
  },
  {
    name: 'a damage family listed twice',
    operation: 'damage',
    referenceId: INVENTED_ID,
    change: (spec,) => ({ ...spec, damages: [...spec.damages, ...spec.damages,], }),
  },
  {
    name: 'damage hashes that no longer match',
    operation: 'damage',
    referenceId: INVENTED_ID,
    change: (spec,) => ({ ...spec, damages: spec.damages.map((damage,) => ({ ...damage, hash: 'wrong', })), }),
  },
  {
    name: 'damage counts one character off',
    operation: 'damage',
    referenceId: INVENTED_ID,
    change: (spec,) => ({
      ...spec,
      damages: spec.damages.map((damage,) => ({ ...damage, changedChars: damage.changedChars + 1, })),
    }),
  },
  {
    name: 'a damage family no builder exists for',
    operation: 'damage',
    referenceId: INVENTED_ID,
    change: (spec,) => ({
      ...spec,
      damages: spec.damages.slice(0, 1,).map((damage,) => ({ ...damage, kind: 'unsupported' as never, })),
    }),
  },
  {
    name: 'edits whose expected text is not what the archive holds',
    operation: 'edit',
    referenceId: INVENTED_ID,
    change: (spec,) => ({ ...spec, edits: spec.edits.map((edit,) => ({ ...edit, expectedHash: 'wrong', })), }),
  },
  {
    name: 'the same edits listed twice, which overlap',
    operation: 'edit',
    referenceId: INVENTED_ID,
    change: (spec,) => ({ ...spec, edits: [...spec.edits, ...spec.edits,], }),
  },
  {
    name: 'edits with no author',
    operation: 'edit',
    referenceId: INVENTED_ID,
    change: (spec,) => ({ ...spec, edits: spec.edits.map((edit,) => ({ ...edit, author: '', })), }),
  },
  {
    name: 'edits with an author of spaces only',
    operation: 'edit',
    referenceId: INVENTED_ID,
    change: (spec,) => ({ ...spec, edits: spec.edits.map((edit,) => ({ ...edit, author: '  ', })), }),
  },
  {
    name: 'edits with no rationale',
    operation: 'edit',
    referenceId: INVENTED_ID,
    change: (spec,) => ({ ...spec, edits: spec.edits.map((edit,) => ({ ...edit, rationale: '', })), }),
  },
  {
    name: 'edits with a rationale of spaces only',
    operation: 'edit',
    referenceId: INVENTED_ID,
    change: (spec,) => ({ ...spec, edits: spec.edits.map((edit,) => ({ ...edit, rationale: '  ', })), }),
  },
  {
    name: 'an edit whose range runs past the folded slice',
    operation: 'edit',
    referenceId: INVENTED_ID,
    change: (spec,) => ({
      ...spec,
      edits: spec.edits.map((edit,) => ({ ...edit, endOffset: 100_000, })),
    }),
  },
  {
    name: 'no review date',
    operation: 'request',
    referenceId: INVENTED_ID,
    change: (spec,) => ({ ...spec, reviewedOn: '', }),
  },
  {
    name: 'a review date of spaces only',
    operation: 'request',
    referenceId: INVENTED_ID,
    change: (spec,) => ({ ...spec, reviewedOn: '  ', }),
  },
  {
    name: 'no reference identifier',
    operation: 'request',
    referenceId: '',
    change: (spec,) => ({ ...spec, id: '', }),
  },
  {
    name: 'a reference identifier of spaces only',
    operation: 'request',
    referenceId: '  ',
    change: (spec,) => ({ ...spec, id: '  ', }),
  },
];

/**
 One request for trial rows the references cannot satisfy.
 */
type TrialRefusalRow = {
  /**
   What the request asks for, as the case name states it.
   */
  readonly name: string;

  /**
   Families requested.
   */
  readonly kinds: readonly FidelityDamageKind[];

  /**
   Families the single reference holds variants of.
   */
  readonly reviewed: readonly FidelityDamageKind[];

  /**
   Identifier the refusal names.
   */
  readonly referenceId: string;
};

/**
 Requests the trial matrix refuses, each with what its refusal names.
 */
const TRIAL_REFUSALS: readonly TrialRefusalRow[] = [
  {
    name: 'a family no reference reviewed',
    kinds: ['alteration',],
    reviewed: ['deletion',],
    referenceId: 'damage selection (alteration)',
  },
  {
    name: 'one reviewed family beside one no reference reviewed',
    kinds: [
      'deletion',
      'alteration',
    ],
    reviewed: ['deletion',],
    referenceId: 'damage selection (alteration)',
  },
  {
    name: 'two families no reference reviewed, named in the order requested',
    kinds: [
      'deletion',
      'insertion',
      'alteration',
    ],
    reviewed: ['deletion',],
    referenceId: 'damage selection (insertion, alteration)',
  },
  {
    name: 'no families',
    kinds: [],
    reviewed: ['deletion',],
    referenceId: 'damage selection',
  },
  {
    name: 'a family named twice',
    kinds: [
      'deletion',
      'deletion',
    ],
    reviewed: ['deletion',],
    referenceId: 'damage selection',
  },
];

/**
 One manifest and filter the selection refuses.
 */
type SelectRefusalRow = {
  /**
   What is wrong with the request, as the case name states it.
   */
  readonly name: string;

  /**
   Manifest handed in.
   */
  readonly specs: readonly FidelityReferenceSpec[];

  /**
   Entry filter handed in.
   */
  readonly onlyEntryIds: readonly string[];

  /**
   Identifier the refusal names.
   */
  readonly referenceId: string;
};

/**
 Requests the selection refuses, each with what its refusal names.
 */
const SELECT_REFUSALS: readonly SelectRefusalRow[] = [
  {
    name: 'a manifest with no references',
    specs: [],
    onlyEntryIds: [],
    referenceId: 'manifest',
  },
  {
    name: 'a manifest that lists one identifier twice',
    specs: [
      reviewedFixture().spec,
      reviewedFixture().spec,
    ],
    onlyEntryIds: [],
    referenceId: 'manifest',
  },
  {
    name: 'an entry filter naming no reviewed entry',
    specs: [reviewedFixture().spec,],
    onlyEntryIds: ['unreviewed',],
    referenceId: 'unreviewed',
  },
];

//endregion Rebound fixtures

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: buildReviewedFidelityReference.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS the folded archive slice with each edit applied at its original coordinates, the source '
            + 'passage and every reviewed damage identity, and keeps the spec as given',
          fn: async () => {
            const fixture = reviewedFixture();
            const result = buildReviewedFidelityReference(fixture,);
            expect(summaryOf({ reference: result, },),).toEqual({
              spec: fixture.spec,
              sourceText: REVIEW_SOURCE,
              referenceText: REVIEW_REFERENCE,
              damages: fixture.spec.damages,
            },);
            expect(result.damages.map((damage,) => damage.damageKind,),).toEqual([
              'deletion',
              'insertion',
              'alteration',
            ],);
            expect(result.spec,).not.toBe(fixture.spec,);
          },
        },),
        it({
          name: 'KEEPS THE WHOLE PAGES the passages were cut from, which the editor probe prepares the context of its '
            + 'sheet from: the neighbouring sections, the declared names and the whole original',
          fn: async () => {
            const fixture = reviewedFixture();
            expect(buildReviewedFidelityReference(fixture,).pages,).toStrictEqual({
              sourceFile: fixture.sourceFile,
              archiveFile: fixture.archiveFile,
            },);
          },
        },),
        it({
          name: 'KEEPS an unedited reference exactly as its archive slice reads',
          fn: async () => {
            const { sourceFile, archiveFile, spec, } = rebound({
              original: REVIEW_REFERENCE,
              reference: REVIEW_REFERENCE,
              edits: [],
              layout: 'after',
            },);
            const result = buildReviewedFidelityReference({
              sourceFile,
              archiveFile,
              spec,
            },);
            expect(summaryOf({ reference: result, },),).toEqual({
              spec,
              sourceText: REVIEW_SOURCE,
              referenceText: REVIEW_REFERENCE,
              damages: spec.damages,
            },);
          },
        },),
        it({
          name: 'APPLIES edits listed out of order by their original coordinates and keeps the order given',
          fn: async () => {
            const fixture = reviewedFixture();
            const spec = {
              ...fixture.spec,
              edits: fixture.spec.edits.toReversed(),
            };
            const result = buildReviewedFidelityReference({
              ...fixture,
              spec,
            },);
            expect(result.referenceText,).toBe(REVIEW_REFERENCE,);
            expect(result.spec,).toEqual(spec,);
          },
        },),
        it({
          name: 'APPLIES two edits that touch end to end, the second starting where the first ends',
          fn: async () => {
            const original = REVIEW_REFERENCE.replace(
              'quiet flat',
              'loud shed',
            );
            const loudAt = original.indexOf('loud',);
            const touching = [
              {
                startOffset: loudAt,
                endOffset: loudAt + 'loud'.length,
                expectedHash: hashContent({ content: 'loud', },),
                replacement: 'quiet',
                author: 'fixture-author',
                rationale: 'Restore the source\'s quiet.',
              },
              {
                startOffset: loudAt + 'loud'.length,
                endOffset: loudAt + 'loud shed'.length,
                expectedHash: hashContent({ content: ' shed', },),
                replacement: ' flat',
                author: 'fixture-author',
                rationale: 'Restore the source\'s flat.',
              },
            ];
            const { sourceFile, archiveFile, spec, } = rebound({
              original,
              reference: REVIEW_REFERENCE,
              edits: touching,
              layout: 'after',
            },);
            const result = buildReviewedFidelityReference({
              sourceFile,
              archiveFile,
              spec,
            },);
            expect(result.referenceText,).toBe(REVIEW_REFERENCE,);
          },
        },),
        it({
          name: 'OWNS ITS PROVENANCE: a later change to the caller\'s spec or its edits leaves the built spec as it was',
          fn: async () => {
            const fixture = reviewedFixture();
            const spec = {
              ...fixture.spec,
              edits: fixture.spec.edits.map((edit,) => ({ ...edit, })),
            };
            const result = buildReviewedFidelityReference({
              ...fixture,
              spec,
            },);
            spec.id = 'changed-after-verification';
            const [first,] = spec.edits;
            if (first !== undefined)
              first.replacement = 'changed';
            expect(result.spec,).toEqual(fixture.spec,);
          },
        },),
        ...DRIFTS.map(function driftCase(row,) {
          return it({
            name: `REFUSES ${row.name} at the ${row.operation} boundary, never relabelling archive text as gold`,
            fn: async () => {
              const fixture = reviewedFixture();
              const refusal = refusalOfBuild({
                sourceFile: fixture.sourceFile,
                archiveFile: fixture.archiveFile,
                spec: row.change(fixture.spec,),
              },);
              expectRefusal({
                refusal,
                referenceId: row.referenceId,
                operation: row.operation,
              },);
            },
          },);
        },),
        it({
          name: 'REFUSES a source start that slice would wrap around to the reviewed bytes, at the source boundary',
          fn: async () => {
            const fixture = reviewedFixture();
            const source = {
              ...fixture.spec.source,
              startOffset: fixture.spec.source.startOffset - fixture.sourceFile.length,
            };
            expect(fixture.sourceFile.slice(
              source.startOffset,
              source.endOffset,
            ),).toBe(fixture.sourceFile.slice(
              fixture.spec.source.startOffset,
              fixture.spec.source.endOffset,
            ),);
            expectRefusal({
              refusal: refusalOfBuild({
                sourceFile: fixture.sourceFile,
                archiveFile: fixture.archiveFile,
                spec: {
                  ...fixture.spec,
                  source,
                },
              },),
              referenceId: INVENTED_ID,
              operation: 'source',
            },);
          },
        },),
        it({
          name: 'REFUSES the same removal listed twice at the edit boundary, although one removal reconstructs the '
            + 'reference and every damage hash',
          fn: async () => {
            const removed = 'very ';
            const original = REVIEW_REFERENCE.replace(
              'quiet flat',
              `${removed}quiet flat`,
            );
            const startOffset = original.indexOf(removed,);
            const edit = {
              startOffset,
              endOffset: startOffset + removed.length,
              expectedHash: hashContent({ content: removed, },),
              replacement: '',
              author: 'fixture-author',
              rationale: 'Remove unsupported intensifier.',
            };
            const { sourceFile, archiveFile, spec, } = rebound({
              original,
              reference: REVIEW_REFERENCE,
              edits: [edit,],
              layout: 'after',
            },);
            expect(buildReviewedFidelityReference({
              sourceFile,
              archiveFile,
              spec,
            },).referenceText,).toBe(REVIEW_REFERENCE,);
            expectRefusal({
              refusal: refusalOfBuild({
                sourceFile,
                archiveFile,
                spec: {
                  ...spec,
                  edits: [
                    edit,
                    edit,
                  ],
                },
              },),
              referenceId: INVENTED_ID,
              operation: 'edit',
            },);
          },
        },),
        it({
          name: 'REFUSES a donor inside the reference range at the donor boundary even with no insertion variant '
            + 'to hash it',
          fn: async () => {
            const fixture = reviewedFixture();
            const spec = {
              ...fixture.spec,
              damages: fixture.spec.damages.filter((damage,) => damage.kind !== 'insertion',),
            };
            expect(buildReviewedFidelityReference({
              ...fixture,
              spec,
            },).damages.map((damage,) => damage.damageKind,),).toEqual([
              'deletion',
              'alteration',
            ],);
            expectRefusal({
              refusal: refusalOfBuild({
                sourceFile: fixture.sourceFile,
                archiveFile: fixture.archiveFile,
                spec: {
                  ...spec,
                  donor: spec.archive,
                },
              },),
              referenceId: INVENTED_ID,
              operation: 'donor',
            },);
          },
        },),
        it({
          name: 'KEEPS a donor span that ends where the reference range starts, or starts where it ends, sharing '
            + 'no character with it',
          fn: async () => {
            for (const layout of LAYOUTS) {
              const spec = donorSpanSpec({
                layout,
                overlap: 0,
              },);
              expect(buildReviewedFidelityReference(spec,).referenceText,).toBe(REVIEW_REFERENCE,);
            }
          },
        },),
        it({
          name: 'REFUSES a donor span that shares one character with the reference range, at its start or its '
            + 'end, at the donor boundary',
          fn: async () => {
            for (const layout of LAYOUTS) {
              expectRefusal({
                refusal: refusalOfBuild(donorSpanSpec({
                  layout,
                  overlap: 1,
                },),),
                referenceId: INVENTED_ID,
                operation: 'donor',
              },);
            }
          },
        },),
        it({
          name: 'KEEPS a reference of exactly the natural length floor and REFUSES one a character shorter, at '
            + 'the reference boundary',
          fn: async () => {
            const atFloor = REVIEW_REFERENCE.slice(
              0,
              MIN_REVIEWED_REFERENCE_CHARS,
            );
            const shorter = REVIEW_REFERENCE.slice(
              0,
              MIN_REVIEWED_REFERENCE_CHARS - 1,
            );
            const kept = rebound({
              original: atFloor,
              reference: atFloor,
              edits: [],
              layout: 'after',
            },);
            expect(buildReviewedFidelityReference(kept,).referenceText,).toBe(atFloor,);
            expectRefusal({
              refusal: refusalOfBuild(rebound({
                original: shorter,
                reference: shorter,
                edits: [],
                layout: 'after',
              },),),
              referenceId: INVENTED_ID,
              operation: 'reference',
            },);
          },
        },),
        it({
          name: 'REFUSES a reviewed damage family its builder cannot produce for the reference, at the damage '
            + 'boundary, instead of dropping the family',
          fn: async () => {
            const fixture = reviewedFixture();
            const text = 'A cat slept. '.repeat(40,);
            expect(deleteOneSentence({ cleanText: text, },).kind,).toBe('undamageable',);
            expectRefusal({
              refusal: refusalOfBuild({
                sourceFile: fixture.sourceFile,
                archiveFile: `${text}\n\n${REVIEW_DONOR}`,
                spec: {
                  ...fixture.spec,
                  archive: {
                    startOffset: 0,
                    endOffset: text.length,
                    hash: hashContent({ content: text, },),
                  },
                  edits: [],
                  referenceHash: hashContent({ content: text, },),
                  referenceChars: text.length,
                  donor: {
                    startOffset: text.length + 2,
                    endOffset: text.length + 2 + REVIEW_DONOR.length,
                    hash: hashContent({ content: REVIEW_DONOR, },),
                  },
                },
              },),
              referenceId: INVENTED_ID,
              operation: 'damage',
            },);
          },
        },),
      ],
    },),
    describe({
      name: reviewedFidelityTrials.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'EXPANDS each reference and requested family into the four arrangements, in reference, then '
            + 'reviewed damage, then arrangement order',
          fn: async () => {
            const fixture = reviewedFixture();
            const first = buildReviewedFidelityReference(fixture,);
            const second = buildReviewedFidelityReference({
              ...fixture,
              spec: {
                ...fixture.spec,
                id: 'second-reference',
              },
            },);
            const rows = reviewedFidelityTrials({
              references: [
                first,
                second,
              ],
              damageKinds: [
                'alteration',
                'deletion',
              ],
            },);
            expect(rows.map((row,) => [
              row.trial.trialId,
              row.trial.direction,
              row.trial.cleanFirst,
            ],),).toEqual(['invented-reference', 'second-reference',].flatMap((id,) =>
              ['deletion', 'alteration',].flatMap((kind,) => [
                [`${id}/${kind}`, 'preserve', true,],
                [`${id}/${kind}`, 'preserve', false,],
                [`${id}/${kind}`, 'replace', true,],
                [`${id}/${kind}`, 'replace', false,],
              ])
            ),);
          },
        },),
        it({
          name: 'CARRIES a row\'s reviewed spec, changed count, damage detail and whole trial, with no context text',
          fn: async () => {
            const reference = buildReviewedFidelityReference(reviewedFixture(),);
            const [row,] = reviewedFidelityTrials({
              references: [reference,],
              damageKinds: ['deletion',],
            },);
            const [damage,] = reference.damages;
            expect(row,).toEqual({
              spec: reference.spec,
              changedChars: damage?.changedChars,
              damageDetail: damage?.damageDetail,
              trial: {
                trialId: 'invented-reference/deletion',
                direction: 'preserve',
                damageKind: 'deletion',
                sourceText: REVIEW_SOURCE,
                contextText: '',
                cleanText: REVIEW_REFERENCE,
                damagedText: damage?.damagedText,
                cleanFirst: true,
              },
            },);
          },
        },),
        it({
          name: 'KEEPS only the requested families, four rows for each in the reference\'s reviewed order',
          fn: async () => {
            const reference = buildReviewedFidelityReference(reviewedFixture(),);
            expect(reviewedFidelityTrials({
              references: [reference,],
              damageKinds: ['alteration',],
            },).map((row,) => row.trial.damageKind,),).toEqual([
              'alteration',
              'alteration',
              'alteration',
              'alteration',
            ],);
            expect(reviewedFidelityTrials({
              references: [reference,],
              damageKinds: [
                'alteration',
                'insertion',
                'deletion',
              ],
            },).map((row,) => row.trial.damageKind,),).toEqual([
              ...Array.from({ length: 4, }, () => 'deletion',),
              ...Array.from({ length: 4, }, () => 'insertion',),
              ...Array.from({ length: 4, }, () => 'alteration',),
            ],);
          },
        },),
        ...TRIAL_REFUSALS.map(function refusedCase(row,) {
          return it({
            name: `REFUSES ${row.name} at the request boundary, since a matrix missing a family is no calibration`,
            fn: async () => {
              const reference = buildReviewedFidelityReference(reviewedFixture(),);
              const refusal = caught(function expand(): unknown {
                return reviewedFidelityTrials({
                  references: [{
                    ...reference,
                    damages: reference.damages.filter((damage,) => row.reviewed.includes(damage.damageKind,),),
                  },],
                  damageKinds: row.kinds,
                },);
              },);
              expectRefusal({
                refusal,
                referenceId: row.referenceId,
                operation: 'request',
              },);
            },
          },);
        },),
        it({
          name: 'REFUSES a request over no references at the request boundary, naming the family it could not find',
          fn: async () => {
            expectRefusal({
              refusal: caught(function expand(): unknown {
                return reviewedFidelityTrials({
                  references: [],
                  damageKinds: ['deletion',],
                },);
              },),
              referenceId: 'damage selection (deletion)',
              operation: 'request',
            },);
          },
        },),
      ],
    },),
    describe({
      name: selectReviewedFidelitySpecs.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS manifest order for an entry filter listed in another order, and drops every other entry',
          fn: async () => {
            const { spec, } = reviewedFixture();
            const first = {
              ...spec,
              id: 'first',
              entryId: 'first-cat',
            };
            const second = {
              ...spec,
              id: 'second',
              entryId: 'second-cat',
            };
            const third = {
              ...spec,
              id: 'third',
              entryId: 'third-cat',
            };
            expect(selectReviewedFidelitySpecs({
              specs: [
                first,
                second,
                third,
              ],
              onlyEntryIds: [
                'third-cat',
                'first-cat',
              ],
            },),).toEqual([
              first,
              third,
            ],);
          },
        },),
        it({
          name: 'RETURNS the whole manifest for an empty filter and every reference of an entry for a named one',
          fn: async () => {
            const { spec, } = reviewedFixture();
            const sibling = {
              ...spec,
              id: 'sibling',
            };
            const other = {
              ...spec,
              id: 'other',
              entryId: 'other-cat',
            };
            const specs = [
              spec,
              sibling,
              other,
            ];
            expect(selectReviewedFidelitySpecs({
              specs,
              onlyEntryIds: [],
            },),).toEqual(specs,);
            expect(selectReviewedFidelitySpecs({
              specs,
              onlyEntryIds: [spec.entryId,],
            },),).toEqual([
              spec,
              sibling,
            ],);
          },
        },),
        it({
          name: 'RETURNS its own copies, so a later change to the manifest leaves the selection as it was',
          fn: async () => {
            const { spec, } = reviewedFixture();
            const specs = [{
              ...spec,
              edits: spec.edits.map((edit,) => ({ ...edit, })),
            },];
            const selected = selectReviewedFidelitySpecs({
              specs,
              onlyEntryIds: [],
            },);
            const [first,] = specs;
            const [edit,] = nonNullishOrThrow(first,).edits;
            nonNullishOrThrow(first,).id = 'changed-after-selection';
            nonNullishOrThrow(edit,).replacement = 'changed-after-selection';
            expect(selected,).toEqual([spec,],);
          },
        },),
        ...SELECT_REFUSALS.map(function refusedCase(row,) {
          return it({
            name: `REFUSES ${row.name} at the request boundary, naming ${row.referenceId}`,
            fn: async () => {
              const refusal = caught(function select(): unknown {
                return selectReviewedFidelitySpecs({
                  specs: row.specs,
                  onlyEntryIds: row.onlyEntryIds,
                },);
              },);
              expectRefusal({
                refusal,
                referenceId: row.referenceId,
                operation: 'request',
              },);
            },
          },);
        },),
      ],
    },),
    describe({
      name: 'REVIEWED_FIDELITY_REFERENCES',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'HOLDS nothing the build and the selection would refuse before a corpus is read: unique identifiers, '
            + 'one 40-digit revision, a reviewed length at the floor or above, a donor clear of its archive range, '
            + 'distinct damage families, and an author and rationale on every edit',
          fn: async () => {
            const specs = REVIEWED_FIDELITY_REFERENCES;
            expect(selectReviewedFidelitySpecs({
              specs,
              onlyEntryIds: [],
            },),).toEqual(specs,);
            expect(new Set(specs.map((spec,) => spec.corpusSha,),).size,).toBe(1,);
            for (const spec of specs) {
              expect(BigInt(`0x${spec.corpusSha}`,).toString(16,)
                .padStart(spec.corpusSha.length, '0',),).toBe(spec.corpusSha,);
              expect(spec.corpusSha,).toHaveLength(40,);
              expect(spec.referenceChars,).toBeGreaterThanOrEqual(MIN_REVIEWED_REFERENCE_CHARS,);
              expect((spec.donor.startOffset >= spec.archive.endOffset) || (spec.archive.startOffset >= spec.donor.endOffset),)
                .toBe(true,);
              expect(new Set(spec.damages.map((damage,) => damage.kind,),).size,).toBe(spec.damages.length,);
              expect(spec.damages.length,).toBeGreaterThan(0,);
              expect(spec.edits.filter((edit,) => (edit.author.trim() === '') || (edit.rationale.trim() === ''),),).toEqual([],);
            }
          },
        },),
      ],
    },),
  ],
},);
