/**
 The running units whose worktree folder is gone, so a unit is still working in a folder a merge or a removal took away: its work
 cannot be committed where a reader looks for it, and it holds a pool slot until its own bound ends it.

 A worktree's name comes from the one place that names worktrees, `worktreeFolderOf`, so this command and the launcher cannot
 disagree about where a run works.

 The listing and the folder question are the seams: a test hands over the text systemd would have printed and answers for the
 folders itself.

 @module
 */

import { worktreeFolderOf, } from './parse-worktree-launcher-options.ts';
import { runNameOfUnit, } from './running-pi-units.ts';

/**
 The end of every systemd unit name.
 */
const UNIT_SUFFIX = '.service';

/**
 One unit whose worktree folder is gone.
 */
export type UnitWithoutItsWorktree = {
  /**
   The unit's name as systemd prints it.
   */
  readonly unitName: string;

  /**
   The worktree folder that is gone.
   */
  readonly folder: string;
};

/**
 Reads the unit names `systemctl list-units` printed, keeping only whole service units.

 `--no-legend` and `--plain` give one unit per line with its name first, and a line that is not a unit, such as a blank one or the
 legend a manager prints without those flags, is left out.

 @param shown - what `systemctl list-units` printed

 @returns the unit names, in the order systemd printed them

 @example
 ```ts
 const units = serviceUnitsIn({ shown: 'pi-copy-route.service loaded active running x\n', },);
 ```
 */
export function serviceUnitsIn({ shown, }: { readonly shown: string },): readonly string[] {
  return shown.split('\n',)
    .map(function firstFieldOf(line,) {
      return line.trim()
        .split(' ',)
        .find(function isAField(word,) {
          return word !== '';
        },)
        ?? '';
    },)
    .filter(function isAService(unit,) {
      return unit.endsWith(UNIT_SUFFIX,);
    },);
}

/**
 Reads every running unit whose worktree folder is gone.

 @param unitNames - the units that are running now

 @param checkoutFolder - the checkout whose worktrees the launcher makes

 @param isThere - how a folder's presence is answered, which a test replaces

 @returns one entry per unit whose folder is gone, in the order systemd listed them

 @example
 ```ts
 const gone = await unitsWhoseWorktreeIsGone({ unitNames, checkoutFolder: '/repo', isThere, },);
 ```
 */
export async function unitsWhoseWorktreeIsGone({
  unitNames,
  checkoutFolder,
  isThere,
}: {
  readonly unitNames: readonly string[];
  readonly checkoutFolder: string;
  readonly isThere: (folder: string) => Promise<boolean>;
},): Promise<readonly UnitWithoutItsWorktree[]> {
  /**
   The worktree folder each unit's run works in, read from the one place that names worktrees.
   */
  const folders = unitNames.map(function ofUnit(unitName,) {
    return worktreeFolderOf({
      checkoutFolder,
      runName: runNameOfUnit({ unitName, },),
    },);
  },);
  /**
   Whether each folder is there, asked all at once: the answer for one unit says nothing about another.
   */
  const there = await Promise.all(folders.map(function ofFolder(folder,) {
    return isThere(folder,);
  },));
  return unitNames.flatMap(function ofAnswer(
    unitName,
    index,
  ) {
    /**
     The folder this unit's run works in.
     */
    const folder = folders[index] ?? '';
    return (there[index] === true)
      ? []
      : [{
        unitName,
        folder,
      },];
  },);
}

/**
 Says one unit whose worktree is gone as the line a reader acts on.

 @param gone - the unit

 @returns the line naming the folder that is gone

 @example
 ```ts
 const line = goneLineOf({ unitName: 'pi-a.service', folder: '/repo/.claude/worktrees/pi-a', },);
 ```
 */
export function goneLineOf(gone: UnitWithoutItsWorktree,): string {
  return `GONE ${gone.folder}`;
}

/**
 Says how many units were listed and how many lack their worktree.

 @param listed - how many units are running now

 @param gone - how many of them have no worktree

 @returns the closing line

 @example
 ```ts
 const line = countLineOf({ listed: 3, gone: 1, },);
 ```
 */
export function countLineOf({
  listed,
  gone,
}: {
  readonly listed: number;
  readonly gone: number;
},): string {
  return `${String(listed,)} running units, ${String(gone,)} without their worktree`;
}
