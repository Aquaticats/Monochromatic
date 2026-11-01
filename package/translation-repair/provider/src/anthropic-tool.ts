import { isAsciiAlphanumeric, } from '@monochromatic-dev/translation-repair-markup/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Anthropic tool
// THE ANSWER TOOL, described twice on purpose.
//
// Charm Hyper speaks the Anthropic Messages protocol, which carries no
// `response_format`. Structured output there is obtained by defining a tool
// whose input schema IS the answer schema and requiring the model to call it.
// The OpenAI-compatible half of this pipeline states the same constraint as
// `JsonSchemaResponseFormat`, so this file translates one into the other and
// nothing downstream learns which provider answered.
//
// WHY THE SCHEMA IS WRITTEN INTO THE SYSTEM PROMPT AS WELL, at the owner's
// instruction: some model and provider pairs behave badly without a detailed
// system prompt, up to and including emitting the wrong tool-call format. The
// schema in `tools` is what the server validates against; the schema in
// `system` is what the model READS. A weak model follows the prose and ignores
// the envelope, so both carry it, and both are rendered from the same argument
// here so they cannot drift apart.
//
// THE FORMAT RULES ARE NOT FILLER. Each line of them names a shape a model has
// actually been seen to emit instead of a tool call: the answer as fenced text,
// the arguments as a JSON string rather than an object, the object wrapped in
// one more envelope key, renamed properties, a required field dropped because
// its honest value was empty, a second call carrying the rest.

/**
 Indent the embedded schema is printed at, so a model reads its structure
 rather than one very long line.
 */
const SCHEMA_INDENT = 2;

/**
 Characters the Messages API accepts in a tool name besides ASCII letters and
 digits.

 TESTED RATHER THAN MATCHED, because a character-class regex over an
 externally supplied name is exactly the shape `RG1` asks to be written as a
 scan instead, and a scan over a name this short costs nothing.
 */
const NAME_MARKS: ReadonlySet<string> = new Set([
  '_',
  '-',
],);

/**
 Longest tool name the Messages API accepts.
 */
const NAME_LIMIT = 64;

/**
 Structured-output constraint, projected so every path through it is readonly.

 WHY THIS EXISTS RATHER THAN THE OPENAI-SIDE TYPE ITSELF: that type carries the
 schema body as an ordinary record, whose index signature is writable, and a
 writable index signature reachable from a parameter describes a function that
 could rewrite its caller's schema. Nothing here does, and this projection is
 how that is stated rather than promised. Every OpenAI-side constraint
 satisfies it structurally, so no caller changes.

 @example
 ```ts
 const format: ReadableResponseFormat = { type: 'json_schema', json_schema, };
 ```
 */
export type ReadableResponseFormat = {
  /**
   Discriminator the OpenAI-compatible API expects, carried through unread.
   */
  readonly type: 'json_schema';

  /**
   Schema envelope: name, optional strictness, JSON schema body.
   */
  readonly json_schema: {
    /**
     Identifier this schema was given, which becomes the tool name.
     */
    readonly name: string;

    /**
     Whether the OpenAI side asked its server to enforce strictly.
     */
    readonly strict?: boolean;

    /**
     JSON schema of the answer object.
     */
    readonly schema: Readonly<Record<string, unknown>>;
  };
};

/**
 Refusal raised when a schema cannot be offered to Anthropic as a tool.

 THROWN RATHER THAN RETURNED, unlike a model's refusal: a name the protocol
 rejects is our own construction error, not an unreliable model's answer, and
 it would otherwise surface as a provider `400` on every call of that stage.

 @example
 ```ts
 throw new UnnameableToolError({ detail: 'name is empty', },);
 ```
 */
export class UnnameableToolError extends Error {
  /**
   Declares this message safe to forward: it names which rule the tool name broke.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds failure naming what disqualified the schema's name.

   @param detail - which naming rule the schema violated

   @example
   ```ts
   new UnnameableToolError({ detail: 'name exceeds 64 characters', },);
   ```
   */
  public constructor(
    { detail, }: { readonly detail: string; },
  ) {
    super(`Schema cannot be offered to Anthropic as a tool: ${detail}`,);
    this.name = 'UnnameableToolError';
  }
}

/**
 Tool entry the Messages API takes, carrying the answer schema.

 FIELD NAMES ARE THE WIRE'S, not the repo's, for the same reason
 `JsonSchemaResponseFormat` carries `json_schema`: this value is serialised
 as-is and a camel-cased copy would need a second translation nobody reads.

 @example
 ```ts
 const tool: AnthropicToolDefinition = { name: 'repair', description, input_schema, };
 ```
 */
export type AnthropicToolDefinition = {
  /**
   Name the model calls, and the one `tool_choice` names when forcing.
   */
  readonly name: string;

  /**
   What calling it means, read by the model when choosing whether to.
   */
  readonly description: string;

  /**
   JSON schema of the single object the call carries.
   */
  readonly input_schema: Readonly<Record<string, unknown>>;
};

