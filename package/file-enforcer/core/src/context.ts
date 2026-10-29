import { resolve, } from 'node:path';

/**
 Sentinel returned by {@link activeConfigPath} when no CLI or watch entry
 point recorded a config path.
 */
export const UNKNOWN_ACTIVE_CONFIG_PATH: unique symbol = Symbol('file-enforcer/context: no active config path recorded by the CLI or watch entry point',);

/**
 Holder for the config path currently being executed through the CLI.
 A map avoids module-root `let` while still allowing {@link setActiveConfigPath}
 to update the active config between imports.
 */
const activeConfigPathByKey: Map<'path', string> = new Map<'path', string>();

/**
 Records the file-enforcer config path for staleness-cache dependency
 tracking, read back via {@link activeConfigPath}.
 
 @param configPath - Config file path that imports the builder API.
 
 @example
 ```ts
 setActiveConfigPath({ configPath: './file-enforcer.config.ts' });
 ```
 */
export function setActiveConfigPath(
  {
    configPath,
  }: {
    readonly configPath: string;
  },
): void {
  activeConfigPathByKey.set(
    'path',
    resolve(configPath,),
  );
}

/**
 Returns the active config path recorded by the CLI or watch mode.
 
 @returns Absolute config path when known, or {@link UNKNOWN_ACTIVE_CONFIG_PATH} when absent.
 
 @example
 ```ts
 const configPath = activeConfigPath();
 ```
 */
export function activeConfigPath(): string | typeof UNKNOWN_ACTIVE_CONFIG_PATH {
  /**
   Config path recorded by CLI or watch mode.
   */
  const activePath = activeConfigPathByKey.get('path',);
  if (activePath === undefined)
    return UNKNOWN_ACTIVE_CONFIG_PATH;

  return activePath;
}

/**
 Returns implicit dependency paths shared by every generated output,
 preferring the path recorded via {@link activeConfigPath}. Including the
 config file prevents a transform change from being skipped when its input
 files and destination metadata stayed unchanged.
 
 @returns Absolute paths that should invalidate every staleness entry.
 
 @example
 ```ts
 const implicitSources = configDependencyPaths();
 ```
 */
export function configDependencyPaths(): readonly string[] {
  /**
   Config path explicitly set by the CLI or watch entry point.
   */
  const activePath = activeConfigPath();
  if (activePath !== UNKNOWN_ACTIVE_CONFIG_PATH)
    return [activePath,];

  /**
   Bun's direct-script entry path, used when a config runs as
   `node file-enforcer.config.ts` without going through the CLI.
   */
  const [, mainPath,] = process.argv;
  if (mainPath === undefined)
    return [];

  return [resolve(mainPath,),];
}
