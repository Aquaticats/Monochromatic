# GitHub.com (version undisclosed, 2026-08-18) omits the license when Monochromatic uses only LICENSES/

## Symptom

The `Aquaticat/Monochromatic` entry on the GitHub repositories tab shows its TypeScript language and star count,
but no license.
The REST repository response reports `"license": null`,
 and the repository-license endpoint returns:

```text
HTTP 404
{"message":"Not Found"}
```

The repository is licensed despite that presentation:

- `package.json:52` declares `LGPL-3.0-or-later AND CC-BY-SA-4.0`.
- `README.md:251-254` says code uses LGPL-3.0-or-later and shareable documentation or content also uses CC-BY-SA-4.0.
- `LICENSES/` contains the LGPL,
   incorporated GPL,
   and Creative Commons texts.

## Root cause

Commit `0057b61d2294764ee30265dc5c521c9adc4b123b` moved the former root `LICENSE` without changing its bytes:

```text
LICENSE => LICENSES/LGPL-3.0-or-later.txt
```

GitHub's current [licensing documentation][github-licensing] recommends `LICENSE.txt`,
 `LICENSE.md`,
 or
`LICENSE.rst` in the repository root.
Its detection section says GitHub compares the repository's `LICENSE` file and recommends keeping that file simple
when a repository has multiple licenses or other complexity.
The production API probe confirms that GitHub.com did not inspect this repository's `LICENSES/` directory on
2026-08-18.
The exact GitHub.com Licensee deployment version is not public,
 so this evidence does not identify a deployed version.

Package metadata and natural-language README prose are not substitutes for a detected license file.
Licensee's [detection documentation][licensee-detection] says package-manager and README detection are disabled by
default because package metadata does not distribute the license text and natural language is not reliably parseable.

The current open-source Licensee release has moved beyond the behavior observed on GitHub.com.
In `licensee/licensee` v10.1.0 (`fb924e7b69b81488092ddbc183afa5ebe45abf1a`),
`lib/licensee/projects/github_project.rb:70-78` loads both root files and `LICENSES/` files:

```ruby
def load_files_with_license_dir
  base_files = dir_files
  license_dir_files = begin
    dir_files('LICENSES')
  rescue Octokit::NotFound
    []
  end

  base_files + license_dir_files
end
```

`lib/licensee/project_files/license_file.rb:114-119` then accepts SPDX-shaped names under `LICENSES/`:

```ruby
def self.name_score(dir, filename = nil)
  dir, filename = normalize_name_score_args(dir, filename)
  return 0.0 unless filename
  return FILENAME_REGEXES.find { |regex, _| filename.match? regex }[1] unless dir == 'LICENSES'

  filename.match?(LICENSES_FILENAME_REGEX) ? 1.0 : 0.0
end
```

