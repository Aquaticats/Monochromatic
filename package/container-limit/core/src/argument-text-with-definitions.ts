/**
 The text a start's arguments reach through the names it uses, in the file or in a module it imports.

 A start spreads names the file declares, and a definition's own text names further declarations, so the words that
 show where the limits come from can sit a chain of definitions away from the argument list.
 */

import {
  isNameCharacter,
} from './positions-in-source-text.ts';

/**
 Names one argument text resolves through.

 An argument list names a handful of names and a chain of them is short, so the cap only stops a file whose names
 refer to each other in a long loop from expanding far past its own size.
 */
const MAXIMUM_RESOLVED_BINDINGS = 64;

/**
 Newline character, which joins the text and its definitions.
 */
const NEWLINE = '\n';

/**
 Every name a text uses.

 @param text - an argument text or a definition

 @returns the names, each once, in the order the text holds them

 @example
 ```ts
 const names = namesIn("'run', ...bounds, image");
 ```
 */

function namesIn(text: string,): readonly string[] {
  /**
   The names the text uses, each once.
   */
  const names = new Set<string>();
  for (let index = 0; index < text.length;) {
    if (!isNameCharacter(text[index],)) {
      index += 1;
      continue;
    }
    /**
     Index past the name that starts at the scan's index.
     */
    let end = index;
    while ((end < text.length) && isNameCharacter(text[end],))
      end += 1;
    names.add(text.slice(
      index,
      end,
    ),);
    index = end;
  }
  return [...names,];
}

/**
 The argument text with the text of every name it uses that the file defines, or that a module it imports defines.

 Words inside a definition count as words of the argument text, no further than `MAXIMUM_RESOLVED_BINDINGS` names.

 @param argumentText - the text that reaches the program
 
 @param definitions - the text each name of the file stands for
 
 @param importedDefinitions - the text each name stands for in a module the file imports from

 @returns the argument text and the text of every definition reachable through the names it uses

 @example
 ```ts
 const reaching = argumentTextWithBoundInitializers({ argumentText, definitions, importedDefinitions });
 ```
 */
export function argumentTextWithBoundInitializers({
  argumentText,
  definitions,
  importedDefinitions,
}: {
  readonly argumentText: string;
  readonly definitions: ReadonlyMap<string, string>;
  readonly importedDefinitions?: ReadonlyMap<string, string>;
},): string {
  /**
   The argument text and the text of every definition it reaches, joined once at the end.
   */
  const pieces: string[] = [argumentText,];
  /**
   Names whose definition is already in the pieces or is being looked for.
   */
  const visited = new Set<string>(namesIn(argumentText,),);
  /**
   Names to resolve, in the order the text names them so the argument text's own names come first.
   */
  const pending = [...visited,];
  for (let next = 0; (next < pending.length) && (visited.size <= MAXIMUM_RESOLVED_BINDINGS); next += 1) {
    /**
     Name to resolve now.
     */
    const name = pending[next];
    if (name === undefined)
      break;
    /**
     Text the file gives the name, either in it or in the module it imports from.
     */
    const definition = definitions.get(name,) ?? importedDefinitions?.get(name,);
    if (definition === undefined)
      continue;
    pieces.push(definition,);
    for (const reached of namesIn(definition,)) {
      if (visited.has(reached,))
        continue;
      visited.add(reached,);
      pending.push(reached,);
    }
  }
  return pieces.join(NEWLINE,);
}
