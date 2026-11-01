import type { BlindSpot, } from './claude-schema-blind-spots.test-fixture.ts';
import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import { CONTEST_IMPORT, } from './claude-schema-destination-forms.test-fixture.ts';
import {
  SENDER_FILE,
  SENDER_IMPORT,
} from './claude-schema-forwarder-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';
import { keyOf, } from './claude-schema-usage.test-fixture.ts';

//region Pairs of a module that declares a forwarder and a module that declares a wrapper of the contest builder
// A file reaches the forwarders of a module that declares no builder as it reaches the builders of one that does: with
// the namespace of a barrel that exports everything of the module or names its members, with a barrel of a barrel, with
// `export * as`, and with a dynamic import. Each form is read as the same form of a module that carries a builder is
// read, so every form is a pair of cases. The forwarder case and its twin write the same files, one with a module that
// declares a forwarder and one with a module that declares a wrapper of the contest builder, and both are reported in
// the same places with the shape each module has: a member held where the search does not read it, the namespace handed
// on whole, or a load in a form no binding follows. Without that, a file that holds the forwarder through a barrel, and
// a third file that calls what the first holds, hide the files that write the arguments of the schema.

/**
 The module that declares the forwarder or the wrapper in every case.
 */
const STAGE = 'imaginary/src/barrel-stage.ts';

/**
 The barrel that exports the module.
 */
export const BARREL = 'imaginary/src/barrel-index.ts';

/**
 The barrel that exports the barrel.
 */
export const OUTER = 'imaginary/src/barrel-outer.ts';

/**
 The file that reaches the module.
 */
export const USER = 'imaginary/src/barrel-user.ts';

/**
 The file that reaches what the first file keeps.
 */
export const SECOND = 'imaginary/src/barrel-second.ts';

/**
 The file that calls what the first file hands on.
 */
export const THIRD = 'imaginary/src/barrel-third.ts';

/**
 The arguments of a call of the function the module declares, as a file writes them.
 */
export const ARGUMENTS = '{ schemaName: \'barrel\', verdicts: [], asksArchive: false, }';

/**
 Where a case reports, told apart from the shape each kind of module gives it.
 */
type Place = {
  /**
   The file the place sits in.
   */
  readonly file: string;

  /**
   Its line.
   */
  readonly line: number;

  /**
   What is wrong there: a member of the module held where the search does not read it, the namespace read where the
   search does not look, a load in a form no binding follows, or the path of an import-equals.
   */
  readonly kind: 'held' | 'load' | 'module' | 'path';

  /**
   The kind of the node that holds it and the field.
   */
  readonly position: string;
};

/**
 One file a form writes.
 */
type WrittenFile = {
  /**
   Its path.
   */
  readonly file: string;

  /**
   Its lines.
   */
  readonly lines: readonly string[];
};

/**
 One form of reaching the module, with the files that write it and where the check reports.
 */
export type Row = {
  /**
   What the form is, which finishes the name of the case.
   */
  readonly form: string;

  /**
   Files besides the module.
   */
  readonly files: readonly WrittenFile[];

  /**
   Files besides the module that the search lists as users of what it declares.
   */
  readonly listed: readonly string[];

  /**
   Where the check reports.
   */
  readonly places: readonly Place[];
};

/**
 The report of a place for the module a case declares.

 @param place - place

 @param isForwarder - whether the module declares a forwarder, and not a wrapper of the contest builder

 @returns Report, with the name the check gives the forwarder when a member is held

 @example
 ```ts
 const place: Place = { file: USER, line: 2, kind: 'module', position: 'CallExpression.arguments', };
 const spot = spotOf({ place, isForwarder: true, },);
 ```
 */
function spotOf(
  {
    place,
    isForwarder,
  }: {
    readonly place: Place;
    readonly isForwarder: boolean;
  },
): BlindSpot {
  /**
   The report without the shape.
   */
  const where = {
    file: place.file,
    line: place.line,
    position: place.position,
  };
  if (place.kind === 'load') {
    return {
      ...where,
      shape: 'load-in-unfollowed-form',
    };
  }
  if (place.kind === 'module') {
    return {
      ...where,
      shape: 'module-binding-outside-a-read-position',
    };
  }
  if (place.kind === 'path') {
    return {
      ...where,
      shape: 'module-path-outside-a-load',
    };
  }
  return isForwarder
    ? {
      ...where,
      shape: 'forwarder-outside-a-read-position',
      held: keyOf({
        file: STAGE,
        name: 'mk',
      },),
    }
    : {
      ...where,
      shape: 'builder-outside-a-read-position',
    };
}

/**
 The two cases of a form: the module declares a forwarder, and its twin declares a wrapper of the contest builder.

 @param row - form

 @returns Forwarder case and case of its twin

 @example
 ```ts
 const [forwarderCase, builderCase,] = pairOf({ form: 'x', files: [], listed: [], places: [], },);
 ```
 */
function pairOf(row: Row,): readonly ShapeCase[] {
  /**
   The files the form writes, as sources.
   */
  const written = row.files
    .map(function sourceOf({
      file,
      lines,
    },) {
      return {
        file,
        text: textOf(lines,),
      };
    },);
  /**
   The imaginary files the search lists, sorted.
   */
  const found = [
    STAGE,
    ...row.listed,
  ].toSorted();
  return [
    {
      name: `REPORTS A FORWARDER OF A MODULE THAT DECLARES NO BUILDER, ${row.form}`,
      files: [
        SENDER_FILE,
        {
          file: STAGE,
          text: textOf([
            CONTEST_IMPORT,
            SENDER_IMPORT,
            'export async function mk(args: Parameters<typeof contestResponseFormat>[0],) {',
            '  return await gatherThroughCache({ responseFormat: contestResponseFormat(args,), },);',
            '}',
          ],),
        },
        ...written,
      ],
      found,
      builders: [],
      spots: row.places
        .map(function spotOfPlace(place,): BlindSpot {
          return spotOf({
            place,
            isForwarder: true,
          },);
        },),
    },
    {
      name: `REPORTS A WRAPPER OF THE CONTEST BUILDER IN THE SAME PLACES, the twin of the forwarder, ${row.form}`,
      files: [
        {
          file: STAGE,
          text: textOf([
            CONTEST_IMPORT,
            'export function mk(args: Parameters<typeof contestResponseFormat>[0],) {',
            '  return contestResponseFormat(args,);',
            '}',
          ],),
        },
        ...written,
      ],
      found,
      builders: ['mk',],
      spots: row.places
        .map(function spotOfPlace(place,): BlindSpot {
          return spotOf({
            place,
            isForwarder: false,
          },);
        },),
    },
  ];
}

/**
 The cases of rows, the forwarder case and the twin of each.

 @param rows - forms

 @returns Cases in the order of the forms

 @example
 ```ts
 const cases = pairsOf({ rows: [], },);
 ```
 */
export function pairsOf({ rows, }: { readonly rows: readonly Row[]; },): readonly ShapeCase[] {
  return rows.flatMap(function pairOfRow(row,): readonly ShapeCase[] {
    return pairOf(row,);
  },);
}

//endregion Pairs of a module that declares a forwarder and a module that declares a wrapper of the contest builder
