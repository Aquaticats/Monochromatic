// Fixture: the system temporary directory should be banned (use scratchRoot or a node_modules/.monochromatic directory).
// Expected violation: no-restricted-syntax(no-system-tmpdir)

import { tmpdir } from 'node:os';

function scratchBase(): string {
  return tmpdir();
}

void scratchBase;

export {};
