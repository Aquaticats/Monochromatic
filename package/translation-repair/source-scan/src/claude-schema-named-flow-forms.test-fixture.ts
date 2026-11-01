import type { BlindSpot, } from './claude-schema-blind-spots.test-fixture.ts';
import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import { CONTEST_IMPORT, } from './claude-schema-destination-forms.test-fixture.ts';
import {
  SENDER_FILE,
  SENDER_IMPORT,
} from './claude-schema-forwarder-forms.test-fixture.ts';
import type { NamedPlace, } from './claude-schema-named-places.test-fixture.ts';
import type { SourceFile, } from './claude-schema-sources.test-fixture.ts';
import { keyOf, } from './claude-schema-usage.test-fixture.ts';

//region What a name of a place stands for when the check reads files
// The reports the check makes over imaginary files, held to one name: a forwarder handed over as the value of a
// property is named, and a second forwarder, or the same forwarder handed over again, is a report the name does not
// stand for, on another line and on the line of the first.

/**
 One case where the check reads imaginary files and the names are held to the reports it makes.
 */
export type NamedFlowCase = {
  /**
   What the case holds.
   */
  readonly name: string;

  /**
   The imaginary files the check reads.
   */
  readonly files: readonly SourceFile[];

  /**
   The reports no name stands for, in the order of the files and then of the lines.
   */
  readonly unnamed: readonly BlindSpot[];
};

/**
 The path of the file that declares the two forwarders of every flow case.
 */
const PLACE_STAGE = 'imaginary/src/place-stage.ts';

/**
 The path of the file that declares a second forwarder with the name of the first, in another module.
 */
const PLACE_OTHER_STAGE = 'imaginary/src/place-other-stage.ts';

/**
 The path of the file that hands the forwarders over as the values of an object.
 */
const PLACE_DEPENDENCIES = 'imaginary/src/place-dependencies.ts';

/**
 The name of the one place the flow cases name: the forwarder `mk` handed over as the value of a property.
 */
export const FLOW_PLACE: NamedPlace = {
  file: PLACE_DEPENDENCIES,
  shape: 'forwarder-outside-a-read-position',
  position: 'Property.value',
  held: keyOf({
    file: PLACE_STAGE,
    name: 'mk',
  },),
  reason: 'a case',
};

/**
 The report for a forwarder handed over as the value of a property in the file of the dependencies.

 @param line - line of the property

 @param name - name of the forwarder

 @param stage - file that declares the forwarder

 @returns Report

 @example
 ```ts
 const spot = flowSpotAt({ line: 4, name: 'other', },);
 ```
 */
function flowSpotAt(
  {
    line,
    name,
    stage = PLACE_STAGE,
  }: {
    readonly line: number;
    readonly name: string;
    readonly stage?: string;
  },
): BlindSpot {
  return {
    file: PLACE_DEPENDENCIES,
    line,
    shape: 'forwarder-outside-a-read-position',
    position: 'Property.value',
    held: keyOf({
      file: stage,
      name,
    },),
  };
}

/**
 A flow case whose dependencies file holds the lines given after its import of both forwarders.

 @param name - name of the case

 @param lines - lines of the dependencies file after the import, the first of which is line two

 @param imports - import of the forwarders the dependencies file starts with; where it holds two lines, the lines
 after it start at line three

 @param unnamed - reports no name stands for

 @returns Case

 @example
 ```ts
 const flowCase = flowCaseOf({ name: 'x', lines: ['export const deps = { settle: mk, };',], unnamed: [], },);
 ```
 */
function flowCaseOf(
  {
    name,
    lines,
    imports = 'import { mk, other, } from \'./place-stage.ts\';',
    unnamed,
  }: {
    readonly name: string;
    readonly lines: readonly string[];
    readonly imports?: string;
    readonly unnamed: readonly BlindSpot[];
  },
): NamedFlowCase {
  return {
    name,
    files: [
      SENDER_FILE,
      {
        file: PLACE_STAGE,
        text: textOf([
          CONTEST_IMPORT,
          SENDER_IMPORT,
          'export async function mk(args: Parameters<typeof contestResponseFormat>[0],) {',
          '  return await gatherThroughCache({ responseFormat: contestResponseFormat(args,), },);',
          '}',
          'export async function other(args: Parameters<typeof contestResponseFormat>[0],) {',
          '  return await gatherThroughCache({ responseFormat: contestResponseFormat(args,), },);',
          '}',
        ],),
      },
      {
        file: PLACE_OTHER_STAGE,
        text: textOf([
          CONTEST_IMPORT,
          SENDER_IMPORT,
          'export async function mk(args: Parameters<typeof contestResponseFormat>[0],) {',
          '  return await gatherThroughCache({ responseFormat: contestResponseFormat(args,), },);',
          '}',
        ],),
      },
      {
        file: PLACE_DEPENDENCIES,
        text: textOf([
          imports,
          ...lines,
        ],),
      },
    ],
    unnamed,
  };
}

