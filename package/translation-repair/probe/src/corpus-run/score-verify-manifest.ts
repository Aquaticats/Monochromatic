import {
  requireArray,
  requireRecord,
  requireString,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import { readRunJson, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Score verify manifest
// The manifest `probe-verify.ts` writes beside the verification sheet, which
// names what the sheet deliberately hides: the entry and the set each position
// came from.

/**
 One manifest row, naming what the sheet deliberately hides.

 @example
 ```ts
 const row: VerifyManifestItem = { position: 1, entryId: 'Whiskers', kind: 'control', };
 ```
 */
export type VerifyManifestItem = {
  /**
   One-based position, matching the sheet.
   */
  readonly position: number;

  /**
   Corpus entry the region belongs to.
   */
  readonly entryId: string;

  /**
   Which set the region came from.
   */
  readonly kind: string;
};

/**
 Reads the rows of a manifest already parsed from JSON.

 @param value - parsed manifest

 @returns Rows in sheet order

 @throws {@link ArtifactParseError} when a field is malformed

 @throws {@link StatedRefusalError} when a row's position is not its place in the file

 @example
 ```ts
 const rows = parseVerifyManifest({ value: JSON.parse(text,), },);
 ```
 */
export function parseVerifyManifest({ value, }: { readonly value: unknown; },): readonly VerifyManifestItem[] {
  /**
   Manifest as a record.
   */
  const manifest = requireRecord({
    value,
    path: 'verify manifest',
  },);

  return requireArray({
    value: manifest.items,
    path: 'verify manifest.items',
  },)
    .map(function toItem(
      item,
      index,
    ): VerifyManifestItem {
      /**
       Row as a record.
       */
      const row = requireRecord({
        value: item,
        path: 'verify manifest.items[]',
      },);

      /**
       Position as written, checked against its own index so a reordered
       manifest cannot mislabel every verdict silently.
       */
      const { position, } = row;
      if (position !== (index + 1)) {
        throw new StatedRefusalError({
          says: `verify manifest item ${String(index + 1,)} does not carry position ${String(index + 1,)}, so the file is `
            + 'not in sheet order and a positional join would mislabel every grade',
        },);
      }

      return {
        position: index + 1,
        entryId: requireString({
          value: row.entryId,
          path: 'verify manifest.items[].entryId',
        },),
        kind: requireString({
          value: row.kind,
          path: 'verify manifest.items[].kind',
        },),
      };
    },);
}

/**
 Reads the manifest written beside the sheet.

 @param path - manifest path

 @returns Rows in sheet order

 @throws {@link ArtifactParseError} when a field is malformed

 @example
 ```ts
 const rows = await readVerifyManifest({ path, },);
 ```
 */
export async function readVerifyManifest(
  { path, }: { readonly path: string; },
): Promise<readonly VerifyManifestItem[]> {
  return parseVerifyManifest({ value: await readRunJson({ path, },), },);
}

//endregion Score verify manifest
