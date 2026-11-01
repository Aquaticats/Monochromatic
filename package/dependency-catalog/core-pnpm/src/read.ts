/**
 Located-file convenience API for pnpm workspace catalogs.
 
 @module
 */

import {
  findRoot,
  PNPM_WORKSPACE,
} from '@monochromatic-dev/fs-path-core/ts';
import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  parseCatalogFromYaml,
} from './parse.ts';
import type {
  CatalogFile,
  ReadCatalogFileOptions,
} from './types.ts';

//region Public file reader

/**
 Workspace file name appended to the directory the walk finds.
 */
const WORKSPACE_FILE_NAME = 'pnpm-workspace.yaml';

/**
 Locates and parses the nearest `pnpm-workspace.yaml`.
 
 The returned `content` is the exact UTF-8 text that was parsed, allowing a
 caller such as catalog-tighten to perform a formatting-preserving rewrite.
 
 @param startDir - optional starting directory for the upward search
 
 @returns located path, original content, and parsed catalogs
 
 @throws RootNotFoundError when no workspace YAML file exists up from the start directory
 
 @example
 ```ts
 const workspace = await readCatalogFile({ startDir: process.cwd(), });
 console.info(workspace.path, workspace.catalogs.defaultCatalog);
 ```
 */
export async function readCatalogFile(
  {
    startDir,
  }: ReadCatalogFileOptions = {},
): Promise<CatalogFile> {
  /**
   Nearest ancestor directory holding the workspace file.
   */
  const workspaceDir = await findRoot(
    startDir === undefined
      ? { marker: PNPM_WORKSPACE, }
      : {
        cwd: startDir,
        marker: PNPM_WORKSPACE,
      },
  );
  /**
   Absolute path to the nearest workspace YAML file.
   */
  const workspaceYamlPath = join(
    workspaceDir,
    WORKSPACE_FILE_NAME,
  );

  /**
   Original workspace YAML text retained for callers that need surgical edits.
   */
  const content = await readFile(
    workspaceYamlPath,
    'utf8',
  );
  /**
   Parsed catalog blocks derived from the retained content.
   */
  const catalogs = parseCatalogFromYaml(content,);

  return {
    path: workspaceYamlPath,
    content,
    catalogs,
  };
}

//endregion Public file reader
