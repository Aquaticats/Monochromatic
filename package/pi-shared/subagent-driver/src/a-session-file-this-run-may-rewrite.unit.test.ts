/**
 The class two walk of the fourth review: every answer that picks a session file to mask at rest, held against the one file this run
 may rewrite. A run that made a copy of the session it was given hands Pi that copy, so no answer of Pi's and no fallback may name the
 session the caller named: the file is another run's, and rewriting it would drop what that run appends.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  reportedSessionFile,
  sessionFileToMaskAtRest,
} from '../dist/final/node/index.mjs';

/** The session the caller named, which belongs to another run. */
const NAMED = '/runs/a-1/session.jsonl';

/** The copy this run made of it. */
const COPY = '/runs/b-2/resumed-session.jsonl';

/** A third file, which only this run's own Pi could have made. */
const OWN = '/runs/b-2/session.jsonl';

await describe({
  name: 'a session file this run may rewrite',
  children: [
    it({
      name: 'answers the copy when Pi names nothing at all, and never the session the caller named',
      fn: async () => {
        expect(sessionFileToMaskAtRest({
          copyThisRunMade: COPY,
          sessionTheCallerNamed: NAMED,
        },),).toEqual({ file: { path: COPY, }, },);
        // The same question with the caller's path as the only other candidate: the answer is still the copy.
        expect(sessionFileToMaskAtRest({
          copyThisRunMade: COPY,
          sessionTheCallerNamed: NAMED,
          reported: COPY,
        },),).toEqual({ file: { path: COPY, }, },);
      },
    },),
    it({
      name: 'refuses the session the caller named when Pi names it, and says which file it left alone',
      fn: async () => {
        /**
         What the decision answered.
         */
        const answered = sessionFileToMaskAtRest({
          copyThisRunMade: COPY,
          sessionTheCallerNamed: NAMED,
          reported: NAMED,
        },);
        expect(answered,).toHaveProperty('refused',);
        expect(JSON.stringify(answered,),).toContain(NAMED,);
        expect(answered,).not.toHaveProperty('file',);
      },
    },),
    it({
      name: 'answers a third file only when this run made no copy of the session it was given',
      fn: async () => {
        // A file this run's own Pi created: no copy stands between it and the run.
        expect(sessionFileToMaskAtRest({
          reported: OWN,
          sessionTheCallerNamed: NAMED,
        },),).toEqual({ file: { path: OWN, }, },);
        // A run that was given a session that is not there hands Pi that path, and its Pi creates the file: no copy was made, so the
        // file is this run's own and may be rewritten.
        expect(sessionFileToMaskAtRest({
          reported: NAMED,
          sessionTheCallerNamed: NAMED,
        },),).toEqual({ file: { path: NAMED, }, },);
      },
    },),
    it({
      name: 'answers nothing when no answer named a file and no copy was made',
      fn: async () => {
        expect(sessionFileToMaskAtRest({},),).toEqual({ absent: true, },);
        expect(sessionFileToMaskAtRest({ sessionTheCallerNamed: NAMED, },),).toEqual({ absent: true, },);
      },
    },),
    it({
      name: 'reads the file Pi named in its statistics first, and in its own state when the statistics say nothing',
      fn: async () => {
        expect(reportedSessionFile({ sessionStats: { sessionFile: OWN, }, state: COPY, },),).toEqual({ reported: OWN, },);
        expect(reportedSessionFile({ sessionStats: {}, state: COPY, },),).toEqual({ reported: COPY, },);
        expect(reportedSessionFile({ sessionStats: {}, },),).toEqual({},);
        expect(reportedSessionFile({ sessionStats: undefined, state: COPY, },),).toEqual({ reported: COPY, },);
        // A statistics answer of any other shape says nothing about a file, which is not a file named.
        expect(reportedSessionFile({ sessionStats: 'no session here', },),).toEqual({},);
        expect(reportedSessionFile({ sessionStats: { sessionFile: 7, }, },),).toEqual({},);
      },
    },),
  ],
},);
