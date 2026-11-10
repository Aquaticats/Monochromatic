// The version line the application prints for `--version`, read as the one question the bundle checks ask of it.
// What: whether a program's whole output opens with the application's own name and a three-part release number,
// `monochromatic-ide <major>.<minor>.<patch>`.
// Why: a copy of the executable alone in its own folder reports its version with no environment at all, which is how a
// check proves the copy starts and knows what it is; a line that opens with anything else, or that states fewer than
// three parts, is another program's answer or a release number the build never writes.
// The read is one linear pass and is tested in application-version-line.test.ts.

/**
 The label the application's version line opens with.
 */
const applicationLabel = 'monochromatic-ide ';

/**
 The parts a release number states, divided by dots.
 */
const releaseParts = 3;

/**
 Whether a character is an ASCII digit, the only kind a release part is written with.
 *
 @param character - one character of the output.
 *
 @returns whether the character is in `0` to `9`.
 */
const isDigit = (character: string): boolean => (character >= '0') && (character <= '9');

/**
 Whether `text` opens with the application's name and a three-part release number.
 *
 The character after the last part is not read, so a suffix such as `-beta` or a commit behind the number leaves the
 line a version line, exactly as a pattern anchored at the start and open at its end found it.
 *
 @param text - the program's whole output.
 *
 @returns whether the output opens with a version line of this application.
 */
export const namesApplicationVersion = (text: string): boolean => {
  if (!text.startsWith(applicationLabel)) {
    return false;
  }
  let at = applicationLabel.length;
  for (let part = 0; part < releaseParts; part++) {
    const partStart = at;
    while (isDigit(text[at] ?? '')) {
      at++;
    }
    if (at === partStart) {
      return false;
    }
    const last = part === releaseParts - 1;
    if (!last) {
      if (text[at] !== '.') {
        return false;
      }
      at++;
    }
  }
  return true;
};
