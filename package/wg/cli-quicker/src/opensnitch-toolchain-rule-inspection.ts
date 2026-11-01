import { constants, } from 'node:fs';
import {
  copyFile,
  mkdir,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  isOwnedRule,
  isRecord,
  readRuleDirectory,
  RULE_FILE_SUFFIX,
  type RuleRecord,
} from './opensnitch-toolchain-rule-directory.ts';
import {
  TOOLCHAIN_RULE_NAME_PREFIX,
  type ToolchainRuleRoots,
  toolchainRuleSpecs,
} from './opensnitch-toolchain-rule-patterns.ts';

/**
 Sentinel for a rule that does not block one exact process path.
 */
const NOT_AN_EXACT_PATH_BLOCK: unique symbol = Symbol('rule does not block one exact process path',);

/**
 A rule that blocks traffic from a path the allowance covers.
 */
export type SupersededRejectRule = {
  /**
   Rule name,
   equal to the file stem.
   */
  readonly name: string;

  /**
   Rule file path.
   */
  readonly path: string;

  /**
   The covered path the rule compares.
   */
  readonly processPath: string;

  /**
   Name of the allowance rule that covers `processPath`.
   */
  readonly coveredBy: string;
};

/**
 A rule sorted before the allowance that could still win over it.
 */
export type EarlierWinningRule = {
  /**
   Rule name.
   */
  readonly name: string;

  /**
   Why the daemon would stop at it first.
   */
  readonly reason: 'precedence' | 'reject' | 'deny';
};

/**
 What the allowance and the surrounding rules look like on disk.
 */
export type ToolchainRulesInspection = {
  /**
   Owned rule names present.
   */
  readonly present: readonly string[];

  /**
   Rule names this run would install that are absent.
   */
  readonly missing: readonly string[];

  /**
   Auto-saved reject or deny rules for covered paths,
   which the allowance already overrides.
   */
  readonly superseded: readonly SupersededRejectRule[];

  /**
   Enabled rules sorting before the allowance that are rejects,
   denies,
   or precedence rules.
   */
  readonly earlierWinners: readonly EarlierWinningRule[];

  /**
   Rule files that could not be parsed.
   */
  readonly unreadable: readonly string[];
};

/**
 Reads the exact process path an enabled reject or deny rule compares.

 @param rule - Parsed rule.

 @returns The compared path,
 or the sentinel when the rule has another shape.

 @example
 ```ts
 exactBlockedPath({ rule });
 ```
 */
function exactBlockedPath(
  { rule, }: { readonly rule: RuleRecord; },
): string | typeof NOT_AN_EXACT_PATH_BLOCK {
  if (rule.enabled !== true)
    return NOT_AN_EXACT_PATH_BLOCK;
  if ((rule.action !== 'reject') && (rule.action !== 'deny'))
    return NOT_AN_EXACT_PATH_BLOCK;
  /**
   Operator comparing the connection.
   */
  const { operator, } = rule;
  if (!isRecord(operator,))
    return NOT_AN_EXACT_PATH_BLOCK;
  if ((operator.type !== 'simple') || (operator.operand !== 'process.path'))
    return NOT_AN_EXACT_PATH_BLOCK;
  return (typeof operator.data) === 'string' ? operator.data : NOT_AN_EXACT_PATH_BLOCK;
}

/**
 Names the reason a rule sorting before the allowance would be reached first.

 @param rule - Parsed rule sorting before the allowance.

 @returns Reason,
 or an empty list for an ordinary non-precedence allow.

 @example
 ```ts
 earlierWinnerReasons({ rule });
 ```
 */
function earlierWinnerReasons(
  { rule, }: { readonly rule: RuleRecord; },
): readonly EarlierWinningRule['reason'][] {
  if (rule.enabled !== true)
    return [];
  if (rule.action === 'reject')
    return ['reject',];
  if (rule.action === 'deny')
    return ['deny',];
  return rule.precedence === true ? ['precedence',] : [];
}

/**
 Reports the allowance's state without changing any file.

 @param rulesDirectory - Daemon rules directory.

 @param roots - Spellings of each covered directory.

 @returns Present,
 missing,
 superseded,
 and earlier-winning rules.

 @example
 ```ts
 await inspectToolchainRules({ rulesDirectory, roots });
 ```
 */
