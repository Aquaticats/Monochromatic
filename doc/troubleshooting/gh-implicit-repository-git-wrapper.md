# GitHub CLI 2.96.0 implicit issue lookup invokes a missing PATH-shadowed Git wrapper

## Symptom

From this repository,
`gh issue list` or `gh issue create` without `--repo` can fail before reaching the issue operation:

```text
failed to run git: node:internal/modules/cjs/loader:1573
  throw err;
  ^

Error: Cannot find module '/var/home/user/Monochromatic/node_modules/@monochromatic-dev/git-policy-cli/dist/final/node/index.mjs'
```

The surprising part is that an issue command invokes Git at all.
GitHub CLI uses Git only to infer the repository from the current checkout;
the issue query or mutation still uses GitHub's API.

In this workspace,
`PATH` puts `node_modules/.bin` before `/usr/bin`.
The selected `git` is therefore the repository's policy wrapper,
not the system Git executable.
The failure occurs when that wrapper shim exists while its ignored build artifact does not.

## Root cause

### The issue command asks for a base repository

GitHub CLI v2.96.0's issue-list implementation calls `BaseRepo` before building its issue query.
`pkg/cmd/issue/list/list.go:135-143` at tag `v2.96.0`:

```go
httpClient, err := opts.HttpClient()
if err != nil {
	return err
}

baseRepo, err := opts.BaseRepo()
if err != nil {
	return err
}
```

Issue creation follows the same ordering.
`pkg/cmd/issue/create/create.go:163-173` calls `BaseRepo` before feature discovery,
prompt handling,
and the issue-creation mutation:

```go
httpClient, err := opts.HttpClient()
if err != nil {
	return
}
apiClient := api.NewClientFromHTTP(httpClient)

baseRepo, err := opts.BaseRepo()
if err != nil {
	return
}
```

Therefore,
this exact repository-resolution error means that invocation did not create an issue.
A separate API listing remains a valid way to check whether another invocation succeeded.

### Implicit base-repository resolution reads Git remotes

Without a repository override,
`SmartBaseRepoFunc` asks the factory for remotes and resolves those remotes to GitHub repositories.
`pkg/cmd/factory/default.go:152-174` at tag `v2.96.0`:

```go
func SmartBaseRepoFunc(f *cmdutil.Factory) func() (ghrepo.Interface, error) {
	return func() (ghrepo.Interface, error) {
		httpClient, err := f.HttpClient()
		if err != nil {
			return nil, err
		}

		apiClient := api.NewClientFromHTTP(httpClient)

		remotes, err := f.Remotes()
		if err != nil {
			return nil, err
		}
		resolvedRepos, err := ghContext.ResolveRemotesToRepos(remotes, apiClient, "")
```

The Git client implements that remote lookup with `git remote -v` followed by a Git-config query.
`git/client.go:164-192` at tag `v2.96.0`:

```go
func (c *Client) Remotes(ctx context.Context) (RemoteSet, error) {
	remoteArgs := []string{"remote", "-v"}
	remoteCmd, err := c.Command(ctx, remoteArgs...)
	if err != nil {
		return nil, err
	}
	remoteOut, remoteErr := remoteCmd.Output()
	if remoteErr != nil {
		return nil, remoteErr
	}

	configArgs := []string{"config", "--get-regexp", `^remote\..*\.gh-resolved$`}
```

GitHub CLI resolves the executable named `git` from `PATH`.
`git/client.go:896-911` at tag `v2.96.0`:

```go
func resolveGitPath() (string, error) {
	path, err := safeexec.LookPath("git")
	if err != nil {
		if errors.Is(err, exec.ErrNotFound) {
			programName := "git"
			if runtime.GOOS == "windows" {
				programName = "Git for Windows"
			}
			return "", &NotInstalled{
				message: fmt.Sprintf("unable to find git executable in PATH; please install %s before retrying", programName),
```

The observed `GH_DEBUG=api gh issue list --limit 1` trace confirms this path:

```text
[git remote -v]
[git config --get-regexp ^remote\..*\.gh-resolved$]
* Request to https://api.github.com/graphql
```

### This workspace shadows Git with an artifact-backed package bin

