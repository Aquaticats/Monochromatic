// Fixture: `main` shapes that are not a wrapper, because the file or other code calls it too.
// Expected: zero rule violations.

export async function main(): Promise<void> {
  await Promise.resolve();
}

function invoke({
  fn,
}: {
  readonly fn: () => Promise<void>;
},): Promise<void> {
  return fn();
}

void invoke({ fn: main, },);

if (import.meta.main)
  await main();
