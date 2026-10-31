import { constants, } from 'node:fs';
import {
  copyFile,
  mkdir,
  open,
  readdir,
  readFile,
  realpath,
  rm,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { OpenSnitchConfigError, } from './errors.ts';
import {
  type PathAliasPair,
  pathSpellings,
  rfc3339,
  TOOLCHAIN_RULE_DESCRIPTION_PREFIX,
  TOOLCHAIN_RULE_NAME_PREFIX,
  toolchainRuleFile,
  type ToolchainRuleRoots,
  type ToolchainRuleSpec,
  toolchainRuleSpecs,
} from './opensnitch-toolchain-rule-patterns.ts';

/**
 Module logger for toolchain daemon-rule maintenance.
 */
const l = tagged({ tag: 'opensnitch-toolchain-rules', },);

/**
 Rule files end with this suffix; the daemon ignores everything else in its rules directory.
 */
const RULE_FILE_SUFFIX = '.json';

/**
 Marker file at the repository root,
 used to find the checkout from the running bundle.
 */
const REPOSITORY_MARKER = 'toolchain.lock.json';

/**
 Path segment `git worktree` agents live under inside the main checkout.
 */
const WORKTREE_SEGMENT = '/.claude/worktrees/';

/**
 Reports whether an unknown failure carries a Node filesystem code.

 @param error - Caught value.

 @param code - Expected code such as `ENOENT`.

 @returns Whether the value is a Node error with that code.

 @example
 ```ts
 hasErrorCode({ error, code: 'ENOENT' });
 ```
 */
function hasErrorCode({ error, code, }: { readonly error: unknown; readonly code: string; },): boolean {
  return Error.isError(error,) && ((error as NodeJS.ErrnoException).code === code);
}

/**
 Reports whether a path exists without following a failure into the caller.

 @param path - Absolute path.

 @returns Resolved real path, or `undefined` when the path is absent.

 @example
 ```ts
 await realpathOrUndefined({ path: '/home' });
 ```
 */
async function realpathOrUndefined({ path, }: { readonly path: string; },): Promise<string | undefined> {
  try {
    return await realpath(path,);
  }
  catch (error) {
    if (hasErrorCode({ error, code: 'ENOENT', },) || hasErrorCode({ error, code: 'ENOTDIR', },))
      return undefined;
    throw new OpenSnitchConfigError(
      `Cannot resolve ${path}`,
      { cause: error, },
    );
  }
}

/**
 Discovers the prefix pairs that name one directory on this host.

 On Fedora Atomic `/home` links to `/var/home`;
 elsewhere the pair list is empty.

 @returns Alias pairs for the `/home` prefix.

 @example
 ```ts
 await homeAliasPairs(); // [['/home', '/var/home']] on Fedora Atomic
 ```
 */
export async function homeAliasPairs(): Promise<readonly PathAliasPair[]> {
  /**
   Real location of `/home`.
   */
  const real = await realpathOrUndefined({ path: '/home', },);
  if ((real === undefined) || (real === '/home'))
    return [];
  return [['/home', real,],];
}

/**
 Expands one directory into the spellings a process path may use: as given, real, and each alias.

 @param path - Absolute directory.

 @param aliasPairs - Host prefix aliases.

 @returns Distinct spellings,
 requested spelling first.

 @example
 ```ts
 await directorySpellings({ path: '/home/me', aliasPairs });
 ```
 */
export async function directorySpellings(
  {
    path,
    aliasPairs,
  }: {
    readonly path: string;
    readonly aliasPairs: readonly PathAliasPair[];
  },
): Promise<readonly string[]> {
  /**
   Real path when the directory exists.
   */
  const real = await realpathOrUndefined({ path, },);
  return [
    ...new Set([
      ...pathSpellings({ path, aliasPairs, },),
      ...(real === undefined ? [] : pathSpellings({ path: real, aliasPairs, },)),
    ],),
  ];
}

/**
 Finds the main checkout containing a directory.

 Walks up from `start` to the nearest directory holding `toolchain.lock.json`,
 then drops a trailing `/.claude/worktrees/<id>` so a worktree run covers the main checkout and all its worktrees.

 @param start - Directory inside a checkout.

 @returns Main checkout root,
 or `undefined` when no ancestor is a checkout.

 @example
 ```ts
 await findRepositoryRoot({ start: import.meta.dirname });
 ```
 */
export async function findRepositoryRoot(
  { start, }: { readonly start: string; },
): Promise<string | undefined> {
  /**
   Directory under inspection.
   */
  let directory = start;
  for (;;) {
    if (await realpathOrUndefined({ path: join(directory, REPOSITORY_MARKER,), },) !== undefined) {
      /**
       Index of the worktree segment when the checkout is a linked worktree.
       */
      const segment = directory.indexOf(WORKTREE_SEGMENT,);
      return segment === -1
        ? directory
        : directory.slice(
          0,
          segment,
        );
    }
    /**
     Parent directory.
     */
    const parent = dirname(directory,);
    if (parent === directory)
      return undefined;
    directory = parent;
  }
}

/**
 Caller choices that move a covered directory away from its default.
 */
export type ToolchainRootOptions = {
  /**
   Caller home,
   the base of every default.
   */
  readonly home: string;

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
   Main checkout roots whose `.cache` holds exposure links.
   */
  readonly repositoryRoots: readonly string[];
};

/**
 Resolves every covered directory into all of its spellings.

 @param options - Home and overrides.

 @returns Roots for {@link toolchainRuleSpecs}.

 @example
 ```ts
 await resolveToolchainRuleRoots({ options: { home: '/home/me', repositoryRoots: [] } });
 ```
 */
export async function resolveToolchainRuleRoots(
  { options, }: { readonly options: ToolchainRootOptions; },
): Promise<ToolchainRuleRoots> {
  /**
   Host prefix aliases applied to each directory.
   */
  const aliasPairs = await homeAliasPairs();
  const [dataHomes, cargoHomes, rustupHomes,] = await Promise.all([
    directorySpellings({ path: options.dataHome ?? join(options.home, '.local', 'share',), aliasPairs, },),
    directorySpellings({ path: options.cargoHome ?? join(options.home, '.cargo',), aliasPairs, },),
    directorySpellings({ path: options.rustupHome ?? join(options.home, '.rustup',), aliasPairs, },),
  ],);
  /**
   Spellings of each requested checkout root.
   */
  const repositoryGroups = await Promise.all(options.repositoryRoots.map(
    function spellRoot(path,): Promise<readonly string[]> {
      return directorySpellings({ path, aliasPairs, },);
    },
  ),);
  return {
    dataHomes,
    cargoHomes,
    rustupHomes,
    repositoryRoots: [...new Set(repositoryGroups.flat(),),],
  };
}

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
 Reports whether a value is a non-null object.

 @param value - Unknown JSON value.

 @returns Whether the value is record-like.

 @example
 ```ts
 isRecord({ name: 'x' });
 ```
 */
function isRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  if ((typeof value) !== 'object')
    return false;
  if (value === null)
    return false;
  return !Array.isArray(value,);
}

