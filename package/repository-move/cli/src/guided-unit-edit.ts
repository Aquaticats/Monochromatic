/**
 The edit of the 860's mount unit that adds the subvolume removal option, made on a copy.

 @module
 */

import {
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import type { GuidedConfig, } from './guided-types.ts';

/**
 The option the 860 mount gains, the plan's adopted decision.
 */
export const SUBVOLUME_REMOVAL_OPTION = 'user_subvol_rm_allowed';

/**
 The mount options the unit has today, which the edit appends the new option to.
 */
const UNIT_MOUNT_OPTIONS = 'compress=zstd:3,noatime';

/**
 Edits a copy of the 860 mount unit: appends the removal option to the one `mount -o` line that carries the unit's
 current options, and refuses when that line is not found exactly once. Writes `mount-860.service.before` (the unit as
 it is, kept for the owner) and `mount-860.service.new` in the state folder; changes nothing else.

 @param config - the configuration
 
 @returns the lines to show, including the changed line
 
 @throws Error when the unit does not hold the expected line exactly once, or already has the option

 @example
 ```ts
 import { prepareUnitEdit, } from '\@monochromatic-dev/repository-move-cli';
 
 const result = await prepareUnitEdit(config,);
 ```
 */
export async function prepareUnitEdit(config: GuidedConfig,): Promise<string> {
  /**
   The text of the 860 mount unit.
   */
  const unit = await readFile(
    config.unitPath,
    'utf8',
  );
  if (unit.includes(SUBVOLUME_REMOVAL_OPTION,)) {
    return `${config.unitPath} already holds ${SUBVOLUME_REMOVAL_OPTION}; the edit is not needed`;
  }
  /**
   The marker this step searches for.
   */
  const marker = `mount -o ${UNIT_MOUNT_OPTIONS} `;
  if (unit.split(marker,)
    .length
    !== 2) {
    throw new Error(`${config.unitPath} does not hold exactly one '${marker.trim()}'; edit the unit by hand with: sudo systemctl edit --full mount-860.service`,);
  }
  await mkdir(
    config.wiz,
    { recursive: true, },
  );
  await writeFile(
    join(
      config.wiz,
      'mount-860.service.before',
    ),
    unit,
  );
  await writeFile(
    join(
      config.wiz,
      'mount-860.service.new',
    ),
    unit.replace(
      marker,
      `mount -o ${UNIT_MOUNT_OPTIONS},${SUBVOLUME_REMOVAL_OPTION} `,
    ),
  );
  return `unit before: ${join(
    config.wiz,
    'mount-860.service.before',
  )}\nunit after:  ${join(
    config.wiz,
    'mount-860.service.new',
  )}\nchanged line: ${marker.trim()} -> mount -o ${UNIT_MOUNT_OPTIONS},${SUBVOLUME_REMOVAL_OPTION}`;
}
