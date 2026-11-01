import { existsSync, } from 'node:fs';
import {
  dirname,
  join,
  relative,
  sep,
} from 'node:path';

import type { Node, } from 'typescript/ast';
import {
  isArrayLiteralExpression,
  isArrowFunction,
  isBlock,
  isClassDeclaration,
  isFunctionLikeDeclaration,
  isIdentifier,
  isInterfaceDeclaration,
  isObjectLiteralExpression,
  isReturnStatement,
  isTypeAliasDeclaration,
  isVariableDeclaration,
} from 'typescript/ast/is';
import type { Project, } from 'typescript/sync';

import { ancestorDirectories, } from './ancestor-directories.ts';
import { callableReturnsBinding, } from './readonly-returned-binding.ts';
import { configuredProjectConfigFileName, } from './semantic-file-name.ts';
import { isWorkspaceSourceFileName, } from './workspace-source-path.ts';

/**
 Workspace marker used to shorten producer locations.
 */
const WORKSPACE_MARKER = 'pnpm-workspace.yaml';

/**
 Display roots already found for configured projects.
 */
const displayRootsByConfig = new Map<string, string>();

/**
 Sentinel for origin boundary carrying no stable identifier name.
 */
const ORIGIN_NAME_UNAVAILABLE: unique symbol = Symbol('origin boundary name unavailable');

/**
 Eager immutable description of one editable semantic type origin.
 */
export type ReadonlyTypeOrigin = {
  readonly identity: string;
  readonly kind: 'callable' | 'type' | 'expression';
  readonly name?: string;
  readonly location: string;
};

/**
 Finds repository display root for configured project.
 
 @param configFileName - Configured TypeScript project path.
 
 @returns nearest pnpm workspace root or configured-project directory.
 
 @example
 ```ts
 displayRoot('/repo/package/module/example/tsconfig.json');
 ```
 */
function displayRoot(configFileName: string,): string {
  /**
   Previously resolved display root for configured project.
   */
  const cached = displayRootsByConfig.get(configFileName,);
  if (cached !== undefined)
    return cached;
  /**
   Package-local fallback when consumer has no pnpm workspace marker.
   */
  const fallback = dirname(configFileName,);
  /**
   Root selected from nearest workspace marker.
   */
  const selected = { value: fallback, };
  for (const directory of ancestorDirectories(fallback,)) {
    /**
     Marker candidate under current ancestor.
     */
    const marker = join(
      directory,
      WORKSPACE_MARKER,
    );
    // oxlint-disable-next-line no-restricted-syntax/no-sync -- Synchronous rule resolves one cached display root before emitting source guidance.
    if (existsSync(marker,)) {
      selected.value = directory;
      break;
    }
  }
  displayRootsByConfig.set(
    configFileName,
    selected.value,
  );
  return selected.value;
}

/**
 Normalizes declaration to reader-facing producer boundary.
 
 @param declaration - Semantic type declaration.
 
 @param project - Project proving local binding return flow.
 
 @returns named type owner,
 proved returning callable,
 or exact original declaration.
 
 @example
 ```ts
 originOwner({ declaration: objectLiteral, project });
 ```
 */
function originOwner({
  declaration,
  project,
}: {
  readonly declaration: Node;
  readonly project: Project;
}): Node {
  /**
   Ancestor cursor retaining nearest actionable local aggregate.
   */
  const cursor = {
    current: declaration,
    hasLocalBinding: false,
    localBinding: declaration,
    localProducer: declaration,
    pending: true,
    returned: false,
  };
  while (cursor.pending) {
    if (isTypeAliasDeclaration(cursor.current,)
      || isInterfaceDeclaration(cursor.current,)
      || isClassDeclaration(cursor.current,))
      return cursor.current;
    if (isFunctionLikeDeclaration(cursor.current,)) {
      /**
       Whether concise arrow body returns declaration without return statement.
       */
      const conciseArrow = isArrowFunction(cursor.current,)
        && (!isBlock(cursor.current
          .body,));
      /**
       Whether local aggregate binding reaches this callable's return.
       */
      const returnedBinding = cursor.hasLocalBinding
        && isIdentifier(cursor.localBinding,)
        && callableReturnsBinding({
          callable: cursor.current,
          binding: cursor.localBinding,
          project,
        },);
      if ((cursor.current === declaration)
        || cursor.returned
        || conciseArrow
        || returnedBinding)
        return cursor.current;
      return cursor.localProducer;
    }
    if (isObjectLiteralExpression(cursor.current,)
      || isArrayLiteralExpression(cursor.current,))
      cursor.localProducer = cursor.current;
    if (isVariableDeclaration(cursor.current,)) {
      /**
       Binding name candidate for semantic return-flow proof.
       */
      const { name, } = cursor.current;
      if (isIdentifier(name,)) {
        cursor.hasLocalBinding = true;
        cursor.localBinding = name;
      }
    }
    if (isReturnStatement(cursor.current,))
      cursor.returned = true;
    /**
     Next owner candidate in semantic source tree.
     */
    const { parent, } = cursor.current;
    cursor.pending = parent !== undefined;
    if (parent !== undefined)
      cursor.current = parent;
  }
  return cursor.localProducer;
}

