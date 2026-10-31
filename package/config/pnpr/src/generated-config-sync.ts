import {
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { renderPnprConfig, } from './generated-config-render.ts';
import { selectPnprPublishedNames, } from './generated-config-selection.ts';
import {
  PNPR_CONFIG_PATH,
  PNPR_EXCLUDED_PACKAGES,
} from './generated-config-settings.ts';
import {
  PnprConfigShapeError,
  readPnprPublishTarget,
} from './publish-plan.ts';
import { readWorkspaceEntries, } from './registry-read.ts';

//region Generate and check the pnpr server config

/**
 Logger root for generating and checking the pnpr server config.
 */
const moduleLogger = tagged({ tag: 'pnpr-config', },);

/**
 Sentinel for a config file that does not exist.
 */
const CONFIG_FILE_ABSENT: unique symbol = Symbol('pnpr config file does not exist',);

/**
 Sentinel for a config text that has no readable OIDC trust list.
 */
const TRUST_LIST_UNREADABLE: unique symbol = Symbol('pnpr config has no readable trust list',);

/**
 The config text the current workspace renders, with the package selection behind it.
 */
export type RenderedPnprConfig = {
  /**
   Complete `config.yaml` text.
   */
  readonly content: string;
  /**
   Sorted names that publish to pnpr.
   */
  readonly publishedNames: readonly string[];
  /**
   Number of workspace packages the selection looked at.
   */
  readonly workspacePackageCount: number;
};

/**
 Error for a checked-in pnpr config that differs from what the generator renders.
 */
export class PnprConfigDriftError extends Error {
  /**
   Repository-relative path of the drifted file.
   */
  readonly configPath: string;

  /**
   Package names the generator would trust that the file lacks, sorted.
   */
  readonly missingPackages: readonly string[];

  /**
   Package names the file trusts that the generator would not, sorted.
   */
  readonly unexpectedPackages: readonly string[];

  /**
   Creates the error with a diagnostic naming the file and every differing package.

   @param configPath - Repository-relative path of the drifted file.

   @param missingPackages - Names rendered but absent from the file.

   @param unexpectedPackages - Names in the file but not rendered.

   @param detail - Plain-language account of where the file differs beyond its package list.

   @example
   ```ts
   throw new PnprConfigDriftError({ configPath: PNPR_CONFIG_PATH, missingPackages: [], unexpectedPackages: [], detail: 'x' });
   ```
   */
  constructor(
    {
      configPath,
      missingPackages,
      unexpectedPackages,
      detail,
    }: {
      readonly configPath: string;
      readonly missingPackages: readonly string[];
      readonly unexpectedPackages: readonly string[];
      readonly detail: string;
    },
  ) {
    super([
      `pnpr config drift: ${configPath} is not what the generator renders from the current workspace.`,
      ...missingPackages.length === 0
        ? []
        : [`Packages the file lacks (${missingPackages.length}): ${missingPackages.join(', ',)}`,],
      ...unexpectedPackages.length === 0
        ? []
        : [`Packages the file lists that should not publish (${unexpectedPackages.length}): ${unexpectedPackages.join(', ',)}`,],
      detail,
      'Regenerate it with `mise run //package/config/pnpr:generate:config` (or `mise run sync:files`) and commit the result.',
    ].join('\n',),);
    this.name = 'PnprConfigDriftError';
    this.configPath = configPath;
    this.missingPackages = missingPackages;
    this.unexpectedPackages = unexpectedPackages;
  }
}

/**
 Renders the pnpr server config from the workspace manifests under a repository root.

 @param rootDir - Repository root.

 @param excludedPackages - Reviewed exclusions keyed by package name; defaults to the checked-in list.

 @returns Config text and the selection behind it.

 @throws Error when a manifest lacks a name, a name is invalid, or an exclusion names no workspace package.

 @example
 ```ts
 const { content } = await renderPnprConfigForWorkspace({ rootDir: '.' });
 ```
 */
export async function renderPnprConfigForWorkspace(
  {
    rootDir,
    excludedPackages = PNPR_EXCLUDED_PACKAGES,
  }: {
    readonly rootDir: string;
    readonly excludedPackages?: Readonly<Record<string, string>>;
  }
): Promise<RenderedPnprConfig> {
  /**
   Function-scoped logger for the selection summary.
   */
  const logger = tagged({
    tag: renderPnprConfigForWorkspace.name,
    l: moduleLogger,
  },);
  /**
   Every workspace package with its manifest.
   */
  const workspacePackages = await readWorkspaceEntries({ rootDir, },);
  /**
   Sorted names that publish to pnpr.
   */
  const publishedNames = selectPnprPublishedNames({
    workspacePackages,
    excludedPackages,
  },);
  logger.info(`publishing ${publishedNames.length} of ${workspacePackages.length} workspace packages`,);
  return {
    content: renderPnprConfig({
      publishedNames,
      excludedPackages,
    },),
    publishedNames,
    workspacePackageCount: workspacePackages.length,
  };
}

/**
 Reads a file, mapping absence to a sentinel.

 @param path - Absolute file path.

 @returns File text, or the absence sentinel when the file does not exist.

 @throws Error for any read failure other than absence.

 @example
 ```ts
 await readTextIfPresent({ path: '/absent' });
 // => CONFIG_FILE_ABSENT
 ```
 */
async function readTextIfPresent(
  { path, }: { readonly path: string; },
): Promise<string | typeof CONFIG_FILE_ABSENT> {
  /**
   Function-scoped logger for the absence branch.
   */
  const logger = tagged({
    tag: readTextIfPresent.name,
    l: moduleLogger,
  },);
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error) {
    if ((Error.isError(error,)) && ('code' in error)
      && (error.code === 'ENOENT')) {
      logger.debug(`${path} does not exist: ${String(error,)}`,);
      return CONFIG_FILE_ABSENT;
    }
    throw error;
  }
}

