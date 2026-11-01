/**
 Which call's result each local binding can be holding.
 
 A parallel to `bindingOriginBySymbolId` and deliberately not part of it. That map answers
 which parameter slots a binding can reach, and it is built by `expressionOrigins`, which
 stops at a `CallExpression` because a callee's summary does not exist while its callers
 are walked. So `const local = firstRow(config,)` leaves `local` with no origins, and
 `local.label = 'written'` attributes nothing, which offered `readonly` for a parameter
 the callable writes through.
 
 Filling those origins later cannot fix it. Attribution happens during the syntactic pass,
 and the callee summary only exists during the fixed point, so by the time the origins
 could be known the write has already been walked and not recorded. What crosses that gap
 is the same deferral the rest of this analysis uses: record the call the binding came
 from, resolve the origins where the callee summary sits.
 
 Kept beside the origins rather than folded into them because the two answer different
 questions and converge for different reasons. Origins accumulate parameter slots and
 settle when no binding gains one. These accumulate call-site keys, and an entry here is
 never refined by the fixed point: it names a syntactic fact about where a value came
 from, which is complete as soon as the declarations have been walked.
 
 @module
 */

import type {
  BinaryExpression,
  ParameterDeclaration,
  ForOfStatement,
  Node,
  VariableDeclaration,
} from 'typescript/ast';
import {
  isIdentifier,
  isArrayBindingPattern,
  isBindingElement,
  isObjectBindingPattern,
  isVariableDeclarationList,
} from 'typescript/ast/is';
import type { Project, } from 'typescript/sync';

import {
  deferrableResultSite,
  NOT_A_DEFERRABLE_RESULT,
  transparentValueRoot,
} from './effect-result-substitution.ts';
import { reachableValueSources, } from './effect-result-reach.ts';
import { expressionRoot, } from './effect-summary-model.ts';

/**
 Call sites a binding can be holding the result of, empty when it holds none.
 */
const NO_RESULT_SITE: ReadonlySet<string> = new Set<string>();

/**
 Call sites reachable from one expression, through its access layers and wrappers.
 
 `expressionRoot` strips the access layers and `deferrableResultSite` unwraps the
 identity-keeping ones, which is the same composition the write site uses. An element of a
 returned container is included on purpose: `rowsOf(config,)[0]` is a piece of what the
 call handed back, so a binding holding it holds caller state.
 
 @param project - TypeScript project resolving the root symbol.
 
 @param resultSitesBySymbolId - Call sites already known per binding.
 
 @param node - Expression a binding is initialized or assigned from.
 
 @returns call sites the expression's value can have come from.
 
 @example
 ```ts
 expressionResultSites({ project, resultSitesBySymbolId, node: declaration.initializer });
 ```
 */
function expressionResultSites({
  project,
  resultSitesBySymbolId,
  node,
}: {
  readonly project: Project;
  readonly resultSitesBySymbolId: ReadonlyMap<number, ReadonlySet<string>>;
  readonly node: Node;
},): ReadonlySet<string> {
  /* One loop over both removals rather than one of each in sequence. Doing them once each
   * lost an alias: `const alias = local as Row;` has no access layer to strip, so the root
   * stayed the assertion, `deferrableResultSite` looked inside it and correctly answered
   * that no call was there, and the identifier test then ran against the assertion rather
   * than against `local`. Measured before the loop existed: the asserted and parenthesised
   * aliases recorded no write while the bare one recorded `mutated=[0]`.
   *
   * The two removals also interleave, as in `firstRow(config,).row as Row`, so neither
   * order fixes it on its own. */
  /**
   Value this expression is, past every access layer and identity-keeping wrapper.
   */
  const cursor: { current: Node; } = { current: node, };
  /**
   Whether the last round removed anything, so the walk knows to look again.
   */
  const walk: { removed: boolean; } = { removed: true, };
  while (walk.removed) {
    /**
     Same value with one more round of layers and wrappers taken off.
     */
    const next = transparentValueRoot(expressionRoot(cursor.current,),);
    walk.removed = next !== cursor.current;
    cursor.current = next;
  }
  /**
   Innermost expression this value comes from.
   */
  const root = cursor.current;
  /**
   Call this expression is the result of, when one underlies the root.
   */
  const site = deferrableResultSite({ node: root, },);
  if (site !== NOT_A_DEFERRABLE_RESULT)
    return new Set([site,],);
  /* Past this point the normalisation walk has run out of layers to strip, and a family of
   * shapes lives here that hold a call result rather than layer over one: a conditional, a
   * property of an authored literal, an element of one. Each was falsified.
   *
   * Asked of every source the value can have come from rather than of the root alone, and the
   * widening can only add call sites, so every shape it reaches is a hole closed. The
   * per-source lookup below is the same identifier hop this function has always done. */
  /**
   Sites found through anything the value can have come from.
   */
  const reached = new Set<string>();
  reachableValueSources({
    project,
    node: root,
  },)
    .forEach(function collectSource(source,): void {
      if (source === root)
        return;
      expressionResultSites({
        project,
        resultSitesBySymbolId,
        node: source,
      },)
        .forEach(function collectSite(found,): void {
          reached.add(found,);
        },);
    },);
  if (reached.size > 0)
    return reached;
  /* An alias hop, which is what makes `const alias = local;` carry what `local` carries.
   * Only an identifier root is followed, matching `discoverAliasOrigins`: anything else is
   * a shape this does not model. Naming no call site does NOT withhold, which is worth
   * stating plainly because the first version of this comment claimed it did: both
   * consumers iterate the returned set, so an empty one records nothing and the offer
   * stands. Every shape missing here is a hole, not a conservative choice. */
  if (!isIdentifier(root,))
    return NO_RESULT_SITE;
  /**
   Symbol the root identifier resolves to.
   */
  const symbol = project.checker
    .getSymbolAtLocation(root,);
  return symbol === undefined
    ? NO_RESULT_SITE
    : resultSitesBySymbolId.get(symbol.id,) ?? NO_RESULT_SITE;
}

