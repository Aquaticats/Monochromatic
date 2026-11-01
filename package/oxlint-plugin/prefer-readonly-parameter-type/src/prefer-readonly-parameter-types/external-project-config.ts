/**
 Deterministic TypeScript configuration for reached package implementation files.
 
 @module
 */

import {
  existsSync,
  mkdirSync,
  writeFileSync,
} from 'node:fs';
import { join, } from 'node:path';

import type { Project, } from 'typescript/sync';

import {
  contentDigest,
  effectCacheRoot,
} from './effect-summary-cache-identity.ts';
import { configuredProjectConfigFileName, } from './semantic-file-name.ts';

/**
 Generated external configured-project schema.
 */
const EXTERNAL_PROJECT_SCHEMA = 4;

/**
 Writes deterministic generated TypeScript config when absent.
 
 @param consumerProject - Caller project selecting dependency-local cache root.
 
 @param packageRoot - Exact package root accepted for source analysis.
 
 @param implementationPath - Shipped runtime entry.
 
 @param implementationDigest - Runtime and source-map content identity.
 
 @param implementationFiles - Reached runtime roots admitted to project.
 
 @returns generated config path.
 
 @example
 ```ts
 externalProjectConfigPath({ consumerProject, packageRoot, implementationPath, implementationDigest, implementationFiles });
 ```
 */
export function externalProjectConfigPath({
  consumerProject,
  packageRoot,
  implementationPath,
  implementationDigest,
  implementationFiles,
}: {
  readonly consumerProject: Project;
  readonly packageRoot: string;
  readonly implementationPath: string;
  readonly implementationDigest: string;
  readonly implementationFiles: readonly string[];
}): string {
  /**
   Dependency-local persistent cache root.
   */
  const cacheRoot = effectCacheRoot({
    projectKey: configuredProjectConfigFileName({ project: consumerProject, },),
  },);
  /**
   Exact generated project identity.
   */
  const projectIdentity = contentDigest(
    `${String(EXTERNAL_PROJECT_SCHEMA,)}\0${packageRoot}\0${implementationPath}\0${implementationDigest}\0${
      implementationFiles.join('\0',)
    }`,
  );
  /**
   Generated project directory.
   */
  const directory = join(
    cacheRoot,
    'external-projects',
    projectIdentity,
  );
  /**
   Generated project configuration path.
   */
  const configPath = join(
    directory,
    'tsconfig.json',
  );
  // oxlint-disable-next-line no-restricted-syntax/no-sync -- Synchronous semantic visitor reuses deterministic package-analysis project when already materialized.
  if (existsSync(configPath,))
    return configPath;
  /**
   Deterministic JavaScript-capable TypeScript project config.
   */
  const configText = `${JSON.stringify(
    {
      compilerOptions: {
        allowJs: true,
        checkJs: true,
        maxNodeModuleJsDepth: 10,
        noEmit: true,
        skipLibCheck: true,
        strict: false,
        module: 'NodeNext',
        moduleResolution: 'NodeNext',
        target: 'ESNext',
        resolveJsonModule: true,
      },
      files: implementationFiles,
    },
    null,
    2,
  )}\n`;
  // oxlint-disable-next-line no-restricted-syntax/no-sync -- Synchronous semantic visitor materializes deterministic package-analysis project before TypeScript snapshot.
  mkdirSync(
    directory,
    { recursive: true, },
  );
  try {
    // oxlint-disable-next-line no-restricted-syntax/no-sync -- Synchronous semantic visitor writes generated package-analysis configuration once.
    writeFileSync(
      configPath,
      configText,
      { flag: 'wx', },
    );
    return configPath;
  }
  catch (error) {
    // oxlint-disable-next-line no-restricted-syntax/no-sync -- Concurrent writer may have atomically materialized identical content-addressed config.
    if (existsSync(configPath,))
      return configPath;
    throw error;
  }
}
