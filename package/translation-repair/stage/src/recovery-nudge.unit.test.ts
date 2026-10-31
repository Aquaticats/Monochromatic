/**
 Tests for the recovery round's wording by cause (ledger P10).

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CUT_SHORT_RECOVERY_NUDGE,
  OFF_SHAPE_RECOVERY_NUDGE,
  RECOVERY_NUDGES,
  UNREADABLE_CAUSES,
  unreadableCauseOf,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: unreadableCauseOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS A REPLY THE LENGTH LIMIT STOPPED AS CUT SHORT, whether the cut fell in its answer or in its '
            + 'thinking, since either ran out of room before the shape could be judged',
          fn: async () => {
            expect([
              unreadableCauseOf({
                outcome: { kind: 'schema-mismatch', rawText: '{"purr":', reason: 'truncated-completion', detail: 'cut', },
              },),
              unreadableCauseOf({
                outcome: { kind: 'schema-mismatch', rawText: '', reason: 'truncated-thinking', detail: 'cut', },
              },),
            ],).toEqual(['cut-short', 'cut-short',],);
          },
        },),
        it({
          name: 'READS A LENGTH STOP WITH NO CONTENT AS A RUNAWAY (ledger B1074), whether the content is empty or only '
            + 'whitespace, and keeps a cut that holds content, and a cut inside the thinking, as cut short',
          fn: async () => {
            expect([
              unreadableCauseOf({
                outcome: { kind: 'schema-mismatch', rawText: '', reason: 'truncated-completion', detail: 'cut', },
              },),
              unreadableCauseOf({
                outcome: { kind: 'schema-mismatch', rawText: ' \n\t', reason: 'truncated-completion', detail: 'cut', },
              },),
              unreadableCauseOf({
                outcome: { kind: 'schema-mismatch', rawText: '{"purr":', reason: 'truncated-completion', detail: 'cut', },
              },),
              unreadableCauseOf({
                outcome: { kind: 'schema-mismatch', rawText: '', reason: 'truncated-thinking', detail: 'cut', },
              },),
            ],).toEqual(['runaway', 'runaway', 'cut-short', 'cut-short',],);
          },
        },),
        it({
          name: 'READS EVERY OTHER UNREADABLE ANSWER AS OFF THE SHAPE: unparseable, refused by the guard, a mismatch '
            + 'with no reason recorded, and one that read as a refusal',
          fn: async () => {
            expect([
              unreadableCauseOf({
                outcome: { kind: 'schema-mismatch', rawText: 'meow', reason: 'unparseable-json', detail: 'x', },
              },),
              unreadableCauseOf({
                outcome: { kind: 'schema-mismatch', rawText: '{}', reason: 'caller-guard-rejected', detail: 'x', },
              },),
              unreadableCauseOf({
                outcome: { kind: 'schema-mismatch', rawText: '{}', detail: 'x', },
              },),
              unreadableCauseOf({
                outcome: { kind: 'refusal-shaped', rawText: 'no treats', marker: 'api-refusal-field', },
              },),
            ],).toEqual(['off-shape', 'off-shape', 'off-shape', 'off-shape',],);
          },
        },),
      ],
    },),

    describe({
      name: 'RECOVERY_NUDGES',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS THE CUT-SHORT WORDING FOR A RUNAWAY, which is asked of another model with the plain question and '
            + 'falls back to its own model under the same nudge as before, so a stored payload of that ask still replays',
          fn: async () => {
            expect(UNREADABLE_CAUSES[0],).toBe('runaway',);
            expect(RECOVERY_NUDGES.runaway,).toStrictEqual(CUT_SHORT_RECOVERY_NUDGE,);
          },
        },),
        it({
          name: 'WORDS EACH CAUSE APART, so each is a prompt of its own to the uniqueness claims, and names the cut '
            + 'as running out of room rather than as a shape fault',
          fn: async () => {
            expect({
              causes: Object.keys(RECOVERY_NUDGES,).toSorted(),
              cutShort: RECOVERY_NUDGES['cut-short'],
              offShape: RECOVERY_NUDGES['off-shape'],
              apart: CUT_SHORT_RECOVERY_NUDGE.content !== OFF_SHAPE_RECOVERY_NUDGE.content,
              cutNamesTheLimit: CUT_SHORT_RECOVERY_NUDGE.content.includes('length limit',),
              cutNamesNoShapeFault: !CUT_SHORT_RECOVERY_NUDGE.content.includes('did not match',),
            },).toStrictEqual({
              causes: [...UNREADABLE_CAUSES,].toSorted(),
              cutShort: CUT_SHORT_RECOVERY_NUDGE,
              offShape: OFF_SHAPE_RECOVERY_NUDGE,
              apart: true,
              cutNamesTheLimit: true,
              cutNamesNoShapeFault: true,
            },);
          },
        },),
      ],
    },),
  ],
},);
