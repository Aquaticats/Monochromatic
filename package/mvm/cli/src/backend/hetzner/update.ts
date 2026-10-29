/**
 Hetzner image reporting.
 
 Hetzner manages system images server-side, so there is nothing to download or
 bake locally. `update` validates the token and reports the available
 non-deprecated images and OS flavors.
 
 @module
 */
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { listImages, } from './api-resources.ts';
import { requireToken, } from './config.ts';

/**
 Logger root for mvm after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

/**
 Validates the token and reports available Hetzner system images.
 Unlike the libvirt backend, there is no local image cache or template build.
 
 @throws Error when the API token is missing
 
 @example
 ```ts
 await hetznerUpdate();
 ```
 */
export async function hetznerUpdate(): Promise<void> {
  /**
   Logger scoped to this report so output is namespaced.
   */
  const rl = tagged({
    tag: hetznerUpdate.name,
    l,
  },);
  requireToken();
  /**
   Non-deprecated system images with a usable slug.
   */
  const active = (await listImages({ type: 'system', },)).filter(
    function isActive(image,) {
      return ((typeof image.deprecated) !== 'string') && ((typeof image.name) === 'string');
    },
  );
  rl.info(
    `Hetzner manages images server-side; ${
      String(active.length,)
    } non-deprecated system images available, nothing to build locally.`,
  );
  /**
   Distinct OS flavors among the active images, for at-a-glance guidance.
   */
  const flavors = [...new Set(active.map(function flavorOf(image,) {
    return image.os_flavor;
  },),),].toSorted();
  rl.info(`available OS flavors: ${flavors.join(', ',)}`,);
}
