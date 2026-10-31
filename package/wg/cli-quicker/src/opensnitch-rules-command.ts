import { homedir, } from 'node:os';
import {
  dirname,
  isAbsolute,
} from 'node:path';
import { parseArgs, } from 'node:util';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { CliUsageError, } from './errors.ts';
import { resolveOpenSnitchRulesDirectory, } from './opensnitch-daemon-config.ts';
import { toolchainRuleSpecs, } from './opensnitch-toolchain-rule-patterns.ts';
import {
  applyToolchainRules,
  findRepositoryRoot,
  inspectToolchainRules,
  removeSupersededRejects,
  removeToolchainRules,
  resolveToolchainRuleRoots,
} from './opensnitch-toolchain-rules.ts';

/**
 Module logger for the `opensnitch-rules` subcommand.
 */
const l = tagged({ tag: 'opensnitch-rules-command', },);

/**
 Usage text shown for any argument error.
 */
const USAGE = 'Usage: wg-quicker opensnitch-rules <apply|status|remove|remove-superseded> '
  + '[--rules-directory <dir>] [--repository-root <dir>]... [--data-home <dir>] [--cargo-home <dir>] '
  + '[--rustup-home <dir>] [--backup-directory <dir>] [--dry-run]';

/**
 Actions the subcommand accepts.
 */
type RulesAction = 'apply' | 'status' | 'remove' | 'remove-superseded';

/**
 Validated command line.
 */
export type OpenSnitchRulesArgs = {
  readonly action: RulesAction;
  readonly rulesDirectory: string | undefined;
  readonly repositoryRoots: readonly string[];
  readonly dataHome: string | undefined;
  readonly cargoHome: string | undefined;
  readonly rustupHome: string | undefined;
  readonly backupDirectory: string | undefined;
  readonly dryRun: boolean;
};

/**
 Requires each supplied path option to be absolute,
 so a privileged run and the caller resolve the same directory.

 @param name - Option name for the diagnostic.

 @param values - Supplied values.

 @returns The same values.

 @throws {@link CliUsageError} when a value is relative.

 @example
 ```ts
 requireAbsolute({ name: '--rules-directory', values: ['/etc/opensnitchd/rules',] });
 ```
 */
function requireAbsolute(
  {
    name,
    values,
  }: {
    readonly name: string;
    readonly values: readonly string[];
  },
): readonly string[] {
  for (const value of values) {
    if (!isAbsolute(value,))
      throw new CliUsageError(`${name} must be an absolute path: ${value}`,);
  }
  return values;
}

/**
 Parses the arguments after `opensnitch-rules`.

 @param argv - Arguments after the subcommand name.

 @returns Validated command line.

 @throws {@link CliUsageError} when the action is unknown,
 an option is unrecognized,
 or a path is relative.

 @example
 ```ts
 parseOpenSnitchRulesArgs({ argv: ['apply', '--dry-run',] });
 ```
 */
export function parseOpenSnitchRulesArgs({ argv, }: { readonly argv: readonly string[]; },): OpenSnitchRulesArgs {
  /**
   Parsed tokens or the parser's complaint.
   */
  const parsed = (function parse(): ReturnType<typeof parseArgs> {
    try {
      return parseArgs({
        args: [...argv,],
        allowPositionals: true,
        options: {
          'rules-directory': { type: 'string', },
          'repository-root': { type: 'string', multiple: true, },
          'data-home': { type: 'string', },
          'cargo-home': { type: 'string', },
          'rustup-home': { type: 'string', },
          'backup-directory': { type: 'string', },
          'dry-run': { type: 'boolean', },
        },
      },);
    }
    catch (error) {
      l.error(`argument error: ${String(error,)}`,);
      throw new CliUsageError(USAGE, { cause: error, },);
    }
  })();
  /**
   Positional action.
   */
  const [action, ...extra] = parsed.positionals;
  if ((action !== 'apply') && (action !== 'status') && (action !== 'remove') && (action !== 'remove-superseded'))
    throw new CliUsageError(USAGE,);
  if (extra.length > 0)
    throw new CliUsageError(USAGE,);
  /**
   Option values as the parser typed them.
   */
  const values = parsed.values as Readonly<Record<string, string | boolean | readonly string[] | undefined>>;
  /**
   Reads one optional single-valued path option.

   @param name - Option name without dashes.

   @returns The validated absolute path,
   or `undefined` when absent.
   */
  const single = function single(name: string,): string | undefined {
    /**
     Raw option value.
     */
    const value = values[name];
    if ((value === undefined) || ((typeof value) !== 'string'))
      return undefined;
    requireAbsolute({ name: `--${name}`, values: [value as string,], },);
    return value as string;
  };
  return {
    action,
    rulesDirectory: single('rules-directory',),
    repositoryRoots: requireAbsolute({
      name: '--repository-root',
      values: (values['repository-root'] as readonly string[] | undefined) ?? [],
    },),
    dataHome: single('data-home',),
    cargoHome: single('cargo-home',),
    rustupHome: single('rustup-home',),
    backupDirectory: single('backup-directory',),
    dryRun: values['dry-run'] === true,
  };
}

