import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { isJsonRecord, } from './json-shape.ts';
import {
  orderForPublishing,
  readPnprPublishTarget,
} from './publish-plan.ts';
import {
  FORBIDDEN_RETRY_WINDOW_MS,
  publishWithForbiddenRetry,
} from './publish-retry.ts';
import {
  buildIfDeclared,
  packForPnpr,
  publishTarball,
  requestPublishToken,
} from './publish-steps.ts';
import {
  fetchPublishedVersions,
  readWorkspace,
  REPOSITORY_ROOT,
  type WorkspaceEntry,
} from './registry-read.ts';
import { chooseDistTag, } from './version-order.ts';

//region Run

/**
 Logger root for the pnpr publish run.
 */
const moduleLogger = tagged({ tag: 'pnpr-publish', },);

/**
 Manifest fields whose workspace packages must publish first, because installers resolve them.
 `devDependencies` are left out:
 installers never fetch them,
 each build task resolves its own development inputs from the workspace,
 and the first CI run (34913557042) showed dev edges fold nearly every package into one cycle,
 so one failing config package blocked the whole workspace.
 */
const DEPENDENCY_FIELDS = [
  'dependencies',
  'peerDependencies',
  'optionalDependencies',
] as const;

/**
 Whether to stop after packing, for local checks without credentials.
 */
const DRY_RUN = process.argv
  .includes('--dry-run',);

/**
 One package version waiting to publish.
 */
type PlannedPublish = {
  /**
   Workspace package.
   */
  readonly entry: WorkspaceEntry;
  /**
   Manifest version to publish.
   */
  readonly version: string;
  /**
   Registry's current `latest`, absent for a new package.
   */
  readonly latest?: string;
  /**
   Workspace packages installers resolve alongside this one.
   */
  readonly dependencyNames: readonly string[];
};

/**
 Lists workspace dependency names across the runtime dependency fields.

 @param entry - Workspace package.

 @param workspace - Workspace packages by name.

 @returns Names of workspace packages the entry depends on.

 @example
 ```ts
 workspaceDependencyNames({ entry, workspace });
 ```
 */
function workspaceDependencyNames(
  {
    entry,
    workspace,
  }: {
    readonly entry: WorkspaceEntry;
    readonly workspace: ReadonlyMap<string, WorkspaceEntry>;
  },
): readonly string[] {
  return DEPENDENCY_FIELDS.flatMap(function namesIn(field,) {
    /**
     One dependency map from the manifest.
     */
    const dependencies = entry.manifest[field];
    return isJsonRecord(dependencies,)
      ? Object.keys(dependencies,)
        .filter(function inWorkspace(name,) {
          return workspace.has(name,);
        },)
      : [];
  },);
}

/**
 Narrows configured names to `PNPR_PUBLISH_ONLY` (comma-separated) for local checks and targeted retries.

 @param packageNames - Names trusted in the generated config.

 @returns All names when the variable is unset or empty, otherwise the listed ones.

 @throws Error when a listed name is not in the config, since pnpr would refuse its workload publish.

 @example
 ```ts
 selectedNames({ packageNames: ['@monochromatic-dev/or-throw-core'] });
 ```
 */
function selectedNames({ packageNames, }: { readonly packageNames: readonly string[]; },): readonly string[] {
  /**
   Raw filter value.
   */
  const only = process.env
    .PNPR_PUBLISH_ONLY;
  if ((only === undefined) || (only.trim() === ''))
    return packageNames;
  /**
   Requested names.
   */
  const requested = only.split(',',)
    .map(function trimName(name,) {
      return name.trim();
    },)
    .filter(function isNonEmpty(name,) {
      return name !== '';
    },);
  /**
   Requested names missing from the trusted list.
   */
  const unknown = requested.filter(function notConfigured(name,) {
    return !packageNames.includes(name,);
  },);
  if (unknown.length > 0)
    throw new Error(`PNPR_PUBLISH_ONLY names packages absent from config.yaml: ${unknown.join(', ',)}`,);
  moduleLogger.info(`PNPR_PUBLISH_ONLY limits this run to ${requested.join(', ',)}`,);
  return requested;
}

/**
 Publishes every configured package version pnpr lacks, in dependency order.

 @throws Error listing packages that failed or were blocked by a failed dependency.

 @example
 ```ts
 await publishMissingVersions();
 ```
 */
