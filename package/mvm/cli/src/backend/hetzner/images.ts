/**
 Resolves an mvm image shorthand to a concrete Hetzner image.
 
 Image slugs churn (e.g. `fedora-41` was removed, `fedora-44` added), so a
 distro shorthand resolves to the newest non-deprecated system image of that
 OS flavor at call time, rather than to a hardcoded slug. Unrecognised values
 pass through literally so callers can pin an exact image name or id.
 
 @module
 */

import { listImages, } from './api-resources.ts';

//region Flavor sets

/**
 Shorthands resolved against Hetzner system images by `os_flavor`.
 */
const KNOWN_FLAVORS: ReadonlySet<string> = new Set([
  'alma',
  'centos',
  'debian',
  'fedora',
  'rocky',
  'ubuntu',
],);

/**
 Shorthands Hetzner does not offer as system images.
 */
const UNSUPPORTED_FLAVORS: ReadonlySet<string> = new Set([
  'alpine',
  'windows',
],);

//endregion Flavor sets

//region Resolution

/**
 Resolves an image shorthand to a Hetzner image slug.
 
 For a known distro flavor, queries `GET /images?type=system` and returns the
 newest non-deprecated image of that flavor. For `alpine`/`windows`, throws.
 For anything else, returns the value unchanged so an exact slug or id passes
 through.
 
 @param shorthand - image identifier (`ubuntu`, `fedora`, a literal slug, etc.)
 
 @returns concrete Hetzner image slug or the literal passthrough value
 
 @throws Error for unsupported flavors, or when no non-deprecated image matches
 
 @example
 ```ts
 await resolveHetznerImage({ shorthand: 'ubuntu' }); // 'ubuntu-24.04'
 await resolveHetznerImage({ shorthand: 'ubuntu-22.04' }); // 'ubuntu-22.04' (literal)
 ```
 */
export async function resolveHetznerImage(
  { shorthand, }: { readonly shorthand: string; },
): Promise<string> {
  if (UNSUPPORTED_FLAVORS.has(shorthand,)) {
    throw new Error(
      `image "${shorthand}" is unsupported on the hetzner backend (Hetzner offers no ${shorthand} system image)`,
    );
  }
  if (!KNOWN_FLAVORS.has(shorthand,)) {
    return shorthand;
  }

  /**
   Non-deprecated system images of the requested flavor with a usable slug.
   */
  const matching = (await listImages({ type: 'system', },)).filter(
    function isUsable(image,) {
      return (image.os_flavor === shorthand)
        && ((typeof image.deprecated) !== 'string')
        && ((typeof image.name) === 'string');
    },
  );
  /**
   Head and tail of the candidate list; head seeds the newest-wins reduce.
   */
  const [first, ...others] = matching;
  if (first === undefined) {
    throw new Error(
      `no non-deprecated Hetzner system image found for "${shorthand}"`,
    );
  }
  /**
   Newest candidate by ISO 8601 creation timestamp (lexicographic compare is
   chronological for same-format timestamps).
   */
  const newest = others.reduce(
    function pickNewer(
      best,
      image,
    ) {
      return (image.created > best.created) ? image : best;
    },
    first,
  );
  if ((typeof newest.name) !== 'string') {
    throw new Error(`resolved Hetzner image for "${shorthand}" has no slug`,);
  }
  return newest.name;
}

//endregion Resolution
