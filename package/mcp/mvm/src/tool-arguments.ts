/**
 Shared argument-schema builders for mvm tools.
 
 Every tool declares its arguments once as a valibot schema; `mcp-stdio` converts that
 same declaration into the JSON Schema clients see and validates calls against it, so
 what is advertised and what is enforced cannot drift apart.
 
 @module
 */
import * as v from 'valibot';

//region Argument builders: described strings, required and optional

/**
 String argument carrying the description clients read in `tools/list`.
 */
export type DescribedString = v.SchemaWithPipe<readonly [
  v.StringSchema<undefined>,
  v.DescriptionAction<string, string>,
]>;

/**
 Optional counterpart of {@link DescribedString}.
 */
export type OptionalDescribedString = v.OptionalSchema<DescribedString, undefined>;

/**
 Builds a required string argument.
 
 @param description - Text clients read for this argument
 
 @returns Schema rejecting a missing or non-string value
 
 @example
 ```ts
 requiredString('VM name to execute in');
 ```
 */
export function requiredString(description: string,): DescribedString {
  return v.pipe(
    v.string(),
    v.description(description,),
  );
}

/**
 Builds an optional whole-number argument of at least one.

 The declared type states what callers may rely on: an absent value, or a whole
 number. The pipe's own action types carry message parameters not worth restating
 here, and every caller reads the value through `typeof args.<name> === 'number'`
 anyway.

 @param description - Text clients read for this argument

 @returns Schema accepting absence, rejecting zero, a fraction and a non-number

 @example
 ```ts
 optionalPositiveInteger('Fewest virtual CPUs the machine may have');
 ```
 */
export function optionalPositiveInteger(
  description: string,
): v.OptionalSchema<v.GenericSchema<number>, undefined> {
  return v.optional(v.pipe(
    v.number(),
    v.integer(),
    v.minValue(1,),
    v.description(description,),
  ),);
}

/**
 Builds an optional string argument.
 
 @param description - Text clients read for this argument
 
 @returns Schema accepting absence, rejecting a present non-string value
 
 @example
 ```ts
 optionalString('Clone from this existing VM instead of creating fresh');
 ```
 */
export function optionalString(description: string,): OptionalDescribedString {
  return v.optional(requiredString(description,),);
}

//endregion
