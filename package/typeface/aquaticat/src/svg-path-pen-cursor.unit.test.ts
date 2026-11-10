/**
 The pen cursor that replays SVG path commands has one owner.

 Glyph bounds resolution and OpenType path emission each replayed the same
 M/L/H/V/Z state machine by hand: M and L set both axes, H rewrites X, V rewrites
 Y, and Z closes without moving the pen. Two hand-written replays can disagree on
 a new command or on close handling, and then a glyph's bounds and its drawn
 contour describe different shapes. `svg-path-pen-cursor.ts` owns the replay,
 and the cases below hold both consumers to its events.
 */
/* oxlint-disable test-import/require-eventual-artifact -- this package ships src/ (package.json `files`) and builds font binaries rather than a JS artifact, so the modules beside this test are the shipped ones; see doc/planning/oxlint-test-import-eventual-artifact.md, section "Packages that ship source". */
/* oxlint-disable import/no-named-as-default-member -- opentype.js's UMD bundle defeats cjs-module-lexer's named-export detection under Node's CJS/ESM interop, so the default import's .Path member is the only one that resolves at runtime; see doc/troubleshooting/opentype-js-cjs-esm-interop.md */
import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import opentype from 'opentype.js';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { fontY, } from './build-font-metrics.ts';
import { addFilledPath, } from './build-font-paths.ts';
import { resolveAbsolutePoints, } from './build-font-resolve-points.ts';
import { parseSvgPathD, } from './parse-svg.ts';
import {
  type PenCursorEvent,
  replayPenCursor,
} from './svg-path-pen-cursor.ts';

//region Fixtures

/**
 Describes one replayed event in a form a failure message can show.
 
 @param event - event to describe
 
 @returns compact description of the event
 */
function describeEvent({ event, }: { readonly event: PenCursorEvent; },): string {
  return event.kind === 'close'
    ? 'close'
    : `${event.kind}(${event.x}, ${event.y})`;
}

//endregion Fixtures

//region Source scanning

/**
 Names the modules beside this suite whose text contains a needle.
 
 @param needle - literal to look for
 
 @returns file names of the matching modules
 */
async function modulesContaining({ needle, }: { readonly needle: string; },): Promise<string[]> {
  /**
   File names in this module's directory.
   */
  const names = await readdir(import.meta.dirname,);
  /**
   Non-test modules among them.
   */
  const sourceNames = names.filter(function isSourceModule(name,) {
    return name.endsWith('.ts',)
      && (!name.endsWith('.test.ts',))
      && (name !== 'env.d.ts');
  },);
  /**
   Text of every source module, paired with its name.
   */
  const sources = await Promise.all(sourceNames.map(async function readModule(name,) {
    return {
      name,
      text: await readFile(
        join(
          import.meta.dirname,
          name,
        ),
        'utf8',
      ),
    };
  },),);
  return sources
    .filter(function containsNeedle(source,) {
      return source.text.includes(needle,);
    },)
    .map(function nameOf(source,) {
      return source.name;
    },);
}

//endregion Source scanning

await describe({
  name: '',
  children: [
    describe({
      name: replayPenCursor.name,
      children: [
        it({
          name: 'replays move, horizontal, vertical, and close steps in order',
          fn: async () => {
            /**
             Path data exercising every command the tokenizer emits.
             */
            const commands = parseSvgPathD('M10 20 H30 V40 Z',);

            /**
             Events the replay produced, described for comparison.
             */
            const events = replayPenCursor(commands,)
              .map(function asText(event,) {
                return describeEvent({ event, },);
              },);

            expect(events,).toStrictEqual([
              'moveTo(10, 20)',
              'lineTo(30, 20)',
              'lineTo(30, 40)',
              'close',
            ],);
          },
        },),

        it({
          name: 'keeps the untouched axis of a horizontal or vertical command',
          fn: async () => {
            /**
             Path data whose second horizontal command follows a vertical one.
             */
            const commands = parseSvgPathD('M0 0 H10 V20 H30',);

            /**
             Events the replay produced, described for comparison.
             */
            const events = replayPenCursor(commands,)
              .map(function asText(event,) {
                return describeEvent({ event, },);
              },);

            expect(events,).toStrictEqual([
              'moveTo(0, 0)',
              'lineTo(10, 0)',
              'lineTo(10, 20)',
              'lineTo(30, 20)',
            ],);
          },
        },),

        it({
          name: 'starts a second subpath after a close',
          fn: async () => {
            /**
             Path data holding two closed subpaths.
             */
            const commands = parseSvgPathD('M0 0 L10 0 Z M20 20 L30 20 Z',);

            /**
             Events the replay produced, described for comparison.
             */
            const events = replayPenCursor(commands,)
              .map(function asText(event,) {
                return describeEvent({ event, },);
              },);

            expect(events,).toStrictEqual([
              'moveTo(0, 0)',
              'lineTo(10, 0)',
              'close',
              'moveTo(20, 20)',
              'lineTo(30, 20)',
              'close',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: 'replay consumers',
      children: [
        it({
          name: 'resolveAbsolutePoints returns the point of every drawn event',
          fn: async () => {
            /**
             Path data mixing single-axis and two-axis commands.
             */
            const commands = parseSvgPathD('M0 0 H10 V20',);
            /**
             Points the replay drew.
             */
            const drawnPoints = replayPenCursor(commands,)
              .flatMap(function pointOf(event,) {
                return event.kind === 'close'
                  ? []
                  : [[event.x, event.y,],];
              },);

            expect(resolveAbsolutePoints(commands,),).toStrictEqual(drawnPoints,);
          },
        },),

        it({
          name: 'addFilledPath draws the replay, closing where it closes',
          fn: async () => {
            /**
             X coordinate of the second column of the test contour.
             */
            const secondColumn = 10;
            /**
             Strip Y coordinate of the lower row of the test contour.
             */
            const lowerRow = 20;
            /**
             Path data for one closed triangle-ish contour.
             */
            const commands = parseSvgPathD(
              `M0 0 H${secondColumn} V${lowerRow} Z`,
            );
            /**
             OpenType path the consumer fills.
             */
            const otPath = new opentype.Path();
            /**
             OpenType path the same steps produce when written out by hand.
             */
            const expected = new opentype.Path();
            expected.moveTo(
              0,
              fontY(0,),
            );
            expected.lineTo(
              secondColumn,
              fontY(0,),
            );
            expected.lineTo(
              secondColumn,
              fontY(lowerRow,),
            );
            expected.close();

            addFilledPath({
              otPath,
              commands,
              cellX: 0,
              xShift: 0,
            },);

            expect(otPath.toPathData(6,),).toBe(expected.toPathData(6,),);
          },
        },),
      ],
    },),

    describe({
      name: 'cursor ownership',
      children: [
        it({
          name: 'declares the pen cursor once',
          fn: async () => {
            expect(await modulesContaining({ needle: 'let cx', },),).toStrictEqual(
              ['svg-path-pen-cursor.ts',],
            );
          },
        },),
      ],
    },),
  ],
},);
