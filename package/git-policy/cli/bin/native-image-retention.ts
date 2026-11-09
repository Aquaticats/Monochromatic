/**
 What the gate does with the image its own build replaced.

 Every build gives `localhost/git-policy-native-test:<tag>` a fresh image and leaves the image that held the tag
 behind: no name points at it any more, so `podman images` reports it as `<none>:<none>` and it stays on the disk
 until somebody prunes the store by hand (issue 991). The gate removes exactly the image its own build replaced, and
 only while nothing else names it: a tag another worktree made, and an image a container still runs, are left alone.
 */

/**
 What podman prints for both the repository and the tag of an image no name points at.
 */
const noName = '<none>';

/**
 One name of one local image, as `podman images --format '{{.ID}} {{.Repository}}:{{.Tag}}'` prints it.

 An image carries one entry per name: the image a build replaced is listed under the untagged marker alone, and an
 image another tag still names is listed under that tag too.
 */
export type ImageName = {
  readonly id: string;
  readonly repository: string;
  readonly tag: string;
};

/**
 Read the listing `podman images --format '{{.ID}} {{.Repository}}:{{.Tag}}'` printed.

 @param output - every line podman printed

 @returns one entry per readable line, in the order podman printed them; a line without an ID and a name is dropped

 @example
 ```ts
 imageNames({ output: 'a1b2c3d4e5f6 localhost/git-policy-native-test:development\n', });
 ```
 */
export function imageNames({ output, }: { readonly output: string; },): readonly ImageName[] {
  /**
   One entry per readable line, in podman's order.
   */
  const names: ImageName[] = [];
  for (const line of output.split('\n',)) {
    /**
     The line without the whitespace podman can print around it.
     */
    const trimmed = line.trim();
    if (trimmed.length === 0)
      continue;
    /**
     Where the ID ends and the name begins.
     */
    const idEnd = trimmed.indexOf(' ',);
    if (idEnd === (-1))
      continue;
    /**
     The `repository:tag` part of the line.
     */
    const name = trimmed.slice(idEnd + 1,);
    /**
     Where the repository ends and the tag begins.
     */
    const tagStart = name.lastIndexOf(':',);
    if (tagStart === (-1))
      continue;
    names.push({
      id: trimmed.slice(
        0,
        idEnd,
      ),
      repository: name.slice(
        0,
        tagStart,
      ),
      tag: name.slice(tagStart + 1,),
    },);
  }
  return names;
}

/**
 The IDs the store lists under one name.

 @param names - every name of the local store, as `imageNames` read them

 @param name - `repository:tag` the store is asked for

 @returns one ID per line the store printed for that name, in podman's order
 */
function idsNamed({
  names,
  name,
}: {
  readonly names: readonly ImageName[];
  readonly name: string;
},): readonly string[] {
  /**
   IDs read so far.
   */
  const ids: string[] = [];
  for (const candidate of names) {
    if (`${candidate.repository}:${candidate.tag}` !== name)
      continue;
    ids.push(candidate.id,);
  }
  return ids;
}

/**
 Whether nothing but the untagged marker names an image.

 @param names - every name of the local store, as `imageNames` read them

 @param id - ID of an image the store listed

 @returns `true` when the store lists the ID and every name of it is the untagged marker, `false` when a tag still
 names it or the store lists it nowhere

 @example
 ```ts
 isUnnamedImage({ names: after, id: replacedId, });
 ```
 */
export function isUnnamedImage({
  names,
  id,
}: {
  readonly names: readonly ImageName[];
  readonly id: string;
},): boolean {
  /**
   Every name the store lists for that ID, which is none once another run removed the image.
   */
  const listed = names.filter(function listedUnder(candidate: ImageName,): boolean {
    return candidate.id === id;
  },);
  if (listed.length === 0)
    return false;
  return listed.every(function isUntagged(candidate: ImageName,): boolean {
    return (candidate.repository === noName) && (candidate.tag === noName);
  },);
}

/**
 The image a gate build superseded, while removing it destroys nothing else.

 @param before - every name of the local store, read before the build

 @param after - every name of the local store, read after the build

 @param name - `repository:tag` of the image the build wrote

 @returns the one ID to remove, or no IDs when the tag carried no image before the build, when the build reused the
 image it carried, or when a name or a container still holds it

 @example
 ```ts
 imagesToRemove({ before, after, name: 'localhost/git-policy-native-test:development', });
 ```
 */
export function imagesToRemove({
  before,
  after,
  name,
}: {
  readonly before: readonly ImageName[];
  readonly after: readonly ImageName[];
  readonly name: string;
},): readonly string[] {
  /**
   IDs the tag carried before the build, which is none when the store listed no image under that name.
   */
  const carried = idsNamed({
    names: before,
    name,
  },);
  /**
   IDs the tag carries after the build.
   */
  const current = idsNamed({
    names: after,
    name,
  },);
  return carried
    .filter(function replaced(candidate: string,): boolean {
      return !current.includes(candidate,);
    },)
    .filter(function unnamed(candidate: string,): boolean {
      return isUnnamedImage({
        names: after,
        id: candidate,
      },);
    },);
}
