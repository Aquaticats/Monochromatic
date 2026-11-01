import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import * as v from 'valibot';

import {
  strictArguments,
  TOOL_ARGUMENTS_VALID,
  ToolSchemaError,
  toolInputSchema,
  validateToolArguments,
} from '@monochromatic-dev/mcp-stdio';

//region toolInputSchema: one declaration becomes the advertised JSON Schema

await describe({
  name: toolInputSchema.name,
  children: [
    it({
      name: 'converts declared arguments into an object-rooted 2020-12 schema',
      fn: async () => {
        expect(toolInputSchema({
          schema: strictArguments({ name: v.string(), },),
          toolName: 'demo',
        },),).toEqual({
          type: 'object',
          properties: { name: { type: 'string', }, },
          required: ['name',],
          additionalProperties: false,
          $schema: 'https://json-schema.org/draft/2020-12/schema',
        },);
      },
    },),
    it({
      name: 'restores the object root a union conversion omits',
      fn: async () => {
        /** Union of two object branches, which converts to a bare `anyOf`. */
        const advertised = toolInputSchema({
          schema: v.union([
            strictArguments({ name: v.string(), },),
            strictArguments({ all: v.literal(true,), },),
          ],),
          toolName: 'demo',
        },);

        // Revision 2026-07-28 requires an object root even when composition keywords carry
        // the real constraint, and the converter emits none for a union.
        expect(advertised.type,).toBe('object',);
        // ToolInputSchema carries an index signature, so composition keywords read off it
        // directly without an assertion.
        expect(advertised.anyOf,).toHaveLength(2,);
      },
    },),
    it({
      name: 'REFUSES a schema the converter cannot express, naming the tool',
      fn: async () => {
        // Deliberate catch: the thrown type matters as much as the message, since callers
        // distinguish an authoring mistake from a dispatch failure by it.
        try {
          toolInputSchema({
            schema: v.custom(function always(): boolean {
              return true;
            },),
            toolName: 'unconvertible',
          },);
          expect('conversion succeeded',).toBe('conversion should have thrown',);
        }
        catch (error: unknown) {
          expect(error instanceof ToolSchemaError,).toBe(true,);
          expect((error as Error).message,).toContain('unconvertible',);
        }
      },
    },),
    it({
      name: 'REFUSES a scalar root, which no argument object could satisfy',
      fn: async () => {
        expect(() =>
          toolInputSchema({
            schema: v.string(),
            toolName: 'scalar',
          },), ).toThrow('object root',);
      },
    },),
    it({
      name: 'REFUSES a union whose branches are not all objects',
      fn: async () => {
        expect(() =>
          toolInputSchema({
            schema: v.union([v.string(), v.number(),],),
            toolName: 'mixed',
          },), ).toThrow('not all objects',);
      },
    },),
  ],
},);

//endregion

//region validateToolArguments: the gate that reads the same declaration

await describe({
  name: validateToolArguments.name,
  children: [
    it({
      name: 'ACCEPTS conforming arguments with the exact success sentinel',
      fn: async () => {
        // Identity rather than truthiness: a message string would also be truthy, so an
        // equality check is what separates the two outcomes.
        expect(validateToolArguments({
          schema: strictArguments({ name: v.string(), },),
          args: { name: 'vm1', },
        },),).toBe(TOOL_ARGUMENTS_VALID,);
      },
    },),
    it({
      name: 'REFUSES a missing argument, naming the path that failed',
      fn: async () => {
        /** Verdict for arguments missing the only declared field. */
        const verdict = validateToolArguments({
          schema: strictArguments({ name: v.string(), },),
          args: {},
        },);

        expect(verdict,).not.toBe(TOOL_ARGUMENTS_VALID,);
        expect(String(verdict,),).toContain('name',);
      },
    },),
    it({
      name: 'REFUSES an undeclared key, inherited names included',
      fn: async () => {
        /** Schema declaring nothing at all, so every key is undeclared. */
        const schema = strictArguments({},);

        expect(validateToolArguments({
          schema,
          args: JSON.parse('{"typo":1}',) as Record<string, unknown>,
        },),).not.toBe(TOOL_ARGUMENTS_VALID,);
        // `constructor` reaches valibot as an own property but tests as declared against a
        // prototype-bearing entries object, which strictArguments removes.
        expect(validateToolArguments({
          schema,
          args: JSON.parse('{"constructor":1}',) as Record<string, unknown>,
        },),).not.toBe(TOOL_ARGUMENTS_VALID,);
      },
    },),
    it({
      name: 'describes every violated path, not only the first',
      fn: async () => {
        /** Verdict for arguments violating two declared fields at once. */
        const verdict = String(validateToolArguments({
          schema: strictArguments({
            name: v.string(),
            count: v.number(),
          },),
          args: {},
        },),);

        expect(verdict,).toContain('name',);
        expect(verdict,).toContain('count',);
      },
    },),
  ],
},);

//endregion