`package/git-policy/cli/package.json:8-18` declares both its module entry and its `git` bin as a built file:

```json
"main": "dist/final/node/index.mjs",
"types": "dist/final/node/index.d.mts",
"exports": {
  ".": {
    "types": "./dist/final/node/index.d.mts",
    "import": "./dist/final/node/index.mjs"
  },
  "./ts": "./src/authoring.ts"
},
"bin": {
  "git": "dist/final/node/index.mjs"
}
```

The generated `node_modules/.bin/git` shim executes that path.
The package's Rolldown configuration writes the artifact to the same directory and cleans that directory at build start.
`package/config/rolldown/src/index.node.ts:151-174`:

```ts
export function nodeConfig(
  {
    input = ['./src/index.ts',],
    outputDir = 'dist/final/node',
    outputOverrides = {},
    external,
  } = {},
): RolldownOptions {
  return defineConfig({
    ...baseOptions,
    input: isInputList(input,) ? [...input,] : { ...input, },
    output: {
      ...baseOutput,
      dir: outputDir,
      cleanDir: true,
      ...outputOverrides,
    },
  },);
}
```

`.gitignore:51-53` excludes `dist/`:

```gitignore
# Build outputs and generated code
.dist/
dist/
target/
```

The package has no install lifecycle script that builds this target.
The observed workspace state had a package-manager-generated bin shim while the target was absent,
so the package itself did not guarantee artifact availability.
This investigation did not rule out root-level install hooks in every install path.
A clean build can also expose a short interval after cleaning and before output emission.

For this incident,
the shim existed while Node reported that its target did not.
A later `mise run //package/git-policy/cli:build` process emitted `index.mjs`,
after which implicit `gh issue list` succeeded again.
The evidence proves the missing-target state and later rebuild;
it does not identify which earlier operation removed or failed to build the target.

### Explicit repository selection skips Git

The repository-override function uses `--repo` first and `GH_REPO` second.
When either supplies a value,
it parses that value directly instead of calling the implicit base-repository resolver.
`pkg/cmdutil/repo_override.go:75-84` at tag `v2.96.0`:

```go
func OverrideBaseRepoFunc(baseRepoFunc func() (ghrepo.Interface, error), override string) func() (ghrepo.Interface, error) {
	if override == "" {
		override = os.Getenv("GH_REPO")
	}
	if override != "" {
		return func() (ghrepo.Interface, error) {
			return ghrepo.FromFullName(override)
		}
	}
	return baseRepoFunc
}
```

The `issue` parent command installs this override before registering both `list` and `create`.
`pkg/cmd/issue/issue.go:43-48` at tag `v2.96.0`:

```go
cmdutil.EnableRepoOverride(cmd, f)

cmdutil.AddGroup(cmd, "General commands",
	cmdList.NewCmdList(f, nil),
	cmdCreate.NewCmdCreate(f, nil),
	cmdStatus.NewCmdStatus(f, nil),
)
```

This is why an explicit `gh api repos/Aquaticat/Monochromatic/issues` call works:
the endpoint already names the repository.
API endpoints containing placeholders such as `{owner}` or `{repo}` can still require local repository context.

### Release view treats its positional argument as a tag, never as a URL

Five view subcommands accept a page URL as their positional argument:
`gh issue view`,
`gh pr view`,
`gh pr diff`,
`gh gist view`,
and `gh repo view`.
`gh release view` does not.
It stores the positional argument as a tag name and resolves the repository separately.
`pkg/cmd/release/view/view.go:54-60` at tag `v2.101.0`:

```go
		RunE: func(cmd *cobra.Command, args []string) error {
			// support `-R, --repo` override
			opts.BaseRepo = f.BaseRepo

			if len(args) > 0 {
				opts.TagName = args[0]
			}
```

`pkg/cmd/release/view/view.go:81` at tag `v2.101.0` then resolves the repository through the same
implicit path this file documents:

```go
	baseRepo, err := opts.BaseRepo()
```

`BaseRepo` comes from the factory's remote-listing resolver.
`pkg/cmd/factory/default.go:41` and `:50` at tag `v2.101.0`:

