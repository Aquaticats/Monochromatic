import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { isFunctionLikeDeclaration, } from 'typescript/ast/is';
import { readFileSync, } from 'node:fs';
import { fileURLToPath, } from 'node:url';

import {
  closeSemanticBridge,
  openSemanticFile,
  parameterBindingSlots,
  parameterSlotTable,
} from '../dist/final/node/index.mjs';

/** Fixture whose parameter shapes exercise every allocation rule. */
const SHAPES_PATH = fileURLToPath(new URL(
  '../../../test-fixture/oxlint-no-restricted-syntax/src/valid/readonly-parameter-slot-shapes.ts',
  import.meta.url,
),);

/** Current shape-fixture text. */
const SHAPES_SOURCE = readFileSync(
  SHAPES_PATH,
  'utf8',
);

/**
 Slot facts of one fixture function, flattened for comparison.
 */
type SlotFacts = {
  readonly functionName: string;
  readonly parameterCount: number;
  readonly slotCount: number;
  readonly parameterOfSlot: readonly number[];
  readonly propertyKeys: readonly (readonly string[])[];
  readonly bindings: readonly (readonly [string, number])[];
};

await describe({
  name: parameterSlotTable.name,
  concurrency: 1,
  children: [
    it({
      name: 'allocates one slot per canonical property, and widens every shape that names none',
      fn: async () => {
        /* Allocation is invisible while stage one broadcasts every property slot, so a
         * misfiled binding would not move a single sweep number. It becomes visible only
         * once narrowing lands, as a moved offer indistinguishable from narrowing working.
         * Pinning the table directly is what separates the two. */
        const session = openSemanticFile({
          fileName: SHAPES_PATH,
          sourceText: SHAPES_SOURCE,
          hasBOM: false,
        },);
        /**
         Reads the allocated slots of one fixture function.
         
         @param functionName - Exported fixture function to inspect.
         
         @returns flattened slot facts for comparison.
         */
        function factsFor(functionName: string,): SlotFacts {
          const at = SHAPES_SOURCE.indexOf(`function ${functionName}`,)
            + 'function '.length;
          const declaration = session.nodeAtOffset(at,).parent;
          if (!isFunctionLikeDeclaration(declaration,))
            throw new Error(`Expected function declaration for ${functionName}.`,);
          const table = parameterSlotTable({ declaration, },);
          return {
            functionName,
            parameterCount: table.parameterCount,
            slotCount: table.slotCount,
            parameterOfSlot: [...table.parameterOfSlot,],
            propertyKeys: table.propertySlotsByParameter
              .map(function keysOf(slots,): readonly string[] {
                return [...slots.keys(),];
              },),
            bindings: declaration.parameters
              .flatMap(function boundNames(parameter, parameterIndex,) {
                return parameterBindingSlots({
                  parameter,
                  parameterIndex,
                  table,
                },)
                  .map(function pair(bound,): readonly [string, number] {
                    return [
                      bound.name.getText(),
                      bound.slot,
                    ];
                  },);
              },),
          };
        }
        const facts = [
          'wholeIdentifierSlot',
          'shorthandPropertySlots',
          'renamedPropertySlot',
          'duplicateKeySlots',
          'nestedPatternSlot',
          'restPropertySlots',
          'arrayPatternSlots',
          'computedKeySlot',
          'numericKeySlots',
        ].map(factsFor,);
        closeSemanticBridge();
        expect(facts,).toEqual([
          /* A plain identifier owns exactly its whole-parameter slot. */
          {
            functionName: 'wholeIdentifierSlot',
            parameterCount: 1,
            slotCount: 1,
            parameterOfSlot: [0,],
            propertyKeys: [[],],
            bindings: [['row', 0,],],
          },
          /* Two shorthand properties, two property slots after the whole one. */
          {
            functionName: 'shorthandPropertySlots',
            parameterCount: 1,
            slotCount: 3,
            parameterOfSlot: [0, 0, 0,],
            propertyKeys: [['named', 'unnamed',],],
            bindings: [
              ['named', 1,],
              ['unnamed', 2,],
            ],
          },
          /* The key is the property a caller writes, not the local it binds. */
          {
            functionName: 'renamedPropertySlot',
            parameterCount: 1,
            slotCount: 2,
            parameterOfSlot: [0, 0,],
            propertyKeys: [['named',],],
            bindings: [['bound', 1,],],
          },
          /* One property read twice is one slot, with both locals registered against it. */
          {
            functionName: 'duplicateKeySlots',
            parameterCount: 1,
            slotCount: 2,
            parameterOfSlot: [0, 0,],
            propertyKeys: [['named',],],
            bindings: [
              ['first', 1,],
              ['second', 1,],
            ],
          },
          /* The outer property owns the slot; the nested binding registers against it,
           * because a write through the inner name is a write through the outer property. */
          {
            functionName: 'nestedPatternSlot',
            parameterCount: 1,
            slotCount: 2,
            parameterOfSlot: [0, 0,],
            propertyKeys: [['outer',],],
            bindings: [['inner', 1,],],
          },
          /* A rest element names a complement set rather than a property, so it takes the
           * whole-parameter slot and contributes no key called `rest`. */
          {
            functionName: 'restPropertySlots',
            parameterCount: 1,
            slotCount: 2,
            parameterOfSlot: [0, 0,],
            propertyKeys: [['named',],],
            bindings: [
              ['named', 1,],
              ['rest', 0,],
            ],
          },
          /* Positional element keys are not modelled, so an array pattern widens whole. */
          {
            functionName: 'arrayPatternSlots',
            parameterCount: 1,
            slotCount: 1,
            parameterOfSlot: [0,],
            propertyKeys: [[],],
            bindings: [
              ['first', 0,],
              ['second', 0,],
            ],
          },
          /* Which property a computed name reads is a runtime question. */
          {
            functionName: 'computedKeySlot',
            parameterCount: 1,
            slotCount: 1,
            parameterOfSlot: [0,],
            propertyKeys: [[],],
            bindings: [['value', 0,],],
          },
          /* A numeric key canonicalizes to the same text a quoted one would. */
          {
            functionName: 'numericKeySlots',
            parameterCount: 1,
            slotCount: 2,
            parameterOfSlot: [0, 0,],
            propertyKeys: [['1',],],
            bindings: [['one', 1,],],
          },
        ],);
      },
    },),
  ],
},);
