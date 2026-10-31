/**
 Guarded valibot to TypeBox conversion for the pi adapter.

 Valibot is the single source for tool parameters; pi validates tool arguments with compiled
 TypeBox schemas, so the pi adapter converts instead of declaring twice.
 The converter covers exactly the constructs the shared tool specs use and throws on anything
 else, because a silent best-effort translation would advertise a schema nobody validated.

 @module
 */

import {
  Type,
  type TSchema,
} from 'typebox';

//region Types

/**
 Minimal structural view of a valibot schema node the converter walks.
 */
export type ValibotSchemaNode = {
  /**
   Valibot node discriminator.
   */
  readonly type: string;
  /**
   Object entries keyed by property name.
   */
  readonly entries?: Readonly<Record<string, ValibotSchemaNode>>;
  /**
   Array item schema.
   */
  readonly item?: ValibotSchemaNode;
  /**
   Schema wrapped by optional or similar wrappers.
   */
  readonly wrapped?: ValibotSchemaNode;
  /**
   Pipe stages: base schema copy first, metadata actions after.
   */
  readonly pipe?: readonly ValibotSchemaNode[];
  /**
   Description carried by a description metadata action.
   */
  readonly description?: string;
};

/**
 JSON Schema options carried from valibot metadata actions onto TypeBox builders.
 */
type ConvertedOptions = {
  /**
   Model-facing description text.
   */
  readonly description?: string;
};

//endregion Types

//region Errors

/**
 Thrown when a valibot schema falls outside the supported conversion subset.
 */
export class ValibotToTypeBoxError extends Error {
  /**
   Build a conversion failure naming the offending node and its schema path.

   @param message - offending node type and JSON-style schema path
   */
  constructor(message: string,) {
    super(message,);
    this.name = 'ValibotToTypeBoxError';
  }
}

//endregion Errors

//region Public API

/**
 Convert a valibot schema into the TypeBox schema pi compiles for argument validation.

 @param schema - valibot schema built from the supported subset

 @returns TypeBox schema carrying the same shape and descriptions

 @throws ValibotToTypeBoxError when the schema uses a construct outside the supported subset

 @example
 ```ts
 valibotToTypeBox(webFetchToolSpec.parameters);
 ```
 */
export function valibotToTypeBox(schema: ValibotSchemaNode,): TSchema {
  return convertNode({
    node: schema,
    path: '$',
  },);
}

//endregion Public API

//region Helpers

/**
 Collect JSON Schema options from a node's pipe metadata.

 @param node - valibot node whose pipe stages may carry a description

 @param path - schema path used in failure messages

 @returns builder options carrying recognized metadata only

 @throws ValibotToTypeBoxError when a pipe stage is anything but a description action

 @example
 ```ts
 collectConvertedOptions(node, '$.query');
 ```
 */
function collectConvertedOptions(
  {
    node,
    path,
  }: {
    /**
     Valibot node whose pipe stages may carry a description.
     */
    readonly node: ValibotSchemaNode;
    /**
     Schema path used in failure messages.
     */
    readonly path: string;
  },
): ConvertedOptions {
  /**
   Pipe stages after the leading base-schema copy.
   */
  const actions = (node.pipe ?? [])
    .slice(1,);
  for (const action of actions) {
    /**
    Whether this pipe stage is a recognized description action.
     */
    const isDescription = (action.type === 'description') && (action.description !== undefined);
    if (!isDescription)
      throw new ValibotToTypeBoxError(
        `unsupported valibot pipe action "${action.type}" at ${path}`,
      );
  }
  /**
   First description action found, when any.
   */
  const descriptionAction = actions
    .find(function isDescriptionAction(action,) {
      return action.type === 'description';
    },);
  if (descriptionAction === undefined)
    return {};
  if (descriptionAction.description === undefined)
    return {};
  return { description: descriptionAction.description, };
}

/**
 Convert one valibot node into its TypeBox counterpart.

 @param node - valibot node to convert

 @param path - schema path used in failure messages

 @returns TypeBox schema for this node

 @throws ValibotToTypeBoxError for unsupported nodes, missing sub-schemas, or unknown pipe actions

 @example
 ```ts
 convertNode({ node, path: '$.query' });
 ```
 */
function convertNode(
  {
    node,
    path,
  }: {
    /**
     Valibot node to convert.
     */
    readonly node: ValibotSchemaNode;
    /**
     Schema path used in failure messages.
     */
    readonly path: string;
  },
): TSchema {
  /**
   Builder options carried by pipe metadata.
   */
  const options = collectConvertedOptions({
    node,
    path,
  },);

  if (node.type === 'string')
    return options.description === undefined
      ? Type.String()
      : Type.String({ description: options.description, });

  if (node.type === 'optional') {
    if (node.wrapped === undefined)
      throw new ValibotToTypeBoxError(`optional node without wrapped schema at ${path}`);
    return Type.Optional(convertNode({
      node: node.wrapped,
      path,
    },),);
  }

  if (node.type === 'array') {
    if (node.item === undefined)
      throw new ValibotToTypeBoxError(`array node without item schema at ${path}`);
    /**
     Converted item schema.
     */
    const item = convertNode({
      node: node.item,
      path: `${path}[]`,
    },);
    return options.description === undefined
      ? Type.Array(item)
      : Type.Array(
        item,
        { description: options.description, },
      );
  }

  if ((node.type === 'object') || (node.type === 'strict_object')) {
    if (node.entries === undefined)
      throw new ValibotToTypeBoxError(`${node.type} node without entries at ${path}`);
    /**
     Converted entry map keyed like the valibot schema.
     */
    const entries = convertEntries({
      entries: node.entries,
      path,
    },);
    /**
     Whether callers may pass keys the schema does not list.
     */
    const isOpen = node.type === 'object';
    return Type.Object(
      entries,
      isOpen
        ? (options.description === undefined ? {} : { description: options.description, })
        : {
          additionalProperties: false,
          ...(options.description === undefined ? {} : { description: options.description, }),
        },
    );
  }

  throw new ValibotToTypeBoxError(
    `unsupported valibot node type "${node.type}" at ${path}`,
  );
}

/**
 Convert every entry of a valibot object node.

 @param entries - valibot entry map to convert

 @param path - schema path used in failure messages

 @returns TypeBox entry map with identical keys

 @throws ValibotToTypeBoxError when any entry falls outside the supported subset

 @example
 ```ts
 convertEntries({ entries: node.entries, path: '$' });
 ```
 */
function convertEntries(
  {
    entries,
    path,
  }: {
    /**
     Valibot entry map to convert.
     */
    readonly entries: Readonly<Record<string, ValibotSchemaNode>>;
    /**
     Schema path used in failure messages.
     */
    readonly path: string;
  },
): Record<string, TSchema> {
  /**
   Converted entries accumulated by key.
   */
  const converted: Record<string, TSchema> = {};
  for (const [key, entry,] of Object.entries(entries,)) {
    converted[key] = convertNode({
      node: entry,
      path: `${path}.${key}`,
    },);
  }
  return converted;
}

//endregion Helpers
