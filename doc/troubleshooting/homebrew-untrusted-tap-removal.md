# Homebrew 7.0.12 refuses to untap an untrusted tap containing an installed formula

## Symptom

During removal on `2026-11-07T04:53:59-05:00`,
`brew untap open-pencil/tap valkyrie00/bbrew` removed `open-pencil/tap`,
then exited unsuccessfully with:

```text
Error: Refusing to load formula valkyrie00/bbrew/bbrew from untrusted tap valkyrie00/bbrew.
Run `brew trust --formula valkyrie00/bbrew/bbrew` or `brew trust valkyrie00/bbrew` to trust it.
```

The installed `bbrew` receipt identified version `2.2.1`
and source tap `valkyrie00/bbrew`.
The failed command had partially completed:
the first tap was already removed.

## Root cause

The installed Homebrew source checkout was clean at
`bf464f57275cae0be106b43d190c98564ac59b0e`.
Its origin was `https://github.com/Homebrew/brew`.
Paths in this section are relative to that checkout,
located at `/home/linuxbrew/.linuxbrew/Homebrew` on the observed host.

`Library/Homebrew/cmd/untap.rb:108` enumerates installed formula names,
then loads their definitions before removing the tap:

```ruby
# Library/Homebrew/cmd/untap.rb:108
def installed_formulae_for(tap:)
  tap.formula_names.filter_map do |formula_name|
    next unless installed_formulae_names.include?(Utils.name_from_full_name(formula_name))

    formula = begin
      Formulary.factory(formula_name)
    rescue FormulaUnavailableError, FormulaSpecificationError
      next
    end
```

The observed failure was the trust rejection emitted by
`Library/Homebrew/trust.rb:648`:

```ruby
# Library/Homebrew/trust.rb:648
raise UntrustedTapError, "Refusing to load #{type} #{name} from untrusted tap #{tap.name}.\n" \
                         "Run `brew trust --#{type} #{name}` or `brew trust #{tap.name}` to trust it."
```

The installed formula therefore triggered a definition load that failed trust validation.
After uninstalling that formula separately,
the tap removal succeeded.

Source excerpts were checked locally.
Browser verification of the commit-pinned GitHub selection timed out;
a subsequent rendered-state probe reported `about:blank`.
No verified online selection is claimed.

## Verification

The tested executable was `/home/linuxbrew/.linuxbrew/bin/brew`,
reporting `Homebrew 7.0.12`.
The commands were actual requested removals,
not repeated state-mutating tests.
No disposable reproduction or other Homebrew release was tested.

### Failing invocation

With `bbrew` installed from the untrusted tap,
the combined `brew untap open-pencil/tap valkyrie00/bbrew` invocation
removed the first tap and failed on the second with the quoted diagnostic.

### Successful invocations

- `HOMEBREW_NO_AUTOREMOVE=1 brew uninstall --formula bbrew`
  exited successfully and removed `bbrew` version `2.2.1`.
- `brew untap valkyrie00/bbrew` then exited successfully.
- `brew tap` returned no taps after removal.
- Filesystem checks confirmed that both tap directories and the `bbrew` installation directory were absent.

## Verified workaround

When removing the installed application is intended,
uninstall it before untapping its source:

```sh
# Run from the desired working directory with Homebrew on PATH.
HOMEBREW_NO_AUTOREMOVE=1 brew uninstall --formula bbrew
brew untap valkyrie00/bbrew
brew tap
```

This removes the application as well as its source tap.
It does not provide a procedure for retaining the installed application.
Automatic dependency cleanup was disabled for this uninstall to keep the removal scoped.
No trust grant or manual deletion of Homebrew files was needed.

The Homebrew commands ran in individual user scopes with `CPUQuota=100%`,
`MemoryMax=512M`,
`TasksMax=32`,
and `IOWeight=10`.

## What does not work

Directly untapping the untrusted tap while its formula remained installed failed as recorded.
The diagnostic's suggested trust grant was not attempted because uninstalling separately succeeded.
No claim is made about `--force`,
casks,
or installations with dependents.

## Upstream filing decision

This record documents the requested local removal and its workaround.
It is not a completed upstream bug audit or a recommendation to file.
The `.out-of-scope/` filenames were checked;
none identified a Homebrew exemption.

- Upstream fault:
  the observed command failed in the installed Homebrew source;
  a clean upstream reproduction was not run.
- Fixability:
  no architectural fix was evaluated.
- Supported combination:
  support for untapping an untrusted installed formula was not researched.
- Contribution acceptance:
  upstream intake policies were not reviewed.
- Maintainer intent:
  issue duplicates and ongoing fixes were not searched.
- Prototype:
  none was created;
  the verified application-removal sequence completed the task.

No upstream filing artifact is proposed from this bounded incident record.
Nothing was sent to upstream maintainers.
