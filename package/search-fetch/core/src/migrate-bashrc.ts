/**
 Shell-profile cleanup for the one-time credential migration.
 
 Removes migrated key exports and pins the explicit store path without
 rewriting unrelated lines.
 
 @module
 */

import {
  fileMode,
  readTextIfPresent,
  writeTextAtomic,
} from './migrate-files.ts';

/**
 Remove key exports from `.bashrc` and pin the store path.
 
 @param bashrcPath - shell profile path
 
 @param storePath - absolute encrypted store path
 
 @example
 ```ts
 await cleanBashrc({ bashrcPath: '/home/user/.bashrc', storePath: '/repo/.env.local.json' });
 ```
 */
export async function cleanBashrc(
  {
    bashrcPath,
    storePath,
  }: {
    readonly bashrcPath: string;
    readonly storePath: string;
  },
): Promise<void> {
  /**
   Local value for text.
   */
  const text = await readTextIfPresent({ path: bashrcPath, },);
  if (!text.present)
    return;
  /**
   Local value for exportLine.
   */
  const exportLine = `export ENV_LOCAL_JSON_PATH=${shellQuote(storePath,)}`;
  /**
   Local value for lines.
   */
  const lines = text.value
    .split('\n',)
    .filter(function isNotKeyExport(line,) {
      return !isCredentialExport(line,);
    },);
  /**
   Local value for pathLineIndex.
   */
  const pathLineIndex = lines.findIndex(function isPathLine(line,) {
    return isPathExport(line,);
  },);
  /**
   Local value for nextLines.
   */
  const nextLines = pathLineIndex === (-1)
    ? [
      ...lines,
      exportLine,
    ]
    : lines.map(function replacePathLine(
      line,
      index,
    ) {
      return index === pathLineIndex ? exportLine : line;
    },);
  await writeTextAtomic({
    path: bashrcPath,
    text: nextLines.join('\n',),
    mode: await fileMode(bashrcPath,),
  },);
}

/**
 Return whether a shell line exports one of the migrated keys.
 
 @param line - one line of the shell profile
 
 @returns whether the line is a migrated-key export
 */
function isCredentialExport(line: string,): boolean {
  /**
   Local value for bare.
   */
  const bare = line.trim();
  /**
   Local value for statement.
   */
  const statement = bare.startsWith('export ',)
    ? bare.slice('export '.length,)
    : bare;
  return statement.startsWith('EXA_API_KEY=',)
    || statement.startsWith('LINKUP_API_KEY=',);
}

/**
 Return whether a shell line exports the store path.
 
 @param line - one line of the shell profile
 
 @returns whether the line is the store-path export
 */
function isPathExport(line: string,): boolean {
  return line.trim()
    .startsWith('export ENV_LOCAL_JSON_PATH=',);
}

/**
 Quote one POSIX shell word.
 
 @param value - raw value
 
 @returns single-quoted shell word
 */
function shellQuote(value: string,): string {
  /**
   Local value for escaped.
   */
  const escaped = value.replaceAll(
    "'",
    String.raw`'\''`,
  );
  return `'${escaped}'`;
}
