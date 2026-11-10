// The variable names the confinement check withholds from a confined language server, read as phrases.
// What: whether one variable name is one the check keeps out of a server's environment.
// Why: a server that reaches a token is a confinement failure, so the check names the variables it withholds, and a
// name is withheld when it holds one of the four words a credential is named with anywhere in it.

import { holdsIgnoringAsciiCase } from './ascii-case-fold.ts';

/// The words a credential's variable name holds, in lower case, as the pattern that named them spelled them.
const CREDENTIAL_WORDS = [
  'token',
  'key',
  'secret',
  'password'
] as const;

/**
 Whether a variable's name is one the confinement check withholds from a server.
 
 What: a search for each of the four credential words, in either case, anywhere in the name; the answer an unanchored
 `TOKEN|KEY|SECRET|PASSWORD` pattern carrying the `i` flag gave to the same name.
 
 Why: the words are not whole names, so `GITHUB_TOKEN`, `API_KEY` and `KEYS` are withheld too, and the scan reads the
 name the pattern read rather than a list of the variables one host happens to set.
 
 @param name - one variable name, from the keys of the process environment
 @returns whether the name holds one of the credential words
 */
export function isCredentialEnvironmentName(name: string): boolean {
  return CREDENTIAL_WORDS.some(word => holdsIgnoringAsciiCase(name, word));
}
