/**
 The gate image a new build replaced, removed from the disk so the gate does not leave one untagged image of every earlier
 run behind (issue 991).

 A build that reuses the image the tag already carried replaced nothing. An image another tag names, and one a container
 still runs, stay where they are: podman refuses that removal, and the run prints which image it kept and what podman
 said.
 */
import {
  type ImageName,
  imagesToRemove,
  localImageNames,
} from './native-image-retention.ts';
import { runCommand, } from './native-verification-process.ts';

/**
 Remove the gate image this build replaced, and print which one it was.

 @param before - every name of the image store, read before the build

 @param name - the image tag this build wrote

 @example
 ```ts
 await removeReplacedImage({ before: namesBeforeBuild, name: testImage });
 ```
 */
export async function removeReplacedImage({
  before,
  name,
}: {
  readonly before: readonly ImageName[];
  readonly name: string;
},): Promise<void> {
  /**
   The image this build replaced, which is none when the build reused the image the tag carried or a name holds it.
   */
  const [replacedImage] = imagesToRemove({
    before,
    after: await localImageNames(),
    name,
  },);
  if (replacedImage === undefined)
    return;
  /**
   Result of the removal, which podman refuses while a container still runs the image.
   */
  const removal = await runCommand({
    command: 'podman',
    args: [
      'image',
      'rm',
      replacedImage,
    ],
    capture: true,
    allowFailure: true,
  },);
  console.log(removal.status === 0
    ? `Removed the gate image this build replaced: ${replacedImage}`
    : `Kept the gate image this build replaced: ${replacedImage} (${removal.stderr
      .trim()})`,);
}
