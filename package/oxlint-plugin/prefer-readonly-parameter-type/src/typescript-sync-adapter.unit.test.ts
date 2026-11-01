import {
  mkdtempSync,
  readFileSync,
  renameSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from 'node:fs';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  join,
  relative,
} from 'node:path';
import { fileURLToPath, } from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  closeSemanticBridge,
  configuredProjectConfigFileName,
  openSemanticFile,
  semanticBridgeCacheStats,
  typescriptOffset,
} from '../dist/final/node/index.mjs';

/** Source fixture resolved through configured package project. */
const FIXTURE_PATH = fileURLToPath(new URL(
  '../../../test-fixture/oxlint-no-restricted-syntax/src/valid/typescript-sync-adapter.ts',
  import.meta.url,
),);

/** Configured fixture package root for disposable semantic files. */
const FIXTURE_ROOT = fileURLToPath(new URL(
  '../../../test-fixture/oxlint-no-restricted-syntax/src/',
  import.meta.url,
),);

/** Disposable configured-project directory. */
type SemanticFixtureDirectory = {
  readonly path: string;
  [Symbol.dispose]: () => void;
};

/**
 Creates non-hidden disposable directory included by fixture project.
 
 @returns disposable configured-project directory.
 */
function createSemanticFixtureDirectory(): SemanticFixtureDirectory {
  /** Unique directory path under configured fixture package. */
  const path = mkdtempSync(join(FIXTURE_ROOT, 'semantic-acceptance-',),);
  return {
    path,
    [Symbol.dispose]: function removeSemanticFixtureDirectory(): void {
      rmSync(path, { recursive: true, force: true, },);
    },
  };
}

/** Second source of the same configured project, for asserting overlay retention. */
const SIBLING_PATH = fileURLToPath(new URL(
  '../../../test-fixture/oxlint-no-restricted-syntax/src/valid/all-rules-passing.ts',
  import.meta.url,
),);

/** Original source retained on disk while overlays change in memory. */
const SOURCE = readFileSync(
  FIXTURE_PATH,
  'utf8',
);

/** Sibling source text handed to the bridge alongside the fixture. */
const SIBLING_SOURCE = readFileSync(
  SIBLING_PATH,
  'utf8',
);

