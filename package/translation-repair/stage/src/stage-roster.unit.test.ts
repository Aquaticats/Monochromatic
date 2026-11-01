/**
 Tests for the count that separates a fan-out stage nobody asked from one that
 could not answer.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { summarizeStageRoster, } from '../dist/final/node/index.mjs';

await describe({
  name: summarizeStageRoster.name,
  children: [
    it({
      name: 'counts a unit where NO voice answered, which no other number '
        + 'moves. A stage that produced nothing because nobody replied looks '
        + 'exactly like a stage that produced nothing because there was '
        + 'nothing to do',
      fn: async () => {
        expect(summarizeStageRoster({
          entries: [
            [
              'refine-candidates (0/1 heard, 0 proposing)',
              'refine-candidates (1/1 heard, 1 proposing)',
              'refine-selected',
            ],
          ],
          stage: 'refine',
        },),).toEqual({
          offered: 2,
          degraded: 1,
          silent: 1,
        },);
      },
    },),

    it({
      name: 'counts a DEGRADED unit that was not silent, which is the editor '
        + 'case that matters: one editor of two answering still ships a '
        + 'repair, so the outcome is shaped like a healthy one while the '
        + 'ensemble that was supposed to keep any single model off the '
        + 'shipped text has quietly stopped being an ensemble',
      fn: async () => {
        expect(summarizeStageRoster({
          entries: [
            [
              'editor-candidates (1/2 heard, 1 repairing)',
              'editor-candidates (2/2 heard, 2 repairing)',
            ],
          ],
          stage: 'editor',
        },),).toEqual({
          offered: 2,
          degraded: 1,
          silent: 0,
        },);
      },
    },),

    it({
      name: 'reads only the stage it was asked about, so one stage failing '
        + 'cannot be reported against another',
      fn: async () => {
        expect(summarizeStageRoster({
          entries: [
            [
              'editor-candidates (1/2 heard, 1 repairing)',
              'refine-candidates (0/1 heard, 0 proposing)',
            ],
          ],
          stage: 'editor',
        },),).toEqual({
          offered: 1,
          degraded: 1,
          silent: 0,
        },);
      },
    },),

    it({
      name: 'reads two-digit counts correctly rather than by leading digit, '
        + 'since "10/12 heard" opens with a one and a zero and a prefix test '
        + 'would report a healthy roster as a dead one',
      fn: async () => {
        expect(summarizeStageRoster({
          entries: [
            [
              'critic-candidates (10/12 heard, 8 claiming)',
              'critic-candidates (12/12 heard, 9 claiming)',
            ],
          ],
          stage: 'critic',
        },),).toEqual({
          offered: 2,
          degraded: 1,
          silent: 0,
        },);
      },
    },),

    it({
      name: 'SKIPS a finding it cannot read instead of throwing, because this '
        + 'count exists to notice a stage going quiet and refusing drifted '
        + 'wording would silence it in exactly that case',
      fn: async () => {
        expect(summarizeStageRoster({
          entries: [
            [
              'editor-candidates (garbled',
              'editor-candidates (two/three heard)',
              'editor-candidates (1/2 heard, 1 repairing)',
            ],
          ],
          stage: 'editor',
        },),).toEqual({
          offered: 1,
          degraded: 1,
          silent: 0,
        },);
      },
    },),

    it({
      name: 'SKIPS a finding whose counts are not written in digits as the stage writes them: an empty heard '
        + 'count is no stage that heard nobody, and a sign, a hexadecimal or an exponent is no count it '
        + 'writes (ledger B73)',
      fn: async () => {
        expect(summarizeStageRoster({
          entries: [
            [
              'editor-candidates (/2 heard, 0 repairing)',
              'editor-candidates (-1/2 heard, 0 repairing)',
              'editor-candidates (0x1/2 heard, 1 repairing)',
              'editor-candidates (1e0/2 heard, 1 repairing)',
              'editor-candidates (2/2 heard, 2 repairing)',
            ],
          ],
          stage: 'editor',
        },),).toEqual({
          offered: 1,
          degraded: 0,
          silent: 0,
        },);
      },
    },),

    it({
      name: 'counts nothing for an artifact carrying no findings of that '
        + 'stage, so a run predating the stage reads as zero offered rather '
        + 'than as a stage that failed everywhere',
      fn: async () => {
        expect(summarizeStageRoster({
          entries: [[], ['refine-skip',],],
          stage: 'refine',
        },),).toEqual({
          offered: 0,
          degraded: 0,
          silent: 0,
        },);
      },
    },),

    it({
      name: 'READS NO VOICES from a unit line whose heard count has no asked count beside it, since '
        + 'nothing there says how many were asked',
      fn: async () => {
        expect(summarizeStageRoster({
          entries: [['refine-candidates (voices heard),',],],
          stage: 'refine',
        },),).toEqual({
          offered: 0,
          degraded: 0,
          silent: 0,
        },);
      },
    },),
  ],
},);