/**
 Writes one line of command output to standard output.

 @param line - Text without a newline.

 @example
 ```ts
 printLine({ line: 'created 000-allow-...' });
 ```
 */
function printLine({ line, }: { readonly line: string; },): void {
  process.stdout.write(`${line}\n`,);
}

/**
 Runs `opensnitch-rules`:
 installs,
 reports,
 or removes the daemon rules that let the toolchain's update traffic through OpenSnitch.

 The command must run as root because the daemon's rule files are mode `0600`.

 @param argv - Arguments after the subcommand name.

 @throws {@link CliUsageError} for a bad command line.

 @example
 ```ts
 await runOpenSnitchRulesCommand({ argv: ['status',] });
 ```
 */
export async function runOpenSnitchRulesCommand({ argv, }: { readonly argv: readonly string[]; },): Promise<void> {
  /**
   Function-scoped lifecycle logger.
   */
  const fl = tagged({ tag: runOpenSnitchRulesCommand.name, l, },);
  /**
   Validated command line.
   */
  const args = parseOpenSnitchRulesArgs({ argv, },);
  /**
   Directory the daemon watches.
   */
  const rulesDirectory = args.rulesDirectory ?? await resolveOpenSnitchRulesDirectory();
  /**
   Checkout roots from the command line,
   or the checkout containing this bundle.
   */
  const repositoryRoots = args.repositoryRoots.length > 0
    ? args.repositoryRoots
    : await (async function detectRoot(): Promise<readonly string[]> {
      /**
       Checkout found above this module.
       */
      const root = await findRepositoryRoot({ start: import.meta.dirname, },);
      return root === undefined ? [] : [root,];
    })();
  /**
   Spellings of every covered directory.
   */
  const roots = await resolveToolchainRuleRoots({
    options: {
      home: process.env.HOME ?? homedir(),
      ...(args.dataHome === undefined ? {} : { dataHome: args.dataHome, }),
      ...(args.cargoHome === undefined ? {} : { cargoHome: args.cargoHome, }),
      ...(args.rustupHome === undefined ? {} : { rustupHome: args.rustupHome, }),
      repositoryRoots,
    },
  },);
  fl.debug(`${args.action} in ${rulesDirectory}`,);
  if (repositoryRoots.length === 0) {
    l.warn(
      'No checkout root was found above this bundle and none was given; '
        + 'the exposure-link rule is skipped. Pass --repository-root <dir> to add it.',
    );
  }
  if (args.action === 'apply') {
    /**
     Files written or confirmed.
     */
    const report = await applyToolchainRules({
      rulesDirectory,
      roots,
      now: new Date(),
    },);
    for (const spec of toolchainRuleSpecs({ roots, },))
      printLine({ line: `${spec.name}: ${spec.pattern}`, },);
    printLine({
      line: `created ${String(report.created.length,)}, updated ${String(report.updated.length,)}, `
        + `unchanged ${String(report.unchanged.length,)} in ${rulesDirectory}`,
    },);
    return;
  }
  if (args.action === 'remove') {
    /**
     Names of the deleted allowance rules.
     */
    const removed = await removeToolchainRules({ rulesDirectory, },);
    printLine({ line: `removed ${String(removed.length,)} toolchain allowance rule(s) from ${rulesDirectory}`, },);
    return;
  }
  if (args.action === 'remove-superseded') {
    /**
     Backup directory beside the rules directory,
     outside the watched path.
     */
    const backupDirectory = args.backupDirectory ?? `${dirname(rulesDirectory,)}/rules-superseded-backup`;
    /**
     Rejects removed,
     or listed in a dry run.
     */
    const removed = await removeSupersededRejects({
      rulesDirectory,
      roots,
      backupDirectory,
      dryRun: args.dryRun,
    },);
    for (const rule of removed)
      printLine({ line: `${args.dryRun ? 'would remove' : 'removed'} ${rule.name} (${rule.processPath})`, },);
    printLine({
      line: `${args.dryRun ? 'would remove' : 'removed'} ${String(removed.length,)} superseded rule(s)`
        + (args.dryRun ? '' : `, backups in ${backupDirectory}`),
    },);
    return;
  }
  /**
   State of the allowance and its neighbors.
   */
  const inspection = await inspectToolchainRules({ rulesDirectory, roots, },);
  printLine({ line: `present: ${inspection.present.join(', ',) || 'none'}`, },);
  printLine({ line: `missing: ${inspection.missing.join(', ',) || 'none'}`, },);
  for (const rule of inspection.superseded)
    printLine({ line: `superseded reject or deny: ${rule.name} (${rule.processPath}) covered by ${rule.coveredBy}`, },);
  for (const rule of inspection.earlierWinners)
    printLine({ line: `sorts before the allowance and can win (${rule.reason}): ${rule.name}`, },);
  for (const path of inspection.unreadable)
    printLine({ line: `unreadable rule file: ${path}`, },);
}
