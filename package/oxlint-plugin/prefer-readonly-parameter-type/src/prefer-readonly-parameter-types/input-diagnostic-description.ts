/**
 Plain-language descriptions for semantic input diagnostics.
 
 @module
 */

/**
 Builds subject plus verb from authored input binding names.
 
 @param names - Unquoted input binding names.
 
 @returns subject plus verb matching singular or plural names.
 */
function usageSubject(names: readonly string[],): string {
  /**
   Quoted authored names ready for diagnostic prose.
   */
  const quotedNames = names.map(function quoteName(name,): string {
    return `"${name}"`;
  },);
  if (quotedNames.length === 0)
    return 'The function input at this location is';
  if (quotedNames.length === 1)
    return `The function input named ${quotedNames[0]} is`;
  if (quotedNames.length === 2)
    return `The function inputs named ${quotedNames[0]} and ${quotedNames[1]} are`;
  /**
   Final binding name joined after comma-separated leading names.
   */
  const finalName = quotedNames.at(-1,) ?? '"unknown input"';
  /**
   Leading names joined before final human-readable conjunction.
   */
  const leadingNames = quotedNames
    .slice(
    0,
    -1,
    )
    .join(', ',);
  return `The function inputs named ${leadingNames}, and ${finalName} are`;
}

/**
 Collects binding names belonging to one semantic input index.
 
 @param targetIndexes - Binding names mapped to owning input indexes.
 
 @param parameterIndex - Input index being described.
 
 @returns authored binding names in source order.
 */
function parameterNames({
  targetIndexes,
  parameterIndex,
  affectedNames,
}: {
  readonly targetIndexes: ReadonlyMap<string, number>;
  readonly parameterIndex: number;
  readonly affectedNames?: ReadonlySet<string>;
},): readonly string[] {
  /**
   Authored binding names belonging to current input.
   */
  const names: string[] = [];
  targetIndexes.forEach(function collectName(
    index,
    name,
  ): void {
    if (index === parameterIndex)
      names.push(name,);
  },);
  if (affectedNames === undefined)
    return names;
  /**
   Names whose own slot carries the reported effect, in the order they were authored.
   
   Filtered rather than taken from the set directly, so the order stays the declaration's.
   */
  const affected = names.filter(function carriesEffect(name,): boolean {
    return affectedNames.has(name,);
  },);
  /* An empty result means the effect reached no binding this map knows, which should not happen
   * and would produce a subject naming nothing. Describing the whole input is what the report
   * said before per-property attribution, so falling back to it loses precision and never the
   * reader's ability to find the input. */
  return affected.length === 0 ? names : affected;
}

/**
 Describes identifier names that belong to one function input.
 
 @param targetIndexes - Binding names mapped to owning input indexes.
 
 @param parameterIndex - Input index being described.
 
 @param affectedNames - Bindings whose own slot carries the effect, absent to name them all.
 
 @returns subject plus verb for singular or destructured input names.
 
 @example
 ```ts
 inputUsageSubject({ targetIndexes, parameterIndex: 0, affectedNames });
 ```
 */
export function inputUsageSubject({
  targetIndexes,
  parameterIndex,
  affectedNames,
}: {
  readonly targetIndexes: ReadonlyMap<string, number>;
  readonly parameterIndex: number;
  readonly affectedNames?: ReadonlySet<string>;
},): string {
  return usageSubject(parameterNames({
    targetIndexes,
    parameterIndex,
    ...(affectedNames === undefined) ? {} : { affectedNames, },
  },),);
}

/**
 Describes only bindings used as receivers of unknown method calls.
 
 @param boundaries - Authored unknown method call expressions.
 
 @param targetIndexes - Binding names mapped to owning input indexes.
 
 @param parameterIndex - Input index being described.
 
 @param affectedNames - Bindings whose own slot carries the effect, absent to name them all.
 
 @returns subject plus verb naming method receiver bindings only.
 
 @example
 ```ts
 inputMethodUsageSubject({ boundaries: ['ctx.ui.notify'], targetIndexes, parameterIndex: 0 });
 ```
 */
export function inputMethodUsageSubject({
  boundaries,
  targetIndexes,
  parameterIndex,
  affectedNames,
}: {
  readonly boundaries: readonly string[];
  readonly targetIndexes: ReadonlyMap<string, number>;
  readonly parameterIndex: number;
  readonly affectedNames?: ReadonlySet<string>;
},): string {
  /**
   Receivers among the bindings whose own slot carries the effect.
   */
  const narrowed = receiverNames({
    names: parameterNames({
      targetIndexes,
      parameterIndex,
      ...(affectedNames === undefined) ? {} : { affectedNames, },
    },),
    boundaries,
  },);
  /* Two filters run in sequence and they can disagree, so the second one gets its own fallback.
   * `parameterNames` already falls back to every binding when narrowing empties its list, but the
   * receiver filter runs after that and can empty the result again. The disagreement is real
   * rather than theoretical: the message kind is chosen by `everyBoundaryIsInputMethod` from
   * every binding of the input, while a binding counts as affected only when its own slot carries
   * the effect, so opacity widened to the whole parameter while a boundary names one property
   * leaves the receiver outside the narrowed set. An empty list reads "The function input at this
   * location is", which names nothing a reader can act on, so the unnarrowed receivers answer
   * instead. Widening what counts as affected would fix the symptom by discarding the precision
   * this exists to add. */
  return usageSubject(narrowed.length === 0
    ? receiverNames({
      names: parameterNames({
        targetIndexes,
        parameterIndex,
      },),
      boundaries,
    },)
    : narrowed,);
}

/**
 Keeps the names used as the receiver of at least one unknown call.
 
 @param names - Candidate input binding names.
 
 @param boundaries - Authored unknown method call expressions.
 
 @returns names some boundary calls a method on.
 
 @example
 ```ts
 receiverNames({ names: ['ctx'], boundaries: ['ctx.ui.notify'] });
 ```
 */
function receiverNames({
  names,
  boundaries,
}: {
  readonly names: readonly string[];
  readonly boundaries: readonly string[];
},): readonly string[] {
  return names.filter(function usedAsReceiver(name,): boolean {
    return boundaries.some(function beginsWithName(boundary,): boolean {
      return boundary.startsWith(`${name}.`,);
    },);
  },);
}

/**
 Tests whether every unknown call is a method called on current input.
 
 @param boundaries - Authored unknown call expressions.
 
 @param targetIndexes - Binding names mapped to owning input indexes.
 
 @param parameterIndex - Input index being described.
 
 @returns whether every boundary starts from a current input binding.
 
 @example
 ```ts
 everyBoundaryIsInputMethod({ boundaries: ['api.write'], targetIndexes, parameterIndex: 0 });
 ```
 */
export function everyBoundaryIsInputMethod({
  boundaries,
  targetIndexes,
  parameterIndex,
}: {
  readonly boundaries: readonly string[];
  readonly targetIndexes: ReadonlyMap<string, number>;
  readonly parameterIndex: number;
},): boolean {
  if (boundaries.length === 0)
    return false;
  return boundaries.every(function boundaryIsInputMethod(boundary,): boolean {
    for (const [name, index,] of targetIndexes) {
      if ((index === parameterIndex) && boundary.startsWith(`${name}.`,))
        return true;
    }
    return false;
  },);
}