async function publishMissingVersions(): Promise<void> {
  /**
   Last moment a forbidden publish may be retried; a push that adds names also redeploys the registry.
   */
  const forbiddenRetryDeadline = Date.now() + FORBIDDEN_RETRY_WINDOW_MS;
  /**
   Registry target from the generated config.
   */
  const target = readPnprPublishTarget(await readFile(
    join(
      REPOSITORY_ROOT,
      'package/config/pnpr/config.yaml',
    ),
    'utf8',
  ),);
  /**
   Origin to talk to; overridable for a local registry container.
   */
  const origin = process.env
    .PNPR_REGISTRY_ORIGIN
    ?? target.origin;
  /**
   Workspace packages by name.
   */
  const workspace = await readWorkspace();
  /**
   Configured package versions pnpr lacks.
   */
  const planned: readonly PlannedPublish[] = (await Promise.all(selectedNames({ packageNames: target.packageNames, },)
    .map(async function checkPackage(name,): Promise<readonly PlannedPublish[]> {
    /**
     Workspace entry for a configured name.
     */
      const entry = workspace.get(name,);
      if (entry === undefined)
        throw new Error(`config.yaml lists ${name}, which is not a workspace package; run mise run sync:files`,);
    /**
     Versions already published.
     */
      const published = await fetchPublishedVersions({
        origin,
        name,
      },);
    /**
     Manifest version to publish.
     */
      const version = String(entry.manifest
        .version,);
      if (published.versions
        .has(version,))
        return [];
      return [{
        entry,
        version,
        ...(published.latest === undefined ? {} : { latest: published.latest, }),
        dependencyNames: workspaceDependencyNames({
          entry,
          workspace,
        },),
      },];
    },),)).flat();
  moduleLogger.info(`${planned.length} of ${target.packageNames
    .length} package versions are missing from ${origin}`,);

  /**
   Planned publishes by package name.
   */
  const byName: ReadonlyMap<string, PlannedPublish> = new Map(planned.map(function toPair(item,) {
    return [
      item.entry
        .name,
      item,
    ];
  },),);
  /**
   Dependency-first order, plus any names that sat in cycles.
   */
  const {
    order,
    cycleMembers,
  } = orderForPublishing(planned.map(function toCandidate(item,) {
    return {
      name: item.entry
        .name,
      dependencyNames: item.dependencyNames,
    };
  },),);
  if (cycleMembers.length > 0)
    moduleLogger.warn(`dependency cycle among ${cycleMembers.join(', ',)}; publishing them in name order`,);

  /**
   Packages that failed or were blocked, with the reason.
   */
  const failures = new Map<string, string>();
  for (const name of order) {
    /**
     Planned publish for this name.
     */
    const item = byName.get(name,);
    if (item === undefined)
      throw new Error(`publish order contains unknown package ${name}`,);
    /**
     Failed dependencies that block this package.
     */
    const blockers = item.dependencyNames
      .filter(function hasFailed(dependency,) {
        return failures.has(dependency,);
      },);
    if (blockers.length > 0) {
      failures.set(
        name,
        `blocked by ${blockers.join(', ',)}`,
      );
      continue;
    }
    try {
      // oxlint-disable-next-line eslint/no-await-in-loop -- builds must finish in dependency order before dependents build or publish.
      await buildIfDeclared({ entry: item.entry, },);
      /**
       Rewritten tarball for this package.
       */
      // oxlint-disable-next-line eslint/no-await-in-loop -- packing reads the build output produced in this same ordered step.
      const tarball = await packForPnpr({ entry: item.entry, },);
      if (DRY_RUN) {
        moduleLogger.info(`dry run: packed ${name}@${item.version} at ${tarball}`,);
        continue;
      }
      /**
       Dist-tag that keeps `latest` moving forward only.
       */
      const tag = chooseDistTag({
        version: item.version,
        ...(item.latest === undefined ? {} : { currentLatest: item.latest, }),
      },);
      // oxlint-disable-next-line eslint/no-await-in-loop -- dependents must not publish before their dependencies exist in the registry.
      await publishWithForbiddenRetry({
        attempt: async function publishOnce() {
          /**
           Workload credential requested right before use, since it is short-lived.
           */
          const token = await requestPublishToken({ audience: target.origin, },);
          await publishTarball({
            tarball,
            target,
            origin,
            tag,
            token,
          },);
        },
        deadline: forbiddenRetryDeadline,
        now: Date.now,
        sleep: wait,
        onRetry: function reportRetry({
          retryNumber,
          delayMs,
        },) {
          moduleLogger.warn(`${name}@${item.version}: pnpr refused the publish with E403; retry ${retryNumber} in ${delayMs} ms in case the registry is still loading a new trust list`,);
        },
      },);
      moduleLogger.info(`published ${name}@${item.version} with tag ${tag}`,);
    }
    catch (error) {
      moduleLogger.error(`${name}@${item.version} failed: ${String(error,)}`,);
      failures.set(
        name,
        String(error,),
      );
    }
  }

  if (failures.size > 0) {
    throw new Error(`pnpr publish incomplete; fix each package or add it to PNPR_EXCLUDED_PACKAGES in package/config/pnpr/src/generated-config-settings.ts:\n${[...failures,].map(function toLine([
      name,
      reason,
    ]: readonly [
      string,
      string
    ],) {
      return `- ${name}: ${reason}`;
    },)
      .join('\n',)}`,);
  }
}

await publishMissingVersions();

//endregion Run
