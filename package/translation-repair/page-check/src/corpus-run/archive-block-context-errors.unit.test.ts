/**
 Tests the words and the fields of the two refusals a block's source section can meet
 (ledger B717): a map with no entry for the block, and an alignment with no pair for it.

 Fixtures are invented identities about a cat.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ArchiveBlockAlignmentMissingError,
  ArchiveBlockContextMissingError,
} from '../../dist/final/node/index.mjs';

await describe({
  name: 'archive block context errors',
  children: [
    it({
      name: 'NAMES THE BLOCK AND EVERY IDENTITY THE MAP HOLDS, in the map\'s order, and carries both as fields',
      fn: async () => {
        const refusal = new ArchiveBlockContextMissingError({
          identity: 'identity-0',
          held: ['identity-7', 'identity-9',],
        },);
        expect(refusal.message,).toContain('archive block identity-0',);
        expect(refusal.message,).toContain('holds 2 blocks: identity-7; identity-9',);
        expect(refusal.identity,).toBe('identity-0',);
        expect(refusal.held,).toEqual(['identity-7', 'identity-9',],);
        expect(refusal.messageNamesOnly,).toBe(true,);
        expect(refusal.name,).toBe('ArchiveBlockContextMissingError',);
      },
    },),
    it({
      name: 'SAYS THE MAP HOLDS NO BLOCK when it is empty, and one block in the singular',
      fn: async () => {
        expect(new ArchiveBlockContextMissingError({ identity: 'identity-0', held: [], },).message,)
          .toContain('the map holds no block,',);
        expect(new ArchiveBlockContextMissingError({ identity: 'identity-0', held: ['identity-7',], },).message,)
          .toContain('holds 1 block: identity-7',);
      },
    },),
    it({
      name: 'NAMES THE BLOCK, THE PAIR IT SITS IN AND HOW MANY PAIRS THE ALIGNMENT HOLDS, one pair in the singular',
      fn: async () => {
        const many = new ArchiveBlockAlignmentMissingError({ blockId: 'block/3', pairIndex: 9, pairCount: 2, },);
        expect(many.message,).toContain('archive block block/3 sits in aligned pair 9',);
        expect(many.message,).toContain('holds 2 pairs',);
        expect(many.blockId,).toBe('block/3',);
        expect(many.pairIndex,).toBe(9,);
        expect(many.pairCount,).toBe(2,);
        expect(many.messageNamesOnly,).toBe(true,);
        expect(many.name,).toBe('ArchiveBlockAlignmentMissingError',);
        expect(new ArchiveBlockAlignmentMissingError({ blockId: 'block/3', pairIndex: 1, pairCount: 1, },).message,)
          .toContain('holds 1 pair,',);
      },
    },),
  ],
},);
