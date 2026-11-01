// Fixture: checked lines already anchor at their structural base plus two columns.
// Expected: zero indentation violations.

const top = 1;

function hanging(
  parameter: number,
): number {
  return parameter + top;
}

class Shape {
  value = 1;

  method(): number {
    return this.value;
  }
}

interface Shaped {
  value: number;
}

enum Mode {
  On = 1,
  Off = 2,
}

function nested(): void {
  {
    nested();
  }
}

function choose(mode: Mode,): number {
  switch (mode) {
    case Mode.On: {
      return 1;
    }
    default: {
      return 2;
    }
  }
}

function suffixes(flag: boolean,): number {
  if (flag) {
    return 1;
  }
  else {
    return 2;
  }
  try {
    suffixes(false,);
  }
  catch (caught: unknown) {
    void caught;
  }
  finally {
    suffixes(true,);
  }
  do {
    suffixes(true,);
  }
  while (flag);
}

const product = 1
  * 2
  ;

const message = `
    free-form template content
`;

function commented(): void {
    // misindented comment lines stay exempt
  commented();
}
