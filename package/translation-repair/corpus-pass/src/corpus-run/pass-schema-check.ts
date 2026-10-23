import {
  ClaudeSchemaKeywordError,
  claudeSchemaOf,
} from '@monochromatic-dev/translation-repair-provider/ts';
import type { SchemaProducer, } from '../claude-schema-producers.ts';

//region Pass schema check
// THE CHECK A PASS MAKES BEFORE IT ASKS ANY PROVIDER: every answer schema a stage can send goes through the Claude
// route's own narrowing, and a schema the route refuses stops the pass at its start, naming each schema and the
// keyword. Without it, a stage schema the route cannot send is found at the first call of the stage, inside an entry,
// which records one `status=ERROR` and goes on: each later entry that reaches the stage then fails the same way, after
// the paid calls of the stages that run ahead of it. A schema keyword is a fault in our own code and not a fault of
// any document, so the check is deterministic and costs no quota.

/**
 Refusal raised when stage schemas carry what the Claude route cannot send, one line for each schema.

 A FAULT IN THE COMMAND, not a stated refusal: the operator has nothing to type, and the stage schema has to be
 written again. The message names only the producer files, the schemas' own names, keywords and paths, all of which
 are written in this repository's source.

 @example
 ```ts
 throw new UnsendableSchemasError({ lines: ['cat/src/pick-wire.ts (pick): "oneOf" at root.pick',], },);
 ```
 */
export class UnsendableSchemasError extends Error {
  /**
   Declares this message safe to forward: it names source files, schema names, schema keywords and paths of member
   names, all written in this repository.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds the refusal.

   @param lines - one line for each refused schema: producer file, schema name, keyword and where it sits

   @example
   ```ts
   new UnsendableSchemasError({ lines: ['cat/src/pick-wire.ts (pick): "oneOf" at root.pick',], },);
   ```
   */
  public constructor({ lines, }: { readonly lines: readonly string[]; },) {
    super(
      `The pass did not start: ${String(lines.length,)} stage schema(s) carry what the Claude route cannot send, so `
        + `the API would answer HTTP 400 to every call of the stage that sends them. No provider was asked. Write the `
        + `schema with the keywords the structured outputs page lists as supported:\n${
         lines.join('\n',)}`,
    );
    this.name = 'UnsendableSchemasError';
  }
}

/**
 Refuses the pass when any answer schema a stage can send is one the Claude route cannot send.

 Reads each producer's schemas with `claudeSchemaOf`, the narrowing the Claude body applies, and collects every
 refusal so one run names them all.

 @param producers - files that declare answer schemas, each with every schema it can build

 @throws {@link UnsendableSchemasError} when any schema is refused, naming each schema and keyword

 @example
 ```ts
 assertSchemasSendable({ producers: CLAUDE_SCHEMA_PRODUCERS, },);
 ```
 */
export function assertSchemasSendable({ producers, }: { readonly producers: readonly SchemaProducer[]; },): void {
  /**
   One line for each refused schema, in the order of the producers.
   */
  const lines = producers.flatMap(function refusalsOfProducer({
    file,
    formats,
  },): readonly string[] {
    return formats.flatMap(function refusalsOfFormat({ json_schema: schema, },): readonly string[] {
      try {
        claudeSchemaOf({ schema: schema.schema, },);
        return [];
      }
      catch (error) {
        if (error instanceof ClaudeSchemaKeywordError)
          return [`${file} (${schema.name}): "${error.refusedKeyword}" at ${error.schemaPath}`,];
        throw error;
      }
    },);
  },);
  if (lines.length > 0)
    throw new UnsendableSchemasError({ lines, },);
}

//endregion Pass schema check
