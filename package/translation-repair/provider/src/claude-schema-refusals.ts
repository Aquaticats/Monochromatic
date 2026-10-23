import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Claude schema refusals
// WHAT THE STRUCTURED OUTPUTS PAGE LISTS AS NOT SUPPORTED THAT NO SDK STRIPS, beyond the keywords `claude-schema.ts`
// keeps on an allowlist (https://platform.claude.com/docs/en/build-with-claude/structured-outputs, "JSON Schema
// limitations" and "Complexity limits", fetched 2026-10-23). Quoted: "`anyOf` and `allOf` (with limitations: `allOf`
// with `$ref` not supported)", "`$ref`, `$def`, and `definitions` (external `$ref` not supported)", "`enum` (strings,
// numbers, Booleans, or nulls only, no complex types)" and, under not supported, "Recursive schemas". Each would
// answer HTTP 400 at the first live call, which the retry ladder returns unretried and the router counts as a lost
// voice, so each is refused here with the keyword and the node that carries it.
//
// A `$ref` THAT NAMES NOTHING IS REFUSED TOO, though the page does not list it: the request names a definition that
// is not in the schema, so there is nothing for the grammar to be built from, and a refusal at the node that holds
// the reference costs no call. That the live API would answer 400 is not measured here.
//
// A LIMIT OF THAT REFUSAL: a pointer is read as written and its fragment is not percent-decoded, so `#/$defs/a%20b`
// for a definition named `a b` is refused as naming nothing, where it passed before this refusal existed. No stage
// schema writes a pointer with an escape other than `~0` and `~1`, which are decoded.
//
// THE COMPLEXITY LIMITS, quoted from the page: "Strict tools per request: 20", "Optional parameters: 24 (total
// optional parameters across all strict tool schemas and JSON output schemas. Each parameter not listed in `required`
// counts toward this limit)" and "Parameters with union types: 16 (total parameters that use `anyOf` or type arrays
// ... across all strict schemas)". The body this route builds sends one schema in `output_config.format` and no
// tools, so the limit on strict tools cannot be reached and is not checked; the other two are counted over the one
// schema, member by member. A schema that stays inside both can still answer "Schema is too complex for compilation"
// from the page's internal grammar limits, which the page gives no figure for and this file does not guess.
//
// A COUNT IS ONLY EVER A REFUSAL WHERE THE LIVE API WOULD REFUSE TOO: the 24 stage schemas all answered HTTP 200 on
// 2026-10-22, and `claude-schemas.unit.test.ts` in `corpus-pass` sends each through this check.

/**
 The most optional parameters one request may carry, from the page's table of complexity limits.
 */
const MAX_OPTIONAL_PARAMETERS = 24;

/**
 The most parameters that use a union type one request may carry, from the page's table of complexity limits.
 */
const MAX_UNION_PARAMETERS = 16;

/**
 Keywords whose value maps a NAME to a schema. The names are the caller's, so a member called `minimum` is kept, and
 only the schemas under it are read.
 */
export const NAMED_SCHEMA_MAPS: ReadonlySet<string> = new Set([
  'properties',
  '$defs',
  'definitions',
],);

/**
 Keywords whose value is a schema, in the older tuple form a list of them, or a list of them: each is read the same
 way and sits at the path of its keyword, with an index for a list.
 */
export const SCHEMA_BEARING: ReadonlySet<string> = new Set([
  'items',
  'anyOf',
  'allOf',
],);

/**
 A schema refused, as the keyword the refusal names and where the node sits.
 */
export type SchemaRefusal = {
  /**
   What the schema carries that the feature does not take.
   */
  readonly refusedKeyword: string;

  /**
   Dotted path of the node that carries it, `root` for the top.
   */
  readonly schemaPath: string;
};

/**
 One schema node and where it sits.
 */
type SchemaNode = {
  /**
   Dotted path from the root, in the form the narrowing walk names a node by.
   */
  readonly path: string;

  /**
   The node.
   */
  readonly node: Readonly<Record<string, unknown>>;
};