/** Parameter identifier offset queried through both snapshots. */
const PARAMETER_OFFSET = SOURCE.indexOf('box: SemanticFixtureBox',);

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: openSemanticFile.name,
      children: [
        it({
          name: 'discovers configured project and resolves parameter type',
          fn: async () => {
            const session = openSemanticFile({
              fileName: FIXTURE_PATH,
              sourceText: SOURCE,
              hasBOM: false,
            },);
            const node = session.nodeAtOffset(PARAMETER_OFFSET,);
            const type = session.checker.getTypeAtLocation(node,);

            expect(session.project.configFileName,).toContain(
              'package/test-fixture/oxlint-no-restricted-syntax/tsconfig.json',
            );
            if (type === undefined)
              throw new Error('Expected fixture parameter type.',);
            expect(session.checker.typeToString(type,),).toBe('SemanticFixtureBox<string>',);
          },
        },),
        it({
          name: 'refreshes semantic type from virtual overlay without disk write',
          fn: async () => {
            const overlaid = SOURCE.replace(
              'box: SemanticFixtureBox<string>',
              'box: Readonly<SemanticFixtureBox<string>>',
            );
            const session = openSemanticFile({
              fileName: FIXTURE_PATH,
              sourceText: overlaid,
              hasBOM: false,
            },);
            const node = session.nodeAtOffset(PARAMETER_OFFSET,);
            const type = session.checker.getTypeAtLocation(node,);

            if (type === undefined)
              throw new Error('Expected overlaid fixture parameter type.',);
            expect(session.checker.typeToString(type,),).toBe('Readonly<SemanticFixtureBox<string>>',);
            expect(readFileSync(FIXTURE_PATH, 'utf8',),).toBe(SOURCE,);
          },
        },),
        it({
          name: 'reuses importer from snapshot refreshed by dependency overlay',
          fn: async () => {
            closeSemanticBridge();
            using directory = createSemanticFixtureDirectory();
            /** Imported source changed only through virtual overlay. */
            const dependencyPath = join(directory.path, 'dependency.ts',);
            /** Unchanged importing source reused from refreshed snapshot. */
            const importerPath = join(directory.path, 'importer.ts',);
            /** Dependency text retained on disk. */
            const dependencySource = 'export type Value = { readonly before: string; };\n';
            /** Importer whose parameter reflects dependency overlay. */
            const importerSource = "import type { Value } from './dependency.ts';\nexport function read(value: Value,): Value { return value; }\n";
            writeFileSync(dependencyPath, dependencySource,);
            writeFileSync(importerPath, importerSource,);
            openSemanticFile({
              fileName: importerPath,
              sourceText: importerSource,
              hasBOM: false,
            },);
            const dependencySession = openSemanticFile({
              fileName: dependencyPath,
              sourceText: 'export type Value = { readonly after: number; };\n',
              hasBOM: false,
            },);
            /** Importer source retained by dependency-refreshed snapshot. */
            const importerFromDependencySnapshot = dependencySession
              .project
              .program
              .getSourceFile(importerPath,);
            const refreshed = openSemanticFile({
              fileName: importerPath,
              sourceText: importerSource,
              hasBOM: false,
            },);
            expect(refreshed.sourceFile,).toBe(importerFromDependencySnapshot,);
            const parameter = refreshed.nodeAtOffset(importerSource.indexOf('value:',),);
            const type = refreshed.checker.getTypeAtLocation(parameter,);
            if (type === undefined)
              throw new Error('Expected importer parameter type after dependency overlay.',);
            expect(refreshed.checker.typeToString(type,),).toBe('Value',);
            expect(
              refreshed.checker
                .getPropertiesOfType(type,)
                .map(function propertyName(property,): string {
                  return property.name;
                },),
            ).toEqual(['after',],);
          },
        },),
        it({
          name: 'refreshes overlay for source an outer project already holds',
          fn: async () => {
            closeSemanticBridge();
            using directory = createSemanticFixtureDirectory();
            /** Nested project source the fixture project also contains. */
            const nestedPath = join(directory.path, 'input.ts',);
            /** Nested source text retained on disk. */
            const diskSource = 'export function read(value: { readonly before: string; },): void { void value; }\n';
            /** Nested source text supplied only through overlay. */
            const overlaidSource = 'export function read(value: { readonly after: number; },): void { void value; }\n';
            /* Both files exist before any snapshot, so the fixture project reads the nested source
             * from disk and the service holds it before the nested project is ever discovered. */
            writeFileSync(
              join(directory.path, 'tsconfig.json',),
              `${JSON.stringify({
                compilerOptions: { strict: true, },
                include: ['input.ts',],
              },)}\n`,
            );
            writeFileSync(nestedPath, diskSource,);
            /** Fixture session whose program pulls in nested source from disk. */
            const outer = openSemanticFile({
              fileName: FIXTURE_PATH,
              sourceText: SOURCE,
              hasBOM: false,
            },);
            /* Positive control. Without this the assertion below passes for the wrong reason,
             * since a service that never read the nested source has nothing stale to serve. */
            expect(
              outer.project.program
                .getSourceFile(nestedPath,)
                ?.text,
            ).toBe(diskSource,);
            /** Nested session whose project the walk refuses to answer from cache. */
            const session = openSemanticFile({
              fileName: nestedPath,
              sourceText: overlaidSource,
              hasBOM: false,
            },);
            const node = session.nodeAtOffset(overlaidSource.indexOf('value:',),);
            const type = session.checker.getTypeAtLocation(node,);
            if (type === undefined)
              throw new Error('Expected nested overlay parameter type.',);
            expect(session.checker.typeToString(type,),).toBe('{ readonly after: number; }',);
            expect(readFileSync(nestedPath, 'utf8',),).toBe(diskSource,);
          },
        },),
        it({
          name: 'fails closed when offset is outside source tree',
          fn: async () => {
            const session = openSemanticFile({
              fileName: FIXTURE_PATH,
              sourceText: SOURCE,
              hasBOM: false,
            },);
            let caught: unknown;
            try {
              session.nodeAtOffset(SOURCE.length + 1,);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(Error,);
            expect((caught as Error).message,).toContain('No TypeScript node contains offset',);
          },
        },),
        it({
          name: 'retains semantic nodes through parser recovery',
          fn: async () => {
            /** Incomplete member access retained by recovering parser. */
            const malformedSource = `export function inspect(value: { readonly text: string; },): string {\n  return value.\n`;
            /** Semantic session over malformed in-memory source. */
            const session = openSemanticFile({
              fileName: FIXTURE_PATH,
              sourceText: malformedSource,
              hasBOM: false,
            },);
            /** Recovered parameter identifier. */
            const node = session.nodeAtOffset(malformedSource.indexOf('value:',),);
            /** Recovered parameter type. */
            const type = session.checker.getTypeAtLocation(node,);
            if (type === undefined)
              throw new Error('Expected parser-recovered parameter type.',);
            expect(session.checker.typeToString(type,),).toBe('{ readonly text: string; }',);
          },
        },),
        it({
          name: 'invalidates deleted source during rename',
          fn: async () => {
            closeSemanticBridge();
            using directory = createSemanticFixtureDirectory();
            /** Original source path opened before rename. */
            const originalPath = join(directory.path, 'original.ts',);
            /** Renamed source path created after first snapshot. */
            const renamedPath = join(directory.path, 'renamed.ts',);
            /** Original source text. */
            const originalSource = 'export const originalValue: string = \'before\';\n';
            writeFileSync(originalPath, originalSource,);
            openSemanticFile({
              fileName: originalPath,
              sourceText: originalSource,
              hasBOM: false,
            },);
            renameSync(originalPath, renamedPath,);
            /** Changed text supplied at renamed path. */
            const renamedSource = 'export const renamedValue: number = 1;\n';
            writeFileSync(renamedPath, renamedSource,);
            /** Session after deleted plus created invalidation. */
            const session = openSemanticFile({
              fileName: renamedPath,
              sourceText: renamedSource,
              hasBOM: false,
            },);
            const node = session.nodeAtOffset(renamedSource.indexOf('renamedValue',),);
            const type = session.checker.getTypeAtLocation(node,);
            if (type === undefined)
              throw new Error('Expected renamed source type.',);
            expect(session.checker.typeToString(type,),).toBe('number',);
            /* The type above proves the created half. This proves the deleted half, which nothing
             * asserted while both halves rode on one flag. */
            expect(
              session.project.program
                .getSourceFile(originalPath,),
            ).toBe(undefined,);
          },
        },),
        it({
          name: 'invalidates a deleted source while reopening one already held',
          fn: async () => {
            closeSemanticBridge();
            using directory = createSemanticFixtureDirectory();
            /** Source deleted while another source of same project is reopened. */
            const goingPath = join(directory.path, 'going.ts',);
            /** Source already held by project when deletion is reported. */
            const stayingPath = join(directory.path, 'staying.ts',);
            /** Text of source that disappears. */
            const goingSource = 'export const goingValue: string = \'going\';\n';
            /** Text of source that remains. */
            const stayingSource = 'export const stayingValue: number = 1;\n';
            writeFileSync(goingPath, goingSource,);
            writeFileSync(stayingPath, stayingSource,);
            /* Open the staying source first so the project holds it, then the going one so it is
             * the active source. Reopening the staying source now takes the reuse-free path with no
             * discovery, which is the branch that carries the deletion in the second update. */
            openSemanticFile({
              fileName: stayingPath,
              sourceText: stayingSource,
              hasBOM: false,
            },);
            openSemanticFile({
              fileName: goingPath,
              sourceText: goingSource,
              hasBOM: false,
            },);
            rmSync(goingPath,);
            const session = openSemanticFile({
              fileName: stayingPath,
              sourceText: stayingSource,
              hasBOM: false,
            },);
            const node = session.nodeAtOffset(stayingSource.indexOf('stayingValue',),);
            const type = session.checker.getTypeAtLocation(node,);
            if (type === undefined)
              throw new Error('Expected staying source type.',);
            expect(session.checker.typeToString(type,),).toBe('number',);
            expect(
              session.project.program
                .getSourceFile(goingPath,),
            ).toBe(undefined,);
          },
        },),
        it({
          name: 'opens configured source through symbolic link path',
          fn: async () => {
            closeSemanticBridge();
            using directory = createSemanticFixtureDirectory();
            /** Real source path targeted by symbolic link. */
            const realPath = join(directory.path, 'real.ts',);
            /** Symbolic source path passed to bridge. */
            const linkedPath = join(directory.path, 'linked.ts',);
            /** Source shared by real and symbolic paths. */
            const source = 'export const linkedValue: string = \'linked\';\n';
            writeFileSync(realPath, source,);
            symlinkSync(realPath, linkedPath,);
            const session = openSemanticFile({
              fileName: linkedPath,
              sourceText: source,
              hasBOM: false,
            },);
            const node = session.nodeAtOffset(source.indexOf('linkedValue',),);
            const type = session.checker.getTypeAtLocation(node,);
            if (type === undefined)
              throw new Error('Expected symbolic-link source type.',);
            expect(session.checker.typeToString(type,),).toBe('string',);
          },
        },),
        it({
          name: 'accepts Windows source paths with noncanonical casing',
          fn: async () => {
            if (process.platform !== 'win32') {
              expect(true,).toBe(true,);
              return;
            }
            closeSemanticBridge();
            using directory = createSemanticFixtureDirectory();
            /** Canonically cased source written inside configured project. */
            const canonicalPath = join(directory.path, 'MixedCase.ts',);
            /** Noncanonical source path accepted by case-insensitive Windows filesystem. */
            const noncanonicalPath = join(directory.path, 'mixedcase.ts',);
            /** Source queried through noncanonical path. */
            const source = 'export const mixedCaseValue: string = \'case\';\n';
            writeFileSync(canonicalPath, source,);
            const session = openSemanticFile({
              fileName: noncanonicalPath,
              sourceText: source,
              hasBOM: false,
            },);
            const node = session.nodeAtOffset(source.indexOf('mixedCaseValue',),);
            const type = session.checker.getTypeAtLocation(node,);
            if (type === undefined)
              throw new Error('Expected noncanonical-path source type.',);
            expect(session.checker.typeToString(type,),).toBe('string',);
          },
        },),
        it({
          name: 'rejects inferred project and recovers for next configured source',
          fn: async () => {
            /** Disposable source outside every configured project. */
            const unconfiguredRoot = mkdtempSync(join(scratchRoot(), 'semantic-unconfigured-',),);
            using unconfigured: SemanticFixtureDirectory = {
              path: unconfiguredRoot,
              [Symbol.dispose]: function removeUnconfiguredFixture(): void {
                rmSync(unconfiguredRoot, { recursive: true, force: true, },);
              },
            };
            /** Unconfigured source path. */
            const fileName = join(unconfigured.path, 'input.ts',);
            /** Unconfigured source text. */
            const sourceText = 'export const value: string = \'outside\';\n';
            writeFileSync(fileName, sourceText,);
            let caught: unknown;
            try {
              openSemanticFile({ fileName, sourceText, hasBOM: false, },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(Error,);
            expect((caught as Error).message,).toContain('no configured project',);
            const recovered = openSemanticFile({
              fileName: FIXTURE_PATH,
              sourceText: SOURCE,
              hasBOM: false,
            },);
            expect(recovered.project.configFileName,).toContain(
              'package/test-fixture/oxlint-no-restricted-syntax/tsconfig.json',
            );
          },
        },),
        it({
          name: 'releases the snapshot it replaced when discovery fails',
          fn: async () => {
            closeSemanticBridge();
            using directory = createSemanticFixtureDirectory();
            /** Configured source opened before discovery fails for another one. */
            const heldPath = join(directory.path, 'held.ts',);
            /** Text of source held before failure. */
            const heldSource = 'export const heldValue: string = \'held\';\n';
            writeFileSync(heldPath, heldSource,);
            /** Session reading through snapshot that failure replaces. */
            const before = openSemanticFile({
              fileName: heldPath,
              sourceText: heldSource,
              hasBOM: false,
            },);
            /** Disposable source outside every configured project. */
            const unconfiguredRoot = mkdtempSync(join(scratchRoot(), 'semantic-unconfigured-',),);
            using unconfigured: SemanticFixtureDirectory = {
              path: unconfiguredRoot,
              [Symbol.dispose]: function removeUnconfiguredFixture(): void {
                rmSync(unconfiguredRoot, { recursive: true, force: true, },);
              },
            };
            /** Unconfigured source path. */
            const outsidePath = join(unconfigured.path, 'input.ts',);
            /** Unconfigured source text. */
            const outsideSource = 'export const value: string = \'outside\';\n';
            writeFileSync(outsidePath, outsideSource,);
            let refused: unknown;
            try {
              openSemanticFile({
                fileName: outsidePath,
                sourceText: outsideSource,
                hasBOM: false,
              },);
            }
            catch (error) {
              refused = error;
            }
            expect((refused as Error).message,).toContain('no configured project',);
            /* Keeping the newer snapshot is one half; letting go of the one it replaced is the
             * other, and nothing asks a snapshot whether it was disposed. Reading through the
             * replaced one does ask, because the native service no longer has it. */
            let caughtUse: unknown;
            try {
              before.project.program
                .getSourceFile(heldPath,);
            }
            catch (error) {
              caughtUse = error;
            }
            expect(caughtUse,).toBeInstanceOf(Error,);
            expect((caughtUse as Error).message,).toContain('snapshot',);
          },
        },),
        it({
          name: 'refuses to leave refused text where an importer can reach it',
          fn: async () => {
            closeSemanticBridge();
            /** Disposable dependency directory outside every configured project. */
            const dependencyRoot = mkdtempSync(join(scratchRoot(), 'semantic-refused-',),);
            using dependencyDirectory: SemanticFixtureDirectory = {
              path: dependencyRoot,
              [Symbol.dispose]: function removeRefusedFixture(): void {
                rmSync(dependencyRoot, { recursive: true, force: true, },);
              },
            };
            using directory = createSemanticFixtureDirectory();
            /** Dependency path this bridge refuses before any importer names it. */
            const dependencyPath = join(dependencyDirectory.path, 'outside.ts',);
            /** Dependency text retained on disk. */
            const diskSource = 'export type Value = { readonly fromDisk: string; };\n';
            writeFileSync(dependencyPath, diskSource,);
            let caught: unknown;
            try {
              openSemanticFile({
                fileName: dependencyPath,
                sourceText: 'export type Value = { readonly fromOverlay: number; };\n',
                hasBOM: false,
              },);
            }
            catch (error) {
              caught = error;
            }
            expect((caught as Error).message,).toContain('no configured project',);
            /* The refusal is not the end of that text. Discovery handed it to the service, and the
             * service keeps it until told otherwise, so a configured source importing this path
             * gets typed against text this bridge refused and disk never had. */
            const importerPath = join(directory.path, 'importer.ts',);
            /** Import specifier reaching the refused dependency from configured project. */
            const specifier = relative(directory.path, dependencyPath,)
              .replaceAll('\\', '/',);
            /** Configured importer naming refused dependency. */
            const importerSource =
              `import type { Value } from '${specifier}';\nexport function read(value: Value,): Value { return value; }\n`;
            writeFileSync(importerPath, importerSource,);
            const session = openSemanticFile({
              fileName: importerPath,
              sourceText: importerSource,
              hasBOM: false,
            },);
            const type = session.checker.getTypeAtLocation(
              session.nodeAtOffset(importerSource.indexOf('value:',),),
            );
            if (type === undefined)
              throw new Error('Expected imported dependency type.',);
            expect(
              session.checker
                .getPropertiesOfType(type,)
                .map(function propertyName(property,): string {
                  return property.name;
                },),
            ).toEqual(['fromDisk',],);
          },
        },),
        it({
          name: 'reopens a source the failed discovery reported as deleted',
          fn: async () => {
            closeSemanticBridge();
            using directory = createSemanticFixtureDirectory();
            /** Configured source held before deletion. */
            const heldPath = join(directory.path, 'held.ts',);
            /** Source text present at first open. */
            const firstSource = 'export const heldValue: string = \'first\';\n';
            /** Source text present after recreation. */
            const secondSource = 'export const heldValue: number = 2;\n';
            writeFileSync(heldPath, firstSource,);
            openSemanticFile({
              fileName: heldPath,
              sourceText: firstSource,
              hasBOM: false,
            },);
            /* Deleting the active source makes the next open report it as deleted, and that open
             * fails discovery, so the deletion reaches the service through an update whose snapshot
             * the bridge used to throw away. */
            rmSync(heldPath,);
            /** Disposable source outside every configured project. */
            const unconfiguredRoot = mkdtempSync(join(scratchRoot(), 'semantic-unconfigured-',),);
            using unconfigured: SemanticFixtureDirectory = {
              path: unconfiguredRoot,
              [Symbol.dispose]: function removeUnconfiguredFixture(): void {
                rmSync(unconfiguredRoot, { recursive: true, force: true, },);
              },
            };
            /** Unconfigured source path. */
            const outsidePath = join(unconfigured.path, 'input.ts',);
            /** Unconfigured source text. */
            const outsideSource = 'export const value: string = \'outside\';\n';
            writeFileSync(outsidePath, outsideSource,);
            let caught: unknown;
            try {
              openSemanticFile({
                fileName: outsidePath,
                sourceText: outsideSource,
                hasBOM: false,
              },);
            }
            catch (error) {
              caught = error;
            }
            expect((caught as Error).message,).toContain('no configured project',);
            writeFileSync(heldPath, secondSource,);
            /** Session over recreated source, which the service no longer holds. */
            const session = openSemanticFile({
              fileName: heldPath,
              sourceText: secondSource,
              hasBOM: false,
            },);
            const node = session.nodeAtOffset(secondSource.indexOf('heldValue',),);
            const type = session.checker.getTypeAtLocation(node,);
            if (type === undefined)
              throw new Error('Expected recreated source type.',);
            expect(session.checker.typeToString(type,),).toBe('number',);
          },
        },),
        it({
          name: 'discovers nested project after caching containing parent project',
          fn: async () => {
            closeSemanticBridge();
            openSemanticFile({ fileName: FIXTURE_PATH, sourceText: SOURCE, hasBOM: false, },);
            using directory = createSemanticFixtureDirectory();
            /** Nested project source path also contained by parent project include. */
            const fileName = join(directory.path, 'input.ts',);
            /** Nested project source text. */
            const sourceText = 'export const nestedValue: string = \'nested\';\n';
            writeFileSync(
              join(directory.path, 'tsconfig.json',),
              `${JSON.stringify({
                compilerOptions: { strict: true, },
                include: ['input.ts',],
              },)}\n`,
            );
            writeFileSync(fileName, sourceText,);
            const session = openSemanticFile({
              fileName,
              sourceText,
              hasBOM: false,
            },);
            expect(
              configuredProjectConfigFileName({ project: session.project, },)
                .replaceAll('\\', '/',)
                .toLowerCase(),
            ).toBe(
              join(directory.path, 'tsconfig.json',).replaceAll('\\', '/',).toLowerCase(),
            );
          },
        },),
        it({
          name: 'retains the text handed for each source, and one project per configured root',
          fn: async () => {
            closeSemanticBridge();
            const first = openSemanticFile({
              fileName: FIXTURE_PATH,
              sourceText: SOURCE,
              hasBOM: false,
            },);
            const second = openSemanticFile({
              fileName: FIXTURE_PATH,
              sourceText: SOURCE,
              hasBOM: false,
            },);
            expect(second.sourceFile,).toBe(first.sourceFile,);
            expect(semanticBridgeCacheStats(),).toEqual({
              overlayCount: 1,
              projectRootCount: 1,
              projectDiscoveryCount: 1,
            },);
            /* A second, different source of the same project. Reopening the same path twice
             * cannot tell retention from clearing, which is what the earlier form of this
             * assertion did, so it passed either way and measured nothing. */
            openSemanticFile({
              fileName: SIBLING_PATH,
              sourceText: SIBLING_SOURCE,
              hasBOM: false,
            },);
            /* Two overlays, not one. The bridge no longer clears down to the active source,
             * because clearing left the native server holding text for a source the overlay had
             * stopped claiming, and nothing ever reported that source as changed. */
            /* One discovery, not two. The sibling belongs to a project this bridge has already
             * discovered, and it reaches that answer through the root cache rather than by asking
             * TypeScript again. A root key spelled the way the host spells paths, rather than the
             * way this bridge normalizes them, misses here and asks twice. */
            expect(semanticBridgeCacheStats(),).toEqual({
              overlayCount: 2,
              projectRootCount: 1,
              projectDiscoveryCount: 1,
            },);
          },
        },),
        it({
          name: 'closes idempotently and starts again',
          fn: async () => {
            closeSemanticBridge();
            closeSemanticBridge();
            const session = openSemanticFile({
              fileName: FIXTURE_PATH,
              sourceText: SOURCE,
              hasBOM: false,
            },);
            expect(
              session.sourceFile.fileName.replaceAll('\\', '/',).toLowerCase(),
            ).toBe(FIXTURE_PATH.replaceAll('\\', '/',).toLowerCase(),);
            closeSemanticBridge();
          },
        },),
      ],
    },),
    describe({
      name: typescriptOffset.name,
      children: [
        it({
          name: 'restores one UTF-16 unit for stripped byte-order mark',
          fn: async () => {
            expect(typescriptOffset({ offset: 4, hasBOM: true, },),).toBe(5,);
            expect(typescriptOffset({ offset: 4, hasBOM: false, },),).toBe(4,);
          },
        },),
      ],
    },),
  ],
},);
