import { join, } from 'node:path';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { IMAGES_DIR, } from './config.ts';
import { pathExists, } from './path-exists.ts';
import type { ImageSpec, } from './registry.ts';
import { ensureLinuxTemplate, } from './template-linux.ts';
import { ensureWindowsTemplate, } from './template-windows.ts';

/**
 Logger root for mvm after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

export {
  TEMPLATE_VM_NAME,
  templateVmGuard,
} from './template-shared.ts';

/**
 Ensures a template image exists for the given image spec, creating it
 if not already cached. Dispatches to the appropriate template baking
 pipeline based on the OS family:
 
 - **Linux**: via {@link ensureLinuxTemplate}, creates a temporary VM from
   the cloud image, installs qemu-guest-agent via cloud-init, and captures
   the result
 - **Windows**: via {@link ensureWindowsTemplate}, boots from the evaluation
   ISO with an Autounattend.xml answer file, performs a full unattended
   installation, and captures the result
 
 Each image gets its own template (e.g. `template-ubuntu.qcow2`,
 `template-windows.qcow2`) so multiple distros coexist in the cache.
 
 @param spec - Image specification from the registry
 
 @returns Absolute path to the template qcow2 image
 
 @throws Error when template creation fails
 
 @example
 ```ts
 const linuxTemplate = await ensureTemplate(IMAGES['ubuntu']);
 const windowsTemplate = await ensureTemplate(IMAGES['windows']);
 ```
 */
export async function ensureTemplate(spec: ImageSpec,): Promise<string> {
  /**
   Tagged logger so template-baking messages name the call site.
   */
  const rl = tagged({
    tag: ensureTemplate.name,
    l,
  },);
  /**
   Cached template location reused when present, baked when missing.
   */
  const templatePath = join(
    IMAGES_DIR,
    spec.templateFileName,
  );

  if (await pathExists(templatePath,)) {
    rl.info(`using cached template ${templatePath}`,);
    return templatePath;
  }

  if (spec.osFamily
    === 'windows')
    return await ensureWindowsTemplate(spec,);
  return await ensureLinuxTemplate(spec,);
}
