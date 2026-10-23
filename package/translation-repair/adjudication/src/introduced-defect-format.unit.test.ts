/**
 Tests for the introduced-defect probe's structured-output constraint (P1 of the second review of the Claude route).
 Nothing here reaches a provider: the schema is read, never sent.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  INTRODUCED_DEFECT_RESPONSE_FORMAT,
  INTRODUCED_DEFECT_VERDICTS,
} from '../dist/final/node/index.mjs';

/**
 Reads a schema node as an object.

 @param value - node to read

 @param what - name of the node, for the refusal

 @returns Node

 @throws {@link TypeError} where the node is not an object
 */
function recordOf(
  {
    value,
    what,
  }: {
    readonly value: unknown;
    readonly what: string;
  },
): Readonly<Record<string, unknown>> {
  if (!isJsonRecord(value,))
    throw new TypeError(`The ${what} of the introduced-defect schema is not an object`,);
  return value;
}

/**
 Reads the schema of one check's fields.

 @returns Check's properties

 @throws {@link TypeError} where the schema has another shape

 @example
 ```ts
 const fields = checkFields();
 ```
 */
function checkFields(): Readonly<Record<string, unknown>> {
  /**
   The schema's root.
   */
  const root = recordOf({ value: INTRODUCED_DEFECT_RESPONSE_FORMAT.json_schema.schema, what: 'root', },);
  /**
   The root's properties.
   */
  const rootProperties = recordOf({ value: root.properties, what: 'root properties', },);
  /**
   The array of checks.
   */
  const checks = recordOf({ value: rootProperties.checks, what: 'checks', },);
  /**
   One check.
   */
  const check = recordOf({ value: checks.items, what: 'check', },);
  return recordOf({ value: check.properties, what: 'check fields', },);
}

await describe({
  name: 'the introduced-defect response format',
  children: [
    it({
      name: 'LISTS THE VERDICT AS AN enum EQUAL TO THE VOCABULARY THE GUARD READS (P1): the model is held to the labels '
        + 'the stage accepts, and the two cannot drift',
      fn: async () => {
        expect(checkFields().verdict,).toEqual({
          type: 'string',
          enum: [...INTRODUCED_DEFECT_VERDICTS,],
        },);
      },
    },),

    it({
      name: 'LEAVES THE CATEGORY AND THE SEVERITY AS PLAIN STRINGS (P1): a negative verdict sends both as empty strings, '
        + 'which an enum would forbid',
      fn: async () => {
        expect(checkFields().category,).toEqual({ type: 'string', },);
        expect(checkFields().severity,).toEqual({ type: 'string', },);
      },
    },),

    it({
      name: 'REQUIRES EVERY FIELD OF A CHECK, so a lost field costs no region its telemetry',
      fn: async () => {
        /**
         Names of the fields a check carries.
         */
        const names = Object.keys(checkFields(),);
        expect(names.toSorted(),).toEqual([
          'category',
          'evidence',
          'omittedText',
          'region',
          'severity',
          'verdict',
          'verdictReason',
        ],);
      },
    },),
  ],
},);
