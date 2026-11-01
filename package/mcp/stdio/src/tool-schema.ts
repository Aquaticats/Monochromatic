// Tool argument schemas: one valibot declaration drives both advertisement and validation.

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  type JsonSchema,
  toJsonSchema,
} from '@valibot/to-json-schema';
import * as v from 'valibot';

import { isPlainObject, } from './plain-object.ts';
import type { ToolInputSchema, } from './protocol-tool.ts';

//region Schema vocabulary

/**
 Valibot schema a tool declares for its arguments.
 
 One declaration serves two purposes: it is converted to the JSON Schema advertised in
 `tools/list`, and it validates incoming `tools/call` arguments. Declaring the two
 separately is what lets an advertised contract drift away from the one enforced.
 */
export type ToolArgumentsSchema = v.BaseSchema<unknown, unknown, v.BaseIssue<unknown>>;

/**
 Builds a strict argument object that rejects every key it does not declare.
 
 Prefer this over `v.strictObject` directly. Valibot decides whether a key is declared
 with `key in entries`, and a plain entries object inherits from `Object.prototype`, so
 `constructor`, `__proto__`, `toString`, and their siblings test as declared and pass
 validation. Measured against `valibot@1.4.2`: `{ name, constructor }` satisfies a strict
 schema declaring only `name`. A null-prototype entries object removes the inherited
 names, so those keys are refused like any other undeclared one.
 
 That matters beyond tidiness: the advertised schema carries `additionalProperties:
 false`, so a client validating locally would reject what this server accepted, and the
 gate hands handlers the original object, leaving the undeclared key observable.
 
 @param entries - Argument schemas keyed by argument name
 
 @returns Strict object schema refusing undeclared keys, inherited names included
 
 @example
 ```ts
 strictArguments({ name: v.string() });
 ```
 */
export function strictArguments<const TEntries extends v.ObjectEntries,>(
  entries: TEntries,
): v.StrictObjectSchema<TEntries, undefined> {
  /**
   Same entries on a prototype-free object, so `key in entries` sees only declared names.
   The literal's `__proto__: null` sets the prototype rather than adding a property,
   and the spread keeps the declared entry types intact.
   */
  const bareEntries: TEntries = {
    __proto__: null,
    ...entries,
  };
  return v.strictObject(bareEntries,);
}

/**
 Returned by {@link validateToolArguments} when arguments satisfy the tool's schema.
 */
export const TOOL_ARGUMENTS_VALID: unique symbol = Symbol('mcp-stdio tool arguments valid',);

/**
 Raised when a tool's schema cannot become a conformant JSON Schema.
 
 Thrown during server construction rather than on first `tools/list`, so an unusable
 schema fails loudly at startup instead of being advertised in a degraded form.
 */
export class ToolSchemaError extends Error {
  /**
   Names the offending tool alongside why conversion failed.
   
   @param message - Description naming tool and cause
   
   @example
   ```ts
   throw new ToolSchemaError('Tool "ping" declares a schema that cannot be converted');
   ```
   */
  constructor(message: string,) {
    super(message,);
    this.name = 'ToolSchemaError';
  }
}

//endregion

//region Advertisement: converting a valibot schema to the advertised JSON Schema

/**
 Draft the MCP revision this package implements expects tools to advertise.
 
 The converter defaults to `draft-07`, which would advertise the wrong draft: revision
 2026-07-28 states `inputSchema` "Defaults to JSON Schema 2020-12 when no explicit
 `$schema` is provided", and names 2020-12 as the vocabulary whose keywords may appear.
 */
const JSON_SCHEMA_TARGET = 'draft-2020-12';

/**
 Root type every tool's argument schema must declare.
 
 Revision 2026-07-28 is explicit: "Tool arguments are always JSON objects, so
 `type: "object"` is required at the root." A valibot union converts to a bare `anyOf`
 with no root type, so the root is restored below rather than advertised non-conformant.
 */
const ROOT_TYPE = 'object';

/**
 Converts a tool's valibot schema into the JSON Schema advertised for it.
 
 Conversion runs with `errorMode: 'throw'` so a schema the converter cannot express, such
 as one built from `v.custom`, fails loudly instead of being advertised in a form that no
 longer matches what is enforced. That mode is the current default; passing it explicitly
 keeps a future default change from reintroducing silent degradation.
 
 @param schema - Valibot schema declared by this tool
 
 @param toolName - Tool name, quoted into failure messages
 
 @returns JSON Schema carrying an object root, ready to advertise
 
 @throws ToolSchemaError when conversion fails, or yields a non-object root
 
 @example
 ```ts
 toolInputSchema({ schema: v.strictObject({ name: v.string() }), toolName: 'destroy_vm' });
 ```
 */
