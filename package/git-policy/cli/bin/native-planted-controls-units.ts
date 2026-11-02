/**
 How `native:mutation:parallel` tells a campaign unit that another unit runs the planted controls of its image.
 */

/**
 Environment variable through which `native:mutation:parallel` tells a campaign unit that another unit runs the planted
 controls of its image beside it: a JSON list of `{ image, unit, log }` entries.
 */
export const plantedUnitsEnvironmentVariable = 'NATIVE_MUTATION_PLANTED_UNITS';

/**
 A controls unit, the full image ID it runs the controls of, its log, and the systemd invocation the launcher started, so a
 later unit that reuses the name is not taken for it.
 */
export type PlantedUnit = {
  readonly image: string;
  readonly unit: string;
  readonly log: string;
  readonly invocation: string;
};

/**
 Write the value of {@link plantedUnitsEnvironmentVariable}.

 @param entries - the controls units that started

 @returns the JSON text

 @example
 ```ts
 plantedUnitsText([{ image: 'a'.repeat(64), unit: 'cli-git-mutpar-w5-planted-aaaaaaaaaaaa', log: '/l/x.log.1', invocation: 'abc' }]);
 ```
 */
export function plantedUnitsText(entries: readonly PlantedUnit[]): string {
  return JSON.stringify(entries);
}

/**
 Check that a parsed value is a controls unit entry.

 @param value - parsed JSON

 @returns true for an object whose `image`, `unit`, `log` and `invocation` are non-empty texts

 @example
 ```ts
 isPlantedUnit({ image: 'a', unit: 'u', log: 'l' });
 ```
 */
function isPlantedUnit(value: unknown): value is PlantedUnit {
  if ((value === null) || ((typeof value) !== 'object'))
    return false;
  /**
   The fields, each of unknown type.
   */
  const {
    image,
    unit,
    log,
    invocation,
  } = value as {
    readonly image?: unknown;
    readonly unit?: unknown;
    readonly log?: unknown;
    readonly invocation?: unknown
  };
  return ((typeof image) === 'string') && (image !== '')
    && ((typeof unit) === 'string')
    && (unit !== '')
    && ((typeof log) === 'string')
    && (log !== '')
    && ((typeof invocation) === 'string')
    && (invocation !== '');
}

/**
 Parse the variable's JSON without throwing.

 @param text - the variable's value

 @returns the entries that are controls unit entries, none when the text is not a JSON list

 @example
 ```ts
 plantedUnitsOf('[]');
 ```
 */
function plantedUnitsOf(text: string): readonly PlantedUnit[] {
  try {
    /**
     The value the text holds.
     */
    const value: unknown = JSON.parse(text);
    return Array.isArray(value) ? value.filter(isPlantedUnit) : [];
  }
  catch (error) {
    console.error(`${plantedUnitsEnvironmentVariable} is not JSON, so no controls unit is used: ${String(error)}`);
    return [];
  }
}

/**
 Find the controls unit of an image, in the value of {@link plantedUnitsEnvironmentVariable}.

 @param image - full ID of the image the campaign runs over

 @param text - the variable's value, empty when it is not set

 @returns the unit as the only element, or no element when the text is not the expected JSON or names none for this image

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
}): readonly PlantedUnit[] {
  return text === ''
    ? []
    : plantedUnitsOf(text)
      .filter(function isOfImage(entry) {
        return entry.image === image;
      })
      .slice(
      0,
      1,
      );
}
