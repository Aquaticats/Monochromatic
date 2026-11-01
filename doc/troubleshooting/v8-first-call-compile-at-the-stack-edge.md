# V8: a function first called with the stack almost spent throws the stack-exhaustion `RangeError`

Code that catches a stack exhaustion and turns it into its own error
can still let the engine's `RangeError` escape,
when the handler's code has never run in that process.
Seen in Node v26.11.1 in `package/translation-repair` (2026-10-14).

## Symptom

`src/parse-mdx.unit.test.ts`,
case "REFUSES A BODY THAT EXHAUSTS THE STACK FROM A SHALLOW TEXT",
failed three times in one whole-suite run and passed on the rerun:
the test expected `MdxParseError` and got a raw `RangeError: Maximum call stack size exceeded`
whose top frame was the refusal handler in `parseMdxBody`.
The body was 200 `>` characters and ` cat`,
parsed from a caller that had already spent most of the stack.

Measured with the real fixture run many times per process:

- cold process,
  before the fix:
  299 of 300 parses ended in a raw `RangeError`,
  and 1 overflowed inside the fixture's own recursion;
- with `--no-opt --no-maglev --no-sparkplug`:
  100 of 100 raw `RangeError`s;
- after one refusal taken at a safe depth in the same process:
  99 of 100 refusals.

A smaller `--stack-size` did not reproduce it:
from 66 to 84 KiB the refusal came back as `MdxParseError`,
and Node does not start below about 58 KiB.

## Root cause

V8 compiles a function lazily,
the first time it is called.
A compile that finds too little stack fails,
and when the parser left no other error,
V8 raises its own stack overflow
(`src/codegen/compiler.cc` in V8's source,
`FailWithPreparedException`,
the branch that calls `isolate->StackOverflow()`).
In a fresh process nothing had yet run the refusal handler,
its `new MdxParseError(...)` or the error class's constructor,
so the first refusal built at the stack edge needed a compile there
and threw the engine's `RangeError` instead of the refusal.
Load changed the outcome only through how much stack the caller had spent
and whether an earlier test in the same process had already compiled the handler.

## Workaround

Run the refusal path once when the module loads,
so its code is compiled before any parse reaches the stack edge:
`readyRefusals()` in `package/translation-repair/document-structure/src/parse-mdx.ts`
builds both refusals (`MdxParseError` and `MarkdownParseError`)
with a `RangeError` cause carrying the engine's message,
which takes the same branch a real exhaustion takes.
After the fix:
299 refusals and 1 fixture self-overflow out of 300,
for each grammar.
`src/parse-mdx-cold-refusal.unit.test.ts` runs the stack-edge search in a fresh child process per grammar
and fails without the warm-up.

## Limits that remain

- V8 can flush the bytecode of a function that has not run for a while,
  so a long-idle process can be cold again.
- The last frames of the stack cannot run even a trivial function,
  so a raw `RangeError` stays possible from the nesting check or the handler itself.
- The fixture overflows in its own recursion about once in 100 runs;
  the test was not loosened for it.
