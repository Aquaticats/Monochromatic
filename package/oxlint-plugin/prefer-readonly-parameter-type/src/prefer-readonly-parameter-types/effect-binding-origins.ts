/**
 Parameter and local-alias binding origin analysis.
 
 Seeding a parameter and registering a local alias are separate operations here, and the
 separation is load-bearing. Seeding allocates: a parameter's destructuring pattern is where
 property slots come from, so each binding it introduces registers against its own slot.
 Aliasing never allocates: a local destructured from something else takes whatever slots its
 source already carries, because no caller writes a property of a local. Letting one
 operation do both would invent property slots for patterns no caller can address.
 
 @module
 */

import type {
  BinaryExpression,
  BindingElement,
  ForOfStatement,
  Node,
  ParameterDeclaration,
  VariableDeclaration,
} from 'typescript/ast';
import {
  isArrayBindingPattern,
  isBindingElement,
  isIdentifier,
  isObjectBindingPattern,
  isVariableDeclarationList,
} from 'typescript/ast/is';
import type { Project, } from 'typescript/sync';

import { expressionElementOrigins, } from './effect-element-origin.ts';
import { expressionValueOrigins, } from './effect-expression-provenance.ts';
import {
  parameterBindingSlots,
  type ParameterSlotTable,
} from './effect-parameter-slots.ts';
import type { EffectSlot, } from './effect-slot-identity.ts';
import type { SlotOrigins, } from './effect-summary-model.ts';

/**
 Registers every identifier bound by one name or destructuring pattern against one slot.
 
 Origins accumulate rather than replace. A local reassigned across branches holds
 state from every slot assigned into it, and an earlier revision overwrote, so
 one branch erased the other and the erased parameter was offered `readonly` while
 the body mutated it through the alias. Applying that suggestion failed to compile;
 `doc/decision/prefer-readonly-binding-origin-accumulation.md` records the measurement.
 
 Accumulation also fixes convergence. Under overwrite, an alias with two origins
 flipped between them on every pass and reported progress each time, so
 `discoverAliasOrigins` only stopped at its pass bound. Monotone growth makes
 progress mean "the set grew", which settles on its own.
 
 A pattern handed here spreads one slot over every name it binds, which is what an alias
 needs: `const { a, b } = source` gives both locals the source's slots, because a write
 through either reaches the source. Only a parameter's own pattern allocates finer slots,
 and `seedParameterSlots` is what does that.
 
 @param project - TypeScript project resolving binding symbols.
 
 @param name - Binding name or nested pattern.
 
 @param slot - Slot the bound state belongs to.
 
 @param bindingOriginBySymbolId - Origin map receiving bindings.
 
 @returns whether map changed.
 
 @mutates bindingOriginBySymbolId - Adds slot origin for binding symbols.
 
 @example
 ```ts
 registerBindingOrigin({ project, name, slot, bindingOriginBySymbolId });
 ```
 */
export function registerBindingOrigin({
  project,
  name,
  slot,
  bindingOriginBySymbolId,
}: {
  readonly project: Project;
  readonly name: Node;
  readonly slot: EffectSlot;
  readonly bindingOriginBySymbolId: Map<number, Set<EffectSlot>>;
},): boolean {
  if (isIdentifier(name,)) {
    /**
     Symbol declared by binding identifier.
     */
    const symbol = project.checker
      .getSymbolAtLocation(name,);
    if (symbol === undefined)
      return false;
    /**
     Origins already known for binding, or new accumulator.
     */
    const origins = bindingOriginBySymbolId.get(symbol.id,) ?? new Set<EffectSlot>();
    /**
     Size before insertion detects fixed-point progress.
     */
    const priorSize = origins.size;
    origins.add(slot,);
    bindingOriginBySymbolId.set(
      symbol.id,
      origins,
    );
    return origins.size !== priorSize;
  }
  if ((!isObjectBindingPattern(name,)) && (!isArrayBindingPattern(name,)))
    return false;
  /**
   Whether any nested binding origin changed.
   */
  let changed = false;
  for (const element of name.elements) {
    if ((!isBindingElement(element,)) || (element.name === undefined))
      continue;
    changed = registerBindingOrigin({
      project,
      name: element.name,
      slot,
      bindingOriginBySymbolId,
    },) || changed;
  }
  return changed;
}

