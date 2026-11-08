/**
 What Pi starts with: the driver's own environment with every variable whose name marks a credential removed, so no command Pi
 runs inherits one.

 Pi hands each command the whole of its own environment (`getShellEnv` spreads `process.env`, `dist/utils/shell.js:147` in Pi
 1.5.0), and everything a command prints reaches a model provider. A sentence asking agents not to print it is not a bound; a
 variable missing from Pi's environment cannot be printed from it.

 What this bounds is inheritance, not access: the credential stores stay where they are (`auth.json`, gh's keyring, mise's own
 store), an ancestor process still holds the driver's own environment, and a command that reads a file can still find a secret.

 @module
 */

/**
 Words that mark a variable's name as holding a credential. A name holding any of them is removed, so a credential whose name
 fits the rule and nobody listed yet is removed too: the rule fails closed on names, and the allow list below is the only way
 back.

 The price is false positives, which the allow list pays for one name at a time: `SSH_AUTH_SOCK`, `ICEAUTHORITY`,
 `GIT_AUTHOR_NAME` and `STARSHIP_SESSION_KEY` all match and all go, because nothing an agent is asked to do here was shown to
 need them.
 */
export const CREDENTIAL_NAME_WORDS: readonly string[] = [
  'TOKEN',
  'SECRET',
  'KEY',
  'PASSWORD',
  'CREDENTIAL',
  'AUTH',
];

/**
 A variable kept although its name holds a credential word, with what fails without it.
 */
export type VariableKeptWithItsReason = {
  readonly name: string;

  /**
   What breaks when this variable is removed, measured rather than supposed.
   */
  readonly why: string;
};

/**
 The names the rule would remove and the driver keeps anyway. Each entry names a capability an agent is asked to exercise, so a
 reader can weigh it: dropping one costs that capability, keeping one costs the leak it holds.
 */
export const VARIABLES_KEPT_DESPITE_CREDENTIAL_NAME: readonly VariableKeptWithItsReason[] = [
  {
    name: 'XAUTHORITY',
    why:
      'The X server cookie file a GUI check reads. `xrandr --query` exits 0 with this variable and exits 1 with "Authorization required, but no authorization protocol specified" without it, and the browser checks this repository requires run against that display. Its value is a path, not a secret.',
  },
];

// The rule reads one name at a time, so it cannot keep a family together: `GIT_CONFIG_KEY_n` matches while
// `GIT_CONFIG_COUNT` and `GIT_CONFIG_VALUE_n` do not, and a command that builds Git configuration from those variables would
// find a pair missing. Nothing in this repository does, which is why the rule stays name-only.

/**
 What Pi gets, and the names that were taken from it.
 */
export type EnvironmentWithoutCredentials = {
  /**
   The environment Pi starts with.
   */
  readonly environment: NodeJS.ProcessEnv;

  /**
   The names removed, in order, so a feed line can say which variables a command may no longer read. Never a value.
   */
  readonly removedNames: readonly string[];
};

/**
 Says whether a variable's name marks it as holding a credential.

 @param name - the variable's name

 @returns whether the name holds one of the words, in any letter case

 @example
 ```ts
 carriesCredentialName({ name: 'MISE_GITHUB_TOKEN', },); // true
 ```
 */
export function carriesCredentialName({ name, }: { readonly name: string },): boolean {
  /**
   The name spelled as the words are, so a lower-case variable cannot hide from the rule.
   */
  const shouted = name.toUpperCase();
  return CREDENTIAL_NAME_WORDS.some(function appearsIn(word,) {
    return shouted.includes(word,);
  },);
}

/**
 Says whether the allow list names this variable.

 @param name - the variable's name

 @returns whether the allow list holds it
 */
function isKeptDespiteItsName(name: string,): boolean {
  return VARIABLES_KEPT_DESPITE_CREDENTIAL_NAME.some(function isThisOne(kept,) {
    return kept.name === name;
  },);
}

/**
 Takes every credential-named variable out of what Pi inherits, and reports which ones so the feed can name them.

 @param environment - the environment the driver itself runs with

 @returns what Pi starts with, and the names taken from it

 @example
 ```ts
 const { environment, removedNames } = stripCredentialsFromEnvironment({ environment: process.env, },);
 ```
 */
export function stripCredentialsFromEnvironment(
  { environment, }: { readonly environment: NodeJS.ProcessEnv },
): EnvironmentWithoutCredentials {
  /**
   The environment Pi gets.
   */
  const kept: NodeJS.ProcessEnv = {};
  /**
   The names taken from it.
   */
  const removedNames: string[] = [];
  for (const [name, value,] of Object.entries(environment,)) {
    if (carriesCredentialName({ name, },) && (!isKeptDespiteItsName(name,))) {
      removedNames.push(name,);
      continue;
    }
    kept[name] = value;
  }
  return {
    environment: kept,
    removedNames: removedNames.toSorted(),
  };
}
