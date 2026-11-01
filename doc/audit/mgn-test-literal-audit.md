# MGN test literal audit

Audit of every test suite against rule `MGN`,
 from GitHub issue #3371:
 tests never assert magic numbers or magic lists,
 and derive expectations from the named constant,
 the input,
 or the relation under test.

## Method

A pattern search finds candidates,
not violations,
so the audit ran in three passes over each package.

1. Search the test and sidecar files for the four violation shapes named in the issue:
   expected values that repeat a production constant, default, limit, timeout or count;
   expected lists that repeat a production list;
   sums or products of literals that mirror production arithmetic;
   counts of a collection the code defines.
2. Pair each candidate literal mechanically with the production constants in the same package
   that carry the same value, so a hit pointed at the constant it might copy.
   The pairing is a reading aid, not a verdict.
3. Read every hit against the four classes and the issue's not-a-violation list,
   fix the violations,
   and record why each remaining candidate is not one.

Each fix was verified by changing the guarded production value or list,
running the narrow test that covers the rewritten assertion,
and confirming it fails before restoring the value,
so no fix silently weakened a test.
Each touched package's own task was then run through `mise run`.

## Not violations

These candidate classes stay as written,
because the literal is the input or the specification of the case,
or the number is the behavior under test:

- A literal that is the input of the case, such as the seconds passed to a duration conversion
  whose result the test then asserts.
- A specification literal with no named constant, such as an RFC test vector,
  a documented wire format, or an expected rendered string.
- A boundary case built from the named constant plus or minus one.
- A count that is the behavior under test,
  where the collection is built in the test and its length is what the case checks.
- A fixture name or a hostile-input string chosen to exercise escaping,
  which no production constant owns.

## Verification

Every rewritten assertion is covered by a mutation run recorded in the commit that rewrote it:
the production constant or list entry was changed,
the narrow test failed,
and the value was restored.
Package test tasks were run after the edits,
with the exact commands in each commit's package scope.

## Follow-up

Review keeps rejecting a literal copy of a production value at review time (rule `MGN`),
and a new test derives its expectation from the start.
