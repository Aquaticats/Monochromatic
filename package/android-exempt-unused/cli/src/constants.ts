/**
 Shared string constants and value types for the android-exempt-unused CLI.
 
 Keeping the appops operation name, its modes, and the adb binary name in one
 place avoids magic literals scattered across the adb command builders.
 
 @module
 */

/**
 Name of the adb executable resolved from `PATH`.
 
 @example
 ```ts
 import { ADB, } from './constants.ts';
 await nanoSpawn(ADB, ['devices',],);
 ```
 */
export const ADB = 'adb';

/**
 appops operation that governs whether unused-permission auto-revoke runs for
 an app. Setting it to {@link MODE_IGNORE} exempts the app; clearing it to
 {@link MODE_DEFAULT} re-enables auto-revoke.
 
 @example
 ```ts
 import { AUTO_REVOKE_OP, MODE_IGNORE, } from './constants.ts';
 await runAdb({ args: ['shell', 'cmd', 'appops', 'set', pkg, AUTO_REVOKE_OP, MODE_IGNORE,], },);
 ```
 */
export const AUTO_REVOKE_OP = 'AUTO_REVOKE_PERMISSIONS_IF_UNUSED';

/**
 appops mode that makes the system ignore auto-revoke for an app, so its
 granted permissions are kept even when unused. This is the exempt state.
 */
export const MODE_IGNORE = 'ignore';

/**
 appops mode that clears any explicit override, restoring the system default
 (auto-revoke active). This is the un-exempt state.
 */
export const MODE_DEFAULT = 'default';

/**
 appops mode this CLI writes: either exempt ({@link MODE_IGNORE}) or revert to
 default ({@link MODE_DEFAULT}).
 */
export type AppOpsMode = typeof MODE_IGNORE | typeof MODE_DEFAULT;

/**
 `adb devices` state string for a device ready to accept shell commands.
 Other states (`offline`, `unauthorized`) are filtered out before use.
 */
export const CONNECTED_STATE = 'device';
