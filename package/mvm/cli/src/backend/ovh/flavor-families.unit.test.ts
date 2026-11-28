/**
 Unit tests for the families mvm may pick a machine size from without being told.

 The flavor names here are the catalogue's own names, copied from a live read of
 one project's `BHS5` flavor list on 2026-11-28 (`diagnosis-2.log`,
 `diagnosis-3-marks.log` in that round's scratch): family tokens such as `d2`,
 `c3`, `b2`, `t1`, `i1` and `bm`, and the bare-metal names `bm-l1`, `bm-m1`,
 `bm-s1`. Which families OVHcloud documents as GPU, storage shaped or bare metal
 is cited in the module doc of `flavor-families.ts`. The vCPU, memory and disk
 numbers are the ones that read carried.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  flavorFamilyOf,
  isKeptFlavor,
  KEPT_FLAVOR_FAMILIES,
} from '../../../dist/final/node/index.mjs';
import { flavorAnswer, } from './fixture.ovh-api-stand-in.ts';

//region Fixtures

/**
 Families whose sizes mvm may pick, as the module under test states them.
 */
const EXPECTED_KEPT = [
  'b2',
  'b3',
  'c2',
  'c3',
  'd2',
  'r2',
  'r3',
] as const;

/**
 Names `mvm` must never choose on its own, one per documented special family, each
 with the reason it is special and where its name comes from.

 A name marked `captured` appears in the live `BHS5` flavor list this file's
 header cites. A name marked `doc-family` is a family OVHcloud documents with a
 `-*` wildcard, so the trailing number is a stand-in for a size and not a claim
 that a flavor of that number exists: the rule reads only the family token before
 the first hyphen, which is the part the document names.
 */
const LEFT_OUT = [
  {
    name: 't1-45',
    reason: 'a GPU machine',
    source: 'captured',
  },
  {
    name: 't1-le-45',
    reason: 'a GPU machine',
    source: 'captured',
  },
  {
    name: 't2-1',
    reason: 'a GPU machine',
    source: 'doc-family',
  },
  {
    name: 'g1-1',
    reason: 'a GPU machine',
    source: 'doc-family',
  },
  {
    name: 'g2-1',
    reason: 'a GPU machine',
    source: 'doc-family',
  },
  {
    name: 'g3-1',
    reason: 'a GPU machine',
    source: 'doc-family',
  },
  {
    name: 'a100-1',
    reason: 'a GPU machine',
    source: 'doc-family',
  },
  {
    name: 'l4-1',
    reason: 'a GPU machine',
    source: 'doc-family',
  },
  {
    name: 'l40s-1',
    reason: 'a GPU machine',
    source: 'doc-family',
  },
  {
    name: 'rtx5000-1',
    reason: 'a GPU machine',
    source: 'doc-family',
  },
  {
    name: 'i1-45',
    reason: 'a storage shaped machine',
    source: 'captured',
  },
  {
    name: 'bm-l1',
    reason: 'a bare metal machine',
    source: 'captured',
  },
  {
    name: 'bm-m1',
    reason: 'a bare metal machine',
    source: 'captured',
  },
  {
    name: 'bm-s1',
    reason: 'a bare metal machine',
    source: 'captured',
  },
  {
    name: 'win-i1-45',
    reason: 'a Windows machine',
    source: 'captured',
  },
] as const;

/**
 Names `mvm` may pick on its own, each from a family the real catalogue carries in
 stock and that is a general-purpose Linux virtual machine.
 */
const KEPT = [
  'd2-2',
  'd2-4',
  'c3-4',
  'c3-4-flex',
  'b2-7',
  'b2-7-flex',
  'b3-8',
  'r2-15',
  'r3-16',
] as const;

//endregion Fixtures

await describe({
  name: '',
  concurrency: 1,
  children: [
    //region The families

    describe({
      name: 'KEPT_FLAVOR_FAMILIES',
      children: [
        it({
          name: 'keeps exactly the documented general-purpose Linux families',
          fn: async () => {
            expect([...KEPT_FLAVOR_FAMILIES,],).toEqual([...EXPECTED_KEPT,],);
          },
        },),
      ],
    },),

    describe({
      name: flavorFamilyOf.name,
      children: [
        it({
          name: 'reads the family token before the first hyphen',
          fn: async () => {
            expect(flavorFamilyOf({ name: 'd2-2', },),).toBe('d2',);
            expect(flavorFamilyOf({ name: 'b2-7-flex', },),).toBe('b2',);
            expect(flavorFamilyOf({ name: 't1-le-45', },),).toBe('t1',);
          },
        },),
        it({
          name: 'reads a name with no hyphen as its own family, in lower case',
          fn: async () => {
            expect(flavorFamilyOf({ name: 'D2', },),).toBe('d2',);
          },
        },),
      ],
    },),

    //endregion The families

    //region What may be picked

    describe({
      name: isKeptFlavor.name,
      children: [
        ...KEPT.map(function mapKept(name,) {
          return it({
            name: `keeps ${name}`,
            fn: async () => {
              const flavor = flavorAnswer({ id: name, },);
              expect(isKeptFlavor({ flavor, },),).toBe(true,);
            },
          },);
        },),
        ...LEFT_OUT.map(function mapLeftOut(entry,) {
          return it({
            name: `leaves out ${entry.name}, which is ${entry.reason}`,
            fn: async () => {
              const flavor = flavorAnswer({ id: entry.name, },);
              expect(isKeptFlavor({ flavor, },),).toBe(false,);
            },
          },);
        },),
        it({
          name: 'leaves out a family it does not know, so a size added later is never picked unseen',
          fn: async () => {
            const flavor = flavorAnswer({ id: 'z9-64', },);
            expect(isKeptFlavor({ flavor, },),).toBe(false,);
          },
        },),
        it({
          name: 'leaves out a kept family whose flavor targets another operating system',
          fn: async () => {
            const flavor = flavorAnswer({
              id: 'b3-8',
              osType: 'windows',
            },);
            expect(isKeptFlavor({ flavor, },),).toBe(false,);
          },
        },),
      ],
    },),

    //endregion What may be picked
  ],
},);
