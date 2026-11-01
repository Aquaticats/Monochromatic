import {
  mkdirSync,
  mkdtempSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from 'node:fs';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { isParameterDeclaration, } from 'typescript/ast/is';

import {
  classifyReadonlyType,
  closeSemanticBridge,
  openSemanticFile,
  readonlyPreferenceGuidance,
  type ReadonlyTypeOriginEvidence,
  readonlyTypeOriginEvidence,
} from '../dist/final/node/index.mjs';

/** Source declaring the type and the callback-taking function every case shares. */
const PACKAGE_SOURCE = 'export type LinkedConfig = { value: string; };\nexport function useConfig(callback: (config: LinkedConfig,) => string,): string { return callback({ value: "", },); }\n';

/** Importer whose callback parameter carries no annotation, so its type comes from the context. */
const LINKED_CALLBACK_SOURCE = "import { useConfig, } from 'linked-workspace-package';\nuseConfig((config,) => config.value,);\n";

/** Importer reaching the same declaration by a relative path inside the workspace. */
const LOCAL_CALLBACK_SOURCE = "import { useConfig, } from './local-package/index.ts';\nuseConfig((config,) => config.value,);\n";

/** Configuration letting the importer name its relative `.ts` import. */
const TSCONFIG_SOURCE = '{"compilerOptions":{"strict":true,"allowImportingTsExtensions":true,"noEmit":true},"include":["*.ts"]}\n';

/** Disposable directory removed after the test scope. */
type DisposableRoot = {
  readonly path: string;
  readonly [Symbol.dispose]: () => void;
};

/**
 Creates a disposable directory for one fixture project.

 @returns directory removed after the test scope.
 */
function disposableRoot(): DisposableRoot {
  /** Directory created for one case. */
  const path = mkdtempSync(join(scratchRoot(), 'readonly-origin-link-',),);
  return {
    path,
    [Symbol.dispose](): void {
      rmSync(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Writes the project reaching its declaration through a `node_modules` symbolic link.

 @param root - Disposable directory receiving the importer and the package.

 @returns real path of the importer file.
 */
function writeLinkedProject(root: string,): string {
  /** Real directory of the importing project. */
  const importer = join(root, 'importer',);
  /** Real directory of the workspace package, whose path carries no `node_modules` segment. */
  const workspacePackage = join(root, 'workspace-package',);
  mkdirSync(join(importer, 'node_modules',), { recursive: true, },);
  mkdirSync(workspacePackage, { recursive: true, },);
  writeFileSync(join(importer, 'tsconfig.json',), TSCONFIG_SOURCE,);
  writeFileSync(join(importer, 'input.ts',), LINKED_CALLBACK_SOURCE,);
  writeFileSync(
    join(workspacePackage, 'package.json',),
    '{"name":"linked-workspace-package","version":"1.0.0","types":"index.ts","main":"index.ts"}\n',
  );
  writeFileSync(join(workspacePackage, 'index.ts',), PACKAGE_SOURCE,);
  symlinkSync(workspacePackage, join(importer, 'node_modules', 'linked-workspace-package',), 'dir',);
  return join(realpathSync(importer,), 'input.ts',);
}

/**
 Writes the project reaching its declaration by a relative path.

 @param root - Disposable directory receiving the importer and the package.

 @returns real path of the importer file.
 */
function writeLocalProject(root: string,): string {
  /** Real directory of the importing project. */
  const importer = join(root, 'importer',);
  mkdirSync(join(importer, 'local-package',), { recursive: true, },);
  writeFileSync(join(importer, 'tsconfig.json',), TSCONFIG_SOURCE,);
  writeFileSync(join(importer, 'input.ts',), LOCAL_CALLBACK_SOURCE,);
  writeFileSync(join(importer, 'local-package', 'index.ts',), PACKAGE_SOURCE,);
  return join(realpathSync(importer,), 'input.ts',);
}

/**
 Guidance and origin evidence the rule would give the unannotated callback parameter.
 */
type CallbackGuidance = {
  readonly evidence: ReadonlyTypeOriginEvidence;
  readonly guidance: string;
  readonly declarationIsExternalLibrary: boolean;
};

/**
 Computes what the rule says about the unannotated `config` callback parameter.

 @param inputPath - Real path of the importer file.

 @param sourceText - Importer text, which the rule reads from the editor overlay.

 @param declarationPath - Real path of the file declaring the parameter type.

 @returns origin evidence, guidance text and the program's external-library flag for the declaration.
 */
function callbackGuidance({
  inputPath,
  sourceText,
  declarationPath,
}: {
  readonly inputPath: string;
  readonly sourceText: string;
  readonly declarationPath: string;
},): CallbackGuidance {
  /** Semantic session over the importer. */
  const session = openSemanticFile({
    fileName: inputPath,
    sourceText,
    hasBOM: false,
  },);
  /** Name node of the callback parameter. */
  const nameNode = session.nodeAtOffset(sourceText.indexOf('config,) =>',),);
  /** Parameter declaration owning that name. */
  const parameter = nameNode.parent;
  if ((parameter === undefined) || (!isParameterDeclaration(parameter,)))
    throw new Error('Expected the callback parameter declaration.',);
  /** Type the context gives the parameter. */
  const parameterType = session.checker.getTypeAtLocation(nameNode,);
  if (parameterType === undefined)
    throw new Error('Expected the callback parameter type.',);
  /** Declaration file of the type, held under its real path. */
  const declarationFile = session.project.program.getSourceFile(declarationPath,);
  if (declarationFile === undefined)
    throw new Error('Expected the declaration file in the program.',);
  /** Whether the program flags the declaration as an external-library module. */
  const declarationIsExternalLibrary = session.project.program.isSourceFileFromExternalLibrary(declarationFile,);
  /** Mutable classification the guidance is chosen from. */
  const classification = classifyReadonlyType({
    checker: session.checker,
    project: session.project,
    type: parameterType,
  },);
  /** Origin the rule proves for the unannotated parameter. */
  const evidence = readonlyTypeOriginEvidence({
    parameter,
    parameterType,
    project: session.project,
  },);
  closeSemanticBridge();
  if (classification.kind !== 'mutable')
    throw new Error('Expected the callback parameter type to be mutable.',);
  return {
    evidence,
    guidance: readonlyPreferenceGuidance({
      suggestions: [],
      originEvidence: evidence,
      classification,
    },),
    declarationIsExternalLibrary,
  };
}

/**
 Guidance the rule gives when the sole origin is a type declared at a location.

 @param location - Repository-relative location the guidance names.

 @returns the sentence pair the rule prints for a unique type origin.
 */
function uniqueTypeGuidance(location: string,): string {
  return `Its inferred parameter type originates in type "LinkedConfig" at ${location}. Likely edit: make the reported writable path deeply readonly in that type, or introduce a deeply readonly result type at the producer boundary. No exact type syntax was proved for that producer; run type checking after the edit.`;
}

await describe({
  name: readonlyTypeOriginEvidence.name,
  concurrency: 1,
  children: [
    it({
      name: 'names the unique origin of a callback type declared in the workspace',
      fn: async () => {
        using root = disposableRoot();
        /** Importer path of the in-workspace case. */
        const inputPath = writeLocalProject(root.path,);
        /** Outcome for the callback parameter. */
        const outcome = callbackGuidance({
          inputPath,
          sourceText: LOCAL_CALLBACK_SOURCE,
          declarationPath: join(realpathSync(root.path,), 'importer', 'local-package', 'index.ts',),
        },);
        expect(outcome.declarationIsExternalLibrary,).toBe(false,);
        expect(outcome.evidence.kind,).toBe('unique',);
        expect(outcome.guidance,).toBe(uniqueTypeGuidance('local-package/index.ts:1',),);
      },
    },),
    it({
      name: 'names the same unique origin when the declaration is reached through a node_modules symbolic link',
      fn: async () => {
        using root = disposableRoot();
        /** Importer path of the linked case. */
        const inputPath = writeLinkedProject(root.path,);
        /** Outcome for the callback parameter. */
        const outcome = callbackGuidance({
          inputPath,
          sourceText: LINKED_CALLBACK_SOURCE,
          declarationPath: join(realpathSync(root.path,), 'workspace-package', 'index.ts',),
        },);
        /* The premise of the case: the program flags a workspace file reached through the link as external. */
        expect(outcome.declarationIsExternalLibrary,).toBe(true,);
        expect(outcome.evidence.kind,).toBe('unique',);
        expect(outcome.guidance,).toBe(uniqueTypeGuidance('../workspace-package/index.ts:1',),);
      },
    },),
  ],
},);
