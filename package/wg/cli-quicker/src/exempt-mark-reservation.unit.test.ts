import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  assertUnreservedExemptMark,
  formatMark,
  RECOMMENDED_EXEMPT_MARK,
  reservedMarkCollisions,
} from '../dist/final/node/exempt-mark-reservation.mjs';

/**
 Mark `wg-quicker` recommended before the netavark collision was traced.
 */
const HISTORIC_EXEMPT_MARK = 8_888;

await describe({
  name: formatMark.name,
  children: [
    it({
      name: 'renders the historic exempt mark the way nft prints it',
      fn: async () => {
        expect(formatMark({ mark: HISTORIC_EXEMPT_MARK, },),).toBe('0x22b8',);
      },
    },),
    it({
      name: 'renders the netavark masquerade mask',
      fn: async () => {
        expect(formatMark({ mark: 0x20_00, },),).toBe('0x2000',);
      },
    },),
  ],
},);

await describe({
  name: reservedMarkCollisions.name,
  children: [
    it({
      name: 'reports netavark for a mark carrying its masquerade bit',
      fn: async () => {
        expect(reservedMarkCollisions({ mark: HISTORIC_EXEMPT_MARK, },)
          .map(function toMask({ mask, },) {
            return mask;
          },),).toEqual([0x20_00,],);
      },
    },),
    it({
      name: 'reports netavark for the bare masquerade mask',
      fn: async () => {
        expect(reservedMarkCollisions({ mark: 0x20_00, },).length,).toBe(1,);
      },
    },),
    it({
      name: 'reports nothing for the historic mark with the netavark bit cleared',
      fn: async () => {
        expect(reservedMarkCollisions({ mark: 0x02_B8, },).length,).toBe(0,);
      },
    },),
    it({
      name: 'reports nothing for the recommended mark',
      fn: async () => {
        expect(reservedMarkCollisions({ mark: RECOMMENDED_EXEMPT_MARK, },).length,).toBe(0,);
      },
    },),
    it({
      name: 'reports nothing for a mark whose neighbouring bits are set',
      fn: async () => {
        expect(reservedMarkCollisions({ mark: 0xD0_00, },).length,).toBe(0,);
      },
    },),
  ],
},);

await describe({
  name: assertUnreservedExemptMark.name,
  children: [
    it({
      name: 'accepts the recommended mark',
      fn: async () => {
        expect(() => {
          assertUnreservedExemptMark({
            key: 'ExemptMark',
            mark: RECOMMENDED_EXEMPT_MARK,
          },);
        },).not.toThrow();
      },
    },),
    it({
      name: 'rejects the bare netavark masquerade mask',
      fn: async () => {
        expect(() => {
          assertUnreservedExemptMark({
            key: 'ExemptMark',
            mark: 0x20_00,
          },);
        },).toThrow();
      },
    },),
    ...[
      '8888',
      '0x22b8',
      '0x2000',
      'netavark',
      String(RECOMMENDED_EXEMPT_MARK,),
    ].map(function rejectionNames(fragment,) {
      return it({
        name: `rejects the historic mark, naming ${fragment}`,
        fn: async () => {
          expect(() => {
            assertUnreservedExemptMark({
              key: 'ExemptMark',
              mark: HISTORIC_EXEMPT_MARK,
            },);
          },).toThrow(fragment,);
        },
      },);
    },),
  ],
},);