```go
	f.BaseRepo = BaseRepoFunc(f.Remotes)
```

```go
// BaseRepoFunc requests a list of Remotes, and selects the first one.
```

Passing a release page URL therefore yields a tag string that no release carries,
after a repository lookup that needs a Git work tree.
From a directory that is not a work tree,
the failure names Git instead of the unusable tag.
Observed on `gh version 2.101.0 (2026-09-15)` from a directory outside any work tree:

```text
$ gh release view https://github.com/cli/cli/releases/tag/v2.101.0
failed to run git: fatal: not a git repository (or any parent up to mount point /var)
Stopping at filesystem boundary (GIT_DISCOVERY_ACROSS_FILESYSTEM not set).
```

That message text is built by `git/errors.go:30-35` at tag `v2.101.0`:

```go
func (ge *GitError) Error() string {
	if ge.Stderr == "" {
		return fmt.Sprintf("failed to run git: %v", ge.err)
	}
	return fmt.Sprintf("failed to run git: %s", ge.Stderr)
}
```

Inside a work tree the same command fails later and differently,
because the URL is still consumed as a tag.
The working form names the repository explicitly and passes only the tag:

```bash
gh release view v2.101.0 --repo cli/cli
```

`package/search-fetch/pi-plugin` maps `https://github.com/OWNER/REPO/releases/tag/TAG` onto that form in
`package/search-fetch/pi-plugin/src/github-record-plan.ts`,
and runs every gh child in a temporary directory so no ambient work tree can supply a repository.

## Verification

Verified against:

- installed `gh version 2.96.0`.
- GitHub CLI tag `v2.96.0`,
  commit `b300f2ec7ec9dc9addc39b2ad88c54097ded7ca0`.
- `Node.js v26.5.0`.
- `@monochromatic-dev/git-policy-cli` 0.0.1 in this workspace.

### Runnable trace

From the repository root:

```bash
GH_DEBUG=api gh issue list --limit 1
GH_DEBUG=api gh issue list --repo Aquaticat/Monochromatic --limit 1
GH_REPO=Aquaticat/Monochromatic GH_DEBUG=api gh issue list --limit 1
```

The implicit command prints Git invocations before the GraphQL request.
The explicit flag and environment forms start with the GraphQL request and print no Git invocation.

### Missing-target fixture

This fixture copies the generated shim into a disposable directory without copying its target:

```bash
gh_bin=$(command -v gh)
node_bin=$(dirname "$(command -v node)")
fixture=$(mktemp --directory "${HOME}/temp/agent/gh-git-wrapper.XXXXXXXX")
mkdir --parents "$fixture/bin"
cp node_modules/.bin/git "$fixture/bin/git"

PATH="$fixture/bin:$node_bin:/usr/bin:/bin" \
  "$gh_bin" issue list --limit 1
PATH="$fixture/bin:$node_bin:/usr/bin:/bin" \
  "$gh_bin" issue list --repo Aquaticat/Monochromatic --limit 1

rm --recursive "$fixture"
```

The implicit form fails with `MODULE_NOT_FOUND` for the fixture's absent
`@monochromatic-dev/git-policy-cli/dist/final/node/index.mjs`.
The explicit form lists the issue successfully with the same broken `git` first on `PATH`.

### Commands that work cleanly

- `gh issue list --repo Aquaticat/Monochromatic` with a missing wrapper target;
- `GH_REPO=Aquaticat/Monochromatic gh issue list` with a missing wrapper target;
- `gh api repos/Aquaticat/Monochromatic/issues` with an explicit endpoint;
- implicit `gh issue list` after the wrapper artifact has been built.

### Commands that fail in the missing-target state

- `gh issue list` without `--repo` or `GH_REPO`;
- `gh issue create` without `--repo` or `GH_REPO`;
- other GitHub CLI commands that request the implicit base repository.

## Verified workarounds

### Pass `--repo`

```bash
gh issue list --repo Aquaticat/Monochromatic --limit 3
gh issue create --repo Aquaticat/Monochromatic --title 'Title' --body 'Body'
```

This is the preferred command-local workaround.
It preserves the high-level issue command and bypasses Git-based repository discovery.
Its tradeoff is that the caller must provide the intended repository explicitly.

