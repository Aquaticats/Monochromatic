// The `/proc` entry names that are process numbers, read with a character walk instead of a regular expression.
// What: whether one directory entry name of a `/proc` listing is a process number.
// Why: both lifecycle inspection scripts walk `/proc` for a language server and its sandbox, and every other entry
// there names a kernel view (`self`, `net`, `sys`, `uptime`) rather than a process. A process number is ASCII digits
// and nothing else, which a walk over the name answers in one pass.

/**
 Whether one `/proc` directory entry name is a process number.
 
 What: a character walk over ASCII digits, the answer an anchored `^\d+$` pattern gave to the same question: an empty
 name holds no digit and is not a number, and no character outside `0` to `9` is a digit, so a name holding one is other
 than a number.
 
 Why: a `/proc` listing is read by each tick of the lifetime watch, so the name is walked once and the walk stops at the
 first character that is not an ASCII digit, the same work the pattern's engine did without a second pass over the
 name.
 
 @param name - one entry name from a `readdirSync('/proc')` listing
 @returns whether every character of the name is an ASCII digit and the name is not empty
 */
export function isProcessNumberName(name: string): boolean {
  if (name === '') return false;
  for (let index = 0; index < name.length; index += 1) {
    const codeUnit = name.charCodeAt(index);
    if ((codeUnit < 0x30) || (codeUnit > 0x39)) return false;
  }
  return true;
}
