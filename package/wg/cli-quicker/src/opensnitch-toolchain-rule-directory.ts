import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { OpenSnitchConfigError, } from './errors.ts';
import {
  TOOLCHAIN_RULE_DESCRIPTION_PREFIX,
  TOOLCHAIN_RULE_NAME_PREFIX,
} from './opensnitch-toolchain-rule-patterns.ts';

/**
 Module logger for rules-directory reads.
 */
const l = tagged({ tag: 'opensnitch-toolchain-rule-directory', },);

/**
 Rule files end with this suffix;
 the daemon ignores everything else in its rules directory.
 */
export const RULE_FILE_SUFFIX = '.json';

/**
 Parsed JSON object of one rule file.
 */
export type RuleRecord = Readonly<Record<string, unknown>>;

/**
 One rule file read from the rules directory.
 */
export type RuleDirectoryEntry = {
  /**
   File stem,
   which the daemon uses to unload a deleted rule.
   */
  readonly stem: string;

  /**
   Absolute file path.
   */
  readonly path: string;

  /**
   Parsed JSON object,
   absent when the file is unreadable or not an object.
   */
  readonly rule?: RuleRecord;

  /**
   Byte length of the file text,
   zero when the file could not be read.
   */
  readonly size: number;
};

/**
 Reports whether a value is a non-null object.

 @param value - Unknown JSON value.

 @returns Whether the value is record-like.

 @example
 ```ts
 isRecord({ name: 'x' });
 ```
 */
export function isRecord(value: unknown,): value is RuleRecord {
  if ((typeof value) !== 'object')
    return false;
  if (value === null)
    return false;
  return !Array.isArray(value,);
}

/**
 Reads and parses one rule file.

 @param rulesDirectory - Daemon rules directory.

 @param name - File name including the suffix.

 @returns Entry whose `rule` is absent when the file cannot be parsed into an object.

 @example
 ```ts
 await readRuleEntry({ rulesDirectory: '/etc/opensnitchd/rules', name: 'x.json' });
 ```
 */
async function readRuleEntry(
  {
    rulesDirectory,
    name,
  }: {
    readonly rulesDirectory: string;
    readonly name: string;
  },
): Promise<RuleDirectoryEntry> {
  /**
   Absolute rule file path.
   */
  const path = join(
    rulesDirectory,
    name,
  );
  /**
   File stem the daemon derives the rule name from.
   */
  const stem = name.slice(
    0,
    -RULE_FILE_SUFFIX.length,
  );
  try {
    /**
     File text kept for its byte length.
     */
    const text = await readFile(
      path,
      'utf8',
    );
    /**
     Parsed JSON value.
     */
    const parsed: unknown = JSON.parse(text,);
    return {
      stem,
      path,
      size: Buffer.byteLength(
        text,
        'utf8',
      ),
      ...(isRecord(parsed,) ? { rule: parsed, } : {}),
    };
  }
  catch (error) {
    l.warn(`Cannot read rule ${path}: ${String(error,)}`,);
    return {
      stem,
      path,
      size: 0,
    };
  }
}

/**
 Reads every rule file in a directory.

 @param rulesDirectory - Daemon rules directory.

 @returns Entries sorted by file name,
 a malformed file carrying no `rule`.

 @throws {@link OpenSnitchConfigError} when the directory cannot be listed.

 @example
 ```ts
 await readRuleDirectory({ rulesDirectory: '/etc/opensnitchd/rules' });
 ```
 */
export async function readRuleDirectory(
  { rulesDirectory, }: { readonly rulesDirectory: string; },
): Promise<readonly RuleDirectoryEntry[]> {
  /**
   Directory listing.
   */
  const names = await (async function listNames(): Promise<readonly string[]> {
    try {
      return await readdir(rulesDirectory,);
    }
    catch (error) {
      throw new OpenSnitchConfigError(
        `Cannot list OpenSnitch rules directory: ${rulesDirectory}`,
        { cause: error, },
      );
    }
  })();
  return Promise.all(
    names
      .filter(function isRuleFile(name,): boolean {
        return name.endsWith(RULE_FILE_SUFFIX,);
      },)
      .toSorted()
      .map(function readNamed(name,): Promise<RuleDirectoryEntry> {
        return readRuleEntry({
          rulesDirectory,
          name,
        },);
      },),
  );
}

/**
 Reports whether a rule file was written by this allowance.

 @param entry - Rule file.

 @returns Whether name prefix and description prefix both match.

 @example
 ```ts
 isOwnedRule({ entry });
 ```
 */
export function isOwnedRule({ entry, }: { readonly entry: RuleDirectoryEntry; },): boolean {
  if (entry.rule === undefined)
    return false;
  if (!entry.stem
    .startsWith(TOOLCHAIN_RULE_NAME_PREFIX,))
    return false;
  /**
   Description text the allowance prefixes.
   */
  const { description, } = entry.rule;
  return (typeof description) === 'string'
    ? description.startsWith(TOOLCHAIN_RULE_DESCRIPTION_PREFIX,)
    : false;
}
