import {
  ArtifactParseError,
  requireRecord,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  readArtifactSchemaVersion,
  ARTIFACT_SCHEMA_VERSION_V1,
} from './artifact-schema-version.ts';
import {
  type ParsedArtifact,
  parseSettledArtifact,
} from './artifact-v1-read.ts';
import { TWO_LANE_GENERATIONS, } from './corpus-run/artifact-two-lane-contract.ts';
import type { ParsedTwoLaneArtifact, } from './corpus-run/artifact-two-lane-read-contract.ts';
import { parseSettledTwoLaneArtifact, } from './corpus-run/artifact-two-lane-read.ts';

//region Artifact dispatch
// Choosing which generation's reader an artifact belongs to, and nothing else.
//
// EVERY GENERATION GETS ITS OWN READER, and this file holds none of them. A
// single reader that tried to cover all three would have to decide, per field,
// whether a missing value means an older file or a malformed one, which is the
// reading that made version 1's optional fields ambiguous in the first place.
//
// THE RESULT IS DISCRIMINATED BY GENERATION rather than merged into one shape.
// Version 1 answers with one status, one output and one issue list; version 2
// has two lanes, no singular anything, and a comparison. Flattening the second
// into the first would pick a lane, which is the question nobody has decided,
// and a caller that cannot say which generation it is holding cannot ask a
// question either one can answer.

/**
 One artifact, read by whichever generation's reader owns it.

 @example
 ```ts
 const reading: ParsedArtifactReading = readSettledArtifact({ value, },);
 ```
 */
export type ParsedArtifactReading = {
  /**
   Artifact predates the schema version field, so its generation is legible
   only from which fields it carries.
   */
  readonly kind: 'legacy';

  /**
   What the version 1 reader made of it.
   */
  readonly artifact: ParsedArtifact;
} | {
  /**
   Artifact states version 1.
   */
  readonly kind: 'version-1';

  /**
   What the version 1 reader made of it.
   */
  readonly artifact: ParsedArtifact;
} | {
  /**
   Artifact states one of the two-lane generations, which is every version
   from 2 onwards. The kind names the SHAPE rather than the integer, since
   three versions record it and differ only in how four keys are spelled.
   */
  readonly kind: 'version-2';

  /**
   What the two-lane reader made of it, comparison recomputed.
   */
  readonly artifact: ParsedTwoLaneArtifact;
};

/**
 Reads one artifact of any generation this reader understands.

 ACCEPTS AN EXPLICIT VERSION 1, which is not the same as there being version 1
 artifacts on disk: a reader that understands a generation should read it, and
 how many files of it exist is a fact about one corpus rather than about the
 format. An unknown version is refused by the version reading itself.

 @param value - artifact JSON, freshly parsed and still untyped

 @returns Which generation it is, and what that generation's reader made of it

 @throws {@link ArtifactParseError} when the value is not a record, when it
 names a generation this reader does not know, or when the chosen reader
 refuses it

 @throws Error when a generation this build knows has no reader here, a
 fault of the build rather than of the file

 @example
 ```ts
 const reading = readSettledArtifact({ value: parseRunJson({ text, from, },), },);
 ```
 */
export function readSettledArtifact(
  { value, }: { readonly value: unknown; },
): ParsedArtifactReading {
  /**
   Artifact as a record, read once here so the version can be looked at
   before any generation's reader runs.
   */
  const artifact = requireRecord({
    value,
    path: 'artifact',
  },);

  /**
   Which generation it states, or that it states none.
   */
  const reading = readArtifactSchemaVersion({
    artifact,
    path: 'artifact',
  },);
  if (reading.kind === 'unversioned') {
    return {
      kind: 'legacy',
      artifact: parseSettledArtifact({ value, },),
    };
  }
  if (reading.version === ARTIFACT_SCHEMA_VERSION_V1) {
    return {
      kind: 'version-1',
      artifact: parseSettledArtifact({ value, },),
    };
  }
  // EVERY GENERATION OF THE TWO-LANE SHAPE, from the list the reader itself
  // accepts, rather than one integer. Naming version 2 alone is what left
  // generation 3 unreadable here for as long as it existed: the reader had
  // learned it and this had not.
  if (TWO_LANE_GENERATIONS.includes(reading.version,)) {
    return {
      kind: 'version-2',
      artifact: parseSettledTwoLaneArtifact({ value, },),
    };
  }

  // REACHED ONLY when a generation joins `KNOWN_ARTIFACT_SCHEMA_VERSIONS` and
  // no reader claims it, which `artifact-dispatch.unit.test.ts` fails on:
  // returning a reading for the wrong generation is worse than refusing. A
  // fault of this build rather than of the file, so it is not an artifact
  // refusal, which would send an operator to archive a sound run (ledger T8).
  throw new Error(
    `unreachable: version ${
      String(reading.version,)
    } is known to this build and neither the version 1 reader nor the two-lane reader reads it`,
  );
}

// RE-EXPORTED under the names their callers already use. The version 1 reader
// moved to a file naming its generation; every consumer of it wanted exactly
// what it does, and a rename would have been churn on top of a move.
export {
  type ParsedAcceptedIssue,
  type ParsedArtifact,
  parseSettledArtifact,
} from './artifact-v1-read.ts';

//endregion Artifact dispatch
