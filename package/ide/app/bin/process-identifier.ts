// The process folders of the /proc file system, read as the one question the bundle checks ask of them.
// What: whether a folder name below /proc is a process identifier, which is one or more ASCII digits.
// Why: the leftover sweep walks every entry of /proc and reads the process behind each, so an entry that is not a
// process would be read as one; the kernel names those folders with the process id alone, and the other entries there
// carry names such as `self`, `net` or `sysrq-trigger`.
// The predicate is one pass over the characters and is tested in process-identifier.test.ts.

/**
 Whether `name` is the name of a process folder below /proc: one or more ASCII digits, and nothing else.
 *
 The digits are the ASCII ones alone. A pattern's `\d` in this position reaches those, so an Arabic-Indic digit or a
 full-width one names no process here, exactly as the folder listing never holds one.
 *
 @param name - the /proc entry name to read.
 *
 @returns whether the name is a process identifier.
 */
export const isProcessIdentifier = (name: string): boolean => {
  if (name.length === 0) {
    return false;
  }
  for (const character of name) {
    if (character < '0' || character > '9') {
      return false;
    }
  }
  return true;
};