export async function inspectToolchainRules(
  {
    rulesDirectory,
    roots,
  }: {
    readonly rulesDirectory: string;
    readonly roots: ToolchainRuleRoots;
  },
): Promise<ToolchainRulesInspection> {
  /**
   Specs this run would install.
   */
  const specs = toolchainRuleSpecs({ roots, },);
  /**
   Files in the directory.
   */
  const entries = await readRuleDirectory({ rulesDirectory, },);
  /**
   Rules the daemon would consider besides the allowance's own.
   */
  const foreign = entries.filter(function notOwned(entry,): boolean {
    return !isOwnedRule({ entry, },);
  },);
  /**
   Covered blocks.
   */
  const superseded = foreign.flatMap(function coveredBlock(entry,): readonly SupersededRejectRule[] {
    if (entry.rule === undefined)
      return [];
    /**
     Compared path of a reject or deny rule on one exact path.
     */
    const blockedPath = exactBlockedPath({ rule: entry.rule, },);
    if (blockedPath === NOT_AN_EXACT_PATH_BLOCK)
      return [];
    /**
     Allowance rule covering the blocked path.
     */
    const covering = specs.find(function covers(spec,): boolean {
      return spec.covers(blockedPath,);
    },);
    if (covering === undefined)
      return [];
    return [
      {
        name: entry.stem,
        path: entry.path,
        processPath: blockedPath,
        coveredBy: covering.name,
      },
    ];
  },);
  /**
   Rules the daemon would reach before the allowance and stop at.
   */
  const earlierWinners = foreign.flatMap(function earlierWinner(entry,): readonly EarlierWinningRule[] {
    if ((entry.rule === undefined) || (entry.stem >= TOOLCHAIN_RULE_NAME_PREFIX))
      return [];
    return earlierWinnerReasons({ rule: entry.rule, },)
      .map(function named(reason,): EarlierWinningRule {
        return {
          name: entry.stem,
          reason,
        };
      },);
  },);
  /**
   Owned names present on disk.
   */
  const present = entries
    .filter(function owns(entry,): boolean {
      return isOwnedRule({ entry, },);
    },)
    .map(function stemOf(entry,): string {
      return entry.stem;
    },);
  return {
    present,
    missing: specs
      .map(function nameOf(spec,): string {
        return spec.name;
      },)
      .filter(function absent(name,): boolean {
        return !present.includes(name,);
      },),
    superseded,
    earlierWinners,
    unreadable: entries
      .filter(function unparsed(entry,): boolean {
        return entry.rule === undefined;
      },)
      .map(function pathOf(entry,): string {
        return entry.path;
      },),
  };
}

/**
 Removes the auto-saved reject and deny rules for covered paths after backing each up.

 A removal needs no rename out of the rules directory: the daemon unloads a rule on a `Remove` event,
 so the file is copied to the backup directory first and then unlinked.

 @param rulesDirectory - Daemon rules directory.

 @param roots - Spellings of each covered directory.

 @param backupDirectory - Directory receiving a copy of each removed file.

 @param dryRun - Whether to list without copying or removing.

 @returns The rules removed,
 or that would be removed in a dry run.

 @example
 ```ts
 await removeSupersededRejects({ rulesDirectory, roots, backupDirectory, dryRun: true });
 ```
 */
export async function removeSupersededRejects(
  {
    rulesDirectory,
    roots,
    backupDirectory,
    dryRun,
  }: {
    readonly rulesDirectory: string;
    readonly roots: ToolchainRuleRoots;
    readonly backupDirectory: string;
    readonly dryRun: boolean;
  },
): Promise<readonly SupersededRejectRule[]> {
  /**
   Rules the allowance already overrides.
   */
  const { superseded, } = await inspectToolchainRules({
    rulesDirectory,
    roots,
  },);
  if (dryRun || (superseded.length === 0))
    return superseded;
  await mkdir(
    backupDirectory,
    {
      recursive: true,
      mode: 0o700,
    },
  );
  for (const rule of superseded) {
    // Each removal follows its own backup so an interrupted run never loses a rule without a copy.
    // oxlint-disable-next-line eslint/no-await-in-loop -- The copy must finish before the original is unlinked.
    await copyFile(
      rule.path,
      join(
        backupDirectory,
        `${rule.name}${RULE_FILE_SUFFIX}`,
      ),
      // Fail rather than overwrite a backup from an earlier run.
      constants.COPYFILE_EXCL,
    );
    // oxlint-disable-next-line eslint/no-await-in-loop -- The unlink follows its own backup.
    await rm(rule.path,);
  }
  return superseded;
}