/**
 Records every call site one binding name can be holding a result of.
 
 @param project - TypeScript project resolving the binding symbol.
 
 @param name - Binding name receiving the value.
 
 @param sites - Call sites the source expression can have come from.
 
 @param resultSitesBySymbolId - Map receiving the binding's call sites.
 
 @mutates resultSitesBySymbolId - Adds every source call site for the named binding.
 
 @returns whether the map gained a call site.
 
 @example
 ```ts
 registerResultSites({ project, name, sites, resultSitesBySymbolId });
 ```
 */
function registerResultSites({
  project,
  name,
  sites,
  resultSitesBySymbolId,
}: {
  readonly project: Project;
  readonly name: Node;
  readonly sites: ReadonlySet<string>;
  readonly resultSitesBySymbolId: Map<number, Set<string>>;
},): boolean {
  if (sites.size === 0)
    return false;
  /* A pattern names several bindings and every one of them can be holding a piece of what the
   * call handed back. This returned false for any non-identifier, so
   * `const { row, } = { row: firstRow(config,), };` registered nothing and a later write
   * through `row` attributed nothing. Falsified.
   *
   * Every leaf takes every site, without asking which key or index a leaf reads. Neither is
   * tracked here, so narrowing would need a claim this cannot support, and taking all of them
   * only adds sites, which only adds effects. */
  if (!isIdentifier(name,))
    return patternLeaves({ name, },)
      .map(function registerLeaf(leaf,): boolean {
        return registerResultSites({
          project,
          name: leaf,
          sites,
          resultSitesBySymbolId,
        },);
      },)
      .includes(true,);
  /**
   Symbol the binding name resolves to.
   */
  const symbol = project.checker
    .getSymbolAtLocation(name,);
  if (symbol === undefined)
    return false;
  /**
   Call sites already recorded for this binding, or a new accumulator.
   */
  const known = resultSitesBySymbolId.get(symbol.id,) ?? new Set<string>();
  /**
   Site count before this registration, deciding whether the map grew.
   */
  const knownBefore = known.size;
  sites.forEach(function addSite(site,): void {
    known.add(site,);
  },);
  resultSitesBySymbolId.set(
    symbol.id,
    known,
  );
  return known.size !== knownBefore;
}

/**
 Resolves which call's result every local binding can be holding.
 
 Converges the same way `discoverAliasOrigins` does and for the same reason: an alias of an
 alias needs a pass per hop, and the sets only ever grow so repetition settles.
 
 @param project - TypeScript project resolving binding symbols.
 
 @param variableDeclarations - Declarations that may bind a call result, parameters included.
 
 A parameter default binds a result exactly as a local declaration does, and only `name` and
 `initializer` are read here, so the two forms need no separate pass. Parameters were absent,
 so `row: Row = firstRow(config,)` bound a result the record never learned about and a later
 write through `row` attributed nothing to `config`. Falsified.
 
 @param aliasAssignments - Simple assignments that may rebind one.
 
 @param forOfStatements - Iteration statements whose iterable may be a call result.
 
 @param resultSitesBySymbolId - Map receiving call sites per binding.
 
 @mutates resultSitesBySymbolId - Adds every call site each binding can hold a result of.
 
 @example
 ```ts
 discoverResultBindings({ project, variableDeclarations, aliasAssignments, forOfStatements, resultSitesBySymbolId });
 ```
 */
