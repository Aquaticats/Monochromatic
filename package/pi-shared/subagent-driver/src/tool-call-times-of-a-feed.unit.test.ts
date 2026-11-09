/**
 How long the tool calls of one feed took, and which calls it still has open, read from a feed a test writes and throws away.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { toolCallTimesOfFeed, } from '../dist/final/node/index.mjs';
import { standInRunsFolder, } from './stand-in-runs-folder-for-tests.ts';

/**
 One second, in the milliseconds a feed counts with.
 */
const SECOND = 1_000;

/**
 The arguments a feed keeps for a call.
 */
const ARGUMENTS = 'cd -- /work && mise run test';

await describe({
  name: toolCallTimesOfFeed.name,
  children: [
    it({
      name: "pairs one tool's calls in the order they started, even when two of them overlap",
      fn: async () => {
        await using runs = await standInRunsFolder({
          runs: [
            {
              name: 'a-1',
              feedLines: [
                {
                  t: 0,
                  kind: 'tool_start',
                  tool: 'bash',
                  args: ARGUMENTS,
                },
                {
                  t: SECOND,
                  kind: 'tool_start',
                  tool: 'bash',
                  args: ARGUMENTS,
                },
                {
                  t: 5 * SECOND,
                  kind: 'tool_end',
                  tool: 'bash',
                },
                {
                  t: 9 * SECOND,
                  kind: 'tool_end',
                  tool: 'bash',
                },
              ],
            },
          ],
        },);
        /**
         What the feed says.
         */
        const times = await toolCallTimesOfFeed({ feedFile: runs.entryPathOf('a-1/feed.jsonl',), },);
        expect(times.closedMilliseconds,).toEqual([5 * SECOND, 8 * SECOND,],);
        expect(times.openCalls,).toEqual([],);
      },
    },),
    it({
      name: 'leaves out an end whose start it never saw',
      fn: async () => {
        await using runs = await standInRunsFolder({
          runs: [
            {
              name: 'a-1',
              feedLines: [
                {
                  t: 0,
                  kind: 'tool_start',
                  tool: 'read',
                },
                {
                  t: SECOND,
                  kind: 'tool_end',
                  tool: 'read',
                },
                {
                  t: 2 * SECOND,
                  kind: 'tool_end',
                  tool: 'edit',
                },
              ],
            },
          ],
        },);
        /**
         What the feed says.
         */
        const times = await toolCallTimesOfFeed({ feedFile: runs.entryPathOf('a-1/feed.jsonl',), },);
        expect(times.closedMilliseconds,).toEqual([SECOND,],);
      },
    },),
    it({
      name: 'reports the calls still open, with the arguments and the start the feed recorded',
      fn: async () => {
        await using runs = await standInRunsFolder({
          runs: [
            {
              name: 'a-1',
              feedLines: [
                {
                  t: 4 * SECOND,
                  kind: 'tool_start',
                  tool: 'bash',
                  args: ARGUMENTS,
                },
              ],
            },
          ],
        },);
        /**
         What the feed says.
         */
        const times = await toolCallTimesOfFeed({ feedFile: runs.entryPathOf('a-1/feed.jsonl',), },);
        expect(times.closedMilliseconds,).toEqual([],);
        expect(times.openCalls,).toEqual([
          {
            tool: 'bash',
            startedMilliseconds: 4 * SECOND,
            argumentsPreview: ARGUMENTS,
          },
        ],);
      },
    },),
    it({
      name: 'skips a line cut in half and a line that carries no moment, which a feed being written can hold',
      fn: async () => {
        await using runs = await standInRunsFolder({
          runs: [
            {
              name: 'a-1',
              feedText: [
                '{"t":0,"kind":"tool_start","tool":"bash"}',
                '{"t":1000,"kind":"tool_end","tool":"bash"}',
                '{"kind":"tool_start","tool":"edit"}',
                '{"t":2000,"kind":"assistant","text":"still working"}',
                '{"t":3000,"kind":"tool_st',
              ].join(
                '\n',
                ),
            },
          ],
        },);
        /**
         What the feed says.
         */
        const times = await toolCallTimesOfFeed({ feedFile: runs.entryPathOf('a-1/feed.jsonl',), },);
        expect(times.closedMilliseconds,).toEqual([SECOND,],);
        expect(times.openCalls,).toEqual([],);
      },
    },),
    it({
      name: 'refuses a feed that is not there, so a caller can tell a missing run folder from an empty one',
      fn: async () => {
        await using runs = await standInRunsFolder({ runs: [], },);
        try {
          await toolCallTimesOfFeed({ feedFile: runs.entryPathOf('gone-1/feed.jsonl',), },);
        }
        catch (error) {
          expect(String(error,),).toContain('ENOENT',);
          return;
        }
        throw new Error('a missing feed was read as an empty one',);
      },
    },),
  ],
},);
