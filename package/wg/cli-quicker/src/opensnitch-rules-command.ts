import { stat, } from 'node:fs/promises';
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
  REPOSITORY_ROOT_ABSENT,
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
 Options the subcommand accepts.
 */
const OPENSNITCH_RULES_OPTIONS = {
  'rules-directory': { type: 'string', },
  'repository-root': {
    type: 'string',
    multiple: true,
  },
  'data-home': { type: 'string', },
  'cargo-home': { type: 'string', },
  'rustup-home': { type: 'string', },
  'backup-directory': { type: 'string', },
  'dry-run': { type: 'boolean', },
} as const;

/**
 Argument-parser configuration whose result type the subcommand reads.
 */
type OpenSnitchRulesParseConfig = {
  readonly args: string[];
  readonly allowPositionals: true;
  readonly options: typeof OPENSNITCH_RULES_OPTIONS;
};

/**
 Actions the subcommand accepts.
 */
type RulesAction = 'apply' | 'status' | 'remove' | 'remove-superseded';

/**
 Validated command line.
 */
export type OpenSnitchRulesArgs = {
  /**
   What to do with the daemon rules.
   */
  readonly action: RulesAction;

  /**
   Daemon rules directory,
   defaulting to the daemon config's `Rules.Path`.
   */
  readonly rulesDirectory?: string;

  /**
   Main checkout roots whose `.cache` holds exposure links;
   empty means the checkout containing this bundle.
   */
  readonly repositoryRoots: readonly string[];

  /**
   XDG data home when not `<home>/.local/share`.
   */
  readonly dataHome?: string;

  /**
   Cargo home when not `<home>/.cargo`.
   */
  readonly cargoHome?: string;

  /**
   Rustup home when not `<home>/.rustup`.
   */
  readonly rustupHome?: string;

  /**
   Directory receiving copies of removed rules.
   */
  readonly backupDirectory?: string;

  /**
   Whether to list without changing anything.
   */
  readonly dryRun: boolean;
};

/**
 Requires a path option to be absolute,
 so a privileged run and the caller resolve the same directory.

 @param name - Option name for the diagnostic.

 @param value - Supplied value.

 @returns The same value.

 @throws {@link CliUsageError} when the value is relative.

 @example
 ```ts
 requireAbsolute({ name: '--rules-directory', value: '/etc/opensnitchd/rules' });
 ```
 */
function requireAbsolute(
  {
    name,
    value,
  }: {
    readonly name: string;
    readonly value: string;
  },
): string {
  if (!isAbsolute(value,))
    throw new CliUsageError(`${name} must be an absolute path: ${value}`,);
  return value;
}

/**
 Splits the arguments after `opensnitch-rules` into positionals and options.

 @param argv - Arguments after the subcommand name.

 @returns Parsed positionals and option values.

 @throws {@link CliUsageError} when an option is unrecognized or lacks a value.

 @example
 ```ts
 readOpenSnitchRulesTokens({ argv: ['apply', '--dry-run',] });
 ```
 */
function readOpenSnitchRulesTokens(
  { argv, }: { readonly argv: readonly string[]; },
): ReturnType<typeof parseArgs<OpenSnitchRulesParseConfig>> {
  try {
    return parseArgs({
      args: [...argv,],
      allowPositionals: true,
      options: OPENSNITCH_RULES_OPTIONS,
    },);
  }
  catch (error) {
    l.error(`argument error: ${String(error,)}`,);
    throw new CliUsageError(
      USAGE,
      { cause: error, },
    );
  }
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
   Positionals and typed option values.
   */
  const {
    positionals,
    values,
  } = readOpenSnitchRulesTokens({ argv, },);
  /**
   Positional action and any surplus words.
   */
  const [action, ...extra] = positionals;
  if ((action !== 'apply') && (action !== 'status')
    && (action !== 'remove')
    && (action !== 'remove-superseded'))
    throw new CliUsageError(USAGE,);
  if (extra.length > 0)
    throw new CliUsageError(USAGE,);
  return {
    action,
    repositoryRoots: (values['repository-root'] ?? []).map(function checkRoot(value,): string {
      return requireAbsolute({
        name: '--repository-root',
        value,
      },);
    },),
    dryRun: values['dry-run'] === true,
    ...(values['rules-directory'] === undefined
      ? {}
      : { rulesDirectory: requireAbsolute({
        name: '--rules-directory',
        value: values['rules-directory'],
      },), }),
    ...(values['data-home'] === undefined
      ? {}
      : { dataHome: requireAbsolute({
        name: '--data-home',
        value: values['data-home'],
      },), }),
    ...(values['cargo-home'] === undefined
      ? {}
      : { cargoHome: requireAbsolute({
        name: '--cargo-home',
        value: values['cargo-home'],
      },), }),
    ...(values['rustup-home'] === undefined
      ? {}
      : { rustupHome: requireAbsolute({
        name: '--rustup-home',
        value: values['rustup-home'],
      },), }),
    ...(values['backup-directory'] === undefined
      ? {}
      : { backupDirectory: requireAbsolute({
        name: '--backup-directory',
        value: values['backup-directory'],
      },), }),
  };
}

