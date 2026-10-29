import { homedir, } from 'node:os';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { defaultAsnCacheDirectory, } from '../dist/final/node/asn-networks.mjs';

/**
 Cache environment overrides for one test.
 */
type CacheEnvironment = {
  readonly cacheDirectory?: string;
  readonly xdgCacheHome?: string;
};

/**
 Installs temporary cache environment and restores original values on disposal.
 
 @param cacheDirectory - Explicit application cache override.
 
 @param xdgCacheHome - XDG user cache root.
 
 @returns Synchronous environment restoration guard.
 
 @example
 ```ts
 using environment = overrideCacheEnvironment({ cacheDirectory: '/tmp/cache' });
 ```
 */
function overrideCacheEnvironment(
  {
    cacheDirectory,
    xdgCacheHome,
  }: CacheEnvironment,
): Disposable {
  /**
   Original application cache override.
   */
  const originalCacheDirectory = process.env.WG_ALLOWEDIPS_CACHE_DIRECTORY;
  /**
   Original XDG cache root.
   */
  const originalXdgCacheHome = process.env.XDG_CACHE_HOME;
  if (cacheDirectory === undefined)
    delete process.env.WG_ALLOWEDIPS_CACHE_DIRECTORY;
  else
    process.env.WG_ALLOWEDIPS_CACHE_DIRECTORY = cacheDirectory;
  if (xdgCacheHome === undefined)
    delete process.env.XDG_CACHE_HOME;
  else
    process.env.XDG_CACHE_HOME = xdgCacheHome;
  return {
    [Symbol.dispose](): void {
      if (originalCacheDirectory === undefined)
        delete process.env.WG_ALLOWEDIPS_CACHE_DIRECTORY;
      else
        process.env.WG_ALLOWEDIPS_CACHE_DIRECTORY = originalCacheDirectory;
      if (originalXdgCacheHome === undefined)
        delete process.env.XDG_CACHE_HOME;
      else
        process.env.XDG_CACHE_HOME = originalXdgCacheHome;
    },
  };
}

await describe({
  name: defaultAsnCacheDirectory.name,
  concurrency: 1,
  children: [
    it({
      name: 'prefers explicit application override',
      fn: async () => {
        using environment = overrideCacheEnvironment({
          cacheDirectory: '/tmp/wg-cache',
          xdgCacheHome: '/tmp/xdg-cache',
        },);
        expect(defaultAsnCacheDirectory(),).toBe('/tmp/wg-cache',);
      },
    },),
    it({
      name: 'uses XDG cache root when no application override exists',
      fn: async () => {
        using environment = overrideCacheEnvironment({
          xdgCacheHome: '/tmp/xdg-cache',
        },);
        expect(defaultAsnCacheDirectory(),).toBe(join(
          '/tmp/xdg-cache',
          'wg-allowedips',
          'asn',
        ),);
      },
    },),
    it({
      name: 'falls back to user home cache root',
      fn: async () => {
        using environment = overrideCacheEnvironment({});
        expect(defaultAsnCacheDirectory(),).toBe(join(
          homedir(),
          '.cache',
          'wg-allowedips',
          'asn',
        ),);
      },
    },),
  ],
},);
