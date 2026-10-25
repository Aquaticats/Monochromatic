import type { TreeNode, } from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import {
  acrossFilesOf,
  type Pending,
  pendingIn,
} from './claude-schema-blind-pending.test-fixture.ts';
import {
  type BlindContext,
  enclosingFunctionOf,
} from './claude-schema-blind-positions.test-fixture.ts';
import { readsOfName, } from './claude-schema-blind-reads.test-fixture.ts';
import { stepOf, } from './claude-schema-blind-step.test-fixture.ts';
import {
  isNoNode,
  NO_NODE,
} from './claude-schema-tree.test-fixture.ts';

//region Where the result of a builder's call goes
// What the check behind the schema search (`claude-schema-blind-spots.test-fixture.ts`) asks of a builder's call: where
// its value goes. The value is followed through the objects, arrays, spreads, conditionals and logical operands it is
// put into, through the members read off it and the methods called on it, and through the constants and the parameters
// with a default that a function or the file binds it to. The walk ends in one of these places, and every other place
// is reported:
//
// - a `return`, an arrow function's body or a `yield`, in a function the search knows as a wrapper (in any other
//   function the value leaves it, and the function is reported);
// - the property of a sender's object argument, which `claude-schema-blind-senders.test-fixture.ts` lists by module,
//   function and parameter;
// - a constant, a pattern, a default or a name assigned, whose every reference is followed in turn; a name bound at the
//   top level of a file is also followed into the files that read it, through the imports, the exports from, the
//   namespace imports and the `import()` of the file (`claude-schema-blind-constants.test-fixture.ts`);
// - a parameter of the callback of an array method called on the value (`claude-schema-blind-callbacks.test-fixture.ts`),
//   whose every reference is followed in turn; a callback the search cannot read is reported as code it cannot read.
//
// A place that takes the value and gives nothing the schema could travel in ends the walk without a report: the test
// of a conditional or a loop, a computed key, an operand of an operator, a substitution in a plain template, a
// statement that drops it, an element of a sequence that is not the last, and the target of an assignment. Any other
// place is reported with its position: an argument of a call or of `new`, a substitution of a tagged template (the
// arguments of the tag function), a store into a member or a field, a pattern an assignment takes the value apart
// into, a class that extends it, a thrown value, a loop that iterates it.

/**
 A name read in a function or in a whole file.
 */
type ReadName = {
  /**
   The name.
   */
  readonly name: string;

  /**
   The function that binds it, or the program of the file.
   */
  readonly scope: TreeNode;
};

/**
 Where the value of a builder's call goes.
 */
type Destinations = {
  /**
   The first function the value is returned from that the search does not know as a wrapper, `NO_NODE` when the value
   leaves by no such function.
   */
  readonly leaving: TreeNode;

  /**
   The path of the file the function sits in when it is not the file of the call, empty when it is.
   */
  readonly leavingIn: string;

  /**
   The places the value goes that the check reports, as the kind of node and the field, each once, with the file after
   the place when it is not the file of the call.
   */
  readonly strays: readonly string[];

  /**
   The callbacks the value is handed to that the search cannot read, each once, with the file after the place when it is
   not the file of the call.
   */
  readonly unreadable: readonly string[];
};

/**
 Where a builder's call sends its value: the function it is returned from when the search does not know that function
 as a builder, and every place that takes it that no sender, wrapper or constant answers for. A constant that a file
 exports is followed into the files that read it.

 @param context - what the file's names are read against

 @param call - call of a builder, or the node that holds its value

 @returns First such function found and the file it sits in when that is not the file of the call, `NO_NODE` when the
 value stays in the function, the places reported, and the callbacks the value is handed to that the search cannot read

 @example
 ```ts
 const { leaving, strays, unreadable, } = destinationsOf({ context, call, });
 ```
 */
export function destinationsOf(
  {
    context,
    call,
  }: {
    readonly context: BlindContext;
    readonly call: TreeNode;
  },
): Destinations {
  /**
   Names already followed, so a name that leads back to itself ends.
   */
  const followed = new Set<string>();
  /**
   The places reported so far.
   */
  const strays = new Set<string>();
  /**
   The callbacks the search cannot read, found so far.
   */
  const unreadable = new Set<string>();
  /**
   Nodes whose value is still to follow.
   */
  const pending: Pending[] = [...pendingIn({
    context,
    nodes: [call,],
  },),];
  /**
   The functions left by that the search does not know, in the order they were met, with the context of their file.
   */
  const leavers: Pending[] = [];
  for (let item = pending.pop(); item !== undefined; item = pending.pop()) {
    /**
     What the names of the node's file are read against.
     */
    const here = item.context;
    /**
     The program of the node's file.
     */
    const { program, } = here.facts;
    /**
     Where the value goes next.
     */
    const {
      onward,
      names,
      isFileWide,
      leaves,
      stray,
      callback,
    } = stepOf({
      context: here,
      node: item.node,
    },);
    if ((!isNoNode({ node: leaves, },)) && (!here.recognized
      .has(leaves,))) {
      leavers.push(...pendingIn({
        context: here,
        nodes: [leaves,],
      },),);
    }
    if (stray !== '')
      strays.add((here.file === context.file) ? stray : `${stray} in ${here.file}`,);
    if (callback.unreadable !== '') {
      unreadable.add((here.file === context.file)
        ? callback.unreadable
        : `${callback.unreadable} in ${here.file}`,);
    }
    pending.push(...pendingIn({
      context: here,
      nodes: onward,
    },),);
    /**
     The function that binds the names, `NO_NODE` at the top level.
     */
    const binding = enclosingFunctionOf({
      node: item.node,
      parents: here.parents,
    },);
    /**
     Where the names are read: the function that binds them, the whole file at the top level and for a name that an
     assignment stores the value in.
     */
    const scope = (isFileWide || isNoNode({ node: binding, },)) ? program : binding;
    /**
     The names to follow, each with the function or file it is read in: the names the value is bound to, and the names the
     parameters of a callback bind.
     */
    const reads = [
      ...names.map(function readInScope(name,): ReadName {
        return {
          name,
          scope,
        };
      },),
      ...callback.names
        .map(function readInCallback(name,): ReadName {
          return {
            name,
            scope: callback.scope,
          };
        },),
    ];
    for (const {
      name,
      scope: readIn,
    } of reads) {
      /**
       The name as a place to stop when the walk comes back to it.
       */
      const place = `${here.file}:${String(readIn.start,)}:${name}`;
      if (!followed.has(place,)) {
        followed.add(place,);
        pending.push(...pendingIn({
          context: here,
          nodes: readsOfName({
            context: here,
            name,
            scope: readIn,
          },),
        },),);
        if (readIn === program) {
          /**
           What the other files read of the name.
           */
          const across = acrossFilesOf({
            context: here,
            name,
          },);
          pending.push(...across.pending,);
          for (const reported of across.strays) {
            strays.add(reported,);
          }
        }
      }
    }
  }
  /**
   The first function left by that the search does not know.
   */
  const [first,] = leavers;
  /**
   What the names of that function's file are read against.
   */
  const leftIn = first?.context ?? context;
  return {
    leaving: first?.node ?? NO_NODE,
    leavingIn: (leftIn.file === context.file) ? '' : leftIn.file,
    strays: [...strays,],
    unreadable: [...unreadable,],
  };
}

//endregion Where the result of a builder's call goes
