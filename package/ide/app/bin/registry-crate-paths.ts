// The crates an executable names through their registry source paths, read with a cursor scan instead of a regular
// expression.
// What: every `name version` pair that a `/registry/src/index.crates.io-<hash>/<name>-<version>/` path in a binary
// names, in the order the paths appear.
// Why: cargo stores an unpacked crate under the registry folder and its own directory name, so a binary that names a
// source path also names the crate that path belongs to. The bundle check uses that to confirm every crate whose source
// the executable names has a license text embedded with it.

/// The path segment every registry crate source sits below, hash folder included up to its separating hyphen.
const REGISTRY_SOURCE_PREFIX = '/registry/src/index.crates.io-';

/// The character that separates a crate name from its version in the registry folder's directory name.
const NAME_VERSION_SEPARATOR = '-';

/**
 Whether one UTF-16 code unit is a lowercase hexadecimal digit, the only shape a registry folder's hash takes.

 @param codeUnit - one code unit of the text
 @returns whether the character is `0` to `9` or `a` to `f`
 */
function isHashDigit(codeUnit: number): boolean {
  return (codeUnit >= 0x30 && codeUnit <= 0x39) || (codeUnit >= 0x61 && codeUnit <= 0x66);
}

/**
 Whether one code unit may appear in a crate name.

 @param codeUnit - one code unit of the text
 @returns whether the character is an ASCII letter, a digit, or one of `.`, `+`, `-`, `_`
 */
function isNameCharacter(codeUnit: number): boolean {
  return isVersionSuffixCharacter(codeUnit) || codeUnit === 0x5F;
}

/**
 Whether one code unit may appear in the `-`/`+` suffix a release carries.

 What: the name characters less `_`, which a crate name may hold and a release suffix may not.

 Why: a directory named `serde-1.0.219-_` must name nothing, so the suffix has to stop where the pattern's class
 stopped rather than swallow the underscore.

 @param codeUnit - one code unit of the text
 @returns whether the character is an ASCII letter, a digit, or one of `.`, `+`, `-`
 */
function isVersionSuffixCharacter(codeUnit: number): boolean {
  return (codeUnit >= 0x30 && codeUnit <= 0x39)
    || (codeUnit >= 0x41 && codeUnit <= 0x5A)
    || (codeUnit >= 0x61 && codeUnit <= 0x7A)
    || codeUnit === 0x2B
    || codeUnit === 0x2D
    || codeUnit === 0x2E;
}

/**
 Whether one code unit is an ASCII digit.

 @param codeUnit - one code unit of the text
 @returns whether the character is `0` to `9`
 */
function isDigit(codeUnit: number): boolean {
  return codeUnit >= 0x30 && codeUnit <= 0x39;
}

/**
 The end of the run of characters below `from` that `accepts` holds.

 @param text - text the run sits in
 @param from - first code unit of the run
 @param accepts - predicate one code unit answers
 @returns the index of the first code unit after the run, which is `from` when the run is empty
 */
function runEnd({ text, from, accepts }: { text: string; from: number; accepts: (codeUnit: number) => boolean }): number {
  let end = from;
  while (end < text.length && accepts(text.charCodeAt(end))) end += 1;
  return end;
}

/**
 The end of a `major.minor.patch` release followed by its optional `-`/`+` suffix, or a negative number when `from` holds
 none.

 What: three non-empty digit runs separated by dots, then a maximal run of suffix characters when the next code unit is
 `-` or `+`. A failure after the first field cannot be repaired by a shorter field, because the character a field must
 be followed by is never one a field holds.

 @param text - text the release sits in
 @param from - first code unit of the release
 @returns the index of the first code unit after the release, or a negative number
 */
function releaseEnd({ text, from }: { text: string; from: number }): number {
  let end = runEnd({ text, from, accepts: isDigit });
  if (end === from || text[end] !== '.') return -1;
  const minorStart = end + 1;
  end = runEnd({ text, from: minorStart, accepts: isDigit });
  if (end === minorStart || text[end] !== '.') return -1;
  const patchStart = end + 1;
  end = runEnd({ text, from: patchStart, accepts: isDigit });
  if (end === patchStart) return -1;
  const suffix = text.charCodeAt(end);
  if (suffix === 0x2B || suffix === 0x2D) end = runEnd({ text, from: end + 1, accepts: isVersionSuffixCharacter });
  return end;
}

/**
 The crate one registry source path names, starting at its hash folder, or nothing when the text there names none.

 What: the hash folder is one or more lowercase hexadecimal digits and a `/`; the directory name after it is a crate name
 and a release divided by a hyphen. The name is the shortest one whose release reads and which a `/` closes, which is
 the split the lazy name in the pattern the scan replaced reached first.

 @param text - text one path sits in
 @param hashStart - first code unit of the hash folder's name
 @returns the crate and the index after the closing `/`, or nothing
 */
function crateAt({ text, hashStart }: { text: string; hashStart: number }): { name: string; version: string; end: number } | undefined {
  const hashEnd = runEnd({ text, from: hashStart, accepts: isHashDigit });
  if (hashEnd === hashStart || text[hashEnd] !== '/') return undefined;
  const directoryStart = hashEnd + 1;
  if (!isNameCharacter(text.charCodeAt(directoryStart))) return undefined;
  for (let hyphen = directoryStart + 1; hyphen < text.length; hyphen += 1) {
    const codeUnit = text.charCodeAt(hyphen);
    if (!isNameCharacter(codeUnit)) return undefined;
    if (text[hyphen] !== NAME_VERSION_SEPARATOR) continue;
    const versionStart = hyphen + 1;
    const versionEnd = releaseEnd({ text, from: versionStart });
    if (versionEnd < 0 || text[versionEnd] !== '/') continue;
    return { name: text.slice(directoryStart, hyphen), version: text.slice(versionStart, versionEnd), end: versionEnd + 1 };
  }
  return undefined;
}

/**
 Every crate whose registry source path `text` holds, in the order the paths appear.

 What: one pass with a moving cursor: each path starts at the registry prefix, a failure resumes at the next code unit of
 the text, and a match resumes after the `/` that closes it, so nothing is rescanned.

 Why: a binary names the source of every crate it was built from, and the pattern this replaces found the same paths
 leftmost first.

 @param text - one code unit per byte, as `Buffer#toString('latin1')` reads a binary
 @returns the named crates, duplicates included, in the order they appear
 */
export function registryCratesNamedIn(text: string): { name: string; version: string }[] {
  const crates: { name: string; version: string }[] = [];
  let search = 0;
  while (search < text.length) {
    const prefixStart = text.indexOf(REGISTRY_SOURCE_PREFIX, search);
    if (prefixStart < 0) break;
    const crate = crateAt({ text, hashStart: prefixStart + REGISTRY_SOURCE_PREFIX.length });
    if (crate === undefined) {
      search = prefixStart + 1;
      continue;
    }
    crates.push({ name: crate.name, version: crate.version });
    search = crate.end;
  }
  return crates;
}
