/**
 How much room each distinct volume a new run cannot work without has, and the floor a new run needs on each: the volume of the
 repository and every build folder, the volume of the runs folder and the scratch folders beside it, and the volume that holds Pi's
 session files, the container images and the caches.

 The volume that holds the repository, every agent worktree and every build folder filled on 2026-11-11 at 13:1x (6.5 MB free of
 3.7 TB, from 111 GB at 02:40), and every Pi driver then exited at once with no summary. Measured that day: the pinned nightly's
 build folders held 372 GB for about 40 agent worktrees, so about 9 GB an agent. A new run starts only while
 {@link BUILD_VOLUME_FLOOR_BYTES} are free there, about eleven agents' builds, and the line says why it waits.

 The drive that holds Pi's session files, the container images and the caches filled on 2026-11-12: 1 MiB free of 1906 GiB at
 16:04:12, and twenty Pi runs ended between 15:56:45 and 15:59:48 with `stop=error`, every one on Pi's own write of its session
 file, `ENOSPC: no space left on device, write`. The scratch a run leaves is not on that drive, so the floor there is sized from
 what fills it: on 2026-11-13 one session's container images took about 20 GB of that drive in one hour (nine images of 2.15 to
 2.53 GB, `podman images`), so {@link SESSION_FILES_FLOOR_BYTES} is about two hours of that with nobody looking, and above the
 1 GiB at which twenty runs ended in four minutes.

 A run's own folders fill the volume that holds the runs folder, that folder and the scratch beside it, and the largest scratch
 folder of a single run of 2026-11-12 measured 16.7 GiB with `btrfs filesystem du --summarize --raw` on 2026-11-13, so
 {@link RUNS_FOLDER_FLOOR_BYTES} is about three such runs.

 Where two needs are on one volume it is read and named once, at the strictest of their floors: the room that runs out is the
 drive's, whatever each need would ask of it alone.

 @module
 */

/**
 The volume that holds the repository, every agent worktree and every build folder, whose room a new run needs.
 */
export const BUILD_VOLUME: string = '/var/mnt/encrypted';

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
export const BUILD_VOLUME_FLOOR_BYTES: number = 100 * BYTES_IN_A_GIBIBYTE;

/**
 The room a new run needs on the volume that holds Pi's session files, in whole gibibytes: about two hours of the container images
 one session took there on 2026-11-13, and well above the 1 GiB at which twenty runs ended on 2026-11-12. A run's scratch is not on
 that drive, so this floor is not sized from scratch, and the drive it guards is the one whose fill a run cannot see coming.
 */
const SESSION_FILES_FLOOR_GIBIBYTES = 50;

/**
 {@link SESSION_FILES_FLOOR_GIBIBYTES} in the bytes a room and a floor are counted in.
 */
export const SESSION_FILES_FLOOR_BYTES: number = SESSION_FILES_FLOOR_GIBIBYTES * BYTES_IN_A_GIBIBYTE;

/**
 The room a new run needs on the volume that holds the runs folder, in whole gibibytes: about three of the largest scratch folder
 one run of 2026-11-12 left, at the 16.7 GiB a btrfs reading of that folder gave on 2026-11-13. `du` read that folder as 65,984 MiB
 by counting the data its copy-on-write copies share once for each of them, so the floor follows the btrfs measurement and is not
 raised on a number that counted the same bytes many times.
 */
const RUNS_FOLDER_FLOOR_GIBIBYTES = 50;

/**
 {@link RUNS_FOLDER_FLOOR_GIBIBYTES} in the bytes a room and a floor are counted in.
 */
export const RUNS_FOLDER_FLOOR_BYTES: number = RUNS_FOLDER_FLOOR_GIBIBYTES * BYTES_IN_A_GIBIBYTE;

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
 One volume a new run fills, with the floor one of its needs sets: before the room is read, so a volume two needs share is read
 once.
 */
export type VolumeAgainstItsFloor = {
  /**
   The volume's own path, with every symbolic link to it resolved.
   */
  readonly volume: string;

  /**
   How many bytes a new run needs free on that volume.
   */
  readonly floorBytes: number;
};

