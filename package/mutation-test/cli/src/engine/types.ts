/**
 Shared engine type declarations: mutation operators, candidate
 replacements, and fully-identified mutants.
 
 @example
 ```ts
 const status: MutantStatus = 'killed';
 ```
 */

/**
 Mutation operator family names, mirroring the Stryker mutator families
 whose replacement tables serve as the parity spec.
 */
export type OperatorName =
  | 'arithmetic'
    | 'equality'
    | 'logical'
    | 'conditional'
    | 'boolean'
    | 'string'
    | 'unary'
    | 'update'
    | 'array'
    | 'object'
    | 'optional-chaining'
    | 'block'
    | 'method'
    | 'arrow'
    | 'regex';

/**
 Final classification of one executed (or filtered) mutant.
 
 `compileError` comes from the per-mutant tsgo check; `runtimeError`
 marks infrastructure failures, never test-detected kills.
 */
export type MutantStatus =
  | 'killed'
    | 'survived'
    | 'timeout'
    | 'compileError'
    | 'runtimeError';

/**
 Candidate replacement produced by one operator for one AST node,
 before identity assignment.
 
 Offsets are UTF-16 string indices as returned by yuku-parser's JS
 bindings (probe-verified on astral characters), so
 `source.slice(start, end)` is exact.
 */
export type Replacement = {
  readonly start: number;
  readonly end: number;
  readonly text: string;
  readonly operator: OperatorName;
  readonly description: string;
};

/**
 Fully-identified mutant ready for manifest serialisation.
 */
export type Mutant = {
  readonly id: string;
  readonly file: string;
  readonly start: number;
  readonly end: number;
  readonly line: number;
  readonly column: number;
  readonly operator: OperatorName;
  readonly original: string;
  readonly replacement: string;
  readonly description: string;
};

/**
 Minimal structural view of an ESTree node used by operators; operators
 narrow via `type` plus property checks so their code stays independent
 of the parser's full node union.
 */
export type EstreeNode = {
  readonly type: string;
  readonly start: number;
  readonly end: number;
  readonly [key: string]: unknown;
};
