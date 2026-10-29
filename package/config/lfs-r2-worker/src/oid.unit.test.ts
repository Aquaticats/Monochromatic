/**
 Tests for oid classification.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isOid,
  OID_LENGTH,
} from '@monochromatic-dev/config-lfs-r2-worker';

/**
 Well-formed oid used across cases.
 */
const VALID_OID = '0123456789abcdef'.repeat(4,);

await describe({
  name: '',
  children: [
    describe({
      name: 'OID_LENGTH',
      children: [
        it({
          name: 'is the sha256 hex length',
          fn: async () => {
            expect(OID_LENGTH,).toBe(64,);
            expect(VALID_OID,).toHaveLength(OID_LENGTH,);
          },
        },),
      ],
    },),
    describe({
      name: isOid.name,
      children: [
        it({
          name: 'accepts 64 lowercase hex characters',
          fn: async () => {
            expect(isOid(VALID_OID,),).toBe(true,);
            expect(
              isOid('a'.repeat(64,),),
            ).toBe(true,);
          },
        },),
        it({
          name: 'rejects a shorter or longer string',
          fn: async () => {
            expect(
              isOid(VALID_OID.slice(1,),),
            ).toBe(false,);
            expect(isOid(`${VALID_OID}0`,),).toBe(false,);
            expect(isOid('',),).toBe(false,);
          },
        },),
        it({
          name: 'rejects uppercase hex',
          fn: async () => {
            expect(
              isOid(VALID_OID.toUpperCase(),),
            ).toBe(false,);
          },
        },),
        it({
          name: 'rejects a non-hex character anywhere',
          fn: async () => {
            expect(isOid(`g${VALID_OID.slice(1,)}`,),).toBe(false,);
            expect(isOid(`${VALID_OID.slice(0, -1,)}/`,),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