/**
 One volume's room with the floor a new run needs on it, which is how a room is asked about and how a refusal says it.
 */
export type FreeSpaceAgainstItsFloor = {
  /**
   The room the volume has.
   */
  readonly space: FreeSpace;

  /**
   How many bytes a new run needs free on that volume.
   */
  readonly floorBytes: number;
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
 Reads one entry per distinct volume a new run needs, each with the strictest floor any of its needs sets, in the order the needs
 are given.

 A volume that two needs share is one drive, and the room that runs out on it is the drive's: keeping the lower floor would start a
 run into room another need has already ruled out, and keeping both entries would print and refuse the same volume twice.

 @param volumes - each volume a new run needs, one entry per need

 @returns each distinct volume once, with the strictest floor set for it

 @example
 ```ts
 const volumes = strictestFloorPerVolume({ volumes: [{ volume: BUILD_VOLUME, floorBytes: RUNS_FOLDER_FLOOR_BYTES, },], },);
 ```
 */
export function strictestFloorPerVolume({
  volumes,
}: {
  readonly volumes: readonly VolumeAgainstItsFloor[];
},): readonly VolumeAgainstItsFloor[] {
  /**
   The strictest floor read so far for each volume, in the order the needs named them.
   */
  const strictest = new Map<string, VolumeAgainstItsFloor>();
  for (const volume of volumes) {
    /**
     What is already known about this volume, when an earlier need was on it.
     */
    const held = strictest.get(volume.volume,);
    if ((held === undefined) || (volume.floorBytes > held.floorBytes)) {
      strictest.set(
        volume.volume,
        volume,
      );
    }
  }
  return [...strictest.values(),];
}

/**
 Reads a room in whole gibibytes, dropping the part of one that is left, which is how every line prints it.

 @param bytes - the room in bytes

 @returns the whole gibibytes the room holds

 @example
 ```ts
 const gibibytes = wholeGibibytesOf(BUILD_VOLUME_FLOOR_BYTES,);
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
 const line = freeSpaceTextOf({ volume: BUILD_VOLUME, freeBytes: BUILD_VOLUME_FLOOR_BYTES, },);
 ```
 */
export function freeSpaceTextOf({
  volume,
  freeBytes,
}: FreeSpace,): string {
  return `${String(wholeGibibytesOf(freeBytes,),)} GiB free on ${volume}`;
}

/**
 Says the floor a new run needs on one volume, as the refusal names it.

 @param floorBytes - how many bytes a new run needs free on the volume

 @returns the floor in whole gibibytes

 @example
 ```ts
 const floor = freeSpaceFloorTextOf(SESSION_FILES_FLOOR_BYTES,);
 ```
 */
export function freeSpaceFloorTextOf(floorBytes: number,): string {
  return `${String(wholeGibibytesOf(floorBytes,),)} GiB`;
}

/**
 Says a volume's room against the floor a new run needs on it, as the refusal names one volume it will not start on: the room,
 the volume, and the floor.

 @param space - the room the volume has

 @param floorBytes - how many bytes a new run needs free on that volume

 @returns the room, the volume and the floor, as one clause of a refusal

 @example
 ```ts
 const clause = freeSpaceUnderTheFloorTextOf({ space: freeSpace, floorBytes: SESSION_FILES_FLOOR_BYTES, },);
 ```
 */
export function freeSpaceUnderTheFloorTextOf({
  space,
  floorBytes,
}: FreeSpaceAgainstItsFloor,): string {
  return `${freeSpaceTextOf(space,)}, under the floor of ${freeSpaceFloorTextOf(floorBytes,)}`;
}

/**
 Says whether a volume holds less room than a new run needs on it.

 @param space - the room the volume has

 @param floorBytes - how many bytes a new run needs free on that volume

 @returns whether the room is below that floor

 @example
 ```ts
 const under = freeSpaceIsUnderTheFloor({ space: freeSpace, floorBytes: SESSION_FILES_FLOOR_BYTES, },);
 ```
 */
export function freeSpaceIsUnderTheFloor({
  space,
  floorBytes,
}: FreeSpaceAgainstItsFloor,): boolean {
  return space.freeBytes < floorBytes;
}
