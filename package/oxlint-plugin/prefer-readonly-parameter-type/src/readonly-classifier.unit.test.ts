import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from 'node:fs';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';
import { fileURLToPath, } from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  classifyReadonlyType,
  closeSemanticBridge,
  openSemanticFile,
} from '../dist/final/node/index.mjs';

/** Readonly classifier semantic fixture. */
const FIXTURE_PATH = fileURLToPath(new URL(
  '../../../test-fixture/oxlint-no-restricted-syntax/src/valid/typescript-sync-adapter.ts',
  import.meta.url,
),);

/** Current classifier fixture text. */
const SOURCE = readFileSync(
  FIXTURE_PATH,
  'utf8',
);

/** Project-owned foreign marker fixture source. */
const FOREIGN_FIXTURE_PATH = fileURLToPath(new URL(
  'prefer-readonly-parameter-types.ts',
  import.meta.url,
),);

/** Current foreign marker fixture text. */
const FOREIGN_SOURCE = readFileSync(
  FOREIGN_FIXTURE_PATH,
  'utf8',
);

/** Source of the project that imports a workspace package through a `node_modules` symbolic link. */
const LINKED_IMPORTER_SOURCE = "import type { LinkedConfig, } from 'linked-workspace-package';\nexport function read(config: LinkedConfig,): string { return config.value; }\n";

/** Source of the workspace package, whose real path carries no `node_modules` segment. */
const LINKED_PACKAGE_SOURCE = 'export type LinkedConfig = { value: string; };\n';

/** Disposable directory removed after the test scope. */
type DisposableLinkedRoot = {
  readonly path: string;
  readonly [Symbol.dispose]: () => void;
};

/**
 Creates the disposable directory holding the importer and the linked package.
 
 @returns directory removed after the test scope.
 */
