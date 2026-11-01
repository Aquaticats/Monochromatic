/**
 Slot allocation for one callable's parameters.
 
 A slot is what an effect is attributed to. Slots below the parameter count are the whole
 parameters; the rest are one per statically canonical top-level property an object pattern
 reads. Allocation depends on the declaration and on nothing else, which is what lets a
 caller index a callee's slots without re-analyzing the callee's body: `addOwnedCallEdge`
 holds the callee declaration and computes the same table the callee's own summary did.
 
 The unit is the property key rather than the binding. `{ a: { b } }` gives property `a` one
 slot that `b` registers against, because a write through `b` is a write through `a`. Making
 the binding the unit would leave `a` with no slot there, which is the direction that loses
 writes. `{ a: x, a: y }` reads one property twice and likewise gets one slot.
 
 @module
 */

import type {
  BindingName,
  BindingPattern,
  Node,
  ParameterDeclaration,
} from 'typescript/ast';
import {
  isArrayBindingPattern,
  isBindingElement,
  isIdentifier,
  isObjectBindingPattern,
} from 'typescript/ast/is';

import {
  asEffectSlot,
  asParameterIndex,
  canonicalPropertyKey,
  type EffectSlot,
  NOT_A_STATIC_KEY,
  type ParameterIndex,
} from './effect-slot-identity.ts';
import type { EffectCallableDeclaration, } from './effect-summary-model.ts';

/**
 Slots of one callable, and how they map back to its parameters.
 */
export type ParameterSlotTable = {
  readonly parameterCount: number;
  readonly slotCount: number;
  readonly parameterOfSlot: readonly ParameterIndex[];
  readonly slotsByParameter: readonly (readonly EffectSlot[])[];
  readonly propertySlotsByParameter: readonly ReadonlyMap<string, EffectSlot>[];
};

/**
 One binding name a parameter introduces, and the slot its state attributes to.
 */
export type ParameterBindingSlot = {
  readonly name: Node;
  readonly slot: EffectSlot;
};

/**
 Tests whether a node destructures rather than naming one binding.
 
 Asks the kind predicates rather than probing for an `elements` field. This AST exposes
 every field name on the prototype, so `'elements' in node` answers true for a numeric
 literal, and `{ 1: one }` was classified as a nested pattern and given no property slot at
 all. `numericKeySlots` in the shape fixture is what measured it.
 
 @param node - Binding name to classify.
 
 @returns whether node is a destructuring pattern.
 
 @example
 ```ts
 isBindingPatternNode(parameter.name);
 ```
 */
function isBindingPatternNode(node: Node,): node is BindingPattern {
  return isObjectBindingPattern(node,)
    || isArrayBindingPattern(node,);
}

/**
 Reads the property key one binding element names, when it names a static one.
 
 A rest element names a complement set rather than a property, and a shorthand element whose
 name destructures further names no property either.
 
 @param element - Binding element from an object pattern.
 
 @returns canonical key, or sentinel when the element names no static property.
 
 @example
 ```ts
 elementPropertyKey({ element });
 ```
 */
function elementPropertyKey(
  { element, }: { readonly element: Node; },
): string | typeof NOT_A_STATIC_KEY {
  if ((!isBindingElement(element,)) || (element.dotDotDotToken !== undefined))
    return NOT_A_STATIC_KEY;
  /**
   Property name this element reads, which is its own name when shorthand.
   */
  const name = element.propertyName ?? element.name;
  if ((name === undefined) || isBindingPatternNode(name,))
    return NOT_A_STATIC_KEY;
  return canonicalPropertyKey({ name, },);
}

/**
 Canonical top-level property keys one parameter's pattern reads, without duplicates.
 
 An array pattern contributes none: positional element keys are not modelled, and a caller's
 array literal is not matched against them.
 
 @param parameter - Parameter whose pattern is inspected.
 
 @returns distinct property keys, empty for an identifier or array pattern.
 
 @example
 ```ts
 parameterPropertyKeys({ parameter });
 ```
 */
function parameterPropertyKeys(
  { parameter, }: { readonly parameter: ParameterDeclaration; },
): readonly string[] {
  /**
   Pattern binding this parameter, when it destructures one.
   */
  const pattern = parameter.name;
  if (!isObjectBindingPattern(pattern,))
    return [];
  return [
    ...new Set(pattern.elements
      .flatMap(function keyOf(element,): readonly string[] {
        /**
         Key this element reads, absent when it names no static property.
         */
        const key = elementPropertyKey({ element, },);
        return key === NOT_A_STATIC_KEY ? [] : [key,];
      },),),
  ];
}

/**
 Allocates every slot one callable declaration owns.
 
 @param declaration - Callable whose parameters are being slotted.
 
 @returns slots, ownership and property keys for that declaration.
 
 @example
 ```ts
 parameterSlotTable({ declaration });
 ```
 */
