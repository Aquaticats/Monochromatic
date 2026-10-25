import { expect, } from '@monochromatic-dev/module-test/ts';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import { foldInvisibleVariants, } from '@monochromatic-dev/translation-repair-markup';
import {
  alterSharedNumber,
  type DamageAttempt,
  deleteOneSentence,
  insertBorrowedSentence,
} from '@monochromatic-dev/translation-repair-repair';
import {
  type FidelityExpectedDamage,
  FidelityReferenceError,
  type FidelityReferenceOperation,
  type FidelityReferenceSpec,
  type ReviewedFidelityReference,
} from '../dist/final/node/index.mjs';

//region Invented reviewed-reference fixtures
// These strings are authored test data, never copied corpus passages.

/**
 Width of the invented SHA-1 pin used by non-I/O fixture checks.
 */
const FIXTURE_COMMIT_WIDTH = 40;

/**
 Invented source carrying the deliberately shared year.
 */
export const REVIEW_SOURCE: string = '2023年，灰白相间的小猫搬到旧书店楼上的安静公寓。她每天早上浇灌窗边的花，'
  + '晚上在图书馆读书。她喜欢与邻居分享故事，也喜欢给来访的朋友准备茶点。书店里的角落明亮而温暖，'
  + '朋友们常在那里讨论书籍和花园。她认真保管明信片，记得每位朋友喜欢的图案。';

/**
 Source-faithful invented reference naturally exceeding the calibration floor.
 */
export const REVIEW_REFERENCE: string = 'The grey-and-white cat arrived in 2023 and moved into a quiet flat above the old bookshop. '
  + 'Every morning she watered the flowers beside the window, and every evening she read at the library. '
  + 'She enjoyed sharing stories with her neighbours and preparing tea and snacks for visiting friends. '
  + 'The well-lit corner of the bookshop was warm, and her friends often discussed books and gardens there. '
  + 'She carefully kept their postcards and remembered the patterns each friend liked.';

/**
 Independent invented donor supplying one unsupported but fluent statement.
 */
export const REVIEW_DONOR: string = 'On a winter holiday she travelled to a distant village and spent the afternoon learning how its bakers made bread.';

/**
 Narrows a fixture builder result without pretending an undamageable case is usable.

 @param damage - builder output for invented content

 @returns Definite fixture mutation

 @example
 ```ts
 const built = fixtureDamage(deleteOneSentence({ cleanText }));
 ```
 */
function fixtureDamage(damage: DamageAttempt,): Extract<DamageAttempt, { readonly kind: 'damaged'; }> {
  if (damage.kind !== 'damaged')
    throw new Error('Invented reference fixture must admit its specified mutation',);
  return damage;
}

/**
 Identity the manifest reviews for one damage variant: its family, the hash of
 its damaged text and its changed-content count.

 @param damage - variant a damage builder produced

 @returns Its reviewed identity

 @example
 ```ts
 const expected = damageIdentity({ damage, },);
 ```
 */
function damageIdentity(
  { damage, }: { readonly damage: Extract<DamageAttempt, { readonly kind: 'damaged'; }>; },
): FidelityExpectedDamage {
  return {
    kind: damage.damageKind,
    hash: hashContent({ content: damage.damagedText, },),
    changedChars: damage.changedChars,
  };
}

/**
 Builds disjoint original-coordinate edits, including a length-changing first edit.

 @returns Invented files and independently declared reference identity

 @example
 ```ts
 const fixture = reviewedFixture();
 ```
 */
export function reviewedFixture(): {
  readonly sourceFile: string;
  readonly archiveFile: string;
  readonly spec: FidelityReferenceSpec;
} {
  /**
   Archive retains deliberate factual defects and one invisible character for the transform path.
   */
  const original = REVIEW_REFERENCE.replace(
    'grey-and-white',
    'blue',
  )
    .replace(
    '2023',
    '2022',
    )
    .replace(
    'well-lit',
    'well\u2011lit',
    );
  /**
   Local edit coordinates refer to this original folded slice.
   */
  const folded = foldInvisibleVariants({ text: original, },)
    .text;
  /**
   Source begins after a prefix so offset-zero assumptions cannot pass.
   */
  const sourcePrefix = 'source heading\n';
  /**
   Archive has an independent prefix and donor after the reviewed passage.
   */
  const archivePrefix = 'archive heading\n';
  /**
   Full invented source file.
   */
  const sourceFile = `${sourcePrefix}${REVIEW_SOURCE}\nsource footer`;
  /**
   Full invented archive file.
   */
  const archiveFile = `${archivePrefix}${original}\n\n${REVIEW_DONOR}`;
  /**
   First correction changes length, so later offsets must not be interpreted incrementally.
   */
  const colorAt = folded.indexOf('blue',);
  /**
   Second correction restores the shared source year.
   */
  const yearAt = folded.indexOf('2022',);
  /**
   Fixed insertion donor position, independent of name matching.
   */
  const donorAt = archiveFile.indexOf(REVIEW_DONOR,);
  /**
   Existing mechanisms define fixture variants; reviewed hashes lock their outputs.
   */
  const damages = [
    fixtureDamage(deleteOneSentence({ cleanText: REVIEW_REFERENCE, },),),
    fixtureDamage(insertBorrowedSentence({
      cleanText: REVIEW_REFERENCE,
      donorTexts: [REVIEW_DONOR],
    },),),
    fixtureDamage(alterSharedNumber({
      cleanText: REVIEW_REFERENCE,
      sourceText: REVIEW_SOURCE,
    },),),
  ];
  return {
    sourceFile,
    archiveFile,
    spec: {
      id: 'invented-reference',
      entryId: 'starlit-cat',
      corpusSha: 'a'.repeat(FIXTURE_COMMIT_WIDTH,),
      source: {
        startOffset: sourcePrefix.length,
        endOffset: sourcePrefix.length + REVIEW_SOURCE.length,
        hash: hashContent({ content: REVIEW_SOURCE, },),
      },
      archive: {
        startOffset: archivePrefix.length,
        endOffset: archivePrefix.length + original.length,
        hash: hashContent({ content: original, },),
      },
      referenceHash: hashContent({ content: REVIEW_REFERENCE, },),
      referenceChars: REVIEW_REFERENCE.length,
      edits: [
        {
          startOffset: colorAt,
          endOffset: colorAt + 'blue'.length,
          expectedHash: hashContent({ content: 'blue', },),
          replacement: 'grey-and-white',
          author: 'fixture-author',
          rationale: 'Restore source color.',
        },
        {
          startOffset: yearAt,
          endOffset: yearAt + '2022'.length,
          expectedHash: hashContent({ content: '2022', },),
          replacement: '2023',
          author: 'fixture-author',
          rationale: 'Restore source year.',
        },
      ],
      donor: {
        startOffset: donorAt,
        endOffset: donorAt + REVIEW_DONOR.length,
        hash: hashContent({ content: REVIEW_DONOR, },),
      },
      damages: damages.map(function expected(damage,): FidelityExpectedDamage {
        return damageIdentity({ damage, },);
      },),
      reviewedOn: '2026-09-11',
    },
  };
}