export function toolInputSchema(
  {
    schema,
    toolName,
  }: {
    readonly schema: ToolArgumentsSchema;
    readonly toolName: string;
  },
): ToolInputSchema {
  /**
   Converted schema before its root type is checked and restored.
   */
  const converted = convertSchema({
    schema,
    toolName,
  },);

  /**
   Root type the converter produced; `undefined` for a union, which emits only `anyOf`.
   */
  const rootType = converted.type;
  if ((rootType !== undefined) && (rootType !== ROOT_TYPE)) {
    throw new ToolSchemaError(
      `Tool "${toolName}" declares arguments of type "${String(rootType,)}", but MCP requires an object root`,
    );
  }
  // An absent root type means a union. Restoring `object` over branches that are not all
  // objects would advertise a schema nothing can satisfy: the value would have to be an
  // object and also a string. Rejecting here beats shipping an uncallable tool.
  if (rootType === undefined) {
    /**
     Union branches the converter produced, absent when the schema is not a union.
     */
    const branches = converted.anyOf ?? [];
    /**
     Refusal shared by both ways a union can fail to admit an object root.
     */
    const notAllObjects = new ToolSchemaError(
      `Tool "${toolName}" declares a union whose branches are not all objects, which no object-rooted argument schema can satisfy`,
    );
    if (branches.length === 0)
      throw notAllObjects;
    // Element binding rather than a callback parameter: provenance from the converter's
    // own types does not survive a callback parameter, the gap tracked as #427.
    for (const branch of branches) {
      if (!isPlainObject(branch,))
        throw notAllObjects;
      if (branch.type !== ROOT_TYPE)
        throw notAllObjects;
    }
  }

  /**
   Advertised schema with the object root restored.
   
   Assembled as a plain record because the converter types every optional keyword as
   `X | undefined`, which `exactOptionalPropertyTypes` refuses to spread onto the named
   optional fields of {@link ToolInputSchema}. The keys carrying `undefined` disappear at
   serialization, so the frame a client receives is unaffected.
   */
  const advertised: Record<string, unknown> = {
    ...converted,
    type: ROOT_TYPE,
  };
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- root type is set on the line above, which is the only property ToolInputSchema requires; every other keyword is admitted by its index signature
  return advertised as ToolInputSchema;
}

/**
 Runs the converter, restating failures against the tool that caused them.
 
 @param schema - Valibot schema declared by this tool
 
 @param toolName - Tool name, quoted into failure messages
 
 @returns Raw converted schema as the converter models it
 
 @throws ToolSchemaError when the converter rejects this schema
 
 @example
 ```ts
 convertSchema({ schema: v.strictObject({}), toolName: 'list_vms' });
 ```
 */
function convertSchema(
  {
    schema,
    toolName,
  }: {
    readonly schema: ToolArgumentsSchema;
    readonly toolName: string;
  },
): JsonSchema {
  // Deliberate catch-and-rethrow: the converter names the offending valibot schema type but
  // not the tool, which is the only handle an author has to find it among many.
  try {
    return toJsonSchema(
      schema,
      {
        target: JSON_SCHEMA_TARGET,
        errorMode: 'throw',
      },
    );
  }
  catch (error: unknown) {
    throw new ToolSchemaError(
      `Tool "${toolName}" declares a schema that cannot be advertised as JSON Schema: ${
        caughtValueText(error,)
      }`,
    );
  }
}

//endregion

//region Validation: gating a call on the same schema that was advertised

/**
 Checks `tools/call` arguments against the tool's declared schema.
 
 Acts purely as a gate: the parsed output is discarded and the caller keeps handing the
 handler its original arguments, because valibot's object schemas strip unknown keys and
 substituting the parsed value would quietly change what every handler receives.
 
 @param schema - Valibot schema declared by this tool
 
 @param args - Untrusted argument bag from the client
 
 @returns {@link TOOL_ARGUMENTS_VALID}, or one message naming every violated path
 
 @example
 ```ts
 validateToolArguments({ schema, args: { name: 'vm1' } });
 // TOOL_ARGUMENTS_VALID
 ```
 */
export function validateToolArguments(
  {
    schema,
    args,
  }: {
    readonly schema: ToolArgumentsSchema;
    readonly args: Readonly<Record<string, unknown>>;
  },
): string | typeof TOOL_ARGUMENTS_VALID {
  /**
   Parse outcome; only its success flag and issues are consulted.
   */
  const outcome = v.safeParse(
    schema,
    args,
  );
  if (outcome.success)
    return TOOL_ARGUMENTS_VALID;
  return describeIssues(outcome.issues,);
}

/**
 Renders validation issues as one message a client can act on.
 
 Each issue is prefixed with the path it failed at, since a bare "expected string" gives
 no way to tell which argument is wrong when several share a type.
 
 @param issues - Issues valibot reported for one parse
 
 @returns Semicolon-separated descriptions, path first
 
 @example
 ```ts
 describeIssues(outcome.issues);
 // 'name: Invalid key: Expected "name" but received undefined'
 ```
 */
function describeIssues(
  issues: readonly [
    v.BaseIssue<unknown>,
    ...v.BaseIssue<unknown>[],
  ],
): string {
  return issues
    .map(function describeIssue(issue,): string {
      /**
       Dotted path to the failing value; absent when the whole argument bag failed.
       */
      const path = v.getDotPath(issue,);
      if (path === null)
        return issue.message;
      return `${path}: ${issue.message}`;
    },)
    .join('; ',);
}

//endregion