### Set `GH_REPO`

```bash
GH_REPO=Aquaticat/Monochromatic gh issue list --limit 3
```

This also bypasses Git-based discovery.
Its tradeoff is ambient scope:
an exported value can silently direct later GitHub CLI commands at the wrong repository.
Prefer command-local assignment.

### Rebuild the policy wrapper

```bash
mise run //package/git-policy/cli:build
```

This restores implicit repository discovery when the shim is valid and only the build artifact is absent.
Its tradeoff is coupling unrelated GitHub CLI reads to a workspace build artifact.
It also does not eliminate the clean-before-emit interval during a later wrapper build.

### Use an explicit API endpoint

```bash
gh api repos/Aquaticat/Monochromatic/issues \
  --jq '.[0:3][] | "\(.number) \(.title)"'
```

This bypasses the high-level command's base-repository resolver.
Its tradeoff is that the caller owns REST paths,
pagination,
filtering,
and output shaping instead of receiving `gh issue list` semantics.

### Select system Git explicitly through `PATH`

Calling `gh` with a `PATH` where `/usr/bin/git` precedes `node_modules/.bin/git` also works.
Its tradeoff is that it bypasses this repository's Git policy wrapper for every Git subprocess in that environment.
Use `--repo` instead when the goal is only to avoid repository discovery.

## What does not work

### Piping through `head`

```bash
gh issue list --limit 3 2>&1 | head --lines=5
```

The pipe does not prevent repository discovery.
It truncates Node's diagnostic and,
without pipeline status handling,
can make the displayed status describe `head` rather than `gh`.

### Retrying the same implicit high-level command

Retrying without restoring the artifact or supplying repository context repeats the same pre-API failure.
The issue operation is not partially executed after this exact `BaseRepo` error.

### Treating every `gh api` call as Git-free

An explicit `repos/OWNER/REPO/...` endpoint is Git-free for repository selection.
An endpoint using `{owner}` or `{repo}` placeholders can ask GitHub CLI
to infer those values from the current repository
and therefore re-enter Git discovery.

### Passing a release page URL to `gh release view`

```bash
gh release view https://github.com/cli/cli/releases/tag/v2.101.0
```

The argument is not parsed as a URL.
It becomes the tag name,
and the repository still comes from implicit Git discovery,
so the command fails outside a work tree with a Git diagnostic and fails inside one with a tag that does
not exist.
Supply the tag and `--repo` separately instead.

## Upstream filing decision

No `.out-of-scope/` entry covers GitHub CLI repository discovery or the local Git wrapper.
Searches of open `cli/cli` issues and pull requests for
`issue list invokes git remote`,
`failed to run git repository override`,
and `repo override git remote`
found no matching report.

1. **Is it really upstream's fault?
   ** No.
   GitHub CLI is performing the current-repository inference verified in its source and runtime trace;
   the failure comes from this workspace placing a broken `git` shim first on `PATH`.
2. **Can upstream fix it?
   ** Not as this incident is framed.
   Upstream could redesign repository inference,
   but that is unnecessary for this failure
   because the existing repository override already avoids the broken local executable.
3. **Are they supporting this use case?
   ** Yes.
   `--repo`,
   `GH_REPO`,
   and implicit current-repository selection are all implemented paths,
   and both override paths already avoid Git.
4. **Would the repository welcome our contribution?
   ** An issue would be allowed in principle.
   `.github/CONTRIBUTING.md` says,
   `Open an issue if things aren't working as expected` and requires duplicate checking.
   No AI-assisted-contribution prohibition was found in that file,
   the pull-request template,
   or repository policy search.
5. **Will they likely fix it?
   ** No upstream change is warranted because the observed behavior matches the repository-selection design.
6. **Have we prototyped a minimal upstream fix?
   ** No.
   Constraints one,
   two,
   and five fail,
   so an upstream prototype would target expected behavior rather than the defect.
   The consumer-side override and rebuild paths were verified instead.

Nothing should be filed upstream.
The actionable defect is local artifact availability for the PATH-shadowed Git wrapper,
not GitHub CLI's use of Git to discover the current repository.
