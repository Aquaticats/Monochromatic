import { isJsonRecord, } from '../../dist/final/node/index.mjs';

//region Generation restamp
// A SETTLED ARTIFACT RESTAMPED AS GENERATION FOURTEEN, for the replay cases
// that pin what a run of the generation before fifteen saw. The writer stamps
// generation fifteen, which carries the consistency section; generation
// fourteen's reader refuses that section, so the restamp drops it with the
// stamp rather than leaving each case to remember it.

/**
 Restamps a settled artifact's JSON text as generation fourteen, without the
 consistency section that generation does not carry.

 @param text - artifact JSON as the writer wrote it

 @returns The restamped artifact's JSON text

 @throws {@link Error} when the text is not a JSON object, which no artifact writer writes

 @example
 ```ts
 const older = restampedAsGenerationFourteen({ text: await readFile(artifactPath, 'utf8',), },);
 ```
 */
export function restampedAsGenerationFourteen({ text, }: { readonly text: string; },): string {
  /**
   The artifact as written.
   */
  const written: unknown = JSON.parse(text,);
  if (!isJsonRecord(written,))
    throw new Error('restampedAsGenerationFourteen: the artifact text is not a JSON object',);
  /**
   The artifact apart from the section generation fourteen lacks.
   */
  const {
    consistency: ignored,
    ...older
  } = written;
  return JSON.stringify({
    ...older,
    artifactSchemaVersion: 14,
  },);
}

//endregion Generation restamp