/**
 Writes the pnpr server config when its text changed.

 @param rootDir - Repository root.

 @param excludedPackages - Reviewed exclusions keyed by package name; defaults to the checked-in list.

 @returns Whether the file was written (`false` when it already held the rendered text).

 @throws Error when a manifest lacks a name, a name is invalid, or an exclusion names no workspace package.

 @example
 ```ts
 await generatePnprConfig({ rootDir: '.' });
 ```
 */
export async function generatePnprConfig(
  {
    rootDir,
    excludedPackages = PNPR_EXCLUDED_PACKAGES,
  }: {
    readonly rootDir: string;
    readonly excludedPackages?: Readonly<Record<string, string>>;
  }
): Promise<boolean> {
  /**
   Function-scoped logger for the write decision.
   */
  const logger = tagged({
    tag: generatePnprConfig.name,
    l: moduleLogger,
  },);
  /**
   Config text the workspace renders.
   */
  const { content, } = await renderPnprConfigForWorkspace({
    rootDir,
    excludedPackages,
  },);
  /**
   Absolute destination path.
   */
  const destination = join(
    rootDir,
    PNPR_CONFIG_PATH,
  );
  if (await readTextIfPresent({ path: destination, },) === content) {
    logger.info(`${PNPR_CONFIG_PATH} is up to date`,);
    return false;
  }
  await mkdir(
    dirname(destination,),
    { recursive: true, },
  );
  await writeFile(
    destination,
    content,
    'utf8',
  );
  logger.info(`wrote ${PNPR_CONFIG_PATH}`,);
  return true;
}

/**
 Reads the trusted package names a config text lists.

 @param configText - Contents of a pnpr config file.

 @returns Names in config order, or the unreadable sentinel when the text has no readable OIDC workload.

 @example
 ```ts
 readTrustedNames({ configText: 'not a config' });
 // => TRUST_LIST_UNREADABLE
 ```
 */
function readTrustedNames(
  { configText, }: { readonly configText: string; },
): readonly string[] | typeof TRUST_LIST_UNREADABLE {
  /**
   Function-scoped logger for the unreadable-config branch.
   */
  const logger = tagged({
    tag: readTrustedNames.name,
    l: moduleLogger,
  },);
  try {
    return readPnprPublishTarget(configText,)
      .packageNames;
  }
  catch (error) {
    if (error instanceof PnprConfigShapeError) {
      logger.debug(`trust list unreadable: ${error.message}`,);
      return TRUST_LIST_UNREADABLE;
    }
    throw error;
  }
}

/**
 Quotes one line of a text for a diagnostic.

 @param lines - Lines of the text.

 @param index - Zero-based line index.

 @returns The line as a JSON string, or a note that the text ended before that line.

 @example
 ```ts
 quoteLineAt({ lines: ['a'], index: 1 });
 // => 'end of file'
 ```
 */
function quoteLineAt(
  {
    lines,
    index,
  }: {
    readonly lines: readonly string[];
    readonly index: number;
  },
): string {
  return index < lines.length ? JSON.stringify(lines[index],) : 'end of file';
}

/**
 Describes where two config texts first differ, in plain language.

 @param expected - Text the generator renders.

 @param actual - Text the file holds.

 @returns One sentence naming the first differing line.

 @example
 ```ts
 describeFirstDifference({ expected: 'a\nb', actual: 'a\nc' });
 // => 'The first difference is on line 2: expected "b", found "c".'
 ```
 */
