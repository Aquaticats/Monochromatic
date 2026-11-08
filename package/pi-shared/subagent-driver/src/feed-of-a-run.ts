/**
 The feed of a run: one JSON line per noted event, as it happens, and the failure that stops it.

 @module
 */

import { createWriteStream, } from 'node:fs';
import { finished, } from 'node:stream/promises';
import { setTimeout as wait, } from 'node:timers/promises';
import {
  NO_RUN_FAILURE,
  SHUTDOWN_GRACE_MILLISECONDS,
} from './driver-constants.ts';
import type {
  PiRecord,
  RunFailure,
} from './driver-types.ts';

/**
 The feed of one run.
 */
export type Feed = {
  /**
   Writes one feed line, with how long the run has lasted.
   */
  readonly note: (entry: PiRecord,) => void;

  /**
   Why the feed is incomplete, when writing it failed.
   */
  readonly problem: () => RunFailure;
};

/**
 Starts the feed of a run.

 @param feedFile - file that receives one JSON line per noted event

 @param startedAt - when the run started, as milliseconds since the epoch

 @param now - where the feed reads the time

 @returns the feed, and how to finish writing it

 @example
 ```ts
 const feed = startFeed({ feedFile: 'feed.jsonl', startedAt: Date.now(), },);
 ```
 */
export function startFeed({
  feedFile,
  startedAt,
  now = Date.now,
}: {
  readonly feedFile: string;
  readonly startedAt: number;
  readonly now?: () => number
},): Feed & { readonly close: () => Promise<void> } {
  /**
   Where feed lines go.
   */
  const feed = createWriteStream(
    feedFile,
    { flags: 'a', },
  );
  /**
   What changes about the feed while the run lasts.
   */
  const state: { problem?: string } = {};
  feed.on(
    'error',
    function onFeedError({ message, }: { readonly message: string; },) {
      state.problem = message;
    },
  );

  /**
   Waits for the feed's last write, keeping its failure as the feed's problem.
   */
  async function waitForFeedToEnd(): Promise<void> {
    try {
      await finished(feed,);
    }
    catch (error) {
      state.problem = String(error,);
    }
  }

  return {
    note: function note(entry: PiRecord,) {
      if (state.problem === undefined) {
        feed.write(`${JSON.stringify({
          t: now() - startedAt,
          ...entry,
        },)}\n`,);
      }
    },
    problem: function problem() {
      return (state.problem === undefined) ? NO_RUN_FAILURE : {
        failed: true,
        problem: state.problem,
      };
    },
    close: async function close() {
      feed.end();
      /**
       The feed's last write, whose failure is the feed's problem wherever it shows.
       */
      const written = waitForFeedToEnd();
      /**
       Which came first: the last write, or the bound.
       */
      const outcome = await Promise.race([
        written,
        wait(
          SHUTDOWN_GRACE_MILLISECONDS,
          'still-writing' as const,
          { ref: false, },
        ),
      ],);
      if (outcome === 'still-writing') {
        state.problem = `the feed was still being written ${String(SHUTDOWN_GRACE_MILLISECONDS,)} ms after the run ended`;
      }
    },
  };
}
