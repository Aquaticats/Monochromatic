# pnpm symlinked-store filing recommendation

## Recommendation at 2026-11-05T16:34:41+00:00

Recommend filing a new pnpm bug report with the public reproduction and an accompanying draft PR for the prototype.
The store-path case is distinct from the fixed project-`node_modules` case in pnpm/pnpm#17448.
Reference that prior work and describe the boundary difference explicitly.

Filing remains the poster's decision.
The remaining risk is consuming maintainer time or damaging the poster's credibility by overstating an agent-authored
prototype.
Keep the disclosure,
affected-version evidence,
and prototype limits in both submissions.
This recommendation concerns opening the report and a draft fix;
it does not declare the candidate ready to merge.
No upstream issue or PR has been posted.

## Evidence supporting filing

- The [public reproduction][repro] needs no private checkout,
  registry package,
  or credentials.
  It verifies the Node and pnpm archive checksums.
- [Run 42063180350][run] and [repeat 42063626526][repeat] reproduce the declared dependency import failure.
  Their install,
  physical-store control,
  and canonical-path workaround steps pass.
  Every failing job reaches the intended import assertion with `ERR_MODULE_NOT_FOUND` for `proper-lockfile`.
- Linux reproduces on `10.34.9`,
  `11.28.11`,
  `12.16.1`,
  and current `12.18.0`.
  macOS and Windows reproduce on `12.18.0`.
  The release-list API was checked again on 2026-11-05.
- The v10 coverage gap is closed.
  Its configuration reader disables the global virtual store whenever `ci` is true,
  so the original green v10 job was invalid evidence for this condition.
  The corrected child environment sets `CI=false`,
  and the assertion verifies that the consumer is physically inside the shared store before testing the import.
- The [prototype run][prototype] proves that the new source regression fails before the patch,
  then all 19 selected Linux symlink tests pass after it.
  `git apply --check` also succeeds on current upstream commit
  `1970b171712c7bff0aa57166d50138bd3beb96d2`.

The full command evidence,
source trace,
workaround,
and issue-form draft are in the [troubleshooting record](../troubleshooting/pnpm-symlinked-store-path.md).

## Duplicate and current-work assessment

The fresh audit used the GitHub search API with explicit totals,
not the earlier empty CLI search results.
Each issue query returned all 68 matches:
`repo:pnpm/pnpm "symlinked" "store" is:issue`
and `repo:pnpm/pnpm "store path" "symlink" is:issue`.
Symbol searches included `relative_target_for` and `existing_symlink_up_to_date`.

The closest reports were read with their maintainer comments:

- [Issue 17448][modules-issue] and PR 17451 concern a project whose `node_modules` is symlinked.
  The [maintainer's explanation][modules-comment] says that case is fixed and does not reproduce on v11.
  Our fixture has an ordinary project `node_modules`,
  an aliased store,
  and a failing v11 control pair.
- Issue 14286 concerns cross-checkout relinking and metadata rewrites.
  Our fixture has no shared writable `node_modules` or second checkout.
- Issue 9618 concerns ESM access to hoisted dependencies.
  Our consumer declares its dependency,
  and its installed dependency link is demonstrably dangling.
- Issues 5717,
  10243,
  and 11396 concern relocation preferences,
  `dlx` sharing a virtual store,
  and malformed Windows absolute paths respectively.
  Their inputs and failing operations differ from this fresh-install case.

The open-PR audit examined all 202 open PRs and their changed-file lists at 2026-11-05T16:04:35+00:00.
Lists exceeding the initial 100 files were paginated;
the largest contained 2,565 files,
below the REST endpoint's file cap.
The searched paths covered the Rust link writer,
reuse helper,
store-path and virtual-layout code,
and the TypeScript symlink dependency and store-path packages.
The only match was draft PR 15263,
whose relevant hunks only reformat imports.
Its branch is `use-stat` in `pnpm/pnpm`.
The matching upstream branch `perf/store-fetch-after-resolve` changes fetching,
not these link calculations.
No matching implementation was found in this checked scope.
Unpublished private work is not observable and is not an invented filing blocker.

Prior source work was also read:
PRs 11684,
14767,
16500,
and 17451.
The [maintainer's ordinary-layout discussion][ordinary-layouts] matters for implementation review:
the candidate must not be presented as a mandate to rewrite already-correct links.
The report identifies the new failing store-path input rather than relitigating the earlier project-anchor case.

## Policy and intake assessment

The policy audit used upstream commit `1970b171712c7bff0aa57166d50138bd3beb96d2`.
It read the contribution guide,
code of conduct,
issue form and configuration,
PR template,
issue-triage workflow,
fix-intent parser,
and triage skill.
Tracked root filenames and `.github` and `.agents` filenames and contents were checked for contributor trust lists,
Vouch,
denouncement,
allow/deny lists,
and AI-contribution restrictions.
No separate contributor gate or prohibition on this report was found in that scope.

The [contribution policy][contributing] explicitly says:

> We use AI coding agents ourselves and welcome contributions made with them.

It requires contributor responsibility,
duplicate checks,
relevant tests,
focused changes,
and an agent/model disclosure footer.
The evidence and draft identify which verification was performed by the agent;
they claim no separate human rerun.

Blank issues are disabled.
The draft follows the bug form's section order and includes the measured runtime and platforms.
No field requires an unknown fact from the poster.
The checked `I am working on a fix` item reflects the verified prototype;
the triage workflow attempts to assign the author when that item is checked.
It then runs Oz triage to label and comment on the issue.
The workflow does not establish a maintainer-preapproval requirement.

## Readiness boundary

The six filing conditions are met:
upstream behavior is reproduced,
the cause is fixable,
the supported installation settings are exercised,
contributions are permitted under the stated conditions,
no matching refusal or active fix was found,
and a nontrivial prototype is verified against its failing baseline.

The Rust candidate remains a prototype.
It does not establish fixes for the TypeScript implementations,
full CLI behavior after rebuilding,
or performance of additional filesystem lookups.
These limits belong in the draft PR and its review work;
they are not reasons to withhold a recommendation to report the verified current-release defect.
Keep the upstream cross-version checklist accurate and do not mark the PR ready for merge prematurely.
Recheck the live tracker and contribution policy on the day of an eventual filing.

Ranking:
new issue with draft PR > additive comment on the closed project-link issue > deferral.
The new issue has a distinct input and failure boundary and can track its own affected versions.
A comment would retain related maintainer context but bury the separate case in a resolved report.
Either adds reproducible information;
deferral saves immediate review time but leaves a demonstrated startup failure untracked.

[repro]: https://github.com/Aquaticat/pnpm-symlinked-store-repro
[run]: https://github.com/Aquaticat/pnpm-symlinked-store-repro/actions/runs/42063180350
[repeat]: https://github.com/Aquaticat/pnpm-symlinked-store-repro/actions/runs/42063626526
[prototype]: https://github.com/Aquaticat/pnpm-symlinked-store-repro/actions/runs/42055609601
[modules-issue]: https://github.com/pnpm/pnpm/issues/17448
[modules-comment]: https://github.com/pnpm/pnpm/issues/17448#issuecomment-6468041363
[ordinary-layouts]: https://github.com/pnpm/pnpm/pull/17451#discussion_r4400442122
[contributing]: https://raw.githubusercontent.com/pnpm/pnpm/1970b171712c7bff0aa57166d50138bd3beb96d2/CONTRIBUTING.md#:~:text=We%20use%20AI%20coding%20agents%20ourselves
