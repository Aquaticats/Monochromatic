import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { $, } from './index.ts';

await describe({
  name: $.name,
  children: [
    it({
      name: 'returns a valid UUID v4 format',
      fn: async () => {
        const uuidV4Regex =
          // oxlint-disable-next-line no-restricted-syntax/no-regex -- canonical UUIDv4 format validator; the structure (8-4-4-4-12 hex with fixed version/variant nibbles) is the test's contract. Input is a 36-char UUID, no nested quantifiers, linear matching.
          /^[\da-f]{8}-[\da-f]{4}-4[\da-f]{3}-[89ab][\da-f]{3}-[\da-f]{12}$/i;
        const uuid = $({},);
        expect(uuid,).toMatch(uuidV4Regex,);
      },
    },),

    it({
      name: 'generates different UUIDs on multiple calls',
      fn: async () => {
        const uuid1 = $({},);
        const uuid2 = $({},);
        expect(uuid1,).not.toBe(uuid2,);
      },
    },),

    it({
      name: 'accepts empty object parameter',
      fn: async () => {
        const uuidV4Regex =
          // oxlint-disable-next-line no-restricted-syntax/no-regex -- canonical UUIDv4 format validator; the structure (8-4-4-4-12 hex with fixed version/variant nibbles) is the test's contract. Input is a 36-char UUID, no nested quantifiers, linear matching.
          /^[\da-f]{8}-[\da-f]{4}-4[\da-f]{3}-[89ab][\da-f]{3}-[\da-f]{12}$/i;
        const uuid = $({},);
        expect(uuid,).toMatch(uuidV4Regex,);
      },
    },),

    it({
      name: 'generates unique UUIDs across multiple calls',
      fn: async () => {
        const uuids = new Set<string>();
        const callCount = 100;

        for (let callIndex = 0; callIndex < callCount; callIndex++)
          uuids.add($({},),);

        expect(uuids.size,).toBe(callCount,);
      },
    },),

    it({
      name: 'UUID version is always 4',
      fn: async () => {
        const uuids = Array.from({ length: 50, }, () => $({},),);

        uuids.forEach(uuid => {
          const versionChar = uuid.charAt(14,);
          expect(versionChar,).toBe('4',);
        },);
      },
    },),

    it({
      name: 'UUID variant is correct (8, 9, a, or b at position 19)',
      fn: async () => {
        const uuids = Array.from({ length: 50, }, () => $({},),);
        const validVariants = ['8', '9', 'a', 'b', 'A', 'B',];

        uuids.forEach(uuid => {
          const variantChar = uuid.charAt(19,);
          expect(validVariants,).toContain(variantChar.toLowerCase(),);
        },);
      },
    },),

    it({
      name: 'produces lowercase UUIDs',
      fn: async () => {
        const uuid = $({},);
        expect(uuid,).toBe(uuid.toLowerCase(),);
      },
    },),

    it({
      name: 'produces 36 character string (32 hex + 4 dashes)',
      fn: async () => {
        const uuid = $({},);
        expect(uuid,).toHaveLength(36,);
        expect(uuid.split('-',),).toHaveLength(5,);
      },
    },),
  ],
},);
