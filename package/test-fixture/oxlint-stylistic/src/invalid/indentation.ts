// Fixture: checked lines deviating from their structural base plus two columns.
// Expected: one indentation violation per deviating checked line.

const root = 1;

function hang(
      parameter: number,
): number {
    return parameter;
}

class Shape {
      value = 1;

  method(): number {
  return this.value;
  }
}

function choose(flag: boolean): number {
  if (flag) {
        return 1;
  }
      else {
    return 2;
  }
}

const message = `
    free-form template content
`;
