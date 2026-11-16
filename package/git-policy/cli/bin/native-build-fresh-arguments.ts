/**
 The command line of the build step: the profile, the executable, the package directory and the `cargo` to run.

 A task names none of these, so the defaults are the ones every task of this package means. Each flag is one long-form
 word with its value after it, so a caller reads the command line and no short flag hides what it does.

 In TS you'd write (pseudocode):
 ```ts
 // walk the words, one flag at a time, refusing what this step does not know
 ```
 */
import {
  defaultBinName,
  type Profile,
} from './native-build-fresh-steps.ts';

/**
 What one command line asked for.
 */
export type BuildFreshArguments = {
  /**
   The profile to build.
   */
  readonly profile: Profile;
  /**
   The executable to build.
   */
  readonly binName: string;
  /**
   The package directory to build in, which is this script's own package unless a caller names another.
   */
  readonly packageDirectory: string;
  /**
   The `cargo` to run.
   */
  readonly cargoProgram: string;
};

/**
 The text after one flag, refusing a flag left without one.

 @param words - the command line

 @param index - the flag's own position

 @param flag - the flag, for the failure that names it

 @returns the value and the position after it

 @throws Error when the flag has no value

 @example
 ```ts
 const [value, next] = valueAfter({ words: ['--bin', 'prog'], index: 0, flag: '--bin' });
 ```
 */
function valueAfter({
  words,
  index,
  flag,
}: {
  readonly words: readonly string[];
  readonly index: number;
  readonly flag: string;
}): readonly [
  string,
  number
] {
  /**
   The word after the flag, which is its value when there is one.
   */
  const value = words[index + 1];
  if ((value === undefined) || value.startsWith('--'))
    throw new Error(`${flag} needs a value after it, and the command line ended or named another flag`);
  return [
    value,
    index + 2
  ];
}

/**
 Reads one word of the command line, refusing a position no word is at.

 @param words - the command line

 @param index - the word's position

 @returns the word

 @throws Error when the position is past the last word, which the loop's own bound makes unreachable

 @example
 ```ts
 const word = wordAt({ words: ['--release'], index: 0 });
 ```
 */
function wordAt({
  words,
  index,
}: {
  readonly words: readonly string[];
  readonly index: number;
}): string {
  /**
   The word itself, which is there for every position the caller's loop allows.
   */
  const word = words[index];
  if (word === undefined)
    throw new Error('the command line ended where a word belongs');
  return word;
}

/**
 Tells whether one word is a flag that takes a value.

 @param word - the word

 @returns whether it is `--bin`, `--directory` or `--cargo`

 @example
 ```ts
 const takesOne = takesAValue('--bin');
 ```
 */
function takesAValue(word: string): boolean {
  return (word === '--bin') || (word === '--directory')
    || (word === '--cargo');
}

/**
 Reads the command line of this step.

 @param words - the arguments after the program's own name

 @param ownPackage - the package this script is written into, which is what its tasks build by default

 @returns what the command line asked for

 @throws Error when a word is not one this step knows or a flag has no value

 @example
 ```ts
 const asked = parseArguments({ words: ['--release'], ownPackage: '/repo/package/git-policy/cli' });
 ```
 */
export function parseArguments({
  words,
  ownPackage,
}: {
  readonly words: readonly string[];
  readonly ownPackage: string;
}): BuildFreshArguments {
  /**
   What the command line asks for, as each flag changes it.
   */
  const asked = {
    profile: 'dev' as Profile,
    binName: defaultBinName,
    packageDirectory: ownPackage,
    cargoProgram: 'cargo',
  };
  for (let index = 0; index < words.length;) {
    /**
     The word being read, which is one flag or its value.
     */
    const word = wordAt({
      words,
      index,
    },);
    if (word === '--release') {
      asked.profile = 'release';
      index += 1;
      continue;
    }
    if (!takesAValue(word))
      throw new Error(`this step knows no word ${word}; it takes --release, --bin <name>, --directory <path> and --cargo <program>`);
    /**
     The flag's own value and the position after it.
     */
    const [value, next] = valueAfter({
      words,
      index,
      flag: word,
    },);
    if (word === '--bin') asked.binName = value;
    if (word === '--directory') asked.packageDirectory = value;
    if (word === '--cargo') asked.cargoProgram = value;
    index = next;
  }
  return asked;
}