/**
 Seeds one parameter's bindings, each against the slot its own property owns.
 
 This is the only place property slots enter a summary. A binding under a rest element, an
 array pattern or a computed key takes the whole-parameter slot, because no caller property
 key names it.
 
 @param project - TypeScript project resolving binding symbols.
 
 @param parameter - Parameter whose bindings are seeded.
 
 @param parameterIndex - Declared position of that parameter.
 
 @param table - Slot table allocated for the owning declaration.
 
 @param bindingOriginBySymbolId - Origin map receiving bindings.
 
 @mutates bindingOriginBySymbolId - Adds one slot origin per bound name.
 
 @example
 ```ts
 seedParameterSlots({ project, parameter, parameterIndex, table, bindingOriginBySymbolId });
 ```
 */
export function seedParameterSlots({
  project,
  parameter,
  parameterIndex,
  table,
  bindingOriginBySymbolId,
}: {
  readonly project: Project;
  readonly parameter: ParameterDeclaration;
  readonly parameterIndex: number;
  readonly table: ParameterSlotTable;
  readonly bindingOriginBySymbolId: Map<number, Set<EffectSlot>>;
},): void {
  parameterBindingSlots({
    parameter,
    parameterIndex,
    table,
  },)
    .forEach(function seedOne(bound,): void {
      registerBindingOrigin({
        project,
        name: bound.name,
        slot: bound.slot,
        bindingOriginBySymbolId,
      },);
    },);
}

/**
 Resolves every parameter origin represented by expression root.
 
 {@inheritDoc expressionValueOrigins}
 
 @param project - TypeScript project resolving root symbol.
 
 @param bindingOriginBySymbolId - Known parameter and alias origins.
 
 @param node - Expression whose root may represent parameter state.
 
 @returns source parameter origins, empty when root is not parameter-derived.
 
 @example
 ```ts
 expressionOrigins({ project, bindingOriginBySymbolId, node });
 ```
 */
export function expressionOrigins({
  project,
  bindingOriginBySymbolId,
  node,
}: {
  readonly project: Project;
  readonly bindingOriginBySymbolId: ReadonlyMap<number, SlotOrigins>;
  readonly node: Node;
},): SlotOrigins {
  return expressionValueOrigins({
    project,
    bindingOriginBySymbolId,
    node,
  },);
}

/**
 Registers one binding as holding every origin its source can hold.
 
 @param project - TypeScript project resolving binding symbols.
 
 @param name - Binding name or nested pattern receiving origins.
 
 @param parameterOrigins - Origins resolved for aliased source expression.
 
 @param bindingOriginBySymbolId - Origin map receiving bindings.
 
 @returns whether map changed.
 
 @mutates bindingOriginBySymbolId - Adds every source origin for binding symbols.
 
 @example
 ```ts
 registerBindingOrigins({ project, name, parameterOrigins, bindingOriginBySymbolId });
 ```
 */
function registerBindingOrigins({
  project,
  name,
  parameterOrigins,
  bindingOriginBySymbolId,
}: {
  readonly project: Project;
  readonly name: Node;
  readonly parameterOrigins: SlotOrigins;
  readonly bindingOriginBySymbolId: Map<number, Set<EffectSlot>>;
},): boolean {
  /* Spread because `ReadonlySet` has no `reduce`, and for no stronger reason now.
   *
   * It used to also be a defensive snapshot: `expressionOrigins` returned the live set
   * stored for the source binding, so a self-assignment (`cursor = cursor`) could have
   * one call iterating the object another was inserting into. `expressionValueOrigins`
   * removed that hazard structurally by always returning a freshly built set or the
   * shared empty constant, so the copy is no longer what makes this safe. */
  return [...parameterOrigins,]
    .reduce(
    function registerOne(
      changed,
      slot,
    ): boolean {
      return registerBindingOrigin({
        project,
        name,
        slot,
        bindingOriginBySymbolId,
      },) || changed;
    },
    false,
    );
}

