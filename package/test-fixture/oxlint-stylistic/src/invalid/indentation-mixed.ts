// Fixture: deviating indentation shared with argument, chain, and body fixes.
// Expected: --fix reaches one fixed point shared by every overlapping fixer.

function mixed(): void {
      calls(1, 2);
      const chained = value.then(1, ).finally(2, );
        if (true) {
      mixed();
        }
}
