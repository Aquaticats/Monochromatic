// The directory name the application derives from a project path, read with a code-unit walk.
// What: the shortened name of one project's basename, as the application puts it below its private state root.
// Why: a confined language server writes its private scratch directory inside that path, so the confinement check
// recomputes the name to find the directory a session left behind; a name derived differently makes the check read
// another project's state or none at all.
// The application's own derivation is `component` and `take(40)` in src/language/confine.rs, which replaces one
// Unicode scalar value at a time; the check below replaces one code unit at a time, exactly as the pattern it replaced
// did, so a project whose name holds a character outside the basic plane is named with two underscores here and one
// there. This reader answers the check's expression, not the application's.

import { basename } from 'node:path';

/// The longest name the application keeps, from `take(40)` in src/language/confine.rs.
const NAME_LENGTH_LIMIT = 40;

/// The character a name holds for one the directory cannot carry.
const REPLACEMENT = '_';

/**
 Whether one UTF-16 code unit stays in a state directory name.
 
 What: the ASCII letters, the ten ASCII digits, the dot, the underscore and the hyphen, the set `A-Za-z0-9._-`.
 
 Why: the name goes below the private state root beside a hash, so a character outside that set is replaced rather than
 kept, and no other character of the pattern's negated class survives.
 
 @param codeUnit - one code unit of the name, from `String#charCodeAt`
 @returns whether the character stands in the name unchanged
 */
function isKeptInStateName(codeUnit: number): boolean {
  return ((codeUnit >= 0x41) && (codeUnit <= 0x5a))
    || ((codeUnit >= 0x61) && (codeUnit <= 0x7a))
    || ((codeUnit >= 0x30) && (codeUnit <= 0x39))
    || (codeUnit === 0x2e)
    || (codeUnit === 0x5f)
    || (codeUnit === 0x2d);
}

/**
 The name the application derives for one project's state directory.
 
 What: the project path's last component with every character outside `A-Za-z0-9._-` replaced by one underscore, then
 shortened to the length the application keeps; the answer `basename(project).replace(/[^A-Za-z0-9._-]/g, '_')` and a
 `slice(0, 40)` gave for the same path.
 
 Why: one replaced character is one underscore rather than one per run, so the scan keeps the span between replaced
 characters and adds the underscores between the spans; the shortened result holds ASCII only, so the length is
 counted in characters.
 
 @param projectPath - the project directory a confined server analyzes
 @returns the shortened name of the project
 */
export function projectStateDirectoryName(projectPath: string): string {
  const name = basename(projectPath);
  const kept: string[] = [];
  let keptFrom = 0;
  for (let index = 0; index < name.length; index += 1) {
    if (isKeptInStateName(name.charCodeAt(index))) continue;
    kept.push(name.slice(keptFrom, index), REPLACEMENT);
    keptFrom = index + 1;
  }
  kept.push(name.slice(keptFrom));
  return kept.join('').slice(0, NAME_LENGTH_LIMIT);
}
