/**
 Which image an instance boots from, and which account connects to it.

 Image names carry a space ("Ubuntu 24.04"), so a shorthand is compared against
 a normalized name: lower-cased, spaces turned into hyphens. `ubuntu` then
 matches every Ubuntu image and `ubuntu-24.04` matches that one only, with no
 special case for versions.

 The newest match wins, deprecated-free by construction: the schema has no
 deprecation marker on an image, so only images OVHcloud reports as active are
 considered. A public image is preferred over a private one of the same name.

 The login account is the image's own `user` field, which the schema describes
 as "User to connect with". It is never assumed to be `root`: OVHcloud's Linux
 images log in as an unprivileged account.

 @module
 */

import { listImages, } from './api-resources.ts';
import type { OvhImage, } from './types.ts';

//region Constants

/**
 The image status that means the image can be booted.
 */
const ACTIVE_IMAGE_STATUS = 'active';

/**
 The visibility OVHcloud's catalogue images carry.
 */
const PUBLIC_IMAGE_VISIBILITY = 'public';

/**
 Separator a normalized image name uses where the catalogue uses a space.
 */
const NAME_SEPARATOR = '-';

//endregion Constants

//region Matching

/**
 Normalizes an image name or a shorthand so the two compare: lower case, and
 every run of whitespace replaced by one hyphen.
 
 @param name - catalogue name or user shorthand
 
 @returns the comparable form
 
 @example
 ```ts
 normalizeImageName('Ubuntu 24.04'); // 'ubuntu-24.04'
 normalizeImageName('ubuntu');       // 'ubuntu'
 ```
 */
export function normalizeImageName(name: string,): string {
  // oxlint-disable-next-line no-restricted-syntax/no-regex -- one linear pass over a catalogue name of a few dozen characters; no string API collapses a run of whitespace into one separator
  return name.trim()
    .toLowerCase()
    .replaceAll(
    /\s+/gu,
    NAME_SEPARATOR,
    );
}

/**
 Chooses the image a shorthand names in one region.
 
 @param images - images the region offers
 
 @param region - region the images belong to, named in errors
 
 @param shorthand - image shorthand, such as `ubuntu` or `ubuntu-24.04`
 
 @returns the newest active match, public preferred
 
 @throws Error when nothing matches
 
 @example
 ```ts
 selectImage({ images, region: 'bhs5', shorthand: 'ubuntu' });
 ```
 */
export function selectImage({
  images,
  region,
  shorthand,
}: {
  readonly images: readonly OvhImage[];
  readonly region: string;
  readonly shorthand: string;
},): OvhImage {
  /**
   Comparable form of the shorthand every candidate must start with.
   */
  const wanted = normalizeImageName(shorthand,);
  /**
   Active images whose normalized name starts with the shorthand.
   */
  const matches = images.filter(function matchesShorthand(image,) {
    return (image.status === ACTIVE_IMAGE_STATUS)
      && normalizeImageName(image.name,)
        .startsWith(wanted,);
  },);
  if (matches.length === 0) {
    throw new Error(
      `no active image in region ${region} matches "${shorthand}"; set --image to another name`,
    );
  }
  /**
   Matches OVHcloud publishes for everyone; a private image of the same name
   (a snapshot named after a distribution, say) never displaces them.
   */
  const published = matches.filter(function isPublic(image,) {
    return image.visibility === PUBLIC_IMAGE_VISIBILITY;
  },);
  /**
   Newest match by creation timestamp, over the published ones when there are any.
   */
  return ((published.length > 0) ? published : matches)
    .reduce(function newest(
      best,
      candidate,
    ) {
      return (candidate.creationDate > best.creationDate) ? candidate : best;
    },);
}

//endregion Matching

//region Reading

/**
 Resolves a shorthand against one region's own image list.
 
 @param region - region to create in
 
 @param shorthand - image shorthand, such as `ubuntu`
 
 @returns the image to boot from, with its login account and identifier
 
 @throws Error when nothing in the region matches
 
 @example
 ```ts
 const image = await resolveImage({ region: 'bhs5', shorthand: 'ubuntu' });
 ```
 */
export async function resolveImage({
  region,
  shorthand,
}: {
  readonly region: string;
  readonly shorthand: string;
},): Promise<OvhImage> {
  return selectImage({
    images: await listImages({ region, },),
    region,
    shorthand,
  },);
}
