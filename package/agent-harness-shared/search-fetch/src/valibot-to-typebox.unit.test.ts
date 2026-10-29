/**
 Unit tests for the guarded valibot to TypeBox converter.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { Compile, } from 'typebox/compile';
import * as v from 'valibot';

import {
  valibotToTypeBox,
  ValibotToTypeBoxError,
  webFetchToolSpec,
  webSearchToolSpec,
  type ValibotSchemaNode,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Verdict corpus spanning accepted, rejected, and ignored-key inputs.
 */
const VERDICT_CASES = [
  {
    name: 'valid full',
    input: {
      query: 'q',
      fromDate: '2026-01-01',
      includeDomains: ['a.com',],
      toDate: '2026-02-01',
    },
    accepted: true,
  },
  { name: 'valid bare', input: { query: 'q', }, accepted: true, },
  { name: 'missing required', input: {}, accepted: false, },
  { name: 'wrong type', input: { query: 42, }, accepted: false, },
  {
    name: 'unknown key',
    input: {
      query: 'q',
      depth: 'deep',
    },
    accepted: true,
  },
  {
    name: 'single string for array',
    input: {
      query: 'q',
      includeDomains: 'a.com',
    },
    accepted: false,
  },
] as const;

/**
 Node types outside the supported subset, each expected to fail loud.
 */
const UNSUPPORTED_NODE_TYPES = [
  'number',
  'boolean',
  'literal',
  'custom',
  'union',
  'record',
] as const;

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: valibotToTypeBox.name,
      children: [
        it({
          name: 'converts both shared tool specs with descriptions intact',
          fn: async () => {
            for (const spec of [webSearchToolSpec, webFetchToolSpec,]) {
              /**
               JSON rendering of the converted schema, which hosts forward to models.
               */
              const convertedJson = JSON.stringify(valibotToTypeBox(spec.parameters,),);
              expect(convertedJson,).toContain('"type":"object"',);
              expect(convertedJson,).toContain('"description"',);
            }
          },
        },),
        it({
          name: 'agrees with valibot verdicts across the shared search schema',
          fn: async () => {
            /**
             Compiled checker over the converted schema.
             */
            const check = Compile(valibotToTypeBox(webSearchToolSpec.parameters,),);
            for (const testCase of VERDICT_CASES) {
              /**
               Valibot's own verdict, the single source of truth.
               */
              const valibotAccepts = v.safeParse(
                webSearchToolSpec.parameters,
                testCase.input,
              ).success;
              expect(valibotAccepts,).toBe(testCase.accepted,);
              expect(check.Check(testCase.input,),).toBe(valibotAccepts,);
            }
          },
        },),
        it({
          name: 'keeps open objects open so ignored keys survive to warnings',
          fn: async () => {
            /**
             JSON rendering of the converted open search schema.
             */
            const convertedJson = JSON.stringify(valibotToTypeBox(webSearchToolSpec.parameters,),);
            expect(convertedJson.includes('"additionalProperties":false',),).toBe(false,);
          },
        },),
        it({
          name: 'maps strict objects to additionalProperties false',
          fn: async () => {
            /**
             Converted strict-object schema.
             */
            const converted = valibotToTypeBox({
              type: 'strict_object',
              entries: { url: { type: 'string', }, },
            },);
            expect(JSON.stringify(converted,),).toContain('"additionalProperties":false',);
          },
        },),
        it({
          name: 'carries pipe descriptions into the converted schema',
          fn: async () => {
            /**
             Converted schema whose entry carries a pipe description.
             */
            const converted = valibotToTypeBox({
              type: 'object',
              entries: {
                code: {
                  type: 'string',
                  pipe: [
                    { type: 'string', },
                    {
                      type: 'description',
                      description: 'code token',
                    },
                  ],
                },
              },
            },);
            expect(JSON.stringify(converted,),).toContain('"description":"code token"',);
          },
        },),
        it({
          name: 'throws for every node type outside the supported subset',
          fn: async () => {
            await Promise.all(UNSUPPORTED_NODE_TYPES.map(function assertUnsupportedType(nodeType,) {
              return expectUnsupported({ type: nodeType, },);
            },),);
          },
        },),
        it({
          name: 'throws for unsupported pipe actions instead of translating them silently',
          fn: async () => {
            await expectUnsupported({
              type: 'string',
              pipe: [
                { type: 'string', },
                { type: 'regex', },
              ],
            },);
          },
        },),
        it({
          name: 'throws for wrapper and object nodes missing their sub-schemas',
          fn: async () => {
            await expectUnsupported({ type: 'optional', },);
            await expectUnsupported({ type: 'array', },);
            await expectUnsupported({ type: 'object', },);
            await expectUnsupported({ type: 'strict_object', },);
          },
        },),
      ],
    },),
  ],
},);

//region Helpers

/**
 Assert a node fails conversion loudly.

 @param node - schema node outside the supported subset

 @throws when conversion unexpectedly succeeds

 @example
 ```ts
 await expectUnsupported({ type: 'number' });
 ```
 */
async function expectUnsupported(node: ValibotSchemaNode,): Promise<void> {
  /**
   Whether conversion threw the guarded error, captured without a root binding.
   */
  const threwGuarded = (async function attemptConversion(): Promise<boolean> {
    try {
      valibotToTypeBox(node,);
      return false;
    }
    catch (error: unknown) {
      expect(error instanceof ValibotToTypeBoxError,).toBe(true,);
      return true;
    }
  })();
  expect(await threwGuarded,).toBe(true,);
}

//endregion Helpers
