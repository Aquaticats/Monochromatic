/**
 The volume a folder is on, found from the folder itself: its real path with every symbolic link resolved, walked up to the folder
 whose parent is on another device.

 A gate that named a path written into the code would name the wrong drive on a machine where that path is a link to another one,
 which is how a full drive on 2026-11-12 filled while every floor read free: the runs folder was reached through a link onto the
 build volume, and the drive that failed held Pi's own files.

 @module
 */

import { dirname, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import {
  type FreeBlocksOfAVolume,
  type FreeSpace,
  freeSpaceOnVolume,
} from './free-space-floor.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 What a folder answers about itself, which is what finds the volume it is on: the path it really is, with every symbolic link
 resolved, and the device whose volume holds it. Two folders whose answers carry the same device are on one volume, and the
 folder at the top of that run of folders is the volume's own path.
 */
export type AnswerAboutAFolder = (folder: string) => Promise<{
  readonly realPath: string;
  readonly device: number;
}>;

/**
 Reads what the nearest folder at or above one answers, for a folder that has not been made yet.

 A folder that is not there is on the volume of the nearest folder above it that is: nothing can be mounted below a folder that
 does not exist, so the write that makes it lands on that volume.

 @param folder - the folder to read, or the nearest one above it that is there

 @param answerAbout - what a folder answers about itself, which a test replaces

 @returns what that folder answered

 @throws Error when the folder cannot be read for any reason other than not being there, or when no folder on the way up is
 there, which is a path this command cannot place at all

 @example
 ```ts
 const answer = await nearestFolderThatIsThere({ folder: '/runs/pi-runs', answerAbout, },);
 ```
 */
async function nearestFolderThatIsThere({
  folder,
  answerAbout,
}: {
  readonly folder: string;
  readonly answerAbout: AnswerAboutAFolder;
},): Promise<Awaited<ReturnType<AnswerAboutAFolder>>> {
  try {
    return await answerAbout(folder,);
  }
  catch (error) {
    if (errorCodeOf(error,) !== MISSING_PATH_CODE) {
      throw error;
    }
    /**
     The folder that holds the one that is not there.
     */
    const parent = dirname(folder,);
    if (parent === folder) {
      throw error;
    }
    return nearestFolderThatIsThere({
      folder: parent,
      answerAbout,
    },);
  }
}

/**
 Reads the folder at the top of the run of folders on one device, which is the volume's own path.

 @param volume - the folder whose volume is read, starting at the folder the caller named having every symbolic link resolved

 @param device - the device each folder of that volume answers with

 @param answerAbout - what a folder answers about itself, which a test replaces

 @returns the volume's own path

 @example
 ```ts
 const volume = await volumeAtTheTopOf({ volume: '/var/home/a/temp', device: 2, answerAbout, },);
 ```
 */
async function volumeAtTheTopOf({
  volume,
  device,
  answerAbout,
}: {
  readonly volume: string;
  readonly device: number;
  readonly answerAbout: AnswerAboutAFolder;
},): Promise<string> {
  /**
   The folder that holds this one.
   */
  const parent = dirname(volume,);
  if (parent === volume) {
    return volume;
  }
  /**
   What the folder above answers about itself.
   */
  const above = await answerAbout(parent,);
  return (above.device === device)
    ? await volumeAtTheTopOf({
      volume: parent,
      device,
      answerAbout,
    },)
    : volume;
}

/**
 Reads the volume a folder is on, named by that volume's own path.

 Symbolic links are resolved first, because the folder the command was given may be reached through one, and the volume is the
 folder it really is.

 @param folder - the folder whose volume is read

 @param answerAbout - what a folder answers about itself, which a test replaces

 @returns the path of the volume the folder is on

 @throws Error when no folder on the way up can be read, which is a path this command cannot place at all

 @example
 ```ts
 const volume = await volumeOfFolder({ folder: '/home/a/temp/agent/pi-runs', answerAbout, },);
 ```
 */
export async function volumeOfFolder({
  folder,
  answerAbout,
}: {
  readonly folder: string;
  readonly answerAbout: AnswerAboutAFolder;
},): Promise<string> {
  /**
   What the nearest folder at or above the one given answers about itself.
   */
  const answer = await nearestFolderThatIsThere({
    folder,
    answerAbout,
  },);
  return volumeAtTheTopOf({
    volume: answer.realPath,
    device: answer.device,
    answerAbout,
  },);
}

/**
 Reads how much room the volume a folder is on has, naming that volume by its own path, so a refusal can say which drive to free.

 @param folder - the folder whose volume's room is read

 @param answerAbout - what a folder answers about itself, which a test replaces

 @param statfs - how the room of a volume is read, which a test replaces

 @returns the room the volume has, and that volume

 @example
 ```ts
 const freeSpace = await freeSpaceOfTheFolder({ folder: runsFolder, answerAbout, statfs: statfsSync, },);
 ```
 */
export async function freeSpaceOfTheFolder({
  folder,
  answerAbout,
  statfs,
}: {
  readonly folder: string;
  readonly answerAbout: AnswerAboutAFolder;
  readonly statfs: FreeBlocksOfAVolume;
},): Promise<FreeSpace> {
  return freeSpaceOnVolume({
    volume: await volumeOfFolder({
      folder,
      answerAbout,
    },),
    statfs,
  },);
}
