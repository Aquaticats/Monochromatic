/**
 Public policy category consuming shared readonly evidence.
 
 @example
 ```ts
 const category: ReadonlyRuleCategory = 'preference';
 ```
 */
export type ReadonlyRuleCategory =
  | 'preference'
    | 'mutation'
    | 'opaque-effect'
    | 'effect-contract';
