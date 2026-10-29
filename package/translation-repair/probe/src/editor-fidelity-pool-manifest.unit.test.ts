/**
 Tests for the frozen pool the editor fidelity probe draws its trials from: how many trials it holds, how they spread
 over entries and defect families, and that every pin in it is well formed and carries no corpus text.

 The pool's own references are not read here: the corpus is unlicensed and absent from the test environment, and the
 hash checks run each time the probe reads the pool. These cases pin the manifest's shape and its ruling, not its
 prose. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { CORPUS_COMMIT_SHA, } from '@monochromatic-dev/translation-repair-corpus-source';
import {
  DEFAULT_EDITOR_TRIAL_CAP,
  DRAWN_EDITOR_SPECS,
  EDITOR_DAMAGE_KINDS,
  EDITOR_FIDELITY_POOL,
  type FidelityReferenceSpan,
  type FidelityReferenceSpec,
  MIN_REVIEWED_REFERENCE_CHARS,
  POOL_DRAWN_ON,
  POOL_TRIALS_WANTED,
  REVIEWED_FIDELITY_REFERENCES,
} from '../dist/final/node/index.mjs';

/**
 Lowercase hexadecimal digits a hash is written in.
 */
const HEX_DIGITS = '0123456789abcdef';

/**
 Characters an identifier is written in: letters, digits, underscores, hyphens and dots, which no sentence of prose is.
 */
const ID_CHARACTERS = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_.-';

/**
 Whether every character of a string is among the allowed ones.

 @param text - string read

 @param allowed - characters it may be written in

 @returns Whether the string uses no other

 @example
 ```ts
 const plain = onlyCharactersOf({ text: 'tabby-1', allowed: ID_CHARACTERS, },);
 ```
 */
function onlyCharactersOf({ text, allowed, }: { readonly text: string; readonly allowed: string; },): boolean {
  for (const character of text) {
    if (!allowed.includes(character,))
      return false;
  }
  return true;
}

/**
 Whether a string is a lowercase SHA-256 hash in hexadecimal.

 @param text - string read

 @returns Whether it is sixty-four lowercase hexadecimal digits

 @example
 ```ts
 const pinned = isHash({ text: spec.referenceHash, },);
 ```
 */
function isHash({ text, }: { readonly text: string; },): boolean {
  return (text.length === 64) && onlyCharactersOf({ text, allowed: HEX_DIGITS, },);
}

/**
 Whether a string is an identifier the corpus names entries by, or the pool names its references by.

 @param text - string read

 @returns Whether it is a plain identifier

 @example
 ```ts
 const plain = isPlainId({ text: spec.entryId, },);
 ```
 */
function isPlainId({ text, }: { readonly text: string; },): boolean {
  return (text !== '') && onlyCharactersOf({ text, allowed: ID_CHARACTERS, },);
}

/**
 Trials a specification stands for: one for each damage family the probe builds.

 @param spec - specification read

 @returns Families it carries that the probe builds

 @example
 ```ts
 const kinds = trialKindsOf({ spec, },);
 ```
 */
function trialKindsOf({ spec, }: { readonly spec: FidelityReferenceSpec; },): readonly string[] {
  return spec.damages
    .map(function kindOf(damage,): string {
      return damage.kind;
    },)
    .filter(function isBuilt(kind,): boolean {
      return EDITOR_DAMAGE_KINDS.some(function is(built,): boolean {
        return built === kind;
      },);
    },);
}

/**
 Every trial the pool stands for, as its family and entry.
 */
const POOL_TRIALS = EDITOR_FIDELITY_POOL.flatMap(function trialsOf(spec,): readonly {
  readonly kind: string;
  readonly entryId: string;
}[] {
  return trialKindsOf({ spec, },)
    .map(function trialOf(kind,): { readonly kind: string; readonly entryId: string; } {
      return {
        kind,
        entryId: spec.entryId,
      };
    },);
},);

/**
 Whether a span is well formed: whole offsets with a nonempty range, and a hash.

 @param span - span read

 @returns Whether the range and the hash are well formed

 @example
 ```ts
 const formed = isFormedSpan({ span: spec.source, },);
 ```
 */
function isFormedSpan({ span, }: { readonly span: FidelityReferenceSpan; },): boolean {
  return Number.isSafeInteger(span.startOffset,) && Number.isSafeInteger(span.endOffset,) && (span.startOffset >= 0)
    && (span.endOffset > span.startOffset) && isHash({ text: span.hash, },);
}

