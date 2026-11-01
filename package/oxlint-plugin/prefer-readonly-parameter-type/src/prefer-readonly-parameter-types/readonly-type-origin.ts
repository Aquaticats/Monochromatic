import type {
  Node,
  ParameterDeclaration,
} from 'typescript/ast';
import {
  isArrowFunction,
  isFunctionExpression,
} from 'typescript/ast/is';
import type {
  Project,
  Symbol as TypeScriptSymbol,
  Type,
} from 'typescript/sync';

import {
  readonlyTypeOrigin,
  type ReadonlyTypeOrigin,
  workspaceOrigin,
} from './readonly-type-origin-location.ts';

/**
 Origin evidence for one parameter's readonly preference guidance.
 */
export type ReadonlyTypeOriginEvidence =
  | { readonly kind: 'authored'; }
    | { readonly kind: 'none'; }
    | { readonly kind: 'multiple'; }
    | { readonly kind: 'uncertain'; }
    | {
      readonly kind: 'unique';
      readonly origin: ReadonlyTypeOrigin;
    };

/**
 Returns distinct symbols exposing declarations for one type.
 
 @param type - Semantic type whose declarations are needed.
 
 @returns alias and structural symbols without duplicate identity.
 
 @example
 ```ts
 declarationSymbols(type);
 ```
 */
function declarationSymbols(type: Type,): readonly TypeScriptSymbol[] {
  return [
    type.getAliasSymbol(),
    type.getSymbol(),
  ]
    .filter(function definedSymbol(
      symbol,
    ): symbol is TypeScriptSymbol {
      return symbol !== undefined;
    },)
    .filter(function firstSymbol(
      symbol,
      index,
      symbols,
    ): boolean {
      return symbols.findIndex(function sameId(candidate,): boolean {
        return candidate.id === symbol.id;
      },) === index;
    },);
}

/**
 Eager origin collection plus declaration-resolution completeness.
 */
export type ReadonlyTypeOriginResolution = {
  readonly origins: readonly ReadonlyTypeOrigin[];
  readonly resolutionIncomplete: boolean;
};

/**
 Collects eager editable origins from semantic type graph.
 
 @param type - Inferred callback parameter type.
 
 @param project - Active project resolving declaration handles.
 
 @returns distinct origins plus resolution completeness.
 
 @example
 ```ts
 editableOrigins({ type, project });
 ```
 */
function editableOrigins({
  type,
  project,
}: {
  readonly type: Type;
  readonly project: Project;
},): ReadonlyTypeOriginResolution {
  /**
   Type graph pending union and intersection expansion.
   */
  const pending: Type[] = [type,];
  /**
   Semantic type identities already expanded.
   */
  const visited = new Set<number>();
  /**
   Origins keyed by full source identity after boundary normalization.
   */
  const originsByIdentity = new Map<string, ReadonlyTypeOrigin>();
  /**
   Mutable resolution state preventing partial provenance from becoming unique.
   */
  const resolution = { incomplete: false, };
  while (pending.length > 0) {
    /**
     Next semantic type,
     absent only after unexpected stack mutation.
     */
    const current = pending.pop();
    if ((current === undefined) || visited.has(current.id,))
      continue;
    visited.add(current.id,);
    if (current.isUnionType() || current.isIntersectionType()) {
      pending.push(...current.getTypes(),);
      continue;
    }
    declarationSymbols(current,)
      .flatMap(function symbolDeclarations(symbol,): readonly Node[] {
        return symbol.declarations
          .flatMap(function resolveDeclaration(handle,): readonly Node[] {
            /**
             Declaration eagerly resolved through active project snapshot.
             */
            const declaration = handle.resolve(project,);
            if (declaration === undefined) {
              resolution.incomplete = true;
              return [];
            }
            return [declaration,];
          },);
      },)
      .filter(function editableDeclaration(declaration,): boolean {
        return workspaceOrigin({
          node: declaration,
          project,
        },);
      },)
      .map(function eagerOrigin(declaration,): ReadonlyTypeOrigin {
        return readonlyTypeOrigin({
          declaration,
          project,
        },);
      },)
      .forEach(function recordOrigin(origin,): void {
        originsByIdentity.set(
          origin.identity,
          origin,
        );
      },);
  }
  return {
    origins: [...originsByIdentity.values(),],
    resolutionIncomplete: resolution.incomplete,
  };
}


/**
 Classifies collected origins without overclaiming partial resolution.
 
 @param authored - Whether parameter carries authored type syntax.
 
 @param resolution - Eager editable origins and resolution completeness.
 
 @returns authored,
 absent,
 uncertain,
 multiple,
 or unique evidence.
 
 @example
 ```ts
 readonlyTypeOriginEvidenceFromResolution({ authored: false, resolution });
 ```
 */
export function readonlyTypeOriginEvidenceFromResolution({
  authored,
  resolution,
}: {
  readonly authored: boolean;
  readonly resolution: ReadonlyTypeOriginResolution;
},): ReadonlyTypeOriginEvidence {
  if (authored)
    return { kind: 'authored', };
  /**
   Origin set and completeness separated for branch readability.
   */
  const {
    origins,
    resolutionIncomplete,
  } = resolution;
  if (resolutionIncomplete)
    return { kind: 'uncertain', };
  if (origins.length === 0)
    return { kind: 'none', };
  if (origins.length > 1)
    return { kind: 'multiple', };
  /**
   Sole origin after count narrowing.
   */
  const [origin,] = origins;
  if (origin === undefined)
    return { kind: 'none', };
  return {
    kind: 'unique',
    origin,
  };
}

/**
 Builds origin evidence for readonly preference on one parameter.
 
 @param parameter - Parameter whose type syntax and callback context are inspected.
 
 @param parameterType - Semantic parameter type.
 
 @param project - Active project resolving type declarations eagerly.
 
 @returns authored,
 absent,
 multiple,
 or unique origin evidence.
 
 @example
 ```ts
 readonlyTypeOriginEvidence({ parameter, parameterType, project });
 ```
 */
export function readonlyTypeOriginEvidence({
  parameter,
  parameterType,
  project,
}: {
  readonly parameter: ParameterDeclaration;
  readonly parameterType: Type;
  readonly project: Project;
},): ReadonlyTypeOriginEvidence {
  if (parameter.type !== undefined) {
    return readonlyTypeOriginEvidenceFromResolution({
      authored: true,
      resolution: {
        origins: [],
        resolutionIncomplete: false,
      },
    },);
  }
  /**
   Callable syntactically owning unannotated parameter.
   */
  const callable = parameter.parent;
  if (!(isFunctionExpression(callable,) || isArrowFunction(callable,)))
    return { kind: 'none', };
  /**
   Contextual callback type proving parameter was supplied by surrounding expression.
   */
  const contextualType = project.checker
    .getContextualType(callable,);
  if (contextualType === undefined)
    return { kind: 'none', };
  return readonlyTypeOriginEvidenceFromResolution({
    authored: false,
    resolution: editableOrigins({
      type: parameterType,
      project,
    },),
  },);
}
