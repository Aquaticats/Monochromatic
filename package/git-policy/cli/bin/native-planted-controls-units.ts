/**
 How `native:mutation:parallel` tells a campaign unit that another unit runs the planted controls of its image.
 */

/**
 Environment variable through which `native:mutation:parallel` tells a campaign unit that another unit runs the planted
 controls of its image beside it: comma-separated `<full image ID>@<unit name>` entries.
 */
export const plantedUnitsEnvironmentVariable = 'NATIVE_MUTATION_PLANTED_UNITS';

/**
 Write the value of {@link plantedUnitsEnvironmentVariable}.

 @param entries - the image each controls unit runs the controls of, and the unit

 @returns the comma-separated entries

 @example
 ```ts
 plantedUnitsText([{ image: 'a'.repeat(64), unit: 'cli-git-mutpar-w5-planted-aaaaaaaaaaaa' }]);
 ```
 */
export function plantedUnitsText(entries: readonly {
  readonly image: string;
  readonly unit: string;
}[]): string {
  return entries.map(function entryText(entry) {
    return `${entry.image}@${entry.unit}`;
  })
    .join(',');
}

/**
 Find the unit that runs the planted controls of an image, in the value of {@link plantedUnitsEnvironmentVariable}.

 @param image - full ID of the image the campaign runs over

 @param text - the variable's value, empty when it is not set

 @returns the unit as the only element, or no element when the text names none for this image

 @example
 ```ts
 plantedUnitFor({ image, text: process.env.NATIVE_MUTATION_PLANTED_UNITS ?? '' });
 ```
 */
export function plantedUnitFor({
  image,
  text,
}: {
  readonly image: string;
  readonly text: string;
}): readonly string[] {
  return text.split(',')
    .flatMap(function unitOfEntry(entry) {
      /**
       Position of the separator between the image ID and the unit name.
       */
      const at = entry.indexOf('@');
      return ((at > 0) && (entry.slice(
        0,
        at,
      ) === image)
        && (entry.length > (at
          + 1))) ? [entry.slice(at + 1)] : [];
    })
    .slice(
    0,
    1,
    );
}