export function describeFirstDifference(
  {
    expected,
    actual,
  }: {
    readonly expected: string;
    readonly actual: string;
  },
): string {
  /**
   Expected lines.
   */
  const expectedLines = expected.split('\n',);
  /**
   Actual lines.
   */
  const actualLines = actual.split('\n',);
  /**
   Index of the first expected line that the actual text lacks or changes; -1 when the actual text only adds lines.
   */
  const firstUnequalIndex = expectedLines.findIndex(function isUnequal(
    line,
    lineIndex,
  ) {
    return line !== actualLines[lineIndex];
  },);
  /**
   Index of the first line present in only one text or unequal in both.
   */
  const index = firstUnequalIndex === (-1) ? expectedLines.length : firstUnequalIndex;
  return `The first difference is on line ${index + 1}: expected ${
    quoteLineAt({
      lines: expectedLines,
      index,
    },)
  }, found ${
    quoteLineAt({
      lines: actualLines,
      index,
    },)
  }.`;
}

/**
 Explains in plain language how a config file differs beyond the package names already listed.

 @param expected - Text the generator renders.

 @param actual - Text the file holds.

 @param listsDiffer - Whether the package lists differ, so the first differing line is only a pointer.

 @param listUnreadable - Whether the file has no readable OIDC trust list, so no package comparison was possible.

 @returns One or two sentences for the diagnostic.

 @example
 ```ts
 describeDrift({ expected: 'a', actual: 'b', listsDiffer: false, listUnreadable: true });
 // => 'The file has no readable OIDC trust list, so its packages cannot be compared. The first difference is on line 1: expected "a", found "b".'
 ```
 */
export function describeDrift(
  {
    expected,
    actual,
    listsDiffer,
    listUnreadable,
  }: {
    readonly expected: string;
    readonly actual: string;
    readonly listsDiffer: boolean;
    readonly listUnreadable: boolean;
  },
): string {
  /**
   First differing line, in plain words.
   */
  const firstDifference = describeFirstDifference({
    expected,
    actual,
  },);
  if (listUnreadable)
    return `The file has no readable OIDC trust list, so its packages cannot be compared. ${firstDifference}`;
  if (listsDiffer)
    return firstDifference;
  return `The package lists match; other content differs. ${firstDifference}`;
}

/**
 Verifies that the checked-in pnpr server config equals what the generator renders, without writing anything.

 @param rootDir - Repository root.

 @param excludedPackages - Reviewed exclusions keyed by package name; defaults to the checked-in list.

 @throws {@link PnprConfigDriftError} when the file is absent or differs, naming the file and every differing package.
 
 @throws Error when a manifest lacks a name, a name is invalid, or an exclusion names no workspace package.

 @example
 ```ts
 await checkPnprConfig({ rootDir: '.' });
 ```
 */
export async function checkPnprConfig(
  {
    rootDir,
    excludedPackages = PNPR_EXCLUDED_PACKAGES,
  }: {
    readonly rootDir: string;
    readonly excludedPackages?: Readonly<Record<string, string>>;
  }
): Promise<void> {
  /**
   Function-scoped logger for the verdict.
   */
  const logger = tagged({
    tag: checkPnprConfig.name,
    l: moduleLogger,
  },);
  /**
   Config text and package selection the workspace renders.
   */
  const {
    content,
    publishedNames,
  } = await renderPnprConfigForWorkspace({
    rootDir,
    excludedPackages,
  },);
  /**
   Text of the checked-in file, or the absence sentinel when the file does not exist.
   */
  const actual = await readTextIfPresent({
    path: join(
      rootDir,
      PNPR_CONFIG_PATH,
    ),
  },);
  if (actual === content) {
    logger.info(`${PNPR_CONFIG_PATH} matches the generator`,);
    return;
  }
  if (actual === CONFIG_FILE_ABSENT) {
    throw new PnprConfigDriftError({
      configPath: PNPR_CONFIG_PATH,
      missingPackages: publishedNames,
      unexpectedPackages: [],
      detail: 'The file does not exist.',
    },);
  }

  /**
   Names the file trusts, or the unreadable sentinel when the file has no trust list.
   */
  const trustedNames = readTrustedNames({ configText: actual, },);
  /**
   Names in the file, for set arithmetic.
   */
  const trustedSet: ReadonlySet<string> = trustedNames === TRUST_LIST_UNREADABLE
    ? new Set<string>()
    : new Set(trustedNames,);
  /**
   Names the generator renders, for set arithmetic.
   */
  const publishedSet: ReadonlySet<string> = new Set(publishedNames,);
  /**
   Names the generator renders that the file lacks.
   */
  const missingPackages = publishedNames.filter(function isMissing(name,) {
    return !trustedSet.has(name,);
  },);
  /**
   Names the file trusts that the generator does not render.
   */
  const unexpectedPackages = [...trustedSet,]
    .filter(function isUnexpected(name,) {
      return !publishedSet.has(name,);
    },)
    .toSorted();
  throw new PnprConfigDriftError({
    configPath: PNPR_CONFIG_PATH,
    missingPackages,
    unexpectedPackages,
    detail: describeDrift({
      expected: content,
      actual,
      listsDiffer: (missingPackages.length + unexpectedPackages.length) > 0,
      listUnreadable: trustedNames === TRUST_LIST_UNREADABLE,
    },),
  },);
}

//endregion Generate and check the pnpr server config
