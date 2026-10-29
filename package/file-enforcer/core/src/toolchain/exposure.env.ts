/**
 Tool environment exports: the gitignored `.cache/toolchain.env.json` that carries the plan's `env` blocks, written by
 sync and read back by consumers.
 */

import {
  mkdir,
  readFile,
  rename,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';
import { pathExists, } from './fs-probe.ts';
import type { ToolchainPaths, } from './paths.ts';
import type { ToolPlanEntry, } from './types.ts';

//region Tool environment exports

/**
 Writes the gitignored `.cache/toolchain.env.json`
 (`ANDROID_HOME`, `ANDROID_SDK_ROOT`, `JAVA_HOME`, `RUSTUP_TOOLCHAIN`).
 Values the plan records as toolchain-root relative paths are resolved to
 absolute machine paths here; literal values pass through.

 @param paths - Resolved toolchain paths of the scope

 @param entries - Plan entries whose `env` blocks are merged

 @returns The environment record written

 @example
 ```ts
 const env = await writeToolEnvFile({ paths, entries: ALL_TOOLS, },);
 env.JAVA_HOME; // '/.../jdk/25'
 ```
 */
export async function writeToolEnvFile(
  {
    paths,
    entries,
  }: {
    readonly paths: ToolchainPaths;
    readonly entries: readonly ToolPlanEntry[];
  },
): Promise<Readonly<Record<string, string>>> {
  /**
   Environment record assembled from every entry's `env` block.
   */
  const env: Record<string, string> = {};
  for (const entry of entries) {
    for (const [name, value,] of Object.entries(entry.env,)) {
      env[name] = value.includes('/')
        ? join(
          paths.toolchainRoot,
          value,
        )
        : value;
    }
  }
  await mkdir(
    dirname(paths.envFile,),
    { recursive: true },
  );
  /**
   Temporary file beside the final env file.
   */
  const temp = `${paths.envFile}.${
    process.pid
  }.tmp`;
  await writeFile(
    temp,
    `${
      JSON.stringify(
        env,
        null,
        2,
      )
    }\n`,
    'utf8',
  );
  await rename(
    temp,
    paths.envFile,
  );
  return env;
}

//endregion Tool environment exports

/**
 Whether a parsed JSON value is an object whose values are all strings.

 @param value - Parsed JSON value to test

 @returns `true` when every property value is a string
 */
function isStringRecord(
  value: unknown,
): value is Readonly<Record<string, string>> {
  return ((typeof value) === 'object')
    && (value !== null)
    && (!Array.isArray(value,))
    && Object.values(value,)
      .every(
      function isString(propertyValue: unknown,): boolean {
        return (typeof propertyValue) === 'string';
      },
      );
}

/**
 Reads a generated `.cache/toolchain.env.json` back.

 @param paths - Resolved toolchain paths of the scope

 @returns The recorded environment; empty when the file is absent

 @throws TypeError when the file exists but is not an object of string values

 @example
 ```ts
 const env = await readToolEnvFile(paths,);
 ```
 */
export async function readToolEnvFile(
  paths: ToolchainPaths,
): Promise<Readonly<Record<string, string>>> {
  if (!(await pathExists(paths.envFile,)))
    return {};
  /**
   Parsed file content.
   */
  const parsed: unknown = JSON.parse(await readFile(
    paths.envFile,
    'utf8',
  ),);
  if (!isStringRecord(parsed,))
    throw new TypeError(`${paths.envFile} is not an object of string values`,);
  return parsed;
}
