//region Reason order fixture
// Reads the order a ballot's reply schema and its sheet's reply instruction
// declare their fields in, so a case can require the reason to come before the
// choice without asking the schema builder for the expected answer (owner,
// 2026-10-21, "Reason first everywhere", extending the ruling of 2026-09-27,
// "Reason before vote").

/**
 Whether a value is a plain object, narrowed so its members can be read.

 @param value - value read off a schema

 @returns Whether it is a non-null, non-array object

 @example
 ```ts
 if (isObject(schema)) schema.properties;
 ```
 */
function isObject(value: unknown,): value is Readonly<Record<string, unknown>> {
  if ((typeof value) !== 'object')
    return false;
  if (value === null)
    return false;
  return !Array.isArray(value,);
}

/**
 Thrown when a schema or an instruction does not name the reason ahead of the
 vote, saying where and in what order the fields stand.

 @example
 ```ts
 throw new ReasonOrderError({ where: 'the schema', reason: 'reason', vote: 'best', order: ['best', 'reason',], },);
 ```
 */
class ReasonOrderError extends Error {
  /**
   Builds the report.

   @param where - what was read, a schema list or an instruction

   @param reason - name of the reason field

   @param vote - name of the choice field

   @param order - the names in the order the reader found them
   */
  constructor(
    {
      where,
      reason,
      vote,
      order,
    }: {
      readonly where: string;
      readonly reason: string;
      readonly vote: string;
      readonly order: readonly string[];
    },
  ) {
    /**
     The names as the reader found them.
     */
    const found = order.join(', ',);
    super(`${where} must name ${reason} first and before ${vote}, and it reads: ${found}`,);
    this.name = 'ReasonOrderError';
  }
}

/**
 The object schema of a response format.

 @param format - response format as a stage hands it to the client

 @returns The schema's own members

 @throws Error - when the format carries no object schema

 @example
 ```ts
 const { properties, } = schemaOf({ format, },);
 ```
 */
function schemaOf({ format, }: { readonly format: unknown; },): Readonly<Record<string, unknown>> {
  if (!isObject(format,))
    throw new Error('the response format is not an object',);
  /**
   The JSON schema envelope.
   */
  const { json_schema: envelope, } = format;
  if (!isObject(envelope,))
    throw new Error('the response format carries no json_schema',);
  /**
   The schema inside it.
   */
  const { schema, } = envelope;
  if (!isObject(schema,))
    throw new Error('the response format carries no object schema',);
  return schema;
}

/**
 Property names of a response format's object schema, in declared order.

 @param format - response format as a stage hands it to the client

 @returns Names of the top-level properties, in declaration order

 @throws Error - when the format carries no object schema with properties

 @example
 ```ts
 const names = propertyOrderOf({ format: CANDIDATE_SELECT_RESPONSE_FORMAT, },);
 ```
 */
export function propertyOrderOf({ format, }: { readonly format: unknown; },): readonly string[] {
  /**
   The properties record.
   */
  const { properties, } = schemaOf({ format, },);
  if (!isObject(properties,))
    throw new Error('the response format carries no object schema with properties',);
  return Object.keys(properties,);
}

/**
 Required names of a response format's object schema, in declared order.

 @param format - response format as a stage hands it to the client

 @returns Names the schema requires, in declaration order

 @throws Error - when the format carries no required list

 @example
 ```ts
 const names = requiredOrderOf({ format: CANDIDATE_SELECT_RESPONSE_FORMAT, },);
 ```
 */
function requiredOrderOf({ format, }: { readonly format: unknown; },): readonly string[] {
  /**
   The required list.
   */
  const { required, } = schemaOf({ format, },);
  if (!Array.isArray(required,))
    throw new Error('the response format carries no required list',);
  return required.map(String,);
}

/**
 Requires a schema to declare, and to require, the reason first and before
 the vote.

 @param format - response format as a stage hands it to the client

 @param reason - name of the reason field

 @param vote - name of the choice field the reason must precede

 @throws {@link ReasonOrderError} when either list does not lead with the
 reason or does not name the vote after it

 @example
 ```ts
 expectReasonFirstInSchema({ format, reason: 'reason', vote: 'best', },);
 ```
 */
export function expectReasonFirstInSchema(
  {
    format,
    reason,
    vote,
  }: {
    readonly format: unknown;
    readonly reason: string;
    readonly vote: string;
  },
): void {
  /**
   Declared names, then required names, each with what it is.
   */
  const lists = [
    {
      where: 'the declared properties',
      order: propertyOrderOf({ format, },),
    },
    {
      where: 'the required properties',
      order: requiredOrderOf({ format, },),
    },
  ];
  for (const list of lists) {
    /**
     What the list is, and its names.
     */
    const {
      where,
      order,
    } = list;
    /**
     Where the reason stands in this list, and where the vote.
     */
    const reasonAt = order.indexOf(reason,);
    /**
     Where the vote stands in the same list.
     */
    const voteAt = order.indexOf(vote,);
    if ((reasonAt !== 0) || (voteAt < 1)) {
      throw new ReasonOrderError({
        where,
        reason,
        vote,
        order,
      },);
    }
  }
}

/**
 Requires a sheet's reply instruction to name the reason before the vote.

 @param text - the instruction, or the sheet it closes

 @param reason - phrase naming the reason field in the instruction

 @param vote - phrase naming the choice field in the instruction

 @throws {@link ReasonOrderError} when the instruction names the vote first or
 leaves either out

 @example
 ```ts
 expectReasonBeforeVoteInText({ text: user, reason: 'assessment one sentence', vote: 'verdict one of', },);
 ```
 */
export function expectReasonBeforeVoteInText(
  {
    text,
    reason,
    vote,
  }: {
    readonly text: string;
    readonly reason: string;
    readonly vote: string;
  },
): void {
  /**
   Where the instruction names the reason.
   */
  const reasonAt = text.indexOf(reason,);
  /**
   Where it names the vote.
   */
  const voteAt = text.indexOf(vote,);
  /**
   Whether the reason is named.
   */
  const reasonNamed = text.includes(reason,);
  /**
   Whether the vote is named.
   */
  const voteNamed = text.includes(vote,);
  /**
   Whether the reason stands ahead of the vote.
   */
  const reasonFirst = reasonAt < voteAt;
  if (!reasonNamed) {
    throw new ReasonOrderError({
      where: 'the instruction',
      reason,
      vote,
      order: [`${reason} at ${String(reasonAt,)}`,],
    },);
  }
  if ((!voteNamed) || (!reasonFirst)) {
    throw new ReasonOrderError({
      where: 'the instruction',
      reason,
      vote,
      order: [
        `${reason} at ${String(reasonAt,)}`,
        `${vote} at ${String(voteAt,)}`,
      ],
    },);
  }
}

//endregion Reason order fixture