await describe({
  name: '',
  children: [
    describe({
      name: 'EDITOR_FIDELITY_POOL',
      children: [
        it({
          name: 'KEEPS THE REVIEWED REFERENCES FIRST AND UNCHANGED, so the trials earlier rounds read are still in the pool '
            + 'under the same ids',
          fn: async () => {
            expect(EDITOR_FIDELITY_POOL.slice(0, REVIEWED_FIDELITY_REFERENCES.length,),).toStrictEqual(
              REVIEWED_FIDELITY_REFERENCES,
            );
            expect(EDITOR_FIDELITY_POOL.length,).toBeGreaterThan(REVIEWED_FIDELITY_REFERENCES.length,);
          },
        },),

        it({
          name: 'IS THE REVIEWED REFERENCES AND THE DRAW AND NOTHING ELSE, every drawn specification dated by the draw\'s '
            + 'freeze date, so no hand-written specification sits among the drawn ones',
          fn: async () => {
            expect(EDITOR_FIDELITY_POOL,).toStrictEqual([...REVIEWED_FIDELITY_REFERENCES, ...DRAWN_EDITOR_SPECS,],);
            // The pool was redrawn on the day the sixth round widened it.
            expect(POOL_DRAWN_ON,).toBe('2026-10-24',);
            for (const spec of DRAWN_EDITOR_SPECS)
              expect(spec.reviewedOn,).toBe(POOL_DRAWN_ON,);
          },
        },),

        it({
          name: 'ASKS FOR A HUNDRED AND FIFTY TRIALS AND HOLDS EVERY ONE THE CLONE OFFERS, at least eighty: the paired test '
            + 'that reads the bench gains from each trial, and the draw takes every eligible entry before it stops short',
          fn: async () => {
            expect(POOL_TRIALS_WANTED,).toBe(150,);
            expect(POOL_TRIALS.length,).toBeGreaterThanOrEqual(80,);
          },
        },),

        it({
          name: 'SPREADS THEM over both defect families and over fifty entries or more, none holding more than two, so one '
            + 'document cannot decide a card and an entry is a cluster of at most two trials',
          fn: async () => {
            for (const kind of EDITOR_DAMAGE_KINDS) {
              expect(POOL_TRIALS.filter(function isKind(trial,): boolean {
                return trial.kind === kind;
              },).length,).toBeGreaterThanOrEqual(35,);
            }
            /**
             Trials each entry holds.
             */
            const perEntry = Map.groupBy(
              POOL_TRIALS,
              function entryOf(trial,): string {
                return trial.entryId;
              },
            );
            expect(perEntry.size,).toBeGreaterThanOrEqual(50,);
            for (const trials of perEntry.values())
              expect(trials.length,).toBeLessThanOrEqual(2,);
          },
        },),

        it({
          name: 'NAMES EACH REFERENCE ONCE, with no slash in its id, since a trial is named by the reference and the family '
            + 'joined by one',
          fn: async () => {
            const ids = EDITOR_FIDELITY_POOL.map(function idOf(spec,): string {
              return spec.id;
            },);
            expect(new Set(ids,).size,).toBe(ids.length,);
            expect(ids.some(function hasSlash(id,): boolean {
              return id.includes('/',);
            },),).toBe(false,);
          },
        },),

        it({
          name: 'PINS EVERY REFERENCE TO THE CORPUS COMMIT AND TO HASHES: whole offsets, a nonempty range, sixty-four hex '
            + 'digits for the source, the archive, the donor, the reference and each twin, a reference past the length '
            + 'floor, and a donor outside the archive range',
          fn: async () => {
            for (const spec of EDITOR_FIDELITY_POOL) {
              expect(spec.corpusSha,).toBe(CORPUS_COMMIT_SHA,);
              expect(isFormedSpan({ span: spec.source, },),).toBe(true,);
              expect(isFormedSpan({ span: spec.archive, },),).toBe(true,);
              expect(isFormedSpan({ span: spec.donor, },),).toBe(true,);
              expect(isHash({ text: spec.referenceHash, },),).toBe(true,);
              expect(spec.referenceChars,).toBeGreaterThanOrEqual(MIN_REVIEWED_REFERENCE_CHARS,);
              expect(
                (spec.donor.startOffset < spec.archive.endOffset) && (spec.archive.startOffset < spec.donor.endOffset),
              ).toBe(false,);
              expect(spec.damages.length,).toBeGreaterThan(0,);
              for (const damage of spec.damages) {
                expect(isHash({ text: damage.hash, },),).toBe(true,);
                expect(damage.changedChars,).toBeGreaterThan(0,);
              }
            }
          },
        },),

        it({
          name: 'DRAWS ONLY THE FAMILIES THE PROBE BUILDS and corrects nothing by hand: a drawn reference is the archive '
            + 'as it stands, which no deletion is asked of and no edit is applied to',
          fn: async () => {
            for (const spec of EDITOR_FIDELITY_POOL.slice(REVIEWED_FIDELITY_REFERENCES.length,)) {
              expect(spec.edits,).toStrictEqual([],);
              expect(trialKindsOf({ spec, },).length,).toBe(spec.damages.length,);
            }
          },
        },),

        it({
          name: 'CARRIES NO CORPUS TEXT: every string in it is an identifier, a hash, a date or the commit, because the '
            + 'corpus is unlicensed and only ids and hashes may be committed',
          fn: async () => {
            for (const spec of EDITOR_FIDELITY_POOL) {
              expect(isPlainId({ text: spec.id, },),).toBe(true,);
              expect(isPlainId({ text: spec.entryId, },),).toBe(true,);
              expect(isPlainId({ text: spec.reviewedOn, },),).toBe(true,);
              expect(spec.reviewedOn.length,).toBe(10,);
              for (const damage of spec.damages)
                expect(isPlainId({ text: damage.kind, },),).toBe(true,);
            }
          },
        },),

        it({
          name: 'FITS UNDER THE DEFAULT CAP, which holds the hundred and fifty trials the draw asks for, so an arm run '
            + 'with no --cap asks every trial and every arm sees the same ones',
          fn: async () => {
            expect(DEFAULT_EDITOR_TRIAL_CAP,).toBeGreaterThanOrEqual(150,);
            expect(DEFAULT_EDITOR_TRIAL_CAP,).toBeGreaterThanOrEqual(POOL_TRIALS.length,);
          },
        },),
      ],
    },),
  ],
},);