/**
 Every schema node under a schema and the schema itself, a name under `properties` read as a name and not as a keyword.

 A BOUNDED WALK OVER THE SCHEMA'S OWN NESTING, which a stage writes by hand; a `$ref` is a string here
 and is not followed.

 @param schema - node read

 @param path - where it sits

 @returns Nodes, the node itself first

 @example
 ```ts
 const nodes = schemaNodesOf({ schema: { type: 'object', }, path: 'root', },);
 ```
 */
function schemaNodesOf(
  {
    schema,
    path,
  }: {
    readonly schema: Readonly<Record<string, unknown>>;
    readonly path: string;
  },
): readonly SchemaNode[] {
  return [
    {
      path,
      node: schema,
    },
    ...Object.entries(schema,)
      .flatMap(function nodesBelow([keyword, value,],): readonly SchemaNode[] {
        if (NAMED_SCHEMA_MAPS.has(keyword,) && isJsonRecord(value,)) {
          return Object.entries(value,)
            .flatMap(function nodesOfMember([name, member,],): readonly SchemaNode[] {
              return isJsonRecord(member,)
                ? schemaNodesOf({
                  schema: member,
                  path: (keyword === 'properties') ? `${path}.${name}` : `${path}.${keyword}.${name}`,
                },)
                : [];
            },);
        }
        if (!SCHEMA_BEARING.has(keyword,))
          return [];

        /**
         Where each schema under the keyword sits, one place for a schema and one for each member of a list.
         */
        const places: readonly (readonly [
          string,
          unknown
        ])[] = Array.isArray(value,)
          ? value.map(function placeOf(
            item: unknown,
            index,
          ): readonly [
            string,
            unknown
          ] {
            return [
              `${path}.${keyword}[${String(index,)}]`,
              item,
            ];
          },)
          : [[
            `${path}.${keyword}`,
            value,
          ],];
        return places.flatMap(function nodesAt([place, held,],): readonly SchemaNode[] {
          return isJsonRecord(held,)
            ? schemaNodesOf({
              schema: held,
              path: place,
            },)
            : [];
        },);
      },),
  ];
}

/**
 Whether a value may stand in an `enum`: the page allows strings, numbers, Booleans and nulls only.

 @param member - one member of an `enum`

 @returns Whether it is one of those four

 @example
 ```ts
 const plain = isPlainEnumMember({ member: 'lap', },);
 // => true
 ```
 */
function isPlainEnumMember({ member, }: { readonly member: unknown; },): boolean {
  return (member === null) || [
    'string',
    'number',
    'boolean',
  ].includes(typeof member,);
}

/**
 What one node carries, by itself, that the page lists as not supported.

 @param path - where the node sits

 @param node - node read

 @returns Zero or one refusal: none where the node carries none of the three

 @example
 ```ts
 const [refusal,] = refusalsOfNode({ path: 'root', node: { $ref: 'http://example.test/cat.json', }, },);
 // => { refusedKeyword: '$ref to another document', schemaPath: 'root', }
 ```
 */
function refusalsOfNode(
  {
    path,
    node,
  }: {
    readonly path: string;
    readonly node: Readonly<Record<string, unknown>>;
  },
): readonly SchemaRefusal[] {
  if (((typeof node.$ref) === 'string') && (!(node.$ref as string).startsWith('#',)))
    return [
      {
        refusedKeyword: '$ref to another document',
        schemaPath: path,
      },
    ];
  if (Array.isArray(node.enum,) && (!node.enum
    .every(function isPlain(member,): boolean {
      return isPlainEnumMember({ member, },);
    },)))
    return [
      {
        refusedKeyword: 'enum with a member that is not a string, a number, a Boolean or null',
        schemaPath: path,
      },
    ];
  if (Array.isArray(node.allOf,)) {
    /**
     Position of the first member that holds a `$ref`, or minus one.
     */
    const holding = node.allOf
      .findIndex(function holdsRef(member: unknown,): boolean {
        return isJsonRecord(member,) && Object.hasOwn(
          member,
          '$ref',
        );
      },);
    if (holding !== (-1))
      return [
        {
          refusedKeyword: 'allOf with a $ref',
          schemaPath: `${path}.allOf[${String(holding,)}]`,
        },
      ];
  }
  return [];
}

