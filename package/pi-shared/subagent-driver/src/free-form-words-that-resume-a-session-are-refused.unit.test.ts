/**
 The class four walk: the free-form words a run passes to Pi are read with Pi's own argument reader, and every spelling that names or
 resumes a session is refused with its reason, so no run resumes entries this package never masks.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  refusedFreeFormWords,
  keepsNoSession,
} from '../dist/final/node/index.mjs';

/**
 The free-form word lists, each with whether it resumes a session.
 */
const LISTS: readonly {
  readonly called: string;
  readonly words: readonly string[];
  readonly resumes: boolean;
}[] = [
  { called: 'a session named as its own word', words: ['--session', '/tmp/old.jsonl',], resumes: true, },
  { called: 'a session id', words: ['--session-id', 'abc',], resumes: true, },
  { called: 'a fork of another session', words: ['--fork', 'abc',], resumes: true, },
  { called: 'the most recent session', words: ['--continue',], resumes: true, },
  { called: 'the most recent session by its short word', words: ['-c',], resumes: true, },
  { called: 'a session to pick', words: ['--resume',], resumes: true, },
  { called: 'a session to pick by its short word', words: ['-r',], resumes: true, },
  { called: 'a model and nothing else', words: ['--model', 'x',], resumes: false, },
  { called: 'an extension', words: ['--extension', '/tmp/a.ts',], resumes: false, },
  { called: 'a system prompt', words: ['--system-prompt', 'carry on',], resumes: false, },
  { called: 'no session at all', words: ['--no-session',], resumes: false, },
  { called: 'a session directory alone, which resumes nothing', words: ['--session-dir', '/tmp/sessions',], resumes: false, },
];

await describe({
  name: 'free-form words that resume a session are refused',
  children: [
    ...LISTS.map(function eachList(list,) {
      return it({
        name: `${list.called} is ${list.resumes ? 'refused' : 'accepted'}`,
        fn: async () => {
          /**
           What the driver's reader refuses.
           */
          const refused = refusedFreeFormWords({ words: list.words, },);
          expect(refused.length > 0,).toBe(list.resumes,);
          for (const one of refused) {
            expect(list.words,).toContain(one.word,);
            expect(one.reason,).toContain('--session',);
          }
        },
      },);
    },),
    it({
      name: 'says whether the run keeps no session at all, which leaves nothing for the session\'s own masking to clear',
      fn: async () => {
        expect(keepsNoSession({ words: ['--no-session',], },),).toBe(true,);
        expect(keepsNoSession({ words: ['--model', 'x',], },),).toBe(false,);
      },
    },),
  ],
},);
