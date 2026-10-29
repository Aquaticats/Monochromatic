/**
 Function arity wrappers.
 
 Use `unary` and `binary` when host APIs pass more positional arguments than
 callback logic should receive. Common cases include `Array.prototype.map`,
 `Array.prototype.flatMap`, and other iterator helpers that pass value,
 index, and source collection.
 
 @example
 ```ts
 import { binary, unary, } from '\@monochromatic-dev/function-arity-core';
 
 ['10', '10'].map(unary(Number.parseInt,));
 ['a', 'b'].map(binary(function render(value: string, index: number): string {
   return `${index}:${value}`;
 },),);
 ```
 
 @packageDocumentation
 */

export { binary, } from './binary.ts';

export { unary, } from './unary.ts';
