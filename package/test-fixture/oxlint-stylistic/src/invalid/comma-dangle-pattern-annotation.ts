// Fixture: type-annotated binding patterns whose lists lack trailing commas.
// Expected violations: stylistic(comma-dangle).

function destructured({ salt }: { readonly salt: number; },): void {
  salt.toFixed();
}

function destructuredPresent({ salt, }: { readonly salt: number; },): void {
  salt.toFixed();
}

function destructuredTuple([only]: [number],): void {
  only.toFixed();
}

export {
  destructured,
  destructuredPresent,
  destructuredTuple,
};
