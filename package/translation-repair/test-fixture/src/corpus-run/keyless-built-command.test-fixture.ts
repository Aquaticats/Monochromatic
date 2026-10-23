import { join, } from 'node:path';

//region Keyless built command
// WHAT AN AS-BUILT SUITE SHARES AROUND ITS CHILD: the places a runner reads
// from the operator's machine, pointed at directories the case owns, and the
// whole stderr a runner leaves when it reaches its first call with no provider
// key set.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. No child is started here. A case starts
// its child with `runBuiltCommand` of `child-environment.test-fixture.ts`,
// which removes every provider key and every setting of the package from the
// child's environment, so a case can read what the real command prints and how
// it exits without any chance of a call leaving the machine, whatever the
// parent's own environment holds.

/**
 The whole stderr a runner leaves when it reaches its first call with no
 provider key set: the stated refusal `createRunClient` raises, as
 `reportingRefusals` prints it.

 @param command - name of the built runner, which starts the line

 @returns The refusal line with its newline

 @example
 ```ts
 expect(run.stderr,).toBe(noKeyRefusal({ command: 'probe-sensitivity', },),);
 ```
 */
export function noKeyRefusal({ command, }: { readonly command: string; },): string {
  return `${command}: TRANSLATION_REPAIR_NEURALWATT_API_KEY, TRANSLATION_REPAIR_SYNTHETIC_API_KEY, `
    + 'TRANSLATION_REPAIR_CHARM_HYPER_API_KEY, TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY, DEEPSEEK_API_KEY, '
    + 'TRANSLATION_REPAIR_XIAOMI_MIMO_API_KEY, TRANSLATION_REPAIR_CLAUDE_API_KEY '
    + 'or TRANSLATION_REPAIR_OPENROUTER_API_KEY is not set; '
    + 'run under mise so sops injects it\n';
}

/**
 The three places a runner reads from the operator's machine, each pointed at
 a directory the case owns.

 @param runsDir - directory the child treats as its runs directory

 @param scratchDir - directory the child's lookup cache and corpus clone
 variables point under, so neither names the operator's own

 @returns Variables to hand `runBuiltCommand` as its `env`

 @example
 ```ts
 const run = await runBuiltCommand({
   command: 'probe-sensitivity',
   packageEntry: import.meta.resolve('@monochromatic-dev/translation-repair-probe'),
   env: scratchPlaces({ runsDir: scratch.path, scratchDir: scratch.path, },),
 },);
 ```
 */
export function scratchPlaces(
  {
    runsDir,
    scratchDir,
  }: {
    readonly runsDir: string;
    readonly scratchDir: string;
  },
): Readonly<Record<string, string>> {
  return {
    TRANSLATION_REPAIR_RUNS_DIR: runsDir,
    TRANSLATION_REPAIR_LOOKUP_CACHE_DIR: join(
      scratchDir,
      'lookup-cache',
    ),
    TRANSLATION_REPAIR_CORPUS_CLONE_DIR: join(
      scratchDir,
      'corpus-clone',
    ),
  };
}

//endregion Keyless built command
