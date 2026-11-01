# @monochromatic-dev/mutation-test-cli.fixture

Deliberately half-tested fixture for `cli-mutation-test` integration runs.

Expected outcomes when the mutation tester runs against this package:

- `src/calc.ts` `clampedSum`:
   fully tested,
   so its arithmetic and
  conditional mutants must be killed.
- `src/calc.ts` `describeSign`:
   zero branch untested on purpose,
   so its
  mutants must survive (confirmed).
- `src/untested.ts`:
   no tests select it,
   so every mutant short-circuits
  to survived without a container run.

The source stays intentionally tiny so integration assertions can pin
exact expectations without churn.
