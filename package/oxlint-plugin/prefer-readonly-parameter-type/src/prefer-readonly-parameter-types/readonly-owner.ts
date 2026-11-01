import type { Type, } from 'typescript/sync';

/**
 Returns owner symbol name for resolved type.
 
 @param type - TypeScript semantic type.
 
 @returns declared or alias symbol name, or empty string when anonymous.
 
 @example
 ```ts
 readonlyOwnerName(type);
 ```
 */
export function readonlyOwnerName(type: Type,): string {
  /**
   Direct symbol for interface, class, and object type.
   */
  const symbol = type.getSymbol();
  if (symbol !== undefined)
    return symbol.name;
  /**
   Alias symbol fallback for mapped and projected types.
   */
  const aliasSymbol = type.getAliasSymbol();
  return aliasSymbol === undefined ? '' : aliasSymbol.name;
}

/**
 Detects authored readonly projection aliases.
 
 @param type - TypeScript semantic type.
 
 @returns whether alias claims readonly projection.
 
 @example
 ```ts
 typeClaimsReadonlyProjection(type);
 ```
 */
export function typeClaimsReadonlyProjection(type: Type,): boolean {
  /**
   Authored alias name when type was instantiated through projection.
   */
  const aliasName = type.getAliasSymbol()
    ?.name;
  return (aliasName === 'Readonly')
    || (aliasName === 'ReadonlyDeep')
    || (aliasName === 'ReadonlyArray');
}
