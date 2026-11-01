// Fixture: type-annotated binding patterns whose lists carry trailing commas.
// Expected: zero stylistic/comma-dangle violations.

function destructured({ salt, }: { readonly salt: number; },): void {
  salt.toFixed();
}

function destructuredPresent({ salt, }: { readonly salt: number; },): void {
  salt.toFixed();
}

function destructuredTuple([only,]: [number,],): void {
  only.toFixed();
}

export {
  destructured,
  destructuredPresent,
  destructuredTuple,
};
