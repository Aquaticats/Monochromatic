// Fixture: a file-level `await main();` wrapper should be banned (run the body at top level).
// Expected violation: no-restricted-syntax(no-main-wrapper)

async function wrapperBody(): Promise<void> {
  await Promise.resolve();
}

async function main(): Promise<void> {
  await wrapperBody();
}

await main();
