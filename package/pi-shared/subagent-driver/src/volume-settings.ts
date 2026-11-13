/**
 The two settings a caller gives instead of letting the gate read this machine's disks and symbolic links: each volume's room, and
 which volume each folder is on.

 They exist so that a test of the gate reads no disk of the machine it runs on and depends on no link of it, as
 `PI_RUNS_PROCESS_FOLDER` does for the process table. A caller starting real runs sets neither, and either can only make the gate
 start a run it would otherwise refuse, never refuse one it would start.

 @module
 */

import {
  LARGEST_EXACTLY_HELD_WHOLE_NUMBER,
  wholeNumberIn,
} from './whole-number-setting.ts';

/**
 The variable that gives a volume's room in bytes instead of reading it, as `<volume>=<bytes>` pairs separated by commas.

 A volume is named by the path the gate would otherwise find it under: `BUILD_VOLUME` for the volume of the repository and the
 builds, and the volume's own path for a folder the gate resolved.
 */
export const VOLUME_FREE_SPACE_BYTES_VARIABLE = 'PI_RUNS_VOLUME_FREE_SPACE_BYTES';

/**
 The variable that says which volume a folder is on instead of resolving the folder's symbolic links and devices, as
 `<folder>=<volume>` pairs separated by commas.
 */
export const FOLDER_VOLUMES_VARIABLE = 'PI_RUNS_FOLDER_VOLUMES';

/**
 A volume setting this command cannot use: a pair that is not `<path>=<whole number>` or `<folder>=<volume>`.
 */
export class VolumeSettingsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'VolumeSettingsError';
}

/**
 Reads the pairs of a setting, each written as `<left>=<right>`, and refuses one that is not.

 A leading or trailing comma and empty spaces around a pair are ignored, so a setting written by hand is read as it reads. A pair
 without an `=` is refused rather than dropped: a mistyped pair would otherwise leave a volume or a folder unset, and the gate
 would read the machine's disk and links while the caller believed it had given them.

 @param listed - the variable's value

 @param variable - the variable's name, which the refusal names

 @returns the right side of each pair, by its left side

 @throws VolumeSettingsError when a pair is not `<left>=<right>`

 @example
 ```ts
 const pairs = pairsIn({ listed: '/a-volume=1024', variable: VOLUME_FREE_SPACE_BYTES_VARIABLE, },);
 ```
 */
function pairsIn({
  listed,
  variable,
}: {
  readonly listed: string;
  readonly variable: string;
}): ReadonlyMap<string, string> {
  /**
   The pairs read so far, by the path on their left.
   */
  const pairs = new Map<string, string>();
  for (const pair of listed.split(',',)) {
    if (pair.trim() === '') {
      continue;
    }
    /**
     Where the left path ends.
     */
    const split = pair.indexOf('=',);
    /**
     The path on the left, as the caller wrote it.
     */
    const left = (split === (-1)) ? '' : pair.slice(
      0,
      split,
    )
      .trim();
    /**
     What the pair gives that path, as the caller wrote it.
     */
    const right = (split === (-1)) ? '' : pair.slice(split + 1,)
      .trim();
    if ((left === '') || (right === '')) {
      throw new VolumeSettingsError(`${variable} holds "${pair}", which is no <path>=<value> pair.`,);
    }
    pairs.set(
      left,
      right,
    );
  }
  return pairs;
}

/**
 Reads the room each volume was given, from the value of {@link VOLUME_FREE_SPACE_BYTES_VARIABLE}.

 @param listed - the variable's value, empty when the caller set nothing

 @returns the room in bytes each volume was given, by the volume's path

 @throws VolumeSettingsError when a pair is not `<volume>=<whole number>`

 @example
 ```ts
 const rooms = givenVolumeFreeSpaceBytes({ listed: '/var/mnt/encrypted=107374182400', },);
 ```
 */
export function givenVolumeFreeSpaceBytes({ listed, }: { readonly listed: string },): ReadonlyMap<string, number> {
  /**
   The room each volume was given, by the volume's path.
   */
  const rows = new Map<string, number>();
  for (const [volume, bytes,] of pairsIn({
    listed,
    variable: VOLUME_FREE_SPACE_BYTES_VARIABLE,
  },)) {
    /**
     What this room was read as, or why it could not be read.
     */
    const reading = wholeNumberIn(bytes,);
    if (!reading.read) {
      if (reading.problem === 'too-large-to-hold-exactly') {
        throw new VolumeSettingsError(
          `${VOLUME_FREE_SPACE_BYTES_VARIABLE} holds "${volume}=${bytes}", which gives that volume a whole number of bytes too large for a number to hold exactly, so no floor would ever see it full. Set it to at most ${String(LARGEST_EXACTLY_HELD_WHOLE_NUMBER,)} bytes.`,
        );
      }
      throw new VolumeSettingsError(
        `${VOLUME_FREE_SPACE_BYTES_VARIABLE} holds "${volume}=${bytes}", which gives that volume no whole number of bytes.`,
      );
    }
    rows.set(
      volume,
      reading.value,
    );
  }
  return rows;
}

/**
 Reads which volume each folder was given, from the value of {@link FOLDER_VOLUMES_VARIABLE}.

 @param listed - the variable's value, empty when the caller set nothing

 @returns the volume each folder was given, by the folder's path

 @throws VolumeSettingsError when a pair is not `<folder>=<volume>`

 @example
 ```ts
 const volumes = givenFolderVolumes({ listed: '/home/a/temp/agent/pi-runs=/var/mnt/encrypted', },);
 ```
 */
export function givenFolderVolumes({ listed, }: { readonly listed: string },): ReadonlyMap<string, string> {
  return pairsIn({
    listed,
    variable: FOLDER_VOLUMES_VARIABLE,
  },);
}