/**
 Reads stable local name for origin owner.
 
 @param owner - Normalized producer boundary.
 
 @returns identifier name when boundary declares one.
 
 @example
 ```ts
 originName(callback);
 ```
 */
function originName(
  owner: Node,
): string | typeof ORIGIN_NAME_UNAVAILABLE {
  if (isFunctionLikeDeclaration(owner,)) {
    if (!('name' in owner))
      return ORIGIN_NAME_UNAVAILABLE;
    /**
     Optional callable name narrowed outside property access.
     */
    const callableName = owner.name;
    if (callableName === undefined)
      return ORIGIN_NAME_UNAVAILABLE;
    return isIdentifier(callableName,)
      ? callableName.text
      : ORIGIN_NAME_UNAVAILABLE;
  }
  if (isTypeAliasDeclaration(owner,) || isInterfaceDeclaration(owner,))
    return owner.name
      .text;
  if (!isClassDeclaration(owner,))
    return ORIGIN_NAME_UNAVAILABLE;
  /**
   Optional class name narrowed outside property access.
   */
  const className = owner.name;
  if (className === undefined)
    return ORIGIN_NAME_UNAVAILABLE;
  return className
    .text;
}

/**
 Classifies normalized origin boundary for diagnostic wording.
 
 @param owner - Normalized producer boundary.
 
 @returns reader-facing origin category.
 
 @example
 ```ts
 originKind(callback);
 ```
 */
function originKind(owner: Node,): ReadonlyTypeOrigin['kind'] {
  if (isFunctionLikeDeclaration(owner,))
    return 'callable';
  if (isTypeAliasDeclaration(owner,)
    || isInterfaceDeclaration(owner,)
    || isClassDeclaration(owner,))
    return 'type';
  return 'expression';
}

/**
 Formats eager repository-relative origin location.
 
 @param owner - Origin boundary resolved in active semantic snapshot.
 
 @param project - Project owning active semantic snapshot.
 
 @returns normalized path and one-based line.
 
 @example
 ```ts
 originLocation({ owner, project });
 ```
 */
function originLocation({
  owner,
  project,
}: {
  readonly owner: Node;
  readonly project: Project;
},): string {
  /**
   Source owning producer boundary.
   */
  const sourceFile = owner.getSourceFile();
  /**
   Path relative to nearest workspace marker.
   */
  const relativePath = relative(
    displayRoot(configuredProjectConfigFileName({ project, },),),
    sourceFile.fileName,
  )
    .split(sep,)
    .join('/',);
  /**
   One-based line containing producer boundary.
   */
  const lineAndCharacter = sourceFile
    .getLineAndCharacterOfPosition(owner.getStart(sourceFile,),);
  /**
   One-based source line.
   */
  const line = lineAndCharacter
    .line + 1;
  return `${relativePath}:${String(line,)}`;
}

/**
 Tests whether declaration source is editable workspace source.

 A path outside every installed-module directory is workspace source whatever the program's
 library flag says. A workspace package reached through a `node_modules` symbolic link has such
 a path and is flagged as an external-library module, yet its declarations are the ones an edit
 would change, so origin collection keeps them and the evidence kind agrees with the
 in-workspace case. A path inside an installed-module directory is never workspace source, and
 the default library is refused before the path is read.

 @param node - Resolved declaration or normalized owner.

 @param project - Project classifying source ownership.

 @returns whether source belongs to inspectable workspace implementation.
 
 @example
 ```ts
 workspaceOrigin({ node, project });
 ```
 */
export function workspaceOrigin({
  node,
  project,
}: {
  readonly node: Node;
  readonly project: Project;
},): boolean {
  /**
   Source file inspected through active project metadata.
   */
  const sourceFile = node.getSourceFile();
  /**
   Program metadata recognizing the default library, which has no installed-module path.
   */
  const { program, } = project;
  return isWorkspaceSourceFileName(sourceFile.fileName,)
    && (!program.isSourceFileDefaultLibrary(sourceFile,));
}

/**
 Converts resolved declaration into eager reader-facing origin.
 
 @param declaration - Declaration resolved in active semantic snapshot.
 
 @param project - Project owning active semantic snapshot.
 
 @returns immutable origin metadata safe across later snapshots.
 
 @example
 ```ts
 readonlyTypeOrigin({ declaration, project });
 ```
 */
export function readonlyTypeOrigin({
  declaration,
  project,
}: {
  readonly declaration: Node;
  readonly project: Project;
},): ReadonlyTypeOrigin {
  /**
   Reader-facing callable or named type boundary.
   */
  const owner = originOwner({
    declaration,
    project,
  },);
  /**
   Stable name when boundary declares an identifier.
   */
  const name = originName(owner,);
  /**
   Optional named-origin property after sentinel narrowing.
   */
  const named = (typeof name) === 'symbol' ? {} : { name, };
  /**
   Source owning normalized boundary identity.
   */
  const sourceFile = owner.getSourceFile();
  /**
   Full source identity preserving distinct same-line boundaries.
   */
  const identity = `${sourceFile.fileName}:${String(owner.getStart(sourceFile,),)}`;
  return {
    identity,
    kind: originKind(owner,),
    ...named,
    location: originLocation({
      owner,
      project,
    },),
  };
}
