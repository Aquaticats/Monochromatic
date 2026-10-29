import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  _buildLogKey as buildLogKey,
  _compareLogKeys as compareLogKeys,
  _parseLogKey as parseLogKey,
} from '@monochromatic-dev/logger-core';

/**
 Keys the strict parser must reject: host application keys, the
 sessionStorage sink's flat shape, and malformed run identities. Eviction
 safety rests on every one of these staying foreign.
 */
const FOREIGN_KEYS: readonly string[] = [
  'other.key',
  'monochromatic.log',
  'monochromatic.log.5',
  'monochromatic.log.10.ab',
  'monochromatic.log.10.ab.3.4',
  'monochromatic.log.x10.ab.3',
  'monochromatic.log.10..3',
  'monochromatic.log.10.ab.3x',
  'monochromatic.log...',
];

await describe({
  name: parseLogKey.name,
  children: [
    it({
      name: 'round-trips a built key back to its identity',
      fn: async () => {
        /**
         Identity pushed through build-then-parse; equality proves the pair inverse.
         */
        const identity = {
          stamp: 1_753_000_000_000,
          nonce: 'ab12',
          index: 7,
        };
        /**
         Key the builder produced for the identity.
         */
        const built = buildLogKey(identity,);
        /**
         Identity parsed back out of the built key.
         */
        const { parsed, } = parseLogKey(built,);
        expect(parsed,)
          .toEqual({
            key: 'monochromatic.log.1753000000000.ab12.7',
            ...identity,
          },);
      },
    },),

    ...FOREIGN_KEYS.map(function mapForeign(key,) {
      return it({
        name: `rejects foreign key ${key}`,
        fn: async () => {
          expect(parseLogKey(key,).parsed,)
            .toBeUndefined();
        },
      },);
    },),

    it({
      name: 'compareLogKeys orders by stamp, then nonce, then index',
      fn: async () => {
        /**
         Keys deliberately shuffled across all three ordering dimensions.
         */
        const shuffled = [
          'monochromatic.log.2000.aaaa.0',
          'monochromatic.log.1000.bbbb.1',
          'monochromatic.log.1000.aaaa.1',
          'monochromatic.log.1000.aaaa.0',
        ]
          .flatMap(function parseOne(key,) {
            const { parsed, } = parseLogKey(key,);
            return (parsed === undefined) ? [] : [parsed,];
          },);
        /**
         Oldest-first ordering the eviction queue relies on.
         */
        const sorted = shuffled.toSorted(function byOldestFirst(first, second,) {
          return compareLogKeys({
            first,
            second,
          },);
        },);
        expect(sorted.map(function keyOf(parsed,) {
          return parsed.key;
        },),)
          .toEqual([
            'monochromatic.log.1000.aaaa.0',
            'monochromatic.log.1000.aaaa.1',
            'monochromatic.log.1000.bbbb.1',
            'monochromatic.log.2000.aaaa.0',
          ],);
      },
    },),
  ],
},);
