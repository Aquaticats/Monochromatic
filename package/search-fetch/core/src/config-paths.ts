/**
 Pi Search Fetch config path helpers.
 
 @module
 */

import { join, } from 'node:path';

import {
  LEGACY_PI_LINKUP_CONFIG_FILE,
  PI_EXTENSION_CONFIG_DIR,
  PI_SEARCH_FETCH_CONFIG_FILE,
} from './config-constants.ts';

/**
 Resolve global Pi Search Fetch config path for a home directory.
 
 @param home - home directory
 
 @returns absolute config path
 
 @example
 ```ts
 configPathForHome({ home: '/home/user' });
 ```
 */
function configPathForHome({ home, }: { readonly home: string; }): string {
  return join(
    home,
    PI_EXTENSION_CONFIG_DIR,
    PI_SEARCH_FETCH_CONFIG_FILE,
  );
}

/**
 Resolve legacy Pi Linkup config path for a home directory.
 
 @param home - home directory
 
 @returns absolute legacy config path
 
 @example
 ```ts
 legacyConfigPathForHome({ home: '/home/user' });
 ```
 */
function legacyConfigPathForHome({ home, }: { readonly home: string; }): string {
  return join(
    home,
    PI_EXTENSION_CONFIG_DIR,
    LEGACY_PI_LINKUP_CONFIG_FILE,
  );
}

export {
  configPathForHome,
  legacyConfigPathForHome,
};