Those source paths are linked at the pinned [v10.1.0 GitHub-project source][licensee-github-project] and
[v10.1.0 license-file source][licensee-license-file].
Licensee pull request [#926][licensee-pr-926] added this support and closed issue [#737][licensee-issue-737].
Version v10.1.0 was released on 2026-08-07,
 but GitHub.com's API still exhibited root-only behavior on 2026-08-18.

Scanning all current `LICENSES/` entries would not produce one precise project license.
`lib/licensee/projects/project.rb:25-33` returns `other` when more than one distinct non-copyright license matches:

```ruby
def license
  return @license if defined? @license

  @license = if licenses_without_copyright.one? || lgpl?
               licenses_without_copyright.first
             elsif licenses_without_copyright.count > 1
               Licensee::License.find('other')
             end
end
```

The pinned implementation is in [v10.1.0 project source][licensee-project].
Licensee issue [#57][licensee-issue-57] discusses this repository's exact pattern,
 code under one license and content
under Creative Commons.
A Licensee maintainer recommends placing the dominant material's license in `LICENSE` and explaining the other terms
in the README.
For Monochromatic,
 the existing README already identifies LGPL-3.0-or-later as the code license.

## Verification

### Version and repository state

- GitHub.com deployment version:
   undisclosed,
   observed 2026-08-18.
- Monochromatic pre-fix commit:
   `f840957ae7d64adfe5e17efdf35ebf29663d37f8`.
- Licensee source reviewed:
   v10.1.0,
   commit `fb924e7b69b81488092ddbc183afa5ebe45abf1a`.
- Disposable GitHub probe repository:
   private,
   created and removed during verification.

The pre-fix repository reproduces the symptom:

```bash
gh api repos/Aquaticat/Monochromatic --jq .license
gh api repos/Aquaticat/Monochromatic/license
```

```text
null
gh: Not Found (HTTP 404)
```

A disposable private GitHub repository used the exact three files from Monochromatic.
Each endpoint call pinned `ref` to the tested commit,
 avoiding branch-cache ambiguity.

### Patterns that GitHub.com detects

- Root `LICENSE` containing `LICENSES/LGPL-3.0-or-later.txt`,
   with the three `LICENSES/` files retained:
  `GNU Lesser General Public License v3.0`,
   SPDX `LGPL-3.0`.
- Root `LICENSE` containing that text,
   with no `LICENSES/` directory:
  the same result.

The first case is the positive control for the intended repository fix.
It proves the extra license texts do not prevent GitHub.com's currently deployed detector from selecting the root
code license.

### Patterns that GitHub.com does not detect

- Only `LICENSES/LGPL-3.0-or-later.txt`.
- Monochromatic's current three-file `LICENSES/` directory.
- The existing `package.json` license expression plus README license prose without a root license file.

The exact three-file layout returned HTTP 404 from the license endpoint.

A Licensee v10.1.0 container probe was rejected as evidence because both the real layout and a root-`LICENSE`
positive control returned empty match arrays.
The failed positive control proves that harness did not test the GitHub behavior in question.

## Verified workarounds

### Restore a top-level LGPL license file

Copy `LICENSES/LGPL-3.0-or-later.txt` byte-for-byte to `LICENSE` and keep the README's scope explanation.
The disposable GitHub probe detected `LGPL-3.0` with this exact layout.
This does not select new terms or remove the Creative Commons terms;
 it restores the code-license file that commit
`0057b61d2` relocated.

Tradeoffs:

- `LICENSE` duplicates the canonical text,
   so repository automation must keep the two files byte-identical.
- GitHub's single label describes the code license,
   not every content-specific term in the repository.
- A future GitHub rollout that scans every `LICENSES/` entry may show `Other` because the repository intentionally has
  distinct license scopes.

### Keep the scope explanation in the README

Link the code-license statement to root `LICENSE` and link the content statement to
`LICENSES/CC-BY-SA-4.0.txt`.
This makes GitHub's one-label limitation explicit at the reader boundary.

Tradeoff:
 GitHub's repository list still has room for only one detected license label,
 so readers must open the README
for content-specific terms.

## What does not work

- `package.json` metadata alone:
   the current repository has it and GitHub returns `license: null`.
- README prose alone:
   the current repository has it and the license endpoint returns HTTP 404.
- Moving every text into `LICENSES/`:
   that move caused the regression on GitHub.com's deployed detector.
- Waiting for open-source Licensee's `LICENSES/` support:
   v10.1.0 contains it,
   but GitHub.com's deployment timing is
  undisclosed,
   and all three recognized texts would represent multiple scopes rather than one precise label.
- Treating the failed local v10.1.0 container run as a null result:
   its root-license positive control also failed,
   so
  the harness was not capable of distinguishing the layouts.

## Upstream filing decision

`.out-of-scope/` contains no GitHub license-detection or Licensee exemption.

1.  **Is it really upstream's fault?**
     No.
    GitHub documents a simple root `LICENSE` file as the reliable detection surface,
     while this repository removed that
    surface.
    Open-source Licensee has already added `LICENSES/` support.
2.  **Can upstream fix it?**
     Yes in principle.
    GitHub can deploy a Licensee version or integration that scans `LICENSES/`,
     but a multi-scope repository still needs
    a presentation policy.
3.  **Are they supporting this use case?**
     Partly.
    Licensee v10.1.0 supports `LICENSES/`;
     GitHub's documentation supports a root license plus a README explanation for
    complexity.
4.  **Would the repository welcome our contribution?**
     Licensee accepted pull request #926 for this feature.
    `README.md`,
     `.github/ISSUE_TEMPLATE/bug_report.md`,
     and repository policy searches showed no AI-assistance ban;
    the merged pull request itself received Copilot review.
5.  **Will they likely fix it?**
     The Licensee-side feature is already fixed and released.
    No public evidence establishes GitHub.com's deployment schedule.
6.  **Have we prototyped a minimal upstream fix?**
     Not applicable because constraint 1 fails and upstream already merged
    the relevant implementation.
    The consumer-side root-file workaround was instead verified against GitHub.com's API.

Duplicate searches found Licensee issue #737 and merged pull request #926 for `LICENSES/` scanning,
 plus issue #57 for
multiple license scopes.
The investigation has nothing additive to post to those threads,
 so the upstream filing artifact is explicitly empty:
no issue and no comment should be filed.

[github-licensing]: https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/licensing-a-repository
[licensee-detection]: https://licensee.github.io/licensee/what-we-look-at/
[licensee-github-project]: https://github.com/licensee/licensee/blob/v10.1.0/lib/licensee/projects/github_project.rb#L70-L78
[licensee-license-file]: https://github.com/licensee/licensee/blob/v10.1.0/lib/licensee/project_files/license_file.rb#L114-L119
[licensee-project]: https://github.com/licensee/licensee/blob/v10.1.0/lib/licensee/projects/project.rb#L25-L33
[licensee-pr-926]: https://github.com/licensee/licensee/pull/926
[licensee-issue-737]: https://github.com/licensee/licensee/issues/737
[licensee-issue-57]: https://github.com/licensee/licensee/issues/57
