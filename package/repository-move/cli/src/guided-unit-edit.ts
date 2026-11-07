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
 The words the new stop block carries, which a check and a test look for: the unit reports that the mount is held open.
 */
export const STOP_PATH_MARKER = 'held open';

/**
 The stop block of the unit as it is today (`systemctl cat mount-860.service`, 2026-11-07): it unmounts and closes the drive,
 and hides every failure.
 */
const OLD_STOP_BLOCK = `ExecStop=/bin/bash -c '\\
  umount /var/mnt/encrypted 2>/dev/null; \\
  cryptsetup luksClose crypt_sda 2>/dev/null'
`;

/**
 The stop block the unit gets (plan item W11, `doc/planning/repository-on-the-860-drive.md`): when the mount is held it names
 the holders and stops without closing the drive, and it never unmounts lazily or by force, so a program still using the
 repository keeps a drive that is there rather than one that vanishes under it.
 */
const NEW_STOP_BLOCK = `ExecStop=/bin/bash -c '\\
  if mountpoint -q /var/mnt/encrypted; then \\
    if ! umount /var/mnt/encrypted; then \\
      echo "mount-860: /var/mnt/encrypted is ${STOP_PATH_MARKER}, so the drive was not closed. Holders:"; \\
      fuser -vm /var/mnt/encrypted 2>&1 || true; \\
      echo "mount-860: stop those programs and stop this unit again; no lazy or forced unmount is used."; \\
      exit 1; \\
    fi; \\
  fi; \\
  cryptsetup luksClose crypt_sda'
`;

/**
 Edits a copy of the 860 mount unit: appends the removal option to the one `mount -o` line that carries the unit's
 current options, and replaces the one stop block with the one that names the holders; each is skipped when the unit
 already has it, and the edit refuses when a line it must change is not found exactly once. Writes
 `mount-860.service.before` (the unit as it is, kept for the owner) and `mount-860.service.new` in the state folder;
 changes nothing else.

 @param config - the configuration
 
 @returns the lines to show, including the changed lines
 
 @throws Error when the unit does not hold a line to change exactly once

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
  /**
   The marker this step searches for.
   */
  const marker = `mount -o ${UNIT_MOUNT_OPTIONS} `;
  /**
   Whether the mount option still has to be added.
   */
  const needsOption = !unit.includes(SUBVOLUME_REMOVAL_OPTION,);
  /**
   Whether the stop block still has to be replaced.
   */
  const needsStopPath = !unit.includes(STOP_PATH_MARKER,);
  if ((!needsOption) && (!needsStopPath)) {
    return `${config.unitPath} already holds ${SUBVOLUME_REMOVAL_OPTION} and the stop block that names the holders; the edit is not needed`;
  }
  if (needsOption && (unit.split(marker,)
    .length
    !== 2)) {
    throw new Error(`${config.unitPath} does not hold exactly one '${marker.trim()}'; edit the unit by hand with: sudo systemctl edit --full mount-860.service`,);
  }
  if (needsStopPath && (unit.split(OLD_STOP_BLOCK,)
    .length
    !== 2)) {
    throw new Error(`${config.unitPath} does not hold exactly one stop block as it is today; edit the unit by hand with: sudo systemctl edit --full mount-860.service`,);
  }
  /**
   The unit with the mount option added, when it needs it.
   */
  const withOption = needsOption
    ? unit.replace(
      marker,
      `mount -o ${UNIT_MOUNT_OPTIONS},${SUBVOLUME_REMOVAL_OPTION} `,
    )
    : unit;
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
    needsStopPath ? withOption.replace(
      OLD_STOP_BLOCK,
      NEW_STOP_BLOCK,
    ) : withOption,
  );
  return `unit before: ${join(
    config.wiz,
    'mount-860.service.before',
  )}\nunit after:  ${join(
    config.wiz,
    'mount-860.service.new',
  )}\nchanged: ${needsOption ? `${marker.trim()} -> mount -o ${UNIT_MOUNT_OPTIONS},${SUBVOLUME_REMOVAL_OPTION}` : 'the mount option was already there'}; ${needsStopPath ? 'the stop block now names the holders and never closes a held drive' : 'the stop block was already the new one'}`;
}
