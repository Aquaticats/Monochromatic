/**
 Tests for the class every rendering audit invariant is thrown as.

 SIX SITES, ONE NAME. The invariants are unreachable by construction, so the
 only thing worth pinning is that reaching one is told apart by name from an
 operator refusal or a provider fault, and that the site's own sentence
 travels whole. Fixtures are invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { RenderingAuditInvariantError, } from '../dist/final/node/index.mjs';

await describe({
  name: RenderingAuditInvariantError.name,
  children: [
    it({
      name: 'NAMES ITSELF, so a stack or a refusal line tells an invariant apart from a fault or '
        + 'a stated refusal by name alone',
      fn: async () => {
        /**
         One invariant, broken.
         */
        const broken = new RenderingAuditInvariantError({
          invariant: 'a defect group with no members cannot occur, since groups are built from claims',
        },);

        expect(broken,).toBeInstanceOf(Error,);
        expect(broken.name,).toBe('RenderingAuditInvariantError',);
      },
    },),

    it({
      name: 'CARRIES THE SITE\'S OWN SENTENCE WHOLE behind a fixed prefix, since the sentence is '
        + 'what a reader has to locate the assumption by',
      fn: async () => {
        expect(new RenderingAuditInvariantError({ invariant: 'slice 3 passed the decided filter and is not decided', },).message,)
          .toBe('rendering audit invariant broken: slice 3 passed the decided filter and is not decided',);
      },
    },),
  ],
},);
