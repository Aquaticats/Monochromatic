/**
 How much room the volume that holds the repository has, and the floor a new run needs before it starts.

 The volume that holds the repository, every agent worktree and every build folder filled on 2026-11-11 at 13:1x (6.5 MB free of
 3.7 TB, from 111 GB at 02:40), and every Pi driver then exited at once with no summary. Measured that day: the pinned nightly's
 build folders held 372 GB for about 40 agent worktrees, so about 9 GB an agent. A new run starts only while
 `FREE_SPACE_FLOOR_BYTES` are free, about eleven agents' builds, and the line says why it waits.

 That volume was the 860 drive at `/var/mnt/encrypted` until the 2026-11-17 move to CachyOS, which put the repository, its agent
 worktrees and their build folders in the home folder's volume.

 @module
 */

import { homedir, } from 'node:os';

/**
 The volume that holds the repository, every agent worktree and every build folder, whose room a new run needs: the home folder's
 volume, read through the home folder of whoever runs the command, because a path that does not exist makes `statfs` throw
 `ENOENT` and stops every run from starting. The value is read once, when the module loads, from `HOME`: a gate started under a
 disposable `HOME` measures that folder's volume instead, so the gate runs with the real one, and a test that gives it another
 `HOME` also gives it the room through `PI_RUNS_FREE_SPACE_BYTES`.
 */
export const BUILD_VOLUME: string = homedir();

/**
 How many bytes one kibibyte is, which is the unit the bigger ones are counted in.
 */
export const BYTES_IN_A_KIBIBYTE = 1_024;

/**
 How many bytes one gibibyte is, which is the unit the room and the floor are printed in.
 */
export const BYTES_IN_A_GIBIBYTE: number = BYTES_IN_A_KIBIBYTE
  * BYTES_IN_A_KIBIBYTE
  * BYTES_IN_A_KIBIBYTE;

/**
 The room a new run needs on {@link BUILD_VOLUME}: about eleven agent builds at the 9 GB an agent that the 2026-11-11 fill
 measured. The floor is what keeps every driver from exiting at once when the volume fills, so it is raised only with a measured
 need for more of them.
 */
export const FREE_SPACE_FLOOR_BYTES: number = 100 * BYTES_IN_A_GIBIBYTE;

/**
 How much room one volume has, and which volume that room is on.
 */
export type FreeSpace = {
  /**
   The volume whose room this is.
   */
  readonly volume: string;

  /**
   How many bytes a new file may still take on it.
   */
  readonly freeBytes: number;
};

/**
 What a volume answers about its room, which is the part of `statfs` this reads: the blocks a new file may take, and how many
 bytes a block is.
 */
export type FreeBlocksOfAVolume = (volume: string) => {
  readonly bavail: number;
  readonly bsize: number;
};

/**
 Reads how much room a volume has, in the bytes a new file may still take.

 @param volume - the volume to read the room of

 @param statfs - how the room is read, which a test replaces with a volume of its own

 @returns the room the volume has, and the volume it is on

 @example
 ```ts
 const freeSpace = freeSpaceOnVolume({ volume: BUILD_VOLUME, statfs: statfsSync, },);
 ```
 */
export function freeSpaceOnVolume({
  volume,
  statfs,
}: {
  readonly volume: string;
  readonly statfs: FreeBlocksOfAVolume;
},): FreeSpace {
  /**
   What the volume answered about its room.
   */
  const answered = statfs(volume,);
  return {
    volume,
    freeBytes: answered.bavail * answered.bsize,
  };
}

/**
 Reads a room in whole gibibytes, dropping the part of one that is left, which is how every line prints it.

 @param bytes - the room in bytes

 @returns the whole gibibytes the room holds

 @example
 ```ts
 const gibibytes = wholeGibibytesOf(FREE_SPACE_FLOOR_BYTES,);
 ```
 */
export function wholeGibibytesOf(bytes: number,): number {
  return Math.floor(bytes / BYTES_IN_A_GIBIBYTE,);
}

/**
 Says how much room a volume has, as the counts line and the refusal name it.

 @param volume - the volume the room is on

 @param freeBytes - the room in bytes

 @returns the room in whole gibibytes, naming the volume

 @example
 ```ts
 const line = freeSpaceTextOf({ volume: BUILD_VOLUME, freeBytes: FREE_SPACE_FLOOR_BYTES, },);
 ```
 */
export function freeSpaceTextOf({
  volume,
  freeBytes,
}: FreeSpace,): string {
  return `${String(wholeGibibytesOf(freeBytes,),)} GiB free on ${volume}`;
}

/**
 Says the floor a new run needs, as the refusal names it.

 @returns the floor in whole gibibytes

 @example
 ```ts
 const floor = freeSpaceFloorTextOf();
 ```
 */
export function freeSpaceFloorTextOf(): string {
  return `${String(wholeGibibytesOf(FREE_SPACE_FLOOR_BYTES,),)} GiB`;
}

/**
 Says whether a volume holds less room than a new run needs.

 @param freeBytes - the room in bytes

 @returns whether the room is below {@link FREE_SPACE_FLOOR_BYTES}

 @example
 ```ts
 const under = freeSpaceIsUnderTheFloor({ freeBytes: FREE_SPACE_FLOOR_BYTES - 1, },);
 ```
 */
export function freeSpaceIsUnderTheFloor({
  freeBytes,
}: Pick<FreeSpace, 'freeBytes'>,): boolean {
  return freeBytes < FREE_SPACE_FLOOR_BYTES;
}