/**
 Tests whether an expression root is reachable from any callable parameter.
 
 @param project - TypeScript project resolving root symbol.
 
 @param bindingOriginBySymbolId - Known parameter and alias origins.
 
 @param node - Expression whose root may represent parameter state.
 
 @returns whether root carries at least one parameter origin.
 
 @example
 ```ts
 expressionHasParameterOrigin({ project, bindingOriginBySymbolId, node });
 ```
 */
export function expressionHasParameterOrigin({
  project,
  bindingOriginBySymbolId,
  node,
}: {
  readonly project: Project;
  readonly bindingOriginBySymbolId: ReadonlyMap<number, SlotOrigins>;
  readonly node: Node;
},): boolean {
  /**
   Origins resolved for expression root.
   */
  const origins = expressionOrigins({
    project,
    bindingOriginBySymbolId,
    node,
  },);
  return origins.size > 0;
}

/**
 Discovers local aliases and destructured bindings to fixed point.
 
 @param project - TypeScript project resolving symbols.
 
 @param variableDeclarations - Body declarations eligible for aliasing.
 
 @param aliasAssignments - Simple assignments eligible for aliasing.
 
 @param forOfStatements - Iterations binding elements from parameter-owned iterables.
 
 @param bindingOriginBySymbolId - Origin map receiving aliases.
 
 @mutates bindingOriginBySymbolId - Adds aliases rooted in parameter state.
 
 @example
 ```ts
 discoverAliasOrigins({ project, variableDeclarations, aliasAssignments, forOfStatements, bindingOriginBySymbolId });
 ```
 */
export function discoverAliasOrigins({
  project,
  variableDeclarations,
  aliasAssignments,
  forOfStatements,
  bindingOriginBySymbolId,
}: {
  readonly project: Project;
  readonly variableDeclarations: readonly VariableDeclaration[];
  readonly aliasAssignments: readonly BinaryExpression[];
  readonly forOfStatements: readonly ForOfStatement[];
  readonly bindingOriginBySymbolId: Map<number, Set<EffectSlot>>;
},): void {
  /**
   Convergence state, settling on its own now that origins only grow.
   
   The pass bound is a backstop rather than the actual terminator it was while
   origins could overwrite each other: an alias with two origins used to flip
   between them every pass and report progress each time.
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
      /* A destructuring default carries origins the declaration's own initializer does not. In
       * `const { row = config.row, } = {} as { row?: Row; }` the initializer names nothing, and
       * the parameter is named inside a binding element, so scanning the declaration alone left
       * `row` with no origin and a later store of it attributed nothing. Falsified.
       *
       * Registered against the element's own name, which is the binding the default fills. */
      bindingElementDefaults({ name: declaration.name, },)
        .forEach(function registerDefault(element,): void {
          if (element.initializer === undefined)
            return;
          /**
           Name this element binds, absent for an elision in an array pattern.
           */
          const bound = element.name;
          if (bound === undefined)
            return;
          state.changed = registerBindingOrigins({
            project,
            name: bound,
            parameterOrigins: expressionOrigins({
              project,
              bindingOriginBySymbolId,
              node: element.initializer,
            },),
            bindingOriginBySymbolId,
          },) || state.changed;
        },);
      if (declaration.initializer === undefined)
        return;
      /* An array pattern binds elements, so it takes the element step even though it writes
       * no element access. `const [first] = copy` over a fresh container reached nothing
       * before this, while the same write through `copy[0]` was attributed. An object
       * pattern keeps asking the value question: it binds properties, and a container's
       * properties are its own rather than its elements. */
      /**
       Parameter origins of initializer root, through elements for an array pattern.
       */
      const parameterOrigins = isArrayBindingPattern(declaration.name,)
        ? expressionElementOrigins({
          project,
          bindingOriginBySymbolId,
          node: declaration.initializer,
        },)
        : expressionOrigins({
          project,
          bindingOriginBySymbolId,
          node: declaration.initializer,
        },);
      state.changed = registerBindingOrigins({
        project,
        name: declaration.name,
        parameterOrigins,
        bindingOriginBySymbolId,
      },) || state.changed;
    },);
    forOfStatements.forEach(function discoverIteration(statement,): void {
      /* Iteration is an element step with no element access written, so it asks the element
       * question. Over a parameter the value origins already answer it, and over a fresh
       * container only the container relation does. */
      /**
       Parameter origins reached through the iterated expression's elements.
       */
      const parameterOrigins = expressionElementOrigins({
        project,
        bindingOriginBySymbolId,
        node: statement.expression,
      },);
      if (parameterOrigins.size === 0)
        return;
      if (isVariableDeclarationList(statement.initializer,)) {
        /**
         Declarations receiving iterated elements.
         */
        const { declarations, } = statement.initializer;
        declarations.forEach(function registerIterationDeclaration(declaration,): void {
          state.changed = registerBindingOrigins({
            project,
            name: declaration.name,
            parameterOrigins,
            bindingOriginBySymbolId,
          },) || state.changed;
        },);
        return;
      }
      if (isIdentifier(statement.initializer,)) {
        state.changed = registerBindingOrigins({
          project,
          name: statement.initializer,
          parameterOrigins,
          bindingOriginBySymbolId,
        },) || state.changed;
      }
    },);
    aliasAssignments.forEach(function discoverAssignment(assignment,): void {
      /**
       Parameter origins of assignment right-hand side.
       */
      const parameterOrigins = expressionOrigins({
        project,
        bindingOriginBySymbolId,
        node: assignment.right,
      },);
      if (!isIdentifier(assignment.left,))
        return;
      state.changed = registerBindingOrigins({
        project,
        name: assignment.left,
        parameterOrigins,
        bindingOriginBySymbolId,
      },) || state.changed;
    },);
  }
}