export function discoverResultBindings({
  project,
  variableDeclarations,
  aliasAssignments,
  forOfStatements,
  resultSitesBySymbolId,
}: {
  readonly project: Project;
  readonly variableDeclarations: readonly (VariableDeclaration | ParameterDeclaration)[];
  readonly aliasAssignments: readonly BinaryExpression[];
  readonly forOfStatements: readonly ForOfStatement[];
  readonly resultSitesBySymbolId: Map<number, Set<string>>;
},): void {
  /**
   Convergence state, settling because the recorded sets only grow.
   */
  const state = {
    changed: true,
    pass: 0,
  };
  while (state.changed
    && (state.pass <= (variableDeclarations.length + aliasAssignments.length
      + forOfStatements.length))) {
    state.changed = false;
    state.pass++;
    variableDeclarations.forEach(function discover(declaration,): void {
      if (declaration.initializer === undefined)
        return;
      state.changed = registerResultSites({
        project,
        name: declaration.name,
        sites: expressionResultSites({
          project,
          resultSitesBySymbolId,
          node: declaration.initializer,
        },),
        resultSitesBySymbolId,
      },) || state.changed;
    },);
    aliasAssignments.forEach(function discoverAssignment(assignment,): void {
      state.changed = registerResultSites({
        project,
        name: assignment.left,
        sites: expressionResultSites({
          project,
          resultSitesBySymbolId,
          node: assignment.right,
        },),
        resultSitesBySymbolId,
      },) || state.changed;
    },);
    forOfStatements.forEach(function discoverIteration(statement,): void {
      /**
       Call sites the iterated expression can have come from.
       */
      const sites = expressionResultSites({
        project,
        resultSitesBySymbolId,
        node: statement.expression,
      },);
      if (sites.size === 0)
        return;
      /* An element of a returned container is a piece of what the call handed back, so a
       * binding holding one holds whatever the call carried. Both binding forms count,
       * unlike the retention classification, where a declaration is not a store: here the
       * question is what the binding holds while it lives, not whether it outlives the
       * call. */
      if (isVariableDeclarationList(statement.initializer,)) {
        statement.initializer
          .declarations
          .forEach(function registerIterationDeclaration(declaration,): void {
            state.changed = registerResultSites({
              project,
              name: declaration.name,
              sites,
              resultSitesBySymbolId,
            },) || state.changed;
          },);
        return;
      }
      state.changed = registerResultSites({
        project,
        name: statement.initializer,
        sites,
        resultSitesBySymbolId,
      },) || state.changed;
    },);
  }
}

/**
 Reads the call sites a write or store target can be holding a result of.
 
 @param project - TypeScript project resolving the target root symbol.
 
 @param resultSitesBySymbolId - Call sites recorded per binding.
 
 @param node - Write target or stored expression.
 
 @returns call sites whose returned state the target can carry.
 
 @example
 ```ts
 targetResultSites({ project, resultSitesBySymbolId, node: assignment.left });
 ```
 */
export function targetResultSites({
  project,
  resultSitesBySymbolId,
  node,
}: {
  readonly project: Project;
  readonly resultSitesBySymbolId: ReadonlyMap<number, ReadonlySet<string>>;
  readonly node: Node;
},): ReadonlySet<string> {
  return expressionResultSites({
    project,
    resultSitesBySymbolId,
    node,
  },);
}

/**
 Names every binding a pattern introduces.
 
 @param name - Binding name, which may be a pattern.
 
 @returns identifier leaves the pattern binds.
 
 @example
 ```ts
 patternLeaves({ name });
 ```
 */
function patternLeaves({ name, }: { readonly name: Node; },): readonly Node[] {
  if (isIdentifier(name,))
    return [name,];
  if (isObjectBindingPattern(name,) || isArrayBindingPattern(name,))
    return name.elements
      .flatMap(function elementLeaves(element,): readonly Node[] {
        if (!isBindingElement(element,))
          return [];
        /**
         Name this element binds, absent for an elision in an array pattern.
         */
        const bound = element.name;
        return bound === undefined ? [] : patternLeaves({ name: bound, },);
      },);
  return [];
}
