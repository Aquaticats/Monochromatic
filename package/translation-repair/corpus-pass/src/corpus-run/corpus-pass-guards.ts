import type { PipelineDigest, } from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  assertArtifactsPlaceable,
  assertBuildGenerationResumable,
  assertResumableSchemaGeneration,
  type UnplaceableArtifactError,
  type LegacyPipelineError,
  type SchemaGenerationError,
  type MislabelledArtifactError,
  type GenerationDriftError,
} from '@monochromatic-dev/translation-repair-slice-cache/ts';

//region Corpus pass guards
// The refusals a pass makes about the artifacts already in its runs directory,
// before it settles anything.
//
// EACH GUARD'S REFUSAL IS A STATED REFUSAL BY CLASS (`pass-generation-guard.ts`,
// `pass-schema-guard.ts`), since each tells the operator what to do next and
// breaks nothing. While the five were plain errors the command reported each as
// a fault in itself, at exit 5, under frames. Nothing is caught here: a guard's
// refusal reaches the command's boundary as the class that raised it, and a
// read of the directory that failed passes on as it was.

/**
 Refuses a resume that would add a second build, or an artifact nothing can
 place, to a pool the runs directory already holds.

 A resume builds again, so if anything that runs changed since the entries
 already here were written, continuing would stamp a second pipeline into one
 pool and every reader that computes a rate would then refuse the lot.

 THE THREE REFUSALS RUN IN ORDER OF HOW LITTLE CHOICE THE OPERATOR HAS.
 First an artifact nothing can place, which no opt-in is an opinion about.
 Then the SHAPE, which no commit can reconcile. Only then the BUILD, whose
 refusal is overridable and whose message says so; running that one first
 offered an operator an opt-in that the shape check then refused anyway, so
 the advice was a lie and the second run logged a resume that never
 happened.

 @param artifactsDir - directory holding one JSON per settled entry

 @param pipelineDigest - built pipeline this invocation would stamp on everything it settles

 @param driftAllowed - whether the launch asked for a mixed directory

 @throws {@link UnplaceableArtifactError} when an artifact
 records nothing usable

 @throws {@link LegacyPipelineError} when artifacts predate
 generation identity

 @throws {@link SchemaGenerationError} when an artifact is of
 another schema generation

 @throws {@link MislabelledArtifactError} when an artifact declares
 this generation and fails its reader

 @throws {@link GenerationDriftError} when entries of another
 build are here and drift was not asked for

 @throws whatever a guard's read of the directory failed with

 @example
 ```ts
 await assertPassResumable({ artifactsDir, pipelineDigest, driftAllowed: false, },);
 ```
 */
export async function assertPassResumable(
  {
    artifactsDir,
    pipelineDigest,
    driftAllowed,
  }: {
    readonly artifactsDir: string;
    readonly pipelineDigest: PipelineDigest;
    readonly driftAllowed: boolean;
  },
): Promise<void> {
  /**
   What every placeable artifact records, read once for both guards.
   */
  const generationCensus = await assertArtifactsPlaceable({ artifactsDir, },);
  await assertResumableSchemaGeneration({ artifactsDir, },);
  assertBuildGenerationResumable({
    census: generationCensus,
    digest: pipelineDigest,
    driftAllowed,
  },);
}

//endregion Corpus pass guards
