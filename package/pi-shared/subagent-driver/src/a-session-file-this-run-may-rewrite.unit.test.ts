/**
 The class two walk of the fourth review, and the fifth review's third defect: every answer that picks a session file to mask at rest,
 held against the one file this run may rewrite. A run that made a copy of the session it was given hands Pi that copy, so no answer
 of Pi's and no fallback may name another file; a run that made no copy rewrites only a file that did not exist before it started.

 @module
 */

import { mkdtemp, rm, writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  isAFileThisRunMayRewrite,
  reportedSessionFile,
  sessionFileToMaskAtRest,
} from '../dist/final/node/index.mjs';

/** The session the caller named, which belongs to another run. */
const NAMED = '/runs/a-1/session.jsonl';

/** The copy this run made of it. */
const COPY = '/runs/b-2/resumed-session.jsonl';

/** A third file, which only this run's own Pi could have made. */
const OWN = '/runs/b-2/session.jsonl';

/** A moment before any file of a run made during the test exists. */
const LONG_AGO = new Date(0,);

await describe({
  name: 'a session file this run may rewrite',
  children: [
    it({
      name: 'answers the copy when Pi names nothing at all, and never the session the caller named',
      fn: async () => {
        expect(await sessionFileToMaskAtRest({
          copyThisRunMade: COPY,
          sessionTheCallerNamed: NAMED,
          runStartedAt: LONG_AGO,
        },),).toEqual({ file: { path: COPY, }, },);
        expect(await sessionFileToMaskAtRest({
          copyThisRunMade: COPY,
          sessionTheCallerNamed: NAMED,
          reported: COPY,
          runStartedAt: LONG_AGO,
        },),).toEqual({ file: { path: COPY, }, },);
      },
    },),
    it({
      name: 'refuses a file Pi names when this run made a copy, and says which file it left alone',
      fn: async () => {
        /**
         What the decision answered for the session this run was given.
         */
        const answered = await sessionFileToMaskAtRest({
          copyThisRunMade: COPY,
          sessionTheCallerNamed: NAMED,
          reported: NAMED,
          runStartedAt: LONG_AGO,
        },);
        expect(answered,).toHaveProperty('refused',);
        expect(JSON.stringify(answered,),).toContain(NAMED,);
        expect(answered,).not.toHaveProperty('file',);
        // Any other file Pi names is one this run did not make, which is the fifth review's third defect.
        /**
         What the decision answered for a file this run never made.
         */
        const third = await sessionFileToMaskAtRest({
          copyThisRunMade: COPY,
          sessionTheCallerNamed: NAMED,
          reported: '/runs/c-3/another-runs.jsonl',
          runStartedAt: LONG_AGO,
        },);
        expect(third,).toHaveProperty('refused',);
        expect(JSON.stringify(third,),).toContain('/runs/c-3/another-runs.jsonl',);
      },
    },),
    it({
      name: 'answers a file only when this run made no copy and the file did not exist before the run began',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         A file made now, which a run that started earlier can have made.
         */
        const madeNow = join(folder.path, 'made-now.jsonl',);
        await writeFile(madeNow, '{}\n',);
        expect(await sessionFileToMaskAtRest({
          reported: madeNow,
          runStartedAt: LONG_AGO,
        },),).toEqual({ file: { path: madeNow, }, },);
        // A file that existed before the run began belongs to another run, and a run that made a copy never rewrites it.
        /**
         What the decision answers for a file that was there first.
         */
        const tooOld = await sessionFileToMaskAtRest({
          reported: madeNow,
          runStartedAt: new Date(Date.now() + 60_000,),
        },);
        expect(tooOld,).toHaveProperty('refused',);
        expect(JSON.stringify(tooOld,),).toContain(madeNow,);
        // The session the caller named is one Pi creates for this run when it was not there, and is this run's own.
        expect(await sessionFileToMaskAtRest({
          reported: NAMED,
          sessionTheCallerNamed: NAMED,
          runStartedAt: LONG_AGO,
        },),).toEqual({ file: { path: NAMED, }, },);
      },
    },),
    it({
      name: 'refuses a file it cannot look at, and answers nothing when no answer named a file',
      fn: async () => {
        /**
         What the decision answers for a path that is not there.
         */
        const missing = await sessionFileToMaskAtRest({
          reported: '/runs/c-3/not-there.jsonl',
          runStartedAt: LONG_AGO,
        },);
        expect(missing,).toHaveProperty('refused',);
        expect(await sessionFileToMaskAtRest({ runStartedAt: LONG_AGO, },),).toEqual({ absent: true, },);
        expect(await sessionFileToMaskAtRest({ sessionTheCallerNamed: NAMED, runStartedAt: LONG_AGO, },),).toEqual({ absent: true, },);
      },
    },),
    it({
      name: 'hands out no file a caller can make for itself',
      fn: async () => {
        /**
         A file a caller made up, which this module never handed out.
         */
        const invented = { path: NAMED, };
        expect(isAFileThisRunMayRewrite({ file: invented, },),).toBe(false,);
        /**
         What the decision handed out.
         */
        const answered = await sessionFileToMaskAtRest({
          copyThisRunMade: COPY,
          runStartedAt: LONG_AGO,
        },);
        expect(('file' in answered) && isAFileThisRunMayRewrite({ file: answered.file, },),).toBe(true,);
        expect(answered,).toEqual({ file: { path: COPY, }, },);
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

/**
 Makes a disposable folder.

 @returns the folder and the way to remove it
 */
async function makeFolder(): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  /**
   The folder.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-session-decision-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}