function disposableLinkedRoot(): DisposableLinkedRoot {
  /** Directory created for one case. */
  const path = mkdtempSync(join(scratchRoot(), 'readonly-owner-link-',),);
  return {
    path,
    [Symbol.dispose](): void {
      rmSync(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Writes a project importing a workspace package through a `node_modules` symbolic link.
 
 TypeScript resolves the import through the link, so the package's real source path is a
 workspace path while the program flags the file as an external-library module.
 
 @param root - Disposable directory receiving the importer and the package.
 
 @returns real path of the importer.
 */
function writeLinkedWorkspaceFixture(root: string,): string {
  /** Real directory of the importing project. */
  const importer = join(root, 'importer',);
  /** Real directory of the workspace package. */
  const workspacePackage = join(root, 'workspace-package',);
  mkdirSync(join(importer, 'node_modules',), { recursive: true, },);
  mkdirSync(workspacePackage, { recursive: true, },);
  writeFileSync(join(importer, 'tsconfig.json',), '{"compilerOptions":{"strict":true},"include":["*.ts"]}\n',);
  writeFileSync(join(importer, 'input.ts',), LINKED_IMPORTER_SOURCE,);
  writeFileSync(
    join(workspacePackage, 'package.json',),
    '{"name":"linked-workspace-package","version":"1.0.0","types":"index.ts","main":"index.ts"}\n',
  );
  writeFileSync(join(workspacePackage, 'index.ts',), LINKED_PACKAGE_SOURCE,);
  symlinkSync(workspacePackage, join(importer, 'node_modules', 'linked-workspace-package',), 'dir',);
  return join(realpathSync(importer,), 'input.ts',);
}

await describe({
  name: classifyReadonlyType.name,
  concurrency: 1,
  children: [
    ...[
      {
        parameter: 'mutableObject:',
        expected: 'mutable',
      },
      {
        parameter: 'readonlyObject:',
        expected: 'deep-readonly',
      },
      {
        parameter: 'objectValue:',
        expected: 'opaque-capability',
      },
      {
        parameter: 'callableValue:',
        expected: 'opaque-capability',
      },
      {
        parameter: 'constructorValue:',
        expected: 'opaque-capability',
      },
      {
        parameter: 'unknownValue:',
        expected: 'opaque-capability',
      },
      {
        parameter: 'anyValue:',
        expected: 'opaque-capability',
      },
      {
        parameter: 'readonlyEncoder:',
        expected: 'projected-readonly-capability',
      },
      {
        parameter: 'encodeOnly:',
        expected: 'opaque-capability',
      },
      {
        parameter: 'projectedController:',
        expected: 'deep-readonly',
      },
      {
        parameter: 'capabilityController:',
        expected: 'opaque-capability',
      },
      {
        parameter: 'readonlyMap:',
        expected: 'deep-readonly',
      },
      {
        parameter: 'readonlyValues:',
        expected: 'deep-readonly',
      },
      {
        parameter: 'shallowReadonlyValues:',
        expected: 'mutable',
      },
      {
        parameter: 'shallowReadonlyMap:',
        expected: 'mutable',
      },
      {
        parameter: 'projectedMethod:',
        expected: 'projected-readonly-capability',
      },
      {
        parameter: 'originalMethod:',
        expected: 'opaque-capability',
      },
    ].map(function classificationCase({ parameter, expected, },) {
      return it({
        name: `classifies ${parameter} as ${expected}`,
        fn: async () => {
          const session = openSemanticFile({
            fileName: FIXTURE_PATH,
            sourceText: SOURCE,
            hasBOM: false,
          },);
          const parameterNode = session.nodeAtOffset(SOURCE.indexOf(parameter,),);
          const type = session.checker.getTypeAtLocation(parameterNode,);
          if (type === undefined)
            throw new Error(`Expected semantic type for ${parameter}.`,);
          const classification = classifyReadonlyType({
            checker: session.checker,
            project: session.project,
            type,
          },);
          closeSemanticBridge();
          expect(classification.kind,).toBe(expected,);
        },
      },);
    },),
    it({
      name: 'retains structured workspace ownership without intrinsic collection owners',
      fn: async () => {
        const session = openSemanticFile({
          fileName: FIXTURE_PATH,
          sourceText: SOURCE,
          hasBOM: false,
        },);
        const parameterNode = session.nodeAtOffset(SOURCE.indexOf('shallowReadonlyValues:',),);
        const type = session.checker.getTypeAtLocation(parameterNode,);
        if (type === undefined)
          throw new Error('Expected shallow readonly values type.',);
        const classification = classifyReadonlyType({
          checker: session.checker,
          project: session.project,
          type,
        },);
        closeSemanticBridge();
        if (classification.kind !== 'mutable')
          throw new Error('Expected mutable shallow readonly values.',);
        expect(classification.writablePaths.every(function workspaceLeaf(path,): boolean {
          return path.declarationOwners.includes('workspace',)
            && (!path.declarationOwners.includes('default-library',));
        },),).toBe(true,);
      },
    },),
    it({
      name: 'classifies a workspace package reached through a node_modules symbolic link as workspace',
      fn: async () => {
        using root = disposableLinkedRoot();
        /** Importer whose parameter type is declared in the linked package. */
        const inputPath = writeLinkedWorkspaceFixture(root.path,);
        const session = openSemanticFile({
          fileName: inputPath,
          sourceText: LINKED_IMPORTER_SOURCE,
          hasBOM: false,
        },);
        const parameterNode = session.nodeAtOffset(LINKED_IMPORTER_SOURCE.indexOf('config:',),);
        const type = session.checker.getTypeAtLocation(parameterNode,);
        if (type === undefined)
          throw new Error('Expected the linked package type.',);
        /** Source file of the type, which the program holds under the real workspace path. */
        const declarationFile = session.project.program
          .getSourceFile(join(realpathSync(root.path,), 'workspace-package', 'index.ts',),);
        if (declarationFile === undefined)
          throw new Error('Expected the linked package source in the program.',);
        /* The premise of the case: the program flags a workspace file reached through the link as external. */
        expect(session.project.program.isSourceFileFromExternalLibrary(declarationFile,),).toBe(true,);
        const classification = classifyReadonlyType({
          checker: session.checker,
          project: session.project,
          type,
        },);
        closeSemanticBridge();
        if (classification.kind !== 'mutable')
          throw new Error('Expected the linked package type to be mutable.',);
        expect(classification.writablePaths.map(function owners(path,): readonly string[] {
          return path.declarationOwners;
        },),).toEqual([['workspace',],],);
      },
    },),
    it({
      name: 'merges workspace and default-library owners on one writable path',
      fn: async () => {
        const session = openSemanticFile({
          fileName: FIXTURE_PATH,
          sourceText: SOURCE,
          hasBOM: false,
        },);
        const parameterNode = session.nodeAtOffset(SOURCE.indexOf('mixedNamedValue:',),);
        const type = session.checker.getTypeAtLocation(parameterNode,);
        if (type === undefined)
          throw new Error('Expected mixed writable owner type.',);
        const classification = classifyReadonlyType({
          checker: session.checker,
          project: session.project,
          type,
        },);
        closeSemanticBridge();
        if (classification.kind !== 'mutable')
          throw new Error('Expected mutable mixed owner type.',);
        /**
         Shared `name` path merged across both union branches.
         */
        const mixedPath = classification.writablePaths.find(function namePath(path,): boolean {
          return path.segments.some(function nameSegment(segment,): boolean {
            return (segment.kind === 'property') && (segment.name === 'name');
          },);
        },);
        expect(mixedPath?.declarationOwners,).toEqual([
          'default-library',
          'workspace',
        ],);
      },
    },),
    it({
      name: 'classifies a cycle member the same whichever member is asked for first',
      fn: async () => {
        /* One session for both, because the defect this guards is the shared classification
         * store carrying an answer from the first walk into the second. Two sessions would pass
         * with the guard removed. */
        const session = openSemanticFile({
          fileName: FIXTURE_PATH,
          sourceText: SOURCE,
          hasBOM: false,
        },);
        /**
         Kinds for both cycle parameters, head asked for first.
         */
        const kinds = [
          'cycleHead:',
          'cycleMember:',
        ].map(function classifyCycleParameter(parameter,): string {
          const parameterNode = session.nodeAtOffset(SOURCE.indexOf(parameter,),);
          const type = session.checker
            .getTypeAtLocation(parameterNode,);
          if (type === undefined)
            throw new Error(`Expected semantic type for ${parameter}.`,);
          return classifyReadonlyType({
            checker: session.checker,
            project: session.project,
            type,
          },).kind;
        },);
        closeSemanticBridge();
        /* Both reach the head's writable slot, the member through readonly properties alone.
         * Reading deep-readonly for the member is the wrong-offer direction: it withholds the
         * opaque effect an outward handoff would charge. */
        expect(kinds,).toEqual([
          'mutable',
          'mutable',
        ],);
      },
    },),
    it({
      name: 'classifies exact foreign borrowed marker as capability',
      fn: async () => {
        const session = openSemanticFile({
          fileName: FOREIGN_FIXTURE_PATH,
          sourceText: FOREIGN_SOURCE,
          hasBOM: false,
        },);
        const parameterNode = session.nodeAtOffset(FOREIGN_SOURCE.indexOf('context: ForeignBorrowed',),);
        const type = session.checker.getTypeAtLocation(parameterNode,);
        if (type === undefined)
          throw new Error('Expected semantic foreign borrowed type.',);
        const classification = classifyReadonlyType({
          checker: session.checker,
          project: session.project,
          type,
        },);
        closeSemanticBridge();
        expect(classification.kind,).toBe('opaque-capability',);
      },
    },),
  ],
},);
