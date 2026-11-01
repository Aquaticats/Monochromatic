import type {
  ChunkPair,
  PairedReading,
} from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { photoReferences, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Visual evidence completeness

/**
 Operational pause when image-dependent slice lacks reviewed visual evidence.

 @example
 ```ts
 throw new VisualEvidenceInterruptedError({ unavailableCount: 1, });
 ```
 */
export class VisualEvidenceInterruptedError extends Error {
  /**
   Declares message safe because it carries count only.
   */
  readonly messageNamesOnly: true = true;

  /**
   Constructs unavailable visual evidence diagnostic.

   @param unavailableCount - referenced assets without usable reviewed outcome

   @example
   ```ts
   new VisualEvidenceInterruptedError({ unavailableCount, });
   ```
   */
  public constructor(
    { unavailableCount, }: { readonly unavailableCount: number; },
  ) {
    super(`visual evidence incomplete for ${String(unavailableCount,)} referenced assets`,);
    this.name = 'VisualEvidenceInterruptedError';
  }
}

/**
 Requires every source-referenced asset to be corroborated or confirmed no-text.

 @param slices - prepared source slices naming visual assets

 @param readings - reviewed paired outcomes by asset name

 @throws {@link VisualEvidenceInterruptedError} when any asset is absent or unavailable

 @example
 ```ts
 assertVisualEvidenceComplete({ slices, readings, });
 ```
 */
export function assertVisualEvidenceComplete(
  {
    slices,
    readings,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly readings: ReadonlyMap<string, PairedReading>;
  },
): void {
  /**
   Referenced asset names in source document.
   */
  const names = new Set(slices.flatMap(function namesInSlice(slice,) {
    /**
     Source text whose visual references require evidence.
     */
    const { text, } = slice.source;
    return photoReferences({ text, })
      .map(function name(reference,): string {
        return reference.assetName;
      },);
  },),);
  /**
   References without usable reviewed visual outcome.
   */
  const unavailable = [...names,]
    .filter(function unavailableReading(name,): boolean {
      /**
       Paired visual evidence when reading stage produced one.
       */
      const reading = readings.get(name,);
      return (reading === undefined) || (reading.kind === 'unavailable');
    },);
  if (unavailable.length === 0)
    return;
  throw new VisualEvidenceInterruptedError({
    unavailableCount: unavailable.length,
  },);
}

//endregion Visual evidence completeness