/**
 Follows a same-document pointer (`#` or `#/a/b`) to the node it names.

 @param root - whole schema

 @param ref - `$ref` text, which starts with `#`

 @returns What the pointer ends at: a record or a Boolean where it names a schema, anything else where it names
 something that is no schema or nothing at all

 @example
 ```ts
 const target = resolvedRef({ root, ref: '#/$defs/cat', },);
 ```
 */
function resolvedRef(
  {
    root,
    ref,
  }: {
    readonly root: Readonly<Record<string, unknown>>;
    readonly ref: string;
  },
): unknown {
  return ref
    .slice(1,)
    .split('/',)
    .slice(1,)
    .map(function unescaped(segment,): string {
      return segment
        .replaceAll(
        '~1',
        '/',
        )
        .replaceAll(
        '~0',
        '~',
        );
    },)
    .reduce<unknown>(
    function step(
      at,
      segment,
    ): unknown {
      // An array answers to its index as text, so one lookup serves a list and a record.
      if (!(isJsonRecord(at,) || Array.isArray(at,)))
        return undefined;
      return Object.hasOwn(
        at,
        segment,
      )
        ? Reflect.get(
          at,
          segment,
        )
        : undefined;
    },
    root,
    );
}

/**
 The first `$ref` that leads back into a schema it is already inside, which makes the schema recursive.

 @param root - whole schema, which the pointers resolve against

 @param nodes - nodes to read, with the nodes of every reference being followed already on the way

 @param active - references being followed, the way into these nodes

 @param settled - references followed to their end without a cycle, which are not followed twice

 @returns Zero or one refusal: none where no reference closes a cycle, else the one at the node that holds it

 @example
 ```ts
 const nodes = schemaNodesOf({ schema: root, path: 'root', },);
 const [refusal,] = cyclesIn({ root, nodes, active: [], settled: new Set(), },);
 ```
 */
function cyclesIn(
  {
    root,
    nodes,
    active,
    settled,
  }: {
    readonly root: Readonly<Record<string, unknown>>;
    readonly nodes: readonly SchemaNode[];
    readonly active: readonly string[];
    readonly settled: Set<string>;
  },
): readonly SchemaRefusal[] {
  for (const {
    path,
    node,
  } of nodes) {
    if (((typeof node.$ref) !== 'string') || (!(node.$ref as string).startsWith('#',)))
      continue;

    /**
     The reference this node holds.
     */
    const ref = node.$ref as string;
    if (active.includes(ref,))
      return [
        {
          refusedKeyword: 'a recursive $ref',
          schemaPath: path,
        },
      ];
    if (settled.has(ref,))
      continue;

    /**
     What the reference ends at.
     */
    const target = resolvedRef({
      root,
      ref,
    },);
    // A Boolean schema is a schema and ends the chain; anything else that is not a record names no schema, so no
    // grammar can be built from the reference (that the live API would answer 400 is not measured, see the header).
    if ((typeof target) === 'boolean')
      continue;
    if (!isJsonRecord(target,))
      return [
        {
          refusedKeyword: 'a $ref that names no schema',
          schemaPath: path,
        },
      ];

    /**
     The cycle found under the target, if any.
     */
    const found = cyclesIn({
      root,
      nodes: schemaNodesOf({
        schema: target,
        path,
      },),
      active: [
        ...active,
        ref,
      ],
      settled,
    },);
    // The refusal names the reference where the chain starts in the schema as written, not a node of the expansion.
    if (found.length > 0)
      return found.map(function atSite(refusal,): SchemaRefusal {
        return {
          ...refusal,
          schemaPath: path,
        };
      },);
    settled.add(ref,);
  }
  return [];
}

/**
 One member of an object schema, with whether the object lists it as required.
 */
