import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import {
  ARGUMENTS,
  CALL_ARGUMENT,
  CONTEST_IMPORT,
  spotAt,
} from './claude-schema-destination-forms.test-fixture.ts';
import {
  SENDER_FILE,
  SENDER_IMPORT,
} from './claude-schema-forwarder-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region A name the sender is called by that a namespace binds
// A TypeScript namespace is a scope as a function is: a name it declares (a constant, a function, a class, an enum, a
// namespace, a declared function) or binds with `import x = y` is another thing inside it than the import of the same
// name at the top of the file. The call of the name there is no call of the sender, so the object it takes is reported
// like the argument of any call. The schema in each case is built outside the namespace, so only the call of the
// sender decides: a namespace that binds nothing of that name, and a namespace beside the one that binds it, leave the
// call a call of the sender.

/**
 The line of the stage file that builds the schema, which every case writes the same way.
 */
const SCHEMA_LINE = `const SCHEMA = contestResponseFormat(${ARGUMENTS},);`;

/**
 The call of the name inside the namespace, which hands the schema built outside to it.
 */
const SENDS_SCHEMA = '  export const sent = gatherThroughCache({ responseFormat: SCHEMA, },);';

/**
 A case whose stage file builds the schema, then declares namespaces, the last of which calls the sender's name.

 @param name - name of the case

 @param lines - lines of the stage file after the line that builds the schema

 @param isReported - whether the call of the name is reported at the line that builds the schema

 @returns Case that lists the stage file

 @example
 ```ts
 const shapeCase = namespaceCase({ name: 'x', lines: [], isReported: true, },);
 ```
 */
function namespaceCase(
  {
    name,
    lines,
    isReported,
  }: {
    readonly name: string;
    readonly lines: readonly string[];
    readonly isReported: boolean;
  },
): ShapeCase {
  return {
    name,
    files: [
      SENDER_FILE,
      {
        file: 'imaginary/src/namespace-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          SENDER_IMPORT,
          SCHEMA_LINE,
          ...lines,
        ],),
      },
    ],
    found: ['imaginary/src/namespace-stage.ts',],
    builders: [],
    spots: isReported
      ? [
        spotAt({
          file: 'imaginary/src/namespace-stage.ts',
          line: 3,
          position: CALL_ARGUMENT,
        },),
      ]
      : [],
  };
}

/**
 Cases of a name the sender is called by that a namespace binds, and of the calls of it that stay accepted.
 */
export const SENDER_NAMESPACE_CASES: readonly ShapeCase[] = [
  namespaceCase({
    name: 'REPORTS A SCHEMA BUILT OUTSIDE A NAMESPACE AND HANDED TO A CALL OF A NAME THE NAMESPACE DECLARES AS A '
      + 'CONSTANT, since the call there is no call of the sender',
    lines: [
      'export namespace Stage {',
      '  const gatherThroughCache = (sent: object,) => sent;',
      SENDS_SCHEMA,
      '}',
    ],
    isReported: true,
  },),
  namespaceCase({
    name: 'REPORTS A SCHEMA BUILT OUTSIDE A NAMESPACE AND HANDED TO A CALL OF A NAME THE NAMESPACE EXPORTS AS A '
      + 'FUNCTION, whose declaration the export statement holds',
    lines: [
      'export namespace Stage {',
      '  export function gatherThroughCache(sent: object,) {',
      '    return sent;',
      '  }',
      SENDS_SCHEMA,
      '}',
    ],
    isReported: true,
  },),
  namespaceCase({
    name: 'REPORTS A SCHEMA BUILT OUTSIDE A NAMESPACE AND HANDED TO A CALL OF A NAME THE NAMESPACE EXPORTS AS A CONSTANT',
    lines: [
      'export namespace Stage {',
      '  export const gatherThroughCache = (sent: object,) => sent;',
      SENDS_SCHEMA,
      '}',
    ],
    isReported: true,
  },),
  namespaceCase({
    name: 'REPORTS A SCHEMA BUILT OUTSIDE A NAMESPACE AND HANDED TO A CALL OF A NAME THE NAMESPACE BINDS WITH `import x = '
      + 'y`, since an import-equals declaration binds its name as a constant does',
    lines: [
      'namespace Other {',
      '  export const make = (sent: object,) => sent;',
      '}',
      'export namespace Stage {',
      '  import gatherThroughCache = Other.make;',
      SENDS_SCHEMA,
      '}',
    ],
    isReported: true,
  },),
  namespaceCase({
    name: 'REPORTS A SCHEMA BUILT OUTSIDE A NAMESPACE AND HANDED TO A CALL OF A NAME THE NAMESPACE DECLARES AS A CLASS, '
      + 'AN ENUM, A NAMESPACE OR A DECLARED FUNCTION',
    lines: [
      'export namespace Stage {',
      '  export declare function gatherThroughCache(sent: object,): object;',
      SENDS_SCHEMA,
      '}',
      'export namespace Second {',
      '  export class gatherThroughCache {}',
      SENDS_SCHEMA,
      '}',
      'export namespace Third {',
      '  export enum gatherThroughCache { First, }',
      SENDS_SCHEMA,
      '}',
      'export namespace Fourth {',
      '  export namespace gatherThroughCache {',
      '    export const sent = 0;',
      '  }',
      SENDS_SCHEMA,
      '}',
    ],
    isReported: true,
  },),
  namespaceCase({
    name: 'ACCEPTS A SCHEMA BUILT OUTSIDE A NAMESPACE AND HANDED TO THE SENDER FROM A NAMESPACE THAT BINDS NOTHING OF THAT '
      + 'NAME, and from one beside the namespace that binds it',
    lines: [
      'namespace Binds {',
      '  export function gatherThroughCache(sent: object,) {',
      '    return sent;',
      '  }',
      '}',
      'export namespace Stage {',
      '  const other = 0;',
      SENDS_SCHEMA,
      '}',
    ],
    isReported: false,
  },),
];

//endregion A name the sender is called by that a namespace binds