/**
 Name the answer tool takes, refusing one the protocol would reject.

 READS THE SCHEMA'S OWN NAME rather than inventing one, so a stage that is
 routed to either provider is described to the model identically by both.

 @param responseFormat - structured-output constraint the caller stated

 @returns Validated tool name

 @throws {@link UnnameableToolError} where the name is empty, too long, or
 carries a character the Messages API rejects

 @example
 ```ts
 const name = answerToolName({ responseFormat, },);
 ```
 */
export function answerToolName(
  { responseFormat, }: { readonly responseFormat: ReadableResponseFormat; },
): string {
  /**
   Name the OpenAI-compatible constraint gave this schema.
   */
  const { name, } = responseFormat.json_schema;

  if (name.length === 0)
    throw new UnnameableToolError({ detail: 'schema name is empty', },);

  if (name.length > NAME_LIMIT)
    throw new UnnameableToolError({
      detail: `schema name is ${String(name.length,)} ${
        wordForCount({
          count: name.length,
          one: 'character',
          many: 'characters',
        },)
      }, over the ${String(NAME_LIMIT,)} the protocol allows`,
    },);

  for (const character of name) {
    if ((!isAsciiAlphanumeric({ character, },)) && (!NAME_MARKS.has(character,)))
      throw new UnnameableToolError({
        detail: 'schema name carries a character outside letters, digits, underscore and hyphen',
      },);
  }

  return name;
}

/**
 Answer tool as the Messages API takes it.

 @param responseFormat - structured-output constraint the caller stated

 @returns Tool entry for the request body

 @throws {@link UnnameableToolError} where the schema name is unusable

 @example
 ```ts
 const tools = [answerToolDefinition({ responseFormat, },),];
 ```
 */
export function answerToolDefinition(
  { responseFormat, }: { readonly responseFormat: ReadableResponseFormat; },
): AnthropicToolDefinition {
  /**
   Validated name, shared with the system prompt and with `tool_choice`.
   */
  const name = answerToolName({ responseFormat, },);

  return {
    name,
    description: `Deliver your entire answer by calling this tool. Its input is the ${name} `
      + 'object the schema describes, and nothing else.',
    input_schema: responseFormat
      .json_schema
      .schema,
  };
}

/**
 System prompt carrying the caller's instruction and the whole answer schema.

 THE INSTRUCTION COMES FIRST because it is the task; the answer protocol is
 how to hand the task's result back, and a model that reads only the opening
 of a long system prompt should meet the work rather than the envelope.

 `strict` is deliberately not represented. It is an OpenAI-side server flag
 with no Messages counterpart, and the pipeline validates every answer
 client-side regardless, so nothing here weakens by dropping it.

 @param instruction - caller's own system text, empty where it sent none

 @param responseFormat - structured-output constraint the caller stated

 @returns System text for the request body

 @throws {@link UnnameableToolError} where the schema name is unusable

 @example
 ```ts
 const system = renderToolSystemPrompt({ instruction, responseFormat, },);
 ```
 */
export function renderToolSystemPrompt(
  {
    instruction,
    responseFormat,
  }: {
    readonly instruction: string;
    readonly responseFormat: ReadableResponseFormat;
  },
): string {
  /**
   Validated name, the same one `tools` and `tool_choice` carry.
   */
  const name = answerToolName({ responseFormat, },);

  /**
   Schema as the model reads it, printed rather than described.
   */
  const printedSchema = JSON.stringify(
    responseFormat
      .json_schema
      .schema,
    null,
    SCHEMA_INDENT,
  );

  /**
   Answer protocol, schema included in full.
   */
  const protocol = `HOW TO ANSWER. Deliver your whole answer by calling the tool named ${name}, `
    + 'once. That tool takes one object, and that object is the answer. Nothing you write '
    + 'outside the tool call is read.\n\n'
    + `THE EXACT SCHEMA OF THAT OBJECT:\n\n${printedSchema}\n\n`
    + 'FORMAT RULES. Each of these names a mistake models make here:\n\n'
    + '- Call the tool. Do not write the JSON as ordinary text and do not put it in a fenced '
    + 'code block. Text outside the call is discarded.\n'
    + '- Pass the object itself. Do not pass a string that contains JSON, and do not escape '
    + 'its braces.\n'
    + '- Pass it at the top level of the tool input. Do not wrap it in a further key such as '
    + `input, arguments, parameters, or ${name}.\n`
    + '- Use exactly the property names the schema lists, spelled exactly. Do not rename one, '
    + 'and do not add a property the schema does not list.\n'
    + '- Every property the schema requires must be present, including where the honest value '
    + 'is an empty string or an empty array. Omitting it is not the same as saying it is empty.\n'
    + '- Call the tool once. Put the whole answer in that one call.\n'
    + '- String values are plain text. Do not fence, bullet or otherwise decorate one unless '
    + 'the schema says that field carries markup.';

  if (instruction.length === 0)
    return protocol;

  return `${instruction}\n\n${protocol}`;
}

//endregion Anthropic tool