/**
 Cases where the check reads imaginary files and the one name is held to the reports it makes.
 */
export const NAMED_FLOW_CASES: readonly NamedFlowCase[] = [
  flowCaseOf({
    name: 'LEAVES THE REPORT OF A SECOND FORWARDER HANDED OVER AS A SHORTHAND PROPERTY OF THE SAME OBJECT',
    lines: ['export const deps = { settle: mk, other, };',],
    unnamed: [
      flowSpotAt({
        line: 2,
        name: 'other',
      },),
    ],
  },),
  flowCaseOf({
    name: 'LEAVES THE REPORT OF A SECOND FORWARDER HANDED OVER AS ANOTHER PROPERTY OF THE SAME OBJECT',
    lines: [
      'export const deps = {',
      '  settle: mk,',
      '  run: other,',
      '};',
    ],
    unnamed: [
      flowSpotAt({
        line: 4,
        name: 'other',
      },),
    ],
  },),
  flowCaseOf({
    name: 'LEAVES THE REPORT OF THE NAMED FORWARDER HANDED OVER AGAIN UNDER A SECOND KEY',
    lines: [
      'export const deps = {',
      '  settle: mk,',
      '  again: mk,',
      '};',
    ],
    unnamed: [
      flowSpotAt({
        line: 4,
        name: 'mk',
      },),
    ],
  },),
  flowCaseOf({
    name: 'LEAVES THE REPORT OF A SECOND FORWARDER ON THE LINE OF THE NAMED ONE',
    lines: ['export const deps = { settle: mk, run: other, };',],
    unnamed: [
      flowSpotAt({
        line: 2,
        name: 'other',
      },),
    ],
  },),
  flowCaseOf({
    name: 'LEAVES THE REPORT OF THE NAMED FORWARDER HANDED OVER AGAIN ON THE LINE OF THE FIRST',
    lines: ['export const deps = { settle: mk, again: mk, };',],
    unnamed: [
      flowSpotAt({
        line: 2,
        name: 'mk',
      },),
    ],
  },),
  flowCaseOf({
    name: 'NAMES THE FORWARDER HANDED OVER UNDER ANOTHER LOCAL NAME, since the name is that of the function',
    imports: 'import { mk as renamed, } from \'./place-stage.ts\';',
    lines: ['export const deps = { settle: renamed, };',],
    unnamed: [],
  },),
  flowCaseOf({
    name: 'NAMES THE FORWARDER HANDED OVER AS A MEMBER OF A NAMESPACE IMPORT, since the name is that of the function',
    imports: 'import * as stage from \'./place-stage.ts\';',
    lines: ['export const deps = { settle: stage.mk, };',],
    unnamed: [],
  },),
  flowCaseOf({
    name: 'LEAVES THE REPORT OF A FORWARDER OF THE SAME NAME THAT ANOTHER MODULE DECLARES, since the name is that of '
      + 'the function and the module that declares it',
    imports: 'import { mk, } from \'./place-other-stage.ts\';',
    lines: ['export const deps = { settle: mk, };',],
    unnamed: [
      flowSpotAt({
        line: 2,
        name: 'mk',
        stage: PLACE_OTHER_STAGE,
      },),
    ],
  },),
  flowCaseOf({
    name: 'LEAVES THE REPORT OF THE FORWARDER OF ANOTHER MODULE HANDED OVER BESIDE THE NAMED ONE, under a local name',
    imports: 'import { mk, } from \'./place-stage.ts\';\nimport { mk as again, } from \'./place-other-stage.ts\';',
    lines: [
      'export const deps = {',
      '  settle: mk,',
      '  again,',
      '};',
    ],
    unnamed: [
      flowSpotAt({
        line: 5,
        name: 'mk',
        stage: PLACE_OTHER_STAGE,
      },),
    ],
  },),
  flowCaseOf({
    name: 'LEAVES NO REPORT WHEN THE NAMED FORWARDER IS HANDED OVER ONCE',
    lines: ['export const deps = { settle: mk, };',],
    unnamed: [],
  },),
];

//endregion What a name of a place stands for when the check reads files
