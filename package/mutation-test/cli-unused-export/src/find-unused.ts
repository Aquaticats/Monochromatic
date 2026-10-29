/**
 Workspace-wide unused-export detection on yuku-analyzer's cross-file
 semantic model.
 
 @example
 ```ts
 const findings = await findUnusedExports({ workspaceRoot: '/repo' });
 ```
 */

import { readFile, } from 'node:fs/promises';
import { posix, } from 'node:path';
import { Analyzer, } from 'yuku-analyzer';
import {
  lineStarts,
  positionAt,
} from '@monochromatic-dev/mutation-test-cli/ts/engine/lines.ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { workspaceResolver, } from './resolve.ts';
import { discoverWorkspacePackages, } from './workspace.ts';

/**
 Module logger for unused-export analysis.
 */
const l = tagged({ tag: 'unused-export', },);

/**
 One export with zero resolved references anywhere in the workspace.
 */
export type UnusedExport = {
  /**
   Workspace-relative source path declaring the export.
   */
  readonly file: string;
  /**
   Exported name.
   */
  readonly name: string;
  /**
   One-based declaration line.
   */
  readonly line: number;
  /**
   Zero-based declaration column.
   */
  readonly column: number;
  /**
   Whether the export is type-only.
   */
  readonly typeOnly: boolean;
};

/**
 Finds workspace exports no other resolved reference uses.
 
 yuku-analyzer follows import, named re-export, and `export *` chains
 back to defining symbols, so a consumer anywhere in the workspace
 counts as usage of the original declaration. Exports backed by no
 local symbol (star records, default expression exports, re-export
 specifiers) never appear as findings; their defining symbols are
 judged where they are declared.
 
 @param workspaceRoot - Absolute workspace root holding `pnpm-workspace.yaml`.
 
 @returns Findings sorted by file, line, then name.
 
 @throws Error when the workspace manifest is missing or malformed.
 
 @example
 ```ts
 const findings = await findUnusedExports({ workspaceRoot: '/repo' });
 ```
 */
export async function findUnusedExports({
  workspaceRoot,
}: Readonly<{
  workspaceRoot: string;
}>,): Promise<readonly UnusedExport[]> {
  /**
   Analysis logger tagged with the calling function.
   */
  const fl = tagged({
    tag: findUnusedExports.name,
    l,
  },);
  /**
   Discovered workspace packages with analyzable sources.
   */
  const packages = await discoverWorkspacePackages({ workspaceRoot, },);
  /**
   Every analyzable source path across the workspace.
   */
  const fileSet: ReadonlySet<string> = new Set(packages.flatMap(function toFiles(entry,): readonly string[] {
    return entry.sourceFiles;
  },),);
  /**
   Package directory looked up by published package name.
   */
  const packageDirsByName: ReadonlyMap<string, string> = new Map(packages.map(function toEntry(entry,): readonly [
    string,
    string,
  ] {
    return [
      entry.name,
      entry.dir,
    ];
  },),);

  fl.info(`analyzing ${String(fileSet.size,)} sources across ${String(packages.length,)} packages`,);

  /**
   Cross-file analyzer over workspace sources.
   */
  const analyzer = new Analyzer({
    resolve: workspaceResolver({
      packageDirsByName,
      fileSet,
    },),
  },);
  /**
   Source texts read concurrently, order matching the file set.
   */
  const sources = await Promise.all([...fileSet,].map(async function readSource(file,): Promise<readonly [
    string,
    string,
  ]> {
    return [
      file,
      await readFile(
        posix.join(
          workspaceRoot,
          file,
        ),
        'utf8',
      ),
    ];
  },),);

  for (const [
    file,
    content,
  ] of sources)
    analyzer.setFile(
      file,
      content,
    );

  analyzer.link();

  for (const diagnostic of analyzer.diagnostics)
    fl.debug(`link diagnostic: ${diagnostic.path}: ${diagnostic.message}`,);

  /**
   Findings accumulated across every module's export records.
   */
  const findings: UnusedExport[] = [];

  for (const analyzedModule of analyzer.modules
    .values()) {
    /**
     Line-start offsets for declaration position math.
     */
    const table = lineStarts(analyzedModule.source,);

    for (const moduleExport of analyzedModule.exports) {
      /**
       Backing local symbol, null for record kinds without one.
       */
      const { local, } = moduleExport;

      if ((local === null) || (moduleExport.name === null))
        continue;
      if (local.findReferences()
        .length
        > 0)
        continue;

      /**
       First declaration node carrying the source span.
       */
      const [declaration,] = local.declarations;

      if (declaration === undefined)
        continue;

      /**
       One-based line and zero-based column of the declaration.
       */
      const position = positionAt({
        table,
        offset: declaration.start,
      },);

      findings.push({
        file: analyzedModule.path,
        name: moduleExport.name,
        line: position.line,
        column: position.column,
        typeOnly: moduleExport.typeOnly,
      },);
    }
  }

  fl.info(`${String(findings.length,)} exports have no workspace references`,);

  return findings.toSorted(function byPosition(
    a,
    b,
  ): number {
    return (a.file < b.file ? -1 : (a.file > b.file ? 1 : 0))
      || (a.line - b.line)
      || (a.name < b.name ? -1 : 1);
  },);
}