/**
 Returns the mutable origin set of one binding name, creating it when absent.
 
 Used where a binding's origins have to grow after its own index is registered, such as a
 parameter whose default initializer names an earlier parameter.
 
 @param project - TypeScript project resolving binding symbol.
 
 @param name - Binding name whose origin set is wanted.
 
 @param bindingOriginBySymbolId - Local binding origins by symbol identity.
 
 @returns mutable origin set for that binding, empty and unattached when unresolved.
 
 @mutates bindingOriginBySymbolId - Attaches an origin set for a newly seen binding.
 
 @example
 ```ts
 bindingOriginsFor({ project, name, bindingOriginBySymbolId });
 ```
 */
export function bindingOriginsFor({
  project,
  name,
  bindingOriginBySymbolId,
}: {
  readonly project: Project;
  readonly name: Node;
  readonly bindingOriginBySymbolId: Map<number, Set<EffectSlot>>;
},): Set<EffectSlot> {
  if (!isIdentifier(name,))
    /* A binding pattern spreads over several symbols, so there is no single set to grow.
     * Defaults inside patterns are not represented here. */
    return new Set<EffectSlot>();
  /**
   Symbol declared by this binding name.
   */
  const symbol = project.checker
    .getSymbolAtLocation(name,);
  if (symbol === undefined)
    return new Set<EffectSlot>();
  /**
   Existing origin set for the symbol, created when this is its first mention.
   */
  const existing = bindingOriginBySymbolId.get(symbol.id,);
  if (existing !== undefined)
    return existing;
  /**
   Fresh set attached for the symbol.
   */
  const created = new Set<EffectSlot>();
  bindingOriginBySymbolId.set(
    symbol.id,
    created,
  );
  return created;
}

/**
 Names every binding element a pattern introduces, at any depth.
 
 @param name - Binding name, which may be a pattern.
 
 @returns binding elements the pattern contains.
 
 @example
 ```ts
 bindingElementDefaults({ name });
 ```
 */
function bindingElementDefaults({ name, }: { readonly name: Node; },): readonly BindingElement[] {
  if ((!isObjectBindingPattern(name,)) && (!isArrayBindingPattern(name,)))
    return [];
  return name.elements
    .flatMap(function elementAndNested(element,): readonly BindingElement[] {
      if (!isBindingElement(element,))
        return [];
      /**
       Name this element binds, absent for an elision in an array pattern.
       */
      const bound = element.name;
      return bound === undefined
        ? [element,]
        : [
          element,
          ...bindingElementDefaults({ name: bound, },),
        ];
    },);
}