/**
 Refuses to derive the covered directories from a home that is not the sudo caller's.

 A direct `sudo wg-quicker opensnitch-rules ...` carries no caller context,
 so `HOME` is root's and the rules would silently cover `/root/.local/share/...` instead of the user's directories.
 Running `wg-quicker opensnitch-rules ...` as the user avoids that,
 because the self-elevation carries the caller's home across `sudo`.

 @param home - Home directory the defaults come from.

 @param homeOwnerUid - Owner of that directory.

 @param effectiveUid - Effective UID of this process.

 @param sudoUid - `SUDO_UID` when sudo started this process.

 @param explicitHomes - Whether the data, Cargo and rustup homes were all given on the command line.

 @throws {@link CliUsageError} when root runs on behalf of a sudo caller whose home this is not.

 @example
 ```ts
 assertHomeBelongsToCaller({ home: '/root', homeOwnerUid: 0, effectiveUid: 0, sudoUid: '1000', explicitHomes: false });
 ```
 */
export function assertHomeBelongsToCaller(
  {
    home,
    homeOwnerUid,
    effectiveUid,
    sudoUid,
    explicitHomes,
  }: {
    readonly home: string;
    readonly homeOwnerUid: number;
    readonly effectiveUid: number;
    readonly sudoUid?: string;
    readonly explicitHomes: boolean;
  },
): void {
  if ((effectiveUid !== 0) || explicitHomes
    || (sudoUid === undefined))
    return;
  if (Number(sudoUid,) === homeOwnerUid)
    return;
  throw new CliUsageError(
    `HOME is ${home}, owned by uid ${String(homeOwnerUid,)}, but sudo was started by uid ${sudoUid}. `
      + 'Run `wg-quicker opensnitch-rules ...` as that user so it carries the caller home across sudo, '
      + 'or pass --data-home, --cargo-home and --rustup-home.',
  );
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
  process.stdout
    .write(`${line}\n`,);
}

/**
 Finds the checkout containing this bundle when the command line names none.

 @returns The checkout root as a one-element list,
 or an empty list with a warning when this bundle is outside a checkout.

 @example
 ```ts
 await bundleRepositoryRoots();
 ```
 */
async function bundleRepositoryRoots(): Promise<readonly string[]> {
  /**
   Checkout found above this module.
   */
  const root = await findRepositoryRoot({ start: import.meta.dirname, },);
  if (root === REPOSITORY_ROOT_ABSENT) {
    l.warn(
      'No checkout root was found above this bundle and none was given; '
        + 'the exposure-link rule is skipped. Pass --repository-root <dir> to add it.',
    );
    return [];
  }
  return [root,];
}

/**
 Prints the allowance's state and the rules around it.

 @param rulesDirectory - Daemon rules directory.

 @param roots - Spellings of each covered directory.

 @example
 ```ts
 await printStatus({ rulesDirectory, roots });
 ```
 */
async function printStatus(
  {
    rulesDirectory,
    roots,
  }: {
    readonly rulesDirectory: string;
    readonly roots: Parameters<typeof inspectToolchainRules>[0]['roots'];
  },
): Promise<void> {
  /**
   State of the allowance and its neighbors.
   */
  const inspection = await inspectToolchainRules({
    rulesDirectory,
    roots,
  },);
  printLine({ line: `present: ${inspection.present
    .join(', ',)
    || 'none'}`, },);
  printLine({ line: `missing: ${inspection.missing
    .join(', ',)
    || 'none'}`, },);
  for (const rule of inspection.superseded) {
    printLine({
      line: `superseded reject or deny: ${rule.name} (${rule.processPath}) covered by ${rule.coveredBy}`,
    },);
  }
  for (const rule of inspection.earlierWinners)
    printLine({ line: `sorts before the allowance and can win (${rule.reason}): ${rule.name}`, },);
  for (const path of inspection.unreadable)
    printLine({ line: `unreadable rule file: ${path}`, },);
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
  const fl = tagged({
    tag: runOpenSnitchRulesCommand.name,
    l,
  },);
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
  const repositoryRoots = args.repositoryRoots
    .length
    > 0 ? args.repositoryRoots : await bundleRepositoryRoots();
  /**
   Home directory every default is derived from.
   */
  const home = process.env
    .HOME
    ?? homedir();
  assertHomeBelongsToCaller({
    home,
    homeOwnerUid: (await stat(home,)).uid,
    effectiveUid: process.geteuid?.() ?? 0,
    explicitHomes: (args.dataHome !== undefined) && (args.cargoHome !== undefined)
      && (args.rustupHome !== undefined),
    ...(process.env
      .SUDO_UID
      === undefined ? {} : { sudoUid: process.env
        .SUDO_UID, }),
  },);
  /**
   Spellings of every covered directory.
   */
  const roots = await resolveToolchainRuleRoots({
    options: {
      home,
      repositoryRoots,
      ...(args.dataHome === undefined ? {} : { dataHome: args.dataHome, }),
      ...(args.cargoHome === undefined ? {} : { cargoHome: args.cargoHome, }),
      ...(args.rustupHome === undefined ? {} : { rustupHome: args.rustupHome, }),
    },
  },);
  fl.debug(`${args.action} in ${rulesDirectory}`,);
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
      line: `created ${String(report.created
        .length,)}, updated ${String(report.updated
          .length,)}, `
        + `unchanged ${String(report.unchanged
          .length,)} in ${rulesDirectory}`,
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
    /**
     Verb for the report lines.
     */
    const verb = args.dryRun ? 'would remove' : 'removed';
    for (const rule of removed)
      printLine({ line: `${verb} ${rule.name} (${rule.processPath})`, },);
    printLine({
      line: `${verb} ${String(removed.length,)} superseded rule(s)${args.dryRun ? '' : `, backups in ${backupDirectory}`}`,
    },);
    return;
  }
  await printStatus({
    rulesDirectory,
    roots,
  },);
}
