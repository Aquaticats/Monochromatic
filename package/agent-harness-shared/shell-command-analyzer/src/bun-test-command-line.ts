/**
 Reads one Bun command line as running the Bun test runner.

 `bun test` misleads every reader of this repository's test output,
 so the guard has to recognize the spellings the CLI accepts for that subcommand.
 The caller strips program wrappers first, which leaves the program word and its arguments here.

 The global options whose value is the next word are listed rather than inferred,
 because `bun --cwd /tmp test` puts `test` behind an option value
 while a boolean flag leaves the subcommand where a reader expects it.
 An option this list does not know may still take the next word,
 so a `test` word behind a word that is no known subcommand counts as the subcommand:
 refusing a command that does not reach the test runner costs a rewrite,
 while missing one that does reports `0 pass / 0 fail` as success.

 @module
 */

import { lastPathSegment, } from './executable-path-words.ts';

//region Constants

/**
 Executable whose test subcommand is refused, whatever path spells it.
 */
const BUN_EXECUTABLE_NAME = 'bun';

/**
 Subcommand word that runs the test runner.
 */
const BUN_TEST_SUBCOMMAND = 'test';

/**
 Word that ends option reading, after which every word is positional.
 */
const OPTION_TERMINATOR = '--';

/**
 Prefix that marks a word as an option rather than a positional word.
 */
const OPTION_PREFIX = '-';

/**
 Global options whose value is the next word, as `bun --help` prints it for Bun 1.4.4.

 Bun accepts the separated form for these (`bun --cwd /tmp -e '…'` runs),
 so their value never reads as the subcommand.
 */
const BUN_GLOBAL_OPTIONS_WITH_VALUE: ReadonlySet<string> = new Set([
  '-F',
  '-c',
  '-d',
  '-e',
  '-l',
  '-p',
  '-r',
  '--conditions',
  '--config',
  '--console-depth',
  '--cpu-prof-dir',
  '--cpu-prof-interval',
  '--cpu-prof-name',
  '--cron-period',
  '--cron-title',
  '--cwd',
  '--define',
  '--disable-warning',
  '--dns-result-order',
  '--drop',
  '--elide-lines',
  '--env-file',
  '--eval',
  '--extension-order',
  '--feature',
  '--fetch-preconnect',
  '--filter',
  '--heap-prof-dir',
  '--heap-prof-interval',
  '--heap-prof-name',
  '--import',
  '--inspect',
  '--inspect-brk',
  '--inspect-wait',
  '--install',
  '--jsx-factory',
  '--jsx-fragment',
  '--jsx-import-source',
  '--jsx-runtime',
  '--loader',
  '--main-fields',
  '--max-http-header-size',
  '--port',
  '--preload',
  '--print',
  '--redirect-warnings',
  '--require',
  '--shell',
  '--title',
  '--tsconfig-override',
  '--unhandled-rejections',
  '--user-agent',
  '--watch-kill-signal',
],);

/**
 Subcommands `bun --help` prints for Bun 1.4.4, including the aliases it names.

 A word from this set in the subcommand position proves the test runner never runs,
 which is what lets a `test` word behind it stay allowed.
 */
const BUN_SUBCOMMAND_WORDS: ReadonlySet<string> = new Set([
  'a',
  'add',
  'audit',
  'build',
  'c',
  'check',
  'create',
  'dedupe',
  'dev',
  'exec',
  'fmt',
  'i',
  'info',
  'init',
  'install',
  'link',
  'outdated',
  'patch',
  'pm',
  'prune',
  'publish',
  'repl',
  'rm',
  'remove',
  'run',
  'unlink',
  'update',
  'upgrade',
  'why',
  'x',
],);

//endregion Constants

//region Positional words

/**
 Collects the argument words that name positions rather than options.
 
 @param args - words after the program word
 
 @returns argument words with option words and their values dropped
 
 @example
 ```ts
 bunPositionalWords({ args: ['--cwd', '/tmp', 'test'], }); // ['test']
 ```
 */
function bunPositionalWords({ args, }: { readonly args: readonly string[]; },): readonly string[] {
  /**
   Argument words that name positions, in source order.
   */
  const positionals: string[] = [];

  for (let index = 0; index < args.length; index += 1) {
    /**
     Argument word at the current offset.
     */
    const word = args[index];
    if (word === undefined)
      break;
    if (word === OPTION_TERMINATOR) {
      positionals.push(...args.slice(index + 1,),);
      break;
    }
    if (word.startsWith(OPTION_PREFIX,)) {
      if (BUN_GLOBAL_OPTIONS_WITH_VALUE.has(word,))
        index += 1;
      continue;
    }
    positionals.push(word,);
  }

  return positionals;
}

//endregion Positional words

//region Predicate

/**
 Checks whether one Bun command line runs the test runner.
 
 @param words - program word followed by its arguments, with program wrappers already stripped
 
 @returns whether the program is Bun and its subcommand is `test`
 
 @example
 ```ts
 isBunTestCommandLine({ words: ['bun', 'test'], }); // true
 isBunTestCommandLine({ words: ['bun', 'run', 'test'], }); // false
 ```
 */
function isBunTestCommandLine({ words, }: { readonly words: readonly string[]; },): boolean {
  /**
   Program word of the command line.
   */
  const [program,] = words;
  if (program === undefined)
    return false;
  if (lastPathSegment(program,) !== BUN_EXECUTABLE_NAME)
    return false;

  /**
   Argument words that name positions rather than options.
   */
  const positionals = bunPositionalWords({ args: words.slice(1,), },);
  /**
   Word in the subcommand position.
   */
  const [subcommand, ...laterPositionals] = positionals;
  if (subcommand === BUN_TEST_SUBCOMMAND)
    return true;
  if (subcommand === undefined)
    return false;
  if (BUN_SUBCOMMAND_WORDS.has(subcommand,))
    return false;

  return laterPositionals.includes(BUN_TEST_SUBCOMMAND,);
}

//endregion Predicate

export { isBunTestCommandLine, };
