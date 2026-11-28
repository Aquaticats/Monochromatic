/**
 Which image an instance boots from, and which account connects to it.

 Image names carry a space ("Ubuntu 24.04"), so a shorthand is compared against
 a normalized name: lower-cased, spaces turned into hyphens. `ubuntu` then
 matches every Ubuntu image and `ubuntu-24.04` matches that one only, with no
 special case for versions.

 The newest match wins, deprecated-free by construction: the schema has no
 deprecation marker on an image, so only images OVHcloud reports as active are
 considered. A public image is preferred over a private one of the same name.

 A shorthand stands for a plain distribution image only. A live catalogue lists,
 beside "Ubuntu 26.04", a "Ubuntu 26.04 - UEFI" and a "Ubuntu 26.04 - NVIDIA -
 v595" that were published after it, so "the newest match" alone booted a GPU
 driver image on the smallest flavor, which never became active. Two tests keep
 a shorthand off such an image: one that carries OVHcloud's tag for preinstalled
 software is left out, and so is one whose name extends another match's name.
 Either is still taken when `--image` names it whole.

 An image the chosen flavor cannot boot is left out too, by the image's own
 `minDisk`, `minRam` and `flavorType`.

 The login account is the image's own `user` field, which the schema describes
 as "User to connect with". It is never assumed to be `root`: OVHcloud's Linux
 images log in as an unprivileged account.

 @module
 */

import { listImages, } from './api-resources.ts';
import type {
  OvhFlavor,
  OvhImage,
} from './types.ts';

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

/**
 The tag OVHcloud's catalogue puts on an image that ships preinstalled software
 on top of a distribution. Read from a live project's catalogue on 2026-11-28:
 every such image carried it beside a second tag naming the software
 (`nvidia_ngc` on the GPU driver images, and `docker`, `plesk`, `cpanel`, `n8n`),
 and every plain distribution image carried no tag at all. The schema types
 `tags` as bare strings, so this value is from that answer and not from the
 schema.
 */
const APPLICATION_IMAGE_TAG = 'application';

/**
 Separator between the flavor families an image's `flavorType` lists. The one
 value a live catalogue showed on 2026-11-28 was a comma series of the leading
 parts of flavor names (`t1`, `l4`), not of flavor `type` values.
 */
const FLAVOR_TYPE_SEPARATOR = ',';

/**
 Separator between a flavor's family and its size in a flavor name such as `d2-2`.
 */
const FLAVOR_FAMILY_SEPARATOR = '-';

/**
 Mebibytes in a gibibyte: the schema gives an image's `minRam` in MB and a
 flavor's `ram` in GiB.
 */
const MEBIBYTES_PER_GIBIBYTE = 1_024;

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
 Whether an image ships preinstalled software, by OVHcloud's own tag.

 @param image - catalogue image

 @returns `true` for an application image, such as a GPU driver image

 @example
 ```ts
 isApplicationImage(image); // true for "Ubuntu 26.04 - NVIDIA - v595"
 ```
 */
function isApplicationImage(image: OvhImage,): boolean {
  return (image.tags ?? []).includes(APPLICATION_IMAGE_TAG,);
}

/**
 Whether an image's name extends another candidate's name, which makes it a
 variant of that image ("Ubuntu 26.04 - UEFI" of "Ubuntu 26.04").

 @param candidate - image whose name is tested

 @param candidates - every image the shorthand matched

 @returns `true` when another candidate's whole name opens this one's

 @example
 ```ts
 isVariantOfAnother({ candidate, candidates }); // true for "Ubuntu 26.04 - UEFI" beside "Ubuntu 26.04"
 ```
 */
function isVariantOfAnother({
  candidate,
  candidates,
}: {
  readonly candidate: OvhImage;
  readonly candidates: readonly OvhImage[];
},): boolean {
  /**
   Comparable name of the image under test.
   */
  const own = normalizeImageName(candidate.name,);
  return candidates.some(function isItsBase(other,) {
    /**
     Comparable name of the image that may be its base.
     */
    const base = normalizeImageName(other.name,);
    return (base !== own) && own.startsWith(`${base}${NAME_SEPARATOR}`,);
  },);
}

/**
 Flavor families an image restricts itself to, from its `flavorType`.

 @param image - catalogue image

 @returns the families named, or none when the image boots on any flavor

 @example
 ```ts
 flavorFamiliesOf(image); // ['t1', 't2'] for a flavorType of 't1,t2'
 ```
 */
function flavorFamiliesOf(image: OvhImage,): readonly string[] {
  return (image.flavorType ?? '')
    .split(FLAVOR_TYPE_SEPARATOR,)
    .map(function trimmed(family,) {
      return family.trim();
    },)
    .filter(function isNamed(family,) {
      return family !== '';
    },);
}

/**
 Whether a flavor can boot an image, by the image's own floor of disk and
 memory and by the flavor families it restricts itself to.

 @param flavor - flavor the instance is created with

 @param image - catalogue image

 @returns `true` when the image states nothing the flavor lacks

 @example
 ```ts
 flavorCanBoot({ flavor, image }); // false when image.minDisk exceeds flavor.disk
 ```
 */
