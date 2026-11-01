/**
 Tests for the panel-vote-state guard fencing untrusted model strings
 out of typed ballots.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  isPanelVoteState,
  PANEL_VOTE_STATES,
} from '../dist/final/node/index.mjs';

await describe({
  name: isPanelVoteState.name,
  children: [
    ...PANEL_VOTE_STATES.map(function toCase(state,) {
      return it({
        name: `admits ${state}`,
        fn: async () => {
          expect(isPanelVoteState(state,),).toBe(true,);
        },
      },);
    },),
    it({
      name: 'rejects unlisted strings',
      fn: async () => {
        expect(isPanelVoteState('purring',),).toBe(false,);
      },
    },),
    it({
      name: 'rejects non-strings',
      fn: async () => {
        expect(isPanelVoteState(1,),).toBe(false,);
        expect(isPanelVoteState(undefined,),).toBe(false,);
        expect(isPanelVoteState(['supported',],),).toBe(false,);
      },
    },),
  ],
},);
