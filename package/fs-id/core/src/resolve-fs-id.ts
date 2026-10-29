/**
 Memoized filesystem identity orchestration and production effect adapters.
 
 @module
 */

import {
  realpath,
  stat,
} from 'node:fs/promises';
import { platform, } from 'node:os';
import { resolve, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import nanoSpawn from 'nano-spawn';

import { caughtValueText as caughtMessage, } from '@monochromatic-dev/caught-value-core/ts';
import { UnsupportedFsIdPlatformError, } from './errors.ts';
import {
  resolveDarwinFsId,
  resolveLinuxFsId,
  resolveWindowsFsId,
} from './platform-resolvers.ts';
import type {
  FsIdResolution,
  FsIdResolver,
  FsIdResolverAdapters,
} from './types.ts';

/**
 Package logger root.
 
 @example
 ```ts
 l.debug('resolving');
 ```
 */
const l = tagged({ tag: 'module-fs-id', },);

/**
 Canonicalizes one target through native realpath resolution.
 
 @param path - Caller-supplied target
 
 @returns Canonical absolute path
 
 @example
 ```ts
 await canonicalizePath({ path: '.' });
 ```
 */
function canonicalizePath({ path, }: { readonly path: string; },): Promise<string> {
  return realpath(resolve(path,),);
}

/**
 Runs one platform command and returns captured stdout.
 
 @param command - Executable name
 
 @param args - Exact argument vector
 
 @returns Captured stdout
 
 @example
 ```ts
 await runCommand({ command: 'findmnt', args: ['--help'] });
 ```
 */
async function runCommand({
  command,
  args,
}: {
  readonly command: string;
  readonly args: readonly string[];
},): Promise<string> {
  /**
   Successful command result.
   */
  const { stdout, } = await nanoSpawn(
    command,
    [...args,],
  );
  return stdout;
}

/**
 Reads runtime device number through Node stat.
 
 @param path - Canonical target path
 
 @returns Decimal device number
 
 @example
 ```ts
 await readDeviceNumber({ path: '.' });
 ```
 */
async function readDeviceNumber({ path, }: { readonly path: string; },): Promise<string> {
  /**
   Bigint stat result avoids precision loss for device identifiers.
   */
  const metadata = await stat(
    path,
    { bigint: true, },
  );
  return metadata.dev
    .toString();
}

/**
 Emits mandatory degraded-stability warning.
 
 @param path - Canonical target path
 
 @param reason - Why reboot stability is unavailable
 
 @example
 ```ts
 warnDegraded({ path: '/repo', reason: 'UUID unavailable' });
 ```
 */
function warnDegraded({
  path,
  reason,
}: {
  readonly path: string;
  readonly reason: string;
},): void {
  l.warn(`filesystem identity for ${path} is runtime-only and may change after reboot: ${reason}`,);
}

/**
 Reports production resolution failure through tagged logger.
 
 @param tag - function-boundary logger tag
 
 @param message - complete failure context
 
 @param error - underlying failure
 */
function reportResolutionError({
  tag,
  message,
  error,
}: Readonly<{
  tag: string;
  message: string;
  error: unknown;
}>,): void {
  tagged({
    tag,
    l,
  },)
    .error(`${message}: ${caughtMessage(error,)}`,);
}

/**
 Reports production branch detail through tagged logger.
 
 @param tag - function-boundary logger tag
 
 @param message - complete branch detail
 */
function reportResolutionDebug({
  tag,
  message,
}: Readonly<{
  tag: string;
  message: string;
}>,): void {
  tagged({
    tag,
    l,
  },)
    .debug(message,);
}

/**
 Preserves special handle-backed path without resolving its link target.
 
 @param path - process file-descriptor path
 
 @returns unchanged path
 */
function preserveHandlePath({ path, }: { readonly path: string; },): Promise<string> {
  return Promise.resolve(path,);
}

/**
 Production adapters using current host operating-system facilities.
 
 @example
 ```ts
 productionAdapters.platform();
 ```
 */
const productionAdapters: FsIdResolverAdapters = {
  platform,
  canonicalizePath,
  run: runCommand,
  deviceNumber: readDeviceNumber,
  warn: warnDegraded,
  debug: reportResolutionDebug,
  reportError: reportResolutionError,
};

/**
 Resolves one canonical path on selected platform.
 
 @param path - Canonical target path
 
 @param adapters - Selected effect adapters
 
 @returns Stable or degraded identity
 
 @throws when platform has no strategy
 
 @example
 ```ts
 await resolveCanonicalPath({ path: '/repo', adapters });
 ```
 */
function resolveCanonicalPath({
  path,
  adapters,
}: {
  readonly path: string;
  readonly adapters: FsIdResolverAdapters;
},): Promise<FsIdResolution> {
  /**
   Platform selected once for this resolution.
   */
  const hostPlatform = adapters.platform();
  if (hostPlatform === 'linux')
    return resolveLinuxFsId({
      path,
      adapters,
    },);
  if (hostPlatform === 'darwin')
    return resolveDarwinFsId({
      path,
      adapters,
    },);
  if (hostPlatform === 'win32')
    return resolveWindowsFsId({
      path,
      adapters,
    },);
  throw new UnsupportedFsIdPlatformError(hostPlatform,);
}

/**
 Creates resolver around supplied effect adapters.
 
 Each call resolves platform identity afresh after canonicalization.
 Avoiding cross-call memoization prevents a volume replaced at the same path from inheriting stale identity.
 
 @param adapters - Production or test effects
 
 @returns Fresh canonicalizing resolver
 
 @example
 ```ts
 const resolver = createFsIdResolver({ adapters });
 await resolver({ path: '/repo' });
 ```
 */
export function createFsIdResolver({
  adapters,
}: {
  readonly adapters: FsIdResolverAdapters;
},): FsIdResolver {
  return async function freshResolver({
    path,
  }: {
    readonly path: string;
  }): Promise<FsIdResolution> {
    /**
     Canonical path resolved on every call so symlink and mount changes are observed.
     */
    const canonicalPath = await adapters.canonicalizePath({ path, },);
    try {
      /**
       Fresh platform resolution before mandatory degraded warning.
       */
      const resolution = await resolveCanonicalPath({
        path: canonicalPath,
        adapters,
      },);
      if (!resolution.stable) {
        adapters.warn({
          path: canonicalPath,
          reason: resolution.reason ?? 'stable identity mechanism unavailable',
        },);
      }
      return resolution;
    }
    catch (error: unknown) {
      adapters.reportError?.({
        tag: resolveCanonicalPath.name,
        message: `filesystem identity resolution failed for ${canonicalPath}`,
        error,
      },);
      throw error;
    }
  };
}

/**
 Default production resolver with fresh identity observation per call.
 
 @example
 ```ts
 await defaultResolver({ path: '.' });
 ```
 */
const defaultResolver = createFsIdResolver({ adapters: productionAdapters, },);

/**
 Production resolver for special handle-backed targets.
 */
const handleTargetResolver = createFsIdResolver({
  adapters: {
    ...productionAdapters,
    canonicalizePath: preserveHandlePath,
  },
},);

/**
 Production resolver that leaves degraded reporting to its caller.
 */
const callerReportedResolver = createFsIdResolver({
  adapters: {
    platform: productionAdapters.platform,
    canonicalizePath: productionAdapters.canonicalizePath,
    run: productionAdapters.run,
    deviceNumber: productionAdapters.deviceNumber,
    warn: function deferDegradedWarning(input,): void {
      void input;
    },
  },
},);

/**
 Handle-backed resolver that leaves all reporting to its caller.
 */
const callerReportedHandleTargetResolver = createFsIdResolver({
  adapters: {
    platform: productionAdapters.platform,
    canonicalizePath: preserveHandlePath,
    run: productionAdapters.run,
    deviceNumber: productionAdapters.deviceNumber,
    warn: function deferHandleDegradedWarning(input,): void {
      void input;
    },
  },
},);

/**
 Resolves filesystem identity for existing path on current host.
 
 @param path - Existing path whose containing filesystem is identified
 
 @param emitDiagnostics - whether module logger reports degraded and failed resolution
 
 @param preserveTargetPath - whether special handle-backed path bypasses realpath
 
 @returns Stable preferred or explicitly caller-reported degraded identity
 
 @throws when host has no strategy or no identity mechanism succeeds
 
 @example
 ```ts
 const result = await resolveFsId({ path: process.cwd() });
 ```
 */
export function resolveFsId({
  path,
  emitDiagnostics = true,
  preserveTargetPath = false,
}: {
  readonly path: string;
  readonly emitDiagnostics?: boolean;
  readonly preserveTargetPath?: boolean;
}): Promise<FsIdResolution> {
  if (preserveTargetPath) {
    return emitDiagnostics
      ? handleTargetResolver({ path, },)
      : callerReportedHandleTargetResolver({ path, },);
  }
  return emitDiagnostics
    ? defaultResolver({ path, },)
    : callerReportedResolver({ path, },);
}
