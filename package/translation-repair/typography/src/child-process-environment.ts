//region Child process environment
// THE OWNER'S RULE: no child process of production code is handed a
// credential, and none is handed the package's own settings. A child that
// needs neither (git reading a clone, a program reading a picture, the suite
// the coverage census runs) is started with the environment built here, so a
// key the run holds cannot leak through the child's own log, a crash dump, a
// tool's debug output, or a hook or alias a repository's configuration runs.
// `production-children-keyless.unit.test.ts` fails any start of a child that
// does not pass its `env` through this function.

/**
 Suffix of every variable that holds a credential: every key the package
 reads has a name ending so.
 */
const CREDENTIAL_SUFFIX = '_API_KEY';

/**
 Prefix of every variable the package reads for its own settings.
 */
const SETTING_PREFIX = 'TRANSLATION_REPAIR_';

/**
 Whether a variable's name says it holds a credential.

 @param name - variable's name, as the environment spells it

 @returns Whether the name ends in `_API_KEY`, anywhere in the environment and with
 any prefix, so `TRANSLATION_REPAIR_OPENROUTER_API_KEY` and `EXA_API_KEY` both are

 @example
 ```ts
 const secret = isCredentialName({ name: 'WHISKER_API_KEY', },); // true
 ```
 */
export function isCredentialName({ name, }: { readonly name: string; },): boolean {
  return name.endsWith(CREDENTIAL_SUFFIX,);
}

/**
 Environment for a child process of production code: the parent's, with every
 credential and every setting of the package stated as absent.

 A REMOVED NAME IS STATED AS `undefined`, NOT LEFT OUT. `nano-spawn` merges
 the object it is given over this process's own environment
 (`{ ...process.env, ...env }`, its `source/options.js`), so a name left out
 is put back from the parent; node's own `spawn` and `execFile` read an
 `undefined` value as absent. One shape therefore serves both, and a child
 sees none of the removed names.

 @param parent - environment the child would inherit, passed in so a case can
 hand an invented one and the helper never reads the process's

 @returns A new object holding every name of the parent, with the value of a
 name ending in `_API_KEY` or starting with `TRANSLATION_REPAIR_` replaced by
 `undefined`; the parent is never changed

 @example
 ```ts
 const env = childEnvironment({ parent: process.env, },);
 ```
 */
export function childEnvironment(
  { parent, }: { readonly parent: Readonly<NodeJS.ProcessEnv>; },
): NodeJS.ProcessEnv {
  /**
   The parent's variables, copied; each removed one stated as absent.
   */
  const child: NodeJS.ProcessEnv = { ...parent, };
  for (const name of Object.keys(child,)) {
    if (isCredentialName({ name, },) || name.startsWith(SETTING_PREFIX,))
      child[name] = undefined;
  }
  return child;
}

//endregion Child process environment
