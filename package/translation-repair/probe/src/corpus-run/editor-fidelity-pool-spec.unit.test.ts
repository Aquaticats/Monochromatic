/**
 Tests for the specification a drawn damage is committed as: its ids, ranges and hashes, and that no passage of either
 page is in it. Pages are cat-themed invention.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure/ts';

import {
  joinedBySection,
  sectionsOf,
  specForDamage,
} from '../../dist/final/node/index.mjs';
import {
  DRAW_CHINESE,
  DRAW_ENGLISH,
  DRAW_PIN,
} from '../editor-fidelity-pool.test-fixture.ts';

/**
 The spec of the first section with a damage borrowed from the last.
 */
const SECTIONS = sectionsOf({
  zh: DRAW_CHINESE,
  en: DRAW_ENGLISH,
},);

await describe({
  name: 'editor fidelity pool specification',
  children: [
    it({
      name: 'NAMES THE SECTION BY ENTRY AND POSITION and holds the hash of the section, of its reference text, of the donor '
        + 'and of the damaged twin, with the damage\'s kind and size',
      fn: async () => {
        const section = nonNullishOrThrow(SECTIONS[0],);
        const donor = nonNullishOrThrow(SECTIONS[2],);
        const spec = specForDamage({
          entryId: 'tabby',
          section,
          kind: 'insertion',
          damage: {
            kind: 'damaged',
            damageKind: 'insertion',
            damagedText: `${section.referenceText} borrowed`,
            changedChars: 9,
            damageDetail: 'added 9 characters',
          },
          donor,
          corpusSha: DRAW_PIN.commitSha,
          reviewedOn: '2026-10-23',
        },);
        expect(spec.id,).toBe('tabby-s0',);
        expect(spec.archive.hash,).toBe(hashContent({ content: section.archiveRaw, },),);
        expect(spec.referenceHash,).toBe(hashContent({ content: section.referenceText, },),);
        expect(spec.donor.hash,).toBe(hashContent({ content: donor.archiveRaw, },),);
        expect(spec.damages,).toStrictEqual([
          {
            kind: 'insertion',
            hash: hashContent({ content: `${section.referenceText} borrowed`, },),
            changedChars: 9,
          },
        ],);
      },
    },),

    it({
      name: 'HOLDS NO PASSAGE OF EITHER PAGE: only ids, offsets, counts and hashes',
      fn: async () => {
        const section = nonNullishOrThrow(SECTIONS[1],);
        const written = JSON.stringify(specForDamage({
          entryId: 'tabby',
          section,
          kind: 'alteration',
          damage: {
            kind: 'damaged',
            damageKind: 'alteration',
            damagedText: section.referenceText.replace('1987', '1989',),
            changedChars: 1,
            damageDetail: 'changed 1987 to 1989',
          },
          donor: nonNullishOrThrow(SECTIONS[0],),
          corpusSha: DRAW_PIN.commitSha,
          reviewedOn: '2026-10-23',
        },),);
        expect(written.includes('shopkeeper',),).toBe(false,);
        expect(
          written.includes(DRAW_CHINESE.slice(
            0,
            6,
          ),),
        ).toBe(false,);
      },
    },),

    it({
      name: 'JOINS THE INSERTION AND THE ALTERATION DRAWN IN ONE SECTION into one specification holding both damages',
      fn: async () => {
        const section = nonNullishOrThrow(SECTIONS[1],);
        const donor = nonNullishOrThrow(SECTIONS[0],);
        const base = {
          entryId: 'tabby',
          section,
          donor,
          corpusSha: DRAW_PIN.commitSha,
          reviewedOn: '2026-10-23',
        };
        const insertion = specForDamage({
          ...base,
          kind: 'insertion',
          damage: {
            kind: 'damaged',
            damageKind: 'insertion',
            damagedText: `${section.referenceText} borrowed`,
            changedChars: 9,
            damageDetail: 'added 9 characters',
          },
        },);
        const alteration = specForDamage({
          ...base,
          kind: 'alteration',
          damage: {
            kind: 'damaged',
            damageKind: 'alteration',
            damagedText: section.referenceText.replace('1987', '1989',),
            changedChars: 1,
            damageDetail: 'changed 1987 to 1989',
          },
        },);
        const joined = joinedBySection({ specs: [insertion, alteration,], },);
        expect(joined.length,).toBe(1,);
        expect(joined[0]?.damages.map(function kindOf(damage,): string {
          return damage.kind;
        },),).toStrictEqual(['insertion', 'alteration',],);
      },
    },),

    it({
      name: 'LEAVES SPECIFICATIONS OF TWO SECTIONS, AND A LONE ONE, AS THEY ARE',
      fn: async () => {
        const donor = nonNullishOrThrow(SECTIONS[0],);
        const damage = {
          kind: 'damaged',
          damageKind: 'insertion',
          damagedText: 'x',
          changedChars: 1,
          damageDetail: 'added 1 character',
        } as const;
        const first = specForDamage({
          entryId: 'tabby',
          section: nonNullishOrThrow(SECTIONS[1],),
          kind: 'insertion',
          damage,
          donor,
          corpusSha: DRAW_PIN.commitSha,
          reviewedOn: '2026-10-23',
        },);
        const second = specForDamage({
          entryId: 'tabby',
          section: nonNullishOrThrow(SECTIONS[2],),
          kind: 'alteration',
          damage,
          donor,
          corpusSha: DRAW_PIN.commitSha,
          reviewedOn: '2026-10-23',
        },);
        expect(joinedBySection({ specs: [first, second,], },),).toStrictEqual([first, second,],);
        expect(joinedBySection({ specs: [first,], },),).toStrictEqual([first,],);
        expect(joinedBySection({ specs: [], },),).toStrictEqual([],);
      },
    },),
  ],
},);