export function parameterSlotTable(
  { declaration, }: { readonly declaration: EffectCallableDeclaration; },
): ParameterSlotTable {
  /**
   Declared parameters receiving whole-parameter slots first.
   */
  const { parameters, } = declaration;
  /**
   Owning parameter of every slot, whole parameters first and properties appended.
   */
  const parameterOfSlot: ParameterIndex[] = parameters.map(
    function wholeOwner(
      _unused,
      parameterIndex,
    ): ParameterIndex {
      return asParameterIndex(parameterIndex,);
    },
  );
  /**
   Property key to slot, per parameter, filled as property slots are appended.
   */
  const propertySlotsByParameter = parameters.map(
    function propertySlotsFor(
      parameter,
      parameterIndex,
    ): ReadonlyMap<string, EffectSlot> {
      return new Map(parameterPropertyKeys({ parameter, },)
        .map(function slotForKey(key,): readonly [
          string,
          EffectSlot,
        ] {
          /**
           Next free slot, appended as this key is assigned one.
           */
          const slot = asEffectSlot(parameterOfSlot.length,);
          parameterOfSlot.push(asParameterIndex(parameterIndex,),);
          return [
            key,
            slot,
          ];
        },),);
    },
  );
  return {
    parameterCount: parameters.length,
    slotCount: parameterOfSlot.length,
    parameterOfSlot,
    slotsByParameter: slotsByParameterFrom({ parameterOfSlot, },),
    propertySlotsByParameter,
  };
}

/**
 Groups slots by the parameter that owns them.
 
 Separate from the allocator because a summary restored from the persistent cache has its
 ownership but not its declaration, and the projection back to parameters needs only this.
 
 @param parameterOfSlot - Owning parameter of every slot.
 
 @returns slots owned by each parameter, whole-parameter slot first.
 
 @example
 ```ts
 slotsByParameterFrom({ parameterOfSlot });
 ```
 */
export function slotsByParameterFrom(
  { parameterOfSlot, }: { readonly parameterOfSlot: readonly ParameterIndex[]; },
): readonly (readonly EffectSlot[])[] {
  /**
   Slots accumulated per owning parameter.
   */
  const grouped: EffectSlot[][] = [];
  parameterOfSlot.forEach(function group(
    parameterIndex,
    slot,
  ): void {
    /**
     Slots already owned by this parameter, created on first sight.
     */
    const owned = grouped[parameterIndex] ?? [];
    owned.push(asEffectSlot(slot,),);
    grouped[parameterIndex] = owned;
  },);
  return grouped;
}

/**
 Collects every identifier one binding name introduces, however deeply nested.
 
 @param name - Binding name or pattern to flatten.
 
 @returns identifiers bound, in declaration order.
 
 @example
 ```ts
 bindingNamesUnder({ name: element.name });
 ```
 */
function bindingNamesUnder(
  { name, }: { readonly name: BindingName; },
): readonly Node[] {
  /**
   Identifiers found so far.
   */
  const found: Node[] = [];
  /**
   Patterns still to flatten, each a descendant of one already seen.
   */
  const pending: Node[] = [name,];
  while (pending.length > 0) {
    /**
     Next binding name, absent only when the stack changed unexpectedly.
     */
    const current = pending.pop();
    if (current === undefined)
      continue;
    if (isIdentifier(current,)) {
      found.push(current,);
      continue;
    }
    if (!isBindingPatternNode(current,))
      continue;
    /**
     Names bound directly by this pattern, in declaration order.
     */
    const nested: Node[] = [];
    current.forEachChild(function queueElement(child,): undefined {
      if (isBindingElement(child,) && (child.name !== undefined))
        nested.push(child.name,);
      return undefined;
    },);
    /* Reversed on the way in, because a stack hands them back in the opposite order and a
     * caller comparing bound names against a written pattern should see them as written. */
    nested.toReversed()
      .forEach(function queueNested(bound,): void {
        pending.push(bound,);
      },);
  }
  return found;
}

/**
 Pairs every binding name one parameter introduces with the slot it attributes to.
 
 A binding under a rest element, under an array pattern, or under a computed key attributes
 to the whole parameter, since no caller property key names it. A binding nested inside a
 property attributes to that property's slot.
 
 @param parameter - Parameter whose bindings are wanted.
 
 @param parameterIndex - Declared position of that parameter.
 
 @param table - Slot table the parameter belongs to.
 
 @returns every binding name paired with its slot.
 
 @example
 ```ts
 parameterBindingSlots({ parameter, parameterIndex, table });
 ```
 */
export function parameterBindingSlots({
  parameter,
  parameterIndex,
  table,
}: {
  readonly parameter: ParameterDeclaration;
  readonly parameterIndex: number;
  readonly table: ParameterSlotTable;
},): readonly ParameterBindingSlot[] {
  /**
   Slot standing for the whole parameter, which every unattributable binding takes.
   */
  const wholeSlot = asEffectSlot(parameterIndex,);
  /**
   Property keys this parameter's pattern reads, mapped to their slots.
   */
  const propertySlots = table.propertySlotsByParameter[parameterIndex]
    ?? new Map<string, EffectSlot>();
  if (!isObjectBindingPattern(parameter.name,))
    /* An identifier yields itself and an array pattern flattens, so every pair this returns
     * names one identifier whichever shape the parameter has. Positional element keys are
     * not modelled, so all of them take the whole-parameter slot. */
    return bindingNamesUnder({ name: parameter.name, },)
      .map(function wholeBinding(bound,): ParameterBindingSlot {
        return {
          name: bound,
          slot: wholeSlot,
        };
      },);
  return parameter.name
    .elements
    .flatMap(function bindingsOfElement(element,): readonly ParameterBindingSlot[] {
      if ((!isBindingElement(element,)) || (element.name === undefined))
        return [];
      /**
       Key this element reads, absent for a rest element or a computed name.
       */
      const key = elementPropertyKey({ element, },);
      /**
       Slot every binding under this element attributes to.
       */
      const slot = key === NOT_A_STATIC_KEY
        ? wholeSlot
        : propertySlots.get(key,) ?? wholeSlot;
      return bindingNamesUnder({ name: element.name, },)
        .map(function pair(bound,): ParameterBindingSlot {
          return {
            name: bound,
            slot,
          };
        },);
    },);
}
