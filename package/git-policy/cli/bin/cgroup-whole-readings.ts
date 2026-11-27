/**
 What one reading of a control group answered, and whether a reading answers every part of the one before it.

 A control group gains its files as the kernel makes it and gives them up one at a time as it goes, the directory last:
 `cgroup_destroy_locked` clears the files with `css_clear_dir` and only then calls `kernfs_remove` on the directory
 (`kernel/cgroup/cgroup.c`). A file the kernel has deactivated and not yet unlinked answers `ENODEV`
 (`fs/kernfs/file.c`), so a reader that meets a group on its way out meets a group that is partly there.

 Two readers meet that and judge a reading by the same rule: `bin/native-check-scope-accounting.ts` reads a check
 scope's own accounting, and `bin/native-gate-container-cgroup-memory.ts` reads a gate container's. A reading that lost a
 part of its group never replaces the whole reading before it, so what a caller is judged on is the last reading that
 answered every part of the one before it. This module is that rule, so neither reader knows the other's files.

 Nothing here reads a file: a reading arrives as the object its own reader assembled, and each part it answered is a
 figure the reading holds or a counter of one of its counter files, named `<field>` or `<field>.<counter>`.
 */

//region What one reading answered

/**
 Every part of one reading, by name: each figure it answered and each counter of each counter file.

 A reading's text fields, such as the path it came from and the moment it was taken, are no parts: every reading answers
 them, so they say nothing about what the group held.

 @param sample - one reading of a control group

 @returns the names of the figures and counters it answered

 @example
 ```ts
 partsOf({ sample }); // new Set(['pidsPeak', 'pidsEvents.max', 'cpuStat.nr_periods'])
 ```
 */
export function partsOf({ sample }: { readonly sample: Readonly<Record<string, unknown>>; },): ReadonlySet<string> {
  /**
   The parts answered so far.
   */
  const parts = new Set<string>();
  for (const [name, part] of Object.entries(sample,)) {
    if (part === undefined)
      continue;
    if ((typeof part) === 'number')
      parts.add(name,);
    else if (((typeof part) === 'object') && (part !== null)) {
      for (const counter of Object.keys(part,))
        parts.add(`${name}.${counter}`,);
    }
  }
  return parts;
}

/**
 Tells whether one reading answers every part of the reading it would replace.

 A control group gains its files as the kernel makes it and loses them as it goes, and it does not come back: what a
 reading answers and the one before it did not is a group still being made, and what it does not answer and the one
 before it did is a group that went. So a reading of the moment a group went is never a reading a caller is judged by,
 and the caller keeps the whole one before it.

 A reading that is not there at all is one of the reader's own sentinels, which each reader passes as it stands: nothing
 kept replaces nothing, and a reading that answered nothing replaces nothing either.

 @param read - the reading just taken, or the reader's sentinel when it answered nothing

 @param kept - the reading it would replace, or the reader's sentinel when nothing was read before it

 @returns true when this reading answers every figure and counter the kept one answered

 @example
 ```ts
 answersEveryPartOf({ read, kept: watch.lastRead() });
 ```
 */
export function answersEveryPartOf<Sample extends Readonly<Record<string, unknown>>>({
  read,
  kept,
}: {
  readonly read: Sample | symbol;
  readonly kept: Sample | symbol;
},): boolean {
  if ((typeof kept) === 'symbol')
    return true;
  if ((typeof read) === 'symbol')
    return false;
  /**
   Every part the kept reading answered.
   */
  const keptParts = partsOf({ sample: kept, },);
  /**
   Every part this reading answers.
   */
  const answered = partsOf({ sample: read, },);
  for (const part of keptParts) {
    if (!answered.has(part,))
      return false;
  }
  return true;
}

//endregion What one reading answered

//region The kernel's answer for a file it is removing

/**
 Tells whether an error is the kernel's answer for a file of a control group it is removing.

 A kernfs node the kernel has deactivated and not yet unlinked answers `ENODEV` to an open and to a read of it
 (`fs/kernfs/file.c`), which is what a control group's files do one at a time as the group goes. Such a file answers no
 figure, exactly as a file this host's kernel does not write does, and the reading it was read for is refused for the
 parts it lost.

 Exported so a control can hand it the shape the kernel answers, which no fixture on an ordinary filesystem can make: a
 directory there answers `ENOENT` for a file it does not hold, and only a control group being removed answers `ENODEV`.

 @param error - the caught value

 @returns true when it carries the code `ENODEV`

 @example
 ```ts
 isRemoved(Object.assign(new Error('ENODEV: no such device, read \'/sys/fs/cgroup/a.scope/pids.max\''), { code: 'ENODEV' }));
 ```
 */
export function isRemoved(error: unknown,): boolean {
  return (Error.isError(error,))
    && ('code' in error)
    && (error.code === 'ENODEV');
}

//endregion The kernel's answer for a file it is removing
