/**
 Tests for the editors one editor calibration seats when its command line names editor cards, over the real command
 line declaration and the real roster, in which no model is called.

 THE REFUSALS ARE THE POINT, as in `probe-candidates.unit.test.ts`: a calibration started for a card it then quietly
 ran without would print a standing over the wrong editors.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import {
  type CommandLineOf,
  RUN_WRITERS,
} from '@monochromatic-dev/translation-repair-run';
import {
  EDITOR_LANE_EDITORS,
  editorChairFor,
} from '../../dist/final/node/index.mjs';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';

//region Fixtures

/**
 The Sonnet 5.5 card at the lowest effort, which holds `judge-unmeasured` and so sits on no roster-derived bench.
 */
const LOW = 'claude-sonnet-5-5-effort-low';

/**
 The same wire model at another effort, which one bench may not seat beside {@link LOW}.
 */
const HIGH = 'claude-sonnet-5-5-effort-high';

/**
 The editor calibration's command line, read as `reportingRefusals` reads it.

 @param typed - arguments after the script path

 @returns The line its body would receive

 @example
 ```ts
 const line = calibrateLine({ typed: ['--candidates', LOW, '--candidates-alone',], },);
 ```
 */
function calibrateLine({ typed, }: { readonly typed: readonly string[]; },): CommandLineOf<'editor-calibrate'> {
  return lineOf({
    command: 'editor-calibrate',
    typed,
  },);
}

//endregion Fixtures

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: editorChairFor.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SEATS EXACTLY THE ONE NAMED CARD AND NO OTHER under --candidates-alone: not the roster\'s writers, not '
            + 'plain Sonnet 5.5, which the run names as an editor',
          fn: async () => {
            expect(
              editorChairFor({ line: calibrateLine({ typed: ['--candidates', LOW, '--candidates-alone',], },), },),
            ).toEqual({
              editors: [LOW,],
              named: true,
            },);
          },
        },),

        it({
          name: 'SEATS THE ROSTER\'S WRITERS THEN THE NAMED CARD without --candidates-alone, and leaves the run\'s own '
            + 'named editor out, so a Sonnet effort card sits in a chair its own wire model does not already hold',
          fn: async () => {
            /**
             What the calibration seats for the card beside the writers.
             */
            const chair = editorChairFor({ line: calibrateLine({ typed: ['--candidates', LOW,], },), },);

            expect(chair.editors,).toEqual([...RUN_WRITERS, LOW,],);
            expect(chair.editors,).not.toContain('claude-sonnet-5-5',);
            expect(chair.named,).toBe(true,);
          },
        },),

        it({
          name: 'SEATS THE LANE\'S OWN EDITORS, and says no card was named, when neither flag is written',
          fn: async () => {
            expect(editorChairFor({ line: calibrateLine({ typed: ['3',], },), },),).toEqual({
              editors: EDITOR_LANE_EDITORS,
              named: false,
            },);
          },
        },),

        it({
          name: 'REFUSES A CARD THE ROSTER DOES NOT KNOW, in the words --candidates refuses it in on the fidelity probe, '
            + 'naming the ids it does know',
          fn: async () => {
            /**
             Reads a line that names an unseatable card.
             */
            function readUnknownCard(): void {
              editorChairFor({
                line: calibrateLine({ typed: ['--candidates', 'nobody/such-model', '--candidates-alone',], },),
              },);
            }

            expect(readUnknownCard,).toThrow(StatedRefusalError,);
            expect(readUnknownCard,).toThrow(
              '--candidates names "nobody/such-model", which is not seatable; the roster knows ',
            );
            expect(readUnknownCard,).toThrow(LOW,);
          },
        },),

        it({
          name: 'REFUSES --candidates-alone over nobody, rather than seating the lane\'s editors under a flag that '
            + 'meant to leave them out',
          fn: async () => {
            expect(() => {
              editorChairFor({ line: calibrateLine({ typed: ['--candidates-alone',], },), },);
            },).toThrow(
              '--candidates-alone runs the candidates without the seated roster, and --candidates named none',
            );
          },
        },),

        it({
          name: 'REFUSES TWO EFFORTS OF ONE WIRE MODEL ON ONE CHAIR, alone or beside the writers, since one model heard '
            + 'twice counts as two voices',
          fn: async () => {
            expect(() => {
              editorChairFor({
                line: calibrateLine({ typed: ['--candidates', `${LOW},${HIGH}`, '--candidates-alone',], },),
              },);
            },).toThrow('The editor calibration editors bench seats claude-sonnet-5-5 more than once',);
            expect(() => {
              editorChairFor({ line: calibrateLine({ typed: ['--candidates', `${LOW},${HIGH}`,], },), },);
            },).toThrow('The editor calibration editors bench seats claude-sonnet-5-5 more than once',);
          },
        },),
      ],
    },),
  ],
},);
