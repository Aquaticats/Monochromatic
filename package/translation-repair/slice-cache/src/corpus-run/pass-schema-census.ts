import {
  allInInputOrder,
  isJsonRecord,
  requireJsonSyntaxRefusal,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { textsInCodePointOrder, } from '@monochromatic-dev/translation-repair-markup/ts';
import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  entryIdOfArtifact,
  listArtifactFiles,
  requireArtifactParseRefusal,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import { readArtifactSchemaVersion, } from '@monochromatic-dev/translation-repair-artifact/ts';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

//region Pass schema census
// Which artifact GENERATION each settled file belongs to, as a discriminated
// answer rather than as the sentence a refusal would print.
//
// KEYING A CENSUS BY ITS OWN PROSE loses exactly what the reader of a refusal
// needs. Three unversioned generations collapse into one phrase, and a file
// that is not an artifact at all collapses with a file from a generation this
// build cannot read, so a message built from those keys can tell an operator
// their malformed file is a sound result of another generation and should be
// kept. The classification is the fact; the sentence is built from it later.

/**
 What one settled file says about its generation.

 @example
 ```ts
 const classification: SchemaClassification = { kind: 'declared', version: 2, };
 ```
 */
export type SchemaClassification = {
  /**
   File names a generation this build can read.
   */
  readonly kind: 'declared';

  /**
   Generation it names.
   */
  readonly version: number;
} | {
  /**
   File carries no version field, so its generation is legible only from which
   fields it holds. Three such generations exist and this does not tell them
   apart; nothing downstream needs it to, because none of them is writable.
   */
  readonly kind: 'unversioned';
} | {
  /**
   File names a generation this build cannot read: a version field that is not
   a count, or a generation written after this build.
   */
  readonly kind: 'unreadable-version';

  /**
   What the reading refused, for the message.
   */
  readonly reason: string;
} | {
  /**
   File is not an artifact at all: not JSON, or JSON that is not a record.

   SEPARATE FROM EVERY OTHER ANSWER because its remedy is the opposite one.
   Another generation's artifact is a sound result to keep; this is a file to
   investigate, and the pipeline guard refuses it first with the advice that
   fits.
   */
  readonly kind: 'malformed';

  /**
   What reading it refused.
   */
  readonly reason: string;
};

/**
 One settled entry and the generation its file belongs to.

 @example
 ```ts
 const row: SchemaCensusRow = { entryId: 'Mittens', classification: { kind: 'unversioned', }, };
 ```
 */
export type SchemaCensusRow = {
  /**
   Entry the file settles, which is its name without the suffix.
   */
  readonly entryId: string;

  /**
   What that file says about its generation.
   */
  readonly classification: SchemaClassification;
};

/**
 Classifies one artifact's generation from its parsed body.

 @param artifact - artifact as parsed JSON, of any shape

 @param entryId - entry the artifact belongs to, for the reader's error paths

 @returns Which generation it declares, or why that could not be read

 @throws Whatever a reading raised that is not a parse failure, since this
 classifies artifacts rather than swallowing faults

 @example
 ```ts
 const classification = classifyArtifact({ artifact, entryId: 'Mittens', },);
 ```
 */
function classifyArtifact(
  {
    artifact,
    entryId,
  }: {
    readonly artifact: unknown;
    readonly entryId: string;
  },
): SchemaClassification {
  // AN ARRAY IS NOT A RECORD HERE, and `isJsonRecord` refuses one (ledger
  // B92). Left to the version reader it would carry no version field and be
  // filed as an unversioned generation; a file that is a JSON array is a file
  // to investigate.
  if (!isJsonRecord(artifact,)) {
    return {
      kind: 'malformed',
      reason: 'a JSON object, which is what every generation of this artifact is',
    };
  }

  try {
    /**
     Generation the artifact names, or a named absence.
     */
    const reading = readArtifactSchemaVersion({
      artifact,
      path: entryId,
    },);

    return (reading.kind === 'versioned')
      ? {
        kind: 'declared',
        version: reading.version,
      }
      : { kind: 'unversioned', };
  } catch (error) {
    return {
      kind: 'unreadable-version',
      reason: caughtValueText(requireArtifactParseRefusal({ error, },),),
    };
  }
}

/**
 Parses one artifact file's text, or classifies it malformed where it is not
 JSON.

 A file that is not JSON is CLASSIFIED rather than skipped or thrown out of
 the census. The pipeline guard refuses it before this runs, so reaching
 here means somebody called the census alone, and answering a question about
 a directory by ignoring part of the directory is how a guard reports that
 everything is fine.

 @param text - artifact text as it sits on disk

 @returns Parsed value, or the malformed classification with the
 parser's reason

 @throws Whatever the parse raised that is not a syntax refusal, which a
 parse of a string never does

 @example
 ```ts
 const parsed = parsedArtifactText({ text, },);
 ```
 */
function parsedArtifactText(
  { text, }: { readonly text: string; },
): {
  /**
   The text is JSON.
   */
  readonly kind: 'parsed';

  /**
   The value it holds, of any shape.
   */
  readonly artifact: unknown;
} | Extract<SchemaClassification, { readonly kind: 'malformed'; }> {
  try {
    return {
      kind: 'parsed',
      artifact: JSON.parse(text,),
    };
  } catch (error) {
    return {
      kind: 'malformed',
      reason: caughtValueText(requireJsonSyntaxRefusal({ error, },),),
    };
  }
}

/**
 Classifies every settled entry in a directory.

 @param artifactsDir - directory holding one JSON per settled entry

 @returns One row per artifact, in directory-sorted order so a refusal reads
 the same twice

 @example
 ```ts
 const rows = await censusBySchema({ artifactsDir, },);
 ```
 */
export async function censusBySchema(
  { artifactsDir, }: { readonly artifactsDir: string; },
): Promise<readonly SchemaCensusRow[]> {
  /**
   Artifact names, sorted.
   */
  const names = textsInCodePointOrder({ texts: (await listArtifactFiles({ artifactsDir, },)), },);

  return allInInputOrder({
    members: names.map(async function readOne(name,): Promise<SchemaCensusRow> {
      /**
       Entry id, which is the file name without its suffix.
       */
      const entryId = entryIdOfArtifact({ name, },);

      /**
       Artifact text as it sits on disk.
       */
      const text = await readFile(
        join(
          artifactsDir,
          name,
        ),
        'utf8',
      );

      /**
       The file's value, or its malformed classification.
       */
      const parsed = parsedArtifactText({ text, },);

      return {
        entryId,
        classification: (parsed.kind === 'malformed')
          ? parsed
          : classifyArtifact({
            artifact: parsed.artifact,
            entryId,
          },),
      };
    },),
  },);
}

//endregion Pass schema census
