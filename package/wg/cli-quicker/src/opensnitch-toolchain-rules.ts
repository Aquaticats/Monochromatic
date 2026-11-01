import {
  open,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { OpenSnitchConfigError, } from './errors.ts';
import {
  isOwnedRule,
  readRuleDirectory,
  RULE_FILE_SUFFIX,
} from './opensnitch-toolchain-rule-directory.ts';
import {
  rfc3339,
  toolchainRuleFile,
  type ToolchainRuleRoots,
  toolchainRuleSpecs,
} from './opensnitch-toolchain-rule-patterns.ts';

export {
  homeAliasPairs,
  directorySpellings,
  findRepositoryRoot,
  REPOSITORY_ROOT_ABSENT,
  resolveToolchainRuleRoots,
  type ToolchainRootOptions,
} from './opensnitch-toolchain-rule-roots.ts';
export {
  inspectToolchainRules,
  removeSupersededRejects,
  type SupersededRejectRule,
  type ToolchainRulesInspection,
} from './opensnitch-toolchain-rule-inspection.ts';

/**
 Renders a rule file exactly as the daemon would save it.

 @param rule - Rule object.

 @returns Indented JSON with a trailing newline.

 @example
 ```ts
 renderRuleText({ rule });
 ```
 */
function renderRuleText({ rule, }: { readonly rule: unknown; },): string {
  return `${
    JSON.stringify(
      rule,
      null,
      2,
    )
  }\n`;
}

/**
 Writes a rule through the file descriptor the daemon's watcher sees as `Write`.

 OpenSnitch 1.8 reloads a rule only on a filesystem `Write` event and unloads one only on `Remove`
 (`daemon/rule/loader.go` `liveReloadWorker`);
 a temporary file renamed into place raises `Create` alone and the daemon would ignore it until restart.
 A new file is created and written in place;
 an existing file is rewritten at offset zero and padded with spaces to its old length,
 so a shorter replacement never exposes a truncated document.

 @param path - Rule file path.

 @param text - Complete rule JSON.

 @param existingSize - Byte length of an existing file,
 absent for a new file.

 @example
 ```ts
 await writeRuleInPlace({ path, text });
 ```
 */
async function writeRuleInPlace(
  {
    path,
    text,
    existingSize,
  }: {
    readonly path: string;
    readonly text: string;
    readonly existingSize?: number;
  },
): Promise<void> {
  if (existingSize === undefined) {
    await writeFile(
      path,
      text,
      {
        flag: 'wx',
        mode: 0o600,
      },
    );
    return;
  }
  /**
   Replacement bytes padded to at least the old length.
   */
  const payload = Buffer.from(
    text.padEnd(
      existingSize,
      ' ',
    ),
    'utf8',
  );
  /**
   Watched inode held open for the positional write.
   */
  await using handle = await open(
    path,
    'r+',
  );
  /**
   Positional write result used to reject a partial rule.
   */
  const { bytesWritten, } = await handle.write(
    payload,
    0,
    payload.length,
    0,
  );
  if (bytesWritten !== payload.length)
    throw new OpenSnitchConfigError(`OpenSnitch rule write was incomplete: ${path}`,);
  await handle.sync();
}

/**
 Outcome of installing the allowance.
 */
export type ApplyToolchainRulesReport = {
  /**
   Rule names written as new files.
   */
  readonly created: readonly string[];

  /**
   Rule names rewritten because their content changed.
   */
  readonly updated: readonly string[];

  /**
   Rule names already identical on disk.
   */
  readonly unchanged: readonly string[];
};

/**
 Copies a rule without its `created` and `updated` fields.

 @param rule - Rule object.

 @returns Remaining fields in their original order.

 @example
 ```ts
 withoutTimestamps({ rule });
 ```
 */
function withoutTimestamps(
  { rule, }: { readonly rule: Readonly<Record<string, unknown>>; },
): Readonly<Record<string, unknown>> {
  return Object.fromEntries(
    Object
      .entries(rule,)
      .filter(function isContent([key,],): boolean {
        return (key !== 'created') && (key !== 'updated');
      },),
  );
}

/**
 Compares a stored rule with its replacement ignoring the timestamps.

 @param stored - Rule read from disk.

 @param desired - Rule this run would write.

 @returns Whether every other field is identical.

 @example
 ```ts
 sameRuleContent({ stored, desired });
 ```
 */
function sameRuleContent(
  {
    stored,
    desired,
  }: {
    readonly stored: Readonly<Record<string, unknown>>;
    readonly desired: Readonly<Record<string, unknown>>;
  },
): boolean {
  return JSON.stringify(withoutTimestamps({ rule: stored, },),)
    === JSON.stringify(withoutTimestamps({ rule: desired, },),);
}

/**
 Installs or refreshes the allowance rules,
 leaving byte-identical files untouched.

 @param rulesDirectory - Daemon rules directory.

 @param roots - Spellings of each covered directory.

 @param now - Timestamp source for `created` and `updated`.

 @returns Which rule names were created,
 updated,
 or already current.

 @throws {@link OpenSnitchConfigError} when a same-named file exists that this allowance did not write.

 @example
 ```ts
 await applyToolchainRules({ rulesDirectory, roots, now: new Date() });
 ```
 */
export async function applyToolchainRules(
  {
    rulesDirectory,
    roots,
    now,
  }: {
    readonly rulesDirectory: string;
    readonly roots: ToolchainRuleRoots;
    readonly now: Date;
  },
): Promise<ApplyToolchainRulesReport> {
  /**
   Files already in the directory.
   */
  const entries = await readRuleDirectory({ rulesDirectory, },);
  /**
   Timestamp text for every write of this run.
   */
  const stamp = rfc3339({ moment: now, },);
  /**
   Names written as new files.
   */
  const created: string[] = [];
  /**
   Names rewritten in place.
   */
  const updated: string[] = [];
  /**
   Names already current.
   */
  const unchanged: string[] = [];
  for (const spec of toolchainRuleSpecs({ roots, },)) {
    /**
     Existing file for this spec.
     */
    const existing = entries.find(function sameStem(entry,): boolean {
      return entry.stem === spec.name;
    },);
    if ((existing !== undefined) && (!isOwnedRule({ entry: existing, },))) {
      throw new OpenSnitchConfigError(
        `A rule named ${spec.name} exists at ${existing.path} but was not written by wg-quicker; `
          + 'move it aside before applying the toolchain allowance.',
      );
    }
    /**
     Creation time kept from an existing owned rule.
     */
    const storedCreated = existing?.rule
      ?.created;
    /**
     Rule this run installs.
     */
    const rule = toolchainRuleFile({
      spec,
      created: (typeof storedCreated) === 'string' ? storedCreated : stamp,
      updated: stamp,
    },);
    if ((existing?.rule !== undefined) && sameRuleContent({
      stored: existing.rule,
      desired: rule,
    },)) {
      unchanged.push(spec.name,);
      continue;
    }
    // Writes run in order so a failure leaves earlier rules whole and later rules untouched.
    // oxlint-disable-next-line eslint/no-await-in-loop -- Each rule file is written and synced before the next begins.
    await writeRuleInPlace({
      path: join(
        rulesDirectory,
        `${spec.name}${RULE_FILE_SUFFIX}`,
      ),
      text: renderRuleText({ rule, },),
      ...(existing === undefined ? {} : { existingSize: existing.size, }),
    },);
    (existing === undefined ? created : updated).push(spec.name,);
  }
  return {
    created,
    updated,
    unchanged,
  };
}

/**
 Removes every rule file this allowance owns.

 @param rulesDirectory - Daemon rules directory.

 @returns Names of the removed rules.

 @example
 ```ts
 await removeToolchainRules({ rulesDirectory });
 ```
 */
export async function removeToolchainRules(
  { rulesDirectory, }: { readonly rulesDirectory: string; },
): Promise<readonly string[]> {
  /**
   Owned files only.
   */
  const owned = (await readRuleDirectory({ rulesDirectory, },)).filter(
    function owns(entry,): boolean {
      return isOwnedRule({ entry, },);
    },
  );
  await Promise.all(owned.map(function removeEntry(entry,): Promise<void> {
    return rm(entry.path,);
  },),);
  return owned.map(function stemOf(entry,): string {
    return entry.stem;
  },);
}
