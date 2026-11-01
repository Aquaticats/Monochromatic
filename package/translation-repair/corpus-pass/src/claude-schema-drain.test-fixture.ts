//region Taking items off a work stack
// What the walks of the schema check take their next item with: they push the items still to follow onto a stack while
// they go, and a walk that reads the stack with `for...of` takes the last item pushed first, so the order of the
// reports stays the one a stack gives.

/**
 The items of a stack, taken off its end one by one until it is empty. Items pushed while the loop runs are taken too.

 @param stack - items still to follow, which the caller pushes onto while it loops

 @returns Each item as it is taken off the end of the stack

 @example
 ```ts
 const pending = [1,];
 for (const item of drained({ stack: pending, },)) {
   if (item < 3)
     pending.push(item + 1,);
 }
 ```
 */
export function* drained<Item,>({ stack, }: { readonly stack: Item[]; },): Generator<Item> {
  while (stack.length > 0) {
    /**
     The item at the end of the stack.
     */
    const item = stack.pop();
    if (item !== undefined)
      yield item;
  }
}

//endregion Taking items off a work stack