type SchemaMember = {
  /**
   The member's schema, or `true` where it is a boolean schema.
   */
  readonly schema: Readonly<Record<string, unknown>> | boolean;

  /**
   Whether the object's `required` lists the member's name.
   */
  readonly isRequired: boolean;
};

/**
 The refusal, if any, for the request's schema as a whole: more optional parameters or more union-typed parameters than
 the page allows in one request.

 @param nodes - every node of the schema

 @returns Zero or one refusal: none where both counts are inside the limits, else one at the root

 @example
 ```ts
 const nodes = schemaNodesOf({ schema, path: 'root', },);
 const [refusal,] = complexityRefusalsOf({ nodes, },);
 ```
 */
function complexityRefusalsOf({ nodes, }: { readonly nodes: readonly SchemaNode[]; },): readonly SchemaRefusal[] {
  /**
   Every member of every object, with whether its object lists it as required.
   */
  const members = nodes.flatMap(function membersOf({ node, },): readonly SchemaMember[] {
    if (!isJsonRecord(node.properties,))
      return [];

    /**
     Names the object lists as required.
     */
    const required: readonly unknown[] = Array.isArray(node.required,) ? node.required : [];
    return Object.entries(node.properties,)
      .map(function memberOf([name, member,],): SchemaMember {
        return {
          schema: isJsonRecord(member,) ? member : true,
          isRequired: required.includes(name,),
        };
      },);
  },);

  /**
   Members not listed in `required`.
   */
  const optional = members.filter(function isOptional({ isRequired, },): boolean {
    return !isRequired;
  },)
    .length;
  if (optional > MAX_OPTIONAL_PARAMETERS) {
    return [
      {
        refusedKeyword: `${String(optional,)} optional parameters, where the page allows ${
          String(MAX_OPTIONAL_PARAMETERS,)
        } in one request`,
        schemaPath: 'root',
      },
    ];
  }

  /**
   Members that use `anyOf` or a type array.
   */
  const union = members.filter(function isUnion({ schema, },): boolean {
    return isJsonRecord(schema,) && (Array.isArray(schema.anyOf,) || Array.isArray(schema.type,));
  },)
    .length;
  if (union > MAX_UNION_PARAMETERS) {
    return [
      {
        refusedKeyword: `${String(union,)} parameters that use a union type, where the page allows ${
          String(MAX_UNION_PARAMETERS,)
        } in one request`,
        schemaPath: 'root',
      },
    ];
  }
  return [];
}

/**
 The first thing a schema carries that the page lists as not supported and no keyword allowlist can see: an `allOf`
 member that holds a `$ref`, an `enum` member that is not a string, a number, a Boolean or null, a `$ref` into another
 document, a `$ref` that names no schema, a recursive `$ref`, and a request past the limits on optional
 and union-typed parameters.

 @param schema - schema as the stage wrote it

 @returns Zero or one refusal: none where the schema carries none of them

 @example
 ```ts
 const [refusal,] = structuralRefusalOf({ schema: { type: 'object', properties: { child: { $ref: '#', }, }, }, },);
 // => { refusedKeyword: 'a recursive $ref', schemaPath: 'root.child', }
 ```
 */
export function structuralRefusalOf(
  { schema, }: { readonly schema: Readonly<Record<string, unknown>>; },
): readonly SchemaRefusal[] {
  /**
   Every node of the schema, the root first.
   */
  const nodes = schemaNodesOf({
    schema,
    path: 'root',
  },);

  /**
   What the nodes carry by themselves, each node's first refusal.
   */
  const ofNodes = nodes.flatMap(function ofNode({
    path,
    node,
  },): readonly SchemaRefusal[] {
    return refusalsOfNode({
      path,
      node,
    },);
  },);
  if (ofNodes.length > 0)
    return ofNodes.slice(
      0,
      1,
    );

  /**
   The recursive reference found, if any.
   */
  const cycles = cyclesIn({
    root: schema,
    nodes,
    active: [],
    settled: new Set<string>(),
  },);
  return (cycles.length > 0) ? cycles : complexityRefusalsOf({ nodes, },);
}

//endregion Claude schema refusals
