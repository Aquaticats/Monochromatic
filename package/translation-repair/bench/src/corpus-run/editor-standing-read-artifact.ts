import { refusalText, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  EDITOR_ROUND_STAGES,
  REFINER_ROUND_STAGES,
  selectionRoundsFor,
} from '@monochromatic-dev/translation-repair-assembly/ts';
import { readRunJson, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { SelectionRound, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import {
  OffRosterModelError,
  parseSettledTwoLaneArtifact,
  readRepairRounds,
  RoundsNotRecordedError,
} from '@monochromatic-dev/translation-repair-artifact/ts';

//region Editor standing read artifact
// WHAT ONE ARTIFACT TURNED OUT TO BE, for `editor-standing-read`: the rounds
// each of its seats produced, or which of three named absences it was.
//
// SAYS IDS, PATHS AND MODEL IDS. Never a passage, and never a parse-refusal
// message that quotes the text it disagrees about.
//
// RETURNS WHAT IT HAS TO SAY rather than printing it, since the artifacts are
// read side by side: until 2026-10-10 each read printed its own line, so the
// lines came out in whichever order the reads ended.

/**
 One artifact's rounds, kept grouped by the chunk that produced them.

 @example
 ```ts
 const reading: StandingReading = { digest, entryId: 'Mittens', editor: [[],], refiner: [[],], };
 ```
 */
export type StandingReading = {
  /**
   Built output that produced it, which is what may be pooled.
   */
  readonly digest: string;

  /**
   Corpus entry, so entries can be counted rather than inferred.
   */
  readonly entryId: string;

  /**
   Editor rounds, one list per chunk, empty where a chunk had nothing to edit.
   */
  readonly editor: readonly (readonly SelectionRound[])[];

  /**
   Refiner rounds, grouped the same way.
   */
  readonly refiner: readonly (readonly SelectionRound[])[];
};

/**
 What one artifact turned out to be.

 TWO OF THE THREE NON-READINGS ARE THEIR OWN ANSWER, not a refusal.

 `off-roster` means the artifact names models the roster no longer holds,
 which says the record predates the current seating and nothing bad about the
 record.

 `earlier-schema` means its repair result carries no `chunks` at all, because
 the lane began recording rounds only in a later build. Found by running this
 over the archives: 22 of 41 artifacts were that, every one recording
 `status: repaired`.

 Folding either in with malformed artifacts would report a healthy archive as
 a broken one, and this reader exists to say how much evidence there is.

 @example
 ```ts
 const outcome: StandingArtifactOutcome = { kind: 'earlier-schema', said: [], };
 ```
 */
export type StandingArtifactOutcome =
  | {
    /**
     The artifact parsed under the current roster.
     */
    readonly kind: 'read';

    /**
     Its rounds, by seat and chunk.
     */
    readonly reading: StandingReading;

    /**
     Lines for stderr, none for a reading.
     */
    readonly said: readonly string[];
  }
    | {
    /**
     Named absences: settled under an earlier roster, settled before the lane
     recorded rounds, or refused by the parse.
     */
      readonly kind: 'off-roster' | 'earlier-schema' | 'refused';

    /**
     Lines for stderr: the model the roster dropped, or why the parse refused,
     and none for an artifact settled before the rounds were recorded, which
     the summary counts instead.
     */
      readonly said: readonly string[];
    };

/**
 Reads one artifact into the rounds each of its seats produced.

 @param path - artifact file to read

 @returns Its rounds, that it predates the roster or the rounds' schema, or
 that it would not parse, with a line for stderr for the first and the last

 @example
 ```ts
 const outcome = await readStandingArtifact({ path, },);
 ```
 */
export async function readStandingArtifact(
  { path, }: { readonly path: string; },
): Promise<StandingArtifactOutcome> {
  try {
    /**
     Whole artifact, parsed rather than trusted.
     */
    const artifact = parseSettledTwoLaneArtifact({
      value: await readRunJson({ path, },),
    },);

    /**
     Rounds every chunk recorded, checked here because version 2 hands the
     lane result back unread.
     */
    const perChunk = readRepairRounds({
      raw: artifact
        .lanes
        .repair
        .raw,
      path: `${artifact.id}.lanes.repair.result`,
    },);

    return {
      kind: 'read',
      said: [],
      reading: {
        digest: artifact.pipelineDigest,
        entryId: artifact.id,
        editor: perChunk.map(function editorRounds(rounds,): readonly SelectionRound[] {
          return selectionRoundsFor({
            rounds,
            stages: EDITOR_ROUND_STAGES,
          },);
        },),
        refiner: perChunk.map(function refinerRounds(rounds,): readonly SelectionRound[] {
          return selectionRoundsFor({
            rounds,
            stages: REFINER_ROUND_STAGES,
          },);
        },),
      },
    };
  } catch (error) {
    // BOTH MESSAGES ARE SAFE TO PRINT. `ArtifactParseError` names a path and
    // the shape it wanted; `OffRosterModelError` names a path and a model id.
    // Neither quotes the value it disagrees about, so no passage can reach the
    // report through them.
    if (error instanceof OffRosterModelError) {
      return {
        kind: 'off-roster',
        said: [`editor-standing-read: ${error.message}`,],
      };
    }

    // COUNTED, NOT PRINTED PER ARTIFACT. On the archives this is the commonest
    // outcome by far, and a line each would bury the report it precedes.
    if (error instanceof RoundsNotRecordedError) {
      return {
        kind: 'earlier-schema',
        said: [],
      };
    }

    return {
      kind: 'refused',
      said: [`editor-standing-read: ${path} refused, ${refusalText({ error, },)}`,],
    };
  }
}

//endregion Editor standing read artifact