/**
 Message a reviewed-fidelity refusal carries, spelled out so that no case builds
 its expectation with the class under test.

 @param referenceId - identifier the refusal names, empty when the manifest entry carried none

 @param operation - verification boundary the refusal names

 @returns The whole message

 @example
 ```ts
 const message = refusalMessage({ referenceId: 'invented-reference', operation: 'source', },);
 ```
 */
export function refusalMessage(
  {
    referenceId,
    operation,
  }: {
    readonly referenceId: string;
    readonly operation: FidelityReferenceOperation;
  },
): string {
  return `reviewed fidelity reference ${referenceId} failed ${operation} verification;`
    + ' use its pinned inputs and reviewed manifest, or source-review a replacement before calibration.';
}

/**
 Waits for a call that must refuse and hands back what it rejected with.

 @param pending - call expected to refuse

 @returns The rejection, unchanged

 @throws {@link Error} when the call answers instead of refusing

 @example
 ```ts
 const refusal = await settledRefusal(readReviewedFidelityReferences({ pin, },),);
 ```
 */
export async function settledRefusal(pending: Promise<unknown>,): Promise<unknown> {
  try {
    await pending;
  }
  catch (error) {
    return error;
  }
  throw new Error('expected the call to refuse, but it answered',);
}

/**
 Everything a built reference holds that a case compares whole, each damage
 variant reduced to the identity the manifest reviews: its family, the hash of
 its damaged text and its changed-content count.

 @param reference - built reference

 @returns Its spec, texts and reviewed damage identities

 @example
 ```ts
 expect(summaryOf({ reference, },),).toEqual({ spec, sourceText, referenceText, damages: spec.damages, },);
 ```
 */
export function summaryOf({ reference, }: { readonly reference: ReviewedFidelityReference; },): {
  readonly spec: FidelityReferenceSpec;
  readonly sourceText: string;
  readonly referenceText: string;
  readonly damages: readonly FidelityExpectedDamage[];
} {
  return {
    spec: reference.spec,
    sourceText: reference.sourceText,
    referenceText: reference.referenceText,
    damages: reference.damages
      .map(function identity(damage,): FidelityExpectedDamage {
        return damageIdentity({ damage, },);
      },),
  };
}

/**
 Asserts that a caught value is the reviewed-fidelity refusal for one reference and
 one verification boundary, whole message included.

 @param refusal - value a call threw or rejected with

 @param referenceId - identifier the refusal must name

 @param operation - verification boundary the refusal must name

 @example
 ```ts
 expectRefusal({ refusal, referenceId: 'invented-reference', operation: 'donor', },);
 ```
 */
export function expectRefusal(
  {
    refusal,
    referenceId,
    operation,
  }: {
    readonly refusal: unknown;
    readonly referenceId: string;
    readonly operation: FidelityReferenceOperation;
  },
): void {
  expect(refusal,)
    .toBeInstanceOf(FidelityReferenceError,);
  /**
   The message the refusal must carry.
   */
  const message = refusalMessage({
    referenceId,
    operation,
  },);
  expect(String(refusal,),)
    .toBe(`FidelityReferenceError: ${message}`,);
}

/**
 Drops the second bracketed group of a log line, which holds its timestamp.

 @param line - output line read

 @returns The line without its timestamp, any other line unchanged

 @example
 ```ts
 // yields '[info] [probe] purr'
 const plain = withoutStamp({ line: '[info] [2026-01-01T00:00:00.000Z] [probe] purr', },);
 ```
 */
export function withoutStamp({ line, }: { readonly line: string; },): string {
  if (!line.startsWith('[',))
    return line;
  /**
   Where the level group closes.
   */
  const levelEnd = line.indexOf(']',);
  /**
   Where the timestamp group closes.
   */
  const stampEnd = line.indexOf(
    ']',
    levelEnd + 1,
  );
  if ([
    levelEnd,
    stampEnd,
  ].includes(-1,))
    return line;
  return `${line.slice(
    0,
    levelEnd + 1,
  )}${line.slice(stampEnd + 1,)}`;
}

//endregion Invented reviewed-reference fixtures
