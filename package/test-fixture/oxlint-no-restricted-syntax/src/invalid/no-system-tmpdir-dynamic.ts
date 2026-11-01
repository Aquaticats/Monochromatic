// Fixture: the system temporary directory reached through a dynamic import should be banned too.
// Expected violation: no-restricted-syntax(no-system-tmpdir)

async function scratchBase(): Promise<string> {
  const { tmpdir } = await import('node:os');
  return `${tmpdir()}/work`;
}

void scratchBase;

export {};