function flavorCanBoot({
  flavor,
  image,
}: {
  readonly flavor: OvhFlavor;
  readonly image: OvhImage;
},): boolean {
  /**
   Families the image restricts itself to; none means any flavor.
   */
  const families = flavorFamiliesOf(image,);
  /**
   Whether the flavor's name opens with one of those families.
   */
  const isOfAFamily = families.some(function namesTheFlavor(family,) {
    return flavor
      .name
      .startsWith(`${family}${FLAVOR_FAMILY_SEPARATOR}`,);
  },);
  /**
   Whether the image's family restriction, if it has one, admits the flavor.
   */
  const isAdmitted = (families.length === 0) || isOfAFamily;
  /**
   Whether the flavor's disk and memory reach the image's stated floors.
   */
  const isLargeEnough = (image.minDisk <= flavor.disk)
    && (image.minRam <= (flavor.ram * MEBIBYTES_PER_GIBIBYTE));
  return isAdmitted && isLargeEnough;
}

/**
 Names of images in one sentence part, for a refusal that must say which
 images it left out.

 @param images - images to name

 @returns their catalogue names, quoted and comma-separated

 @example
 ```ts
 namesOf(images); // '"Ubuntu 26.04 - UEFI", "Ubuntu 26.04 - NVIDIA - v595"'
 ```
 */
function namesOf(images: readonly OvhImage[],): string {
  return images
    .map(function quoted(image,) {
      return `"${image.name}"`;
    },)
    .join(', ',);
}

/**
 Chooses the image a shorthand names in one region, for one flavor.

 A match whose whole name is the shorthand is taken as asked. Otherwise the
 shorthand stands for plain images only: a match that carries preinstalled
 software, or whose name extends another match's name, is left out. Of what is
 left, an image the flavor cannot boot is left out, a public image is preferred,
 and the newest wins.

 @param flavor - flavor the instance is created with

 @param images - images the region offers

 @param region - region the images belong to, named in errors

 @param shorthand - image shorthand, such as `ubuntu` or `ubuntu-24.04`, or an image's whole name

 @returns the newest active plain match the flavor can boot, public preferred

 @throws Error when nothing matches, when every match is left out as an application image or a variant, or when the flavor boots none

 @example
 ```ts
 selectImage({ flavor, images, region: 'bhs5', shorthand: 'ubuntu' });
 ```
 */
export function selectImage({
  flavor,
  images,
  region,
  shorthand,
}: {
  readonly flavor: OvhFlavor;
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
   Matches whose whole name is the shorthand: whoever names an image whole gets it.
   */
  const namedWhole = matches.filter(function isNamedWhole(image,) {
    return normalizeImageName(image.name,) === wanted;
  },);
  /**
   Matches a shorthand may stand for: no preinstalled software, no variant.
   */
  const plain = matches.filter(function isPlain(image,) {
    return (!isApplicationImage(image,))
      && (!isVariantOfAnother({
        candidate: image,
        candidates: matches,
      },));
  },);
  /**
   Images this request means: the one named whole, or the plain matches.
   */
  const meant = (namedWhole.length > 0) ? namedWhole : plain;
  if (meant.length === 0) {
    throw new Error(
      `every active image in region ${region} that matches "${shorthand}" carries preinstalled software or is a variant of another image, and a shorthand never stands for one of those: ${
        namesOf(matches,)
      }; set --image to the whole name of the one you want`,
    );
  }
  /**
   Of those, the images the flavor can boot.
   */
  const bootable = meant.filter(function boots(image,) {
    return flavorCanBoot({
      flavor,
      image,
    },);
  },);
  if (bootable.length === 0) {
    throw new Error(
      `flavor ${flavor.name} (${String(flavor.disk,)} GB of disk, ${
        String(flavor.ram,)
      } GiB of memory) boots none of the images in region ${region} that match "${shorthand}": ${
        namesOf(meant,)
      } each state a larger disk, more memory or another flavor family; set --image to another image or --server-type to a larger flavor`,
    );
  }
  /**
   Bootable matches OVHcloud publishes for everyone; a private image of the same
   name (a snapshot named after a distribution, say) never displaces them.
   */
  const published = bootable.filter(function isPublic(image,) {
    return image.visibility === PUBLIC_IMAGE_VISIBILITY;
  },);
  // Newest match by creation timestamp, over the published ones when there are any.
  return ((published.length > 0) ? published : bootable)
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
 Resolves a shorthand against one region's own image list, for one flavor.
 
 @param flavor - flavor the instance is created with
 
 @param region - region to create in
 
 @param shorthand - image shorthand, such as `ubuntu`, or an image's whole name
 
 @returns the image to boot from, with its login account and identifier
 
 @throws Error when nothing in the region matches or the flavor boots no match
 
 @example
 ```ts
 const image = await resolveImage({ flavor, region: 'bhs5', shorthand: 'ubuntu' });
 ```
 */
export async function resolveImage({
  flavor,
  region,
  shorthand,
}: {
  readonly flavor: OvhFlavor;
  readonly region: string;
  readonly shorthand: string;
},): Promise<OvhImage> {
  return selectImage({
    flavor,
    images: await listImages({ region, },),
    region,
    shorthand,
  },);
}