/**
 One rule file read from the rules directory.
 */
type RuleDirectoryEntry = {
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
   Parsed JSON,
   or `undefined` when unreadable or not an object.
   */
  readonly rule: Readonly<Record<string, unknown>> | undefined;

  /**
   Byte length of the file text,
   zero when the file could not be read.
   */
  readonly size: number;
};

/**
 Reads every rule file in a directory.

 @param rulesDirectory - Daemon rules directory.

 @returns Entries sorted by file name,
 a malformed file carrying `rule: undefined`.

 @throws {@link OpenSnitchConfigError} when the directory cannot be listed.

 @example
 ```ts
 await readRuleDirectory({ rulesDirectory: '/etc/opensnitchd/rules' });
 ```
 */
async function readRuleDirectory(
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
      .map(async function readEntry(name,): Promise<RuleDirectoryEntry> {
        /**
         Absolute rule file path.
         */
        const path = join(rulesDirectory, name,);
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
          const text = await readFile(path, 'utf8',);
          /**
           Parsed JSON value.
           */
          const parsed: unknown = JSON.parse(text,);
          return {
            stem,
            path,
            rule: isRecord(parsed,) ? parsed : undefined,
            size: Buffer.byteLength(text, 'utf8',),
          };
        }
        catch (error) {
          l.warn(`Cannot read rule ${path}: ${String(error,)}`,);
          return { stem, path, rule: undefined, size: 0, };
        }
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
function isOwnedRule({ entry, }: { readonly entry: RuleDirectoryEntry; },): boolean {
  if (entry.rule === undefined)
    return false;
  if (!entry.stem.startsWith(TOOLCHAIN_RULE_NAME_PREFIX,))
    return false;
  return ((typeof entry.rule.description) === 'string')
    && (entry.rule.description as string).startsWith(TOOLCHAIN_RULE_DESCRIPTION_PREFIX,);
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
 or `undefined` for a new file.

 @example
 ```ts
 await writeRuleInPlace({ path, text, existingSize: undefined });
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
    readonly existingSize: number | undefined;
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
    text.padEnd(existingSize, ' ',),
    'utf8',
  );
  await using handle = await open(
    path,
    'r+',
  );
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
 Reads the creation time an existing owned rule carries.

 @param entry - Existing rule file.

 @returns Creation timestamp,
 or `undefined` when absent.

 @example
 ```ts
 existingCreated({ entry });
 ```
 */
function existingCreated({ entry, }: { readonly entry: RuleDirectoryEntry | undefined; },): string | undefined {
  if (entry?.rule === undefined)
    return undefined;
  return (typeof entry.rule.created) === 'string' ? entry.rule.created as string : undefined;
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
   Specs this run installs.
   */
  const specs = toolchainRuleSpecs({ roots, },);
  /**
   Files already in the directory.
   */
  const entries = await readRuleDirectory({ rulesDirectory, },);
  const created: string[] = [];
  const updated: string[] = [];
  const unchanged: string[] = [];
  for (const spec of specs) {
    /**
     Existing file for this spec.
     */
    const existing = entries.find(function sameStem(entry,): boolean {
      return entry.stem === spec.name;
    },);
    if ((existing !== undefined) && !isOwnedRule({ entry: existing, },)) {
      throw new OpenSnitchConfigError(
        `A rule named ${spec.name} exists at ${existing.path} but was not written by wg-quicker; `
          + 'move it aside before applying the toolchain allowance.',
      );
    }
    /**
     Timestamp text for this write.
     */
    const stamp = rfc3339({ moment: now, },);
    /**
     Rule keeping the original creation time.
     */
    const rule = toolchainRuleFile({
      spec,
      created: existingCreated({ entry: existing, },) ?? stamp,
      updated: stamp,
    },);
    if (existing?.rule !== undefined) {
      /**
       Stored rule compared without its timestamps.
       */
      const { created: _created, updated: _updated, ...stored } = existing.rule;
      const { created: _newCreated, updated: _newUpdated, ...desired } = rule;
      if (JSON.stringify(stored,) === JSON.stringify(desired,)) {
        unchanged.push(spec.name,);
        continue;
      }
    }
    await writeRuleInPlace({
      path: join(rulesDirectory, `${spec.name}${RULE_FILE_SUFFIX}`,),
      text: renderRuleText({ rule, },),
      existingSize: existing?.size,
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
  for (const entry of owned)
    await rm(entry.path,);
  return owned.map(function stemOf(entry,): string {
    return entry.stem;
  },);
}

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
 Reports whether a rule is an enabled reject or deny that compares one exact process path.

 @param rule - Parsed rule.

 @returns The compared path,
 or `undefined` when the rule has another shape.

 @example
 ```ts
 exactBlockedPath({ rule });
 ```
 */
function exactBlockedPath(
  { rule, }: { readonly rule: Readonly<Record<string, unknown>>; },
): string | undefined {
  if (rule.enabled !== true)
    return undefined;
  if ((rule.action !== 'reject') && (rule.action !== 'deny'))
    return undefined;
  const { operator, } = rule;
  if (!isRecord(operator,))
    return undefined;
  if ((operator.type !== 'simple') || (operator.operand !== 'process.path'))
    return undefined;
  return (typeof operator.data) === 'string' ? operator.data as string : undefined;
}

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
   Rules sorting before the allowance that are not allow rules without precedence.
   */
  readonly earlierWinners: readonly EarlierWinningRule[];

  /**
   Rule files that could not be parsed.
   */
  readonly unreadable: readonly string[];
};

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
   Compiled specs paired with their expressions.
   */
  const compiled = specs.map(function compile(spec,): { readonly spec: ToolchainRuleSpec; readonly expression: RegExp; } {
    return { spec, expression: new RegExp(spec.pattern,), };
  },);
  /**
   Files in the directory.
   */
  const entries = await readRuleDirectory({ rulesDirectory, },);
  /**
   Entries that are not this allowance's own.
   */
  const foreign = entries.filter(function notOwned(entry,): boolean {
    return !isOwnedRule({ entry, },);
  },);
  const superseded: SupersededRejectRule[] = [];
  const earlierWinners: EarlierWinningRule[] = [];
  for (const entry of foreign) {
    if (entry.rule === undefined)
      continue;
    /**
     Compared path of a reject or deny rule on one exact path.
     */
    const blockedPath = exactBlockedPath({ rule: entry.rule, },);
    if (blockedPath !== undefined) {
      /**
       Allowance rule whose expression matches the blocked path.
       */
      const covering = compiled.find(function covers({ expression, },): boolean {
        return expression.test(blockedPath,);
      },);
      if (covering !== undefined) {
        superseded.push({
          name: entry.stem,
          path: entry.path,
          processPath: blockedPath,
          coveredBy: covering.spec.name,
        },);
      }
    }
    if ((entry.rule.enabled !== true) || (entry.stem >= TOOLCHAIN_RULE_NAME_PREFIX))
      continue;
    if (entry.rule.action === 'reject')
      earlierWinners.push({ name: entry.stem, reason: 'reject', },);
    else if (entry.rule.action === 'deny')
      earlierWinners.push({ name: entry.stem, reason: 'deny', },);
    else if (entry.rule.precedence === true)
      earlierWinners.push({ name: entry.stem, reason: 'precedence', },);
  }
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
  const { superseded, } = await inspectToolchainRules({ rulesDirectory, roots, },);
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
    await copyFile(
      rule.path,
      join(backupDirectory, `${rule.name}${RULE_FILE_SUFFIX}`,),
      // Fail rather than overwrite a backup from an earlier run.
      constants.COPYFILE_EXCL,
    );
    await rm(rule.path,);
  }
  return superseded;
}
