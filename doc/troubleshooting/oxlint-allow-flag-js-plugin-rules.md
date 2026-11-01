# Oxlint 1.81.0: `--allow`, `-A`, and `-W` do not reach JS plugin rules; only config `off` disables them

Passing a JS plugin rule to any of oxlint's CLI severity flags changes nothing:
`--allow <rule>`,
`--allow=<rule>`,
`-A <rule>`,
`-W <rule>`,
and `-D <rule>` all leave the rule running at its configured severity.
Setting the same rule to `'off'` in the config file disables it.
The cause is in oxlint itself:
CLI filters are applied to the builtin rule map only,
while JS plugin rules live in a separate map that only the config-file path writes.

Verified 2026-09-06 against oxlint 1.81.0 (`node_modules/.bin/oxlint --version` prints `Version: 1.81.0`),
`@oxlint/plugins` 1.81.0,
Node v26.8.1,
source tag `apps_v1.81.0` at commit `0b4e2e67f4193e7ebfcc64982275eb583ae82c83`.

## Symptom

The rule under test is `prefer-readonly-parameter-type/prefer-readonly-parameter-types`,
a JS plugin rule from `package/oxlint-plugin/prefer-readonly-parameter-type`,
enabled as `'error'` in `package/config/oxlint/src/rule/restriction.ts:167`.
The `lint:oxlint` task template (`mise.toml:590-597`) builds that shared config through `ensureOxlintConfig()`,
then runs `package/monorepo-task/core-exec/src/oxlint-wrapper.ts`,
which forwards its arguments to `oxlint`;
the root `oxlint.config.ts` spreads the shared config.

With a fixture that violates the rule copied into a package's `src/`,
every CLI spelling listed under "Verification" reports the same 11
`x prefer-readonly-parameter-type(prefer-readonly-parameter-types)` errors,
and the trailer still counts every rule:

```text
Finished in 1.1s on 12 files with 485 rules using 1 threads.
Found 1207 warnings and 42 errors.
```

There is no diagnostic:
oxlint accepts the flag silently.
Setting the rule to `'off'` in the config produces `484 rules` and `31 errors`,
the 11 rule findings gone.

## Root cause

### Step 1: the flags become `LintFilter` values

`-A`/`--allow`,
`-W`/`--warn`,
and `-D`/`--deny` are one enum (`apps/oxlint/src/command/lint.rs:191-208`,
`#[bpaf(short('A'), long("allow"), argument("NAME"))]` at `:194`).
`CliRunner::run` turns them into `LintFilter` values through `get_filters` (`apps/oxlint/src/lint.rs:108`,
`:681-687`).
`LintFilterKind::parse` (`crates/oxc_linter/src/options/filter.rs:96-160`) maps `plugin/rule` to
`LintFilterKind::Rule(plugin, rule)`,
a bare name to `LintFilterKind::Generic(name)`,
and `all` to `LintFilterKind::All`.
No spelling is rejected,
so every variant in "Symptom" reaches the same code.

### Step 2: the filters are applied to the builtin rule map only

The root config applies them at `apps/oxlint/src/lint.rs:343` (`.with_filters(&filters)`),
nested configs at `apps/oxlint/src/config_loader.rs:481`.
Both land in `ConfigStoreBuilder::with_filter` (`crates/oxc_linter/src/config/config_builder.rs:404-435`),
whose every arm touches `self.rules` and nothing else:

```rust
// crates/oxc_linter/src/config/config_builder.rs:421-431
            AllowWarnDeny::Allow => match filter {
                LintFilterKind::Category(category) => {
                    self.rules.retain(|rule, _| rule.category() != *category);
                }
                LintFilterKind::Rule(plugin, rule) => {
                    let (plugin, rule) = super::rules::unalias_plugin_name(plugin, rule);
                    self.rules.retain(|r, _| r.plugin_name() != plugin || r.name() != rule);
                }
                LintFilterKind::Generic(name) => self.rules.retain(|rule, _| rule.name() != name),
                LintFilterKind::All => self.rules.clear(),
            },
```

`self.rules` is `FxHashMap<RuleEnum, AllowWarnDeny>` (`config_builder.rs:35`):
`RuleEnum` is the generated enum of builtin rules.
The `Deny`/`Warn` arms go through `upsert_where` (`:465-482`),
which iterates `get_all_rules()` (`:440-462`),
and that is the static builtin `RULES` table:

```rust
// crates/oxc_linter/src/config/config_builder.rs:451-452
        if builtin_plugins.is_all() {
            RULES.clone()
```

No arm can match a JS plugin rule.

### Step 3: JS plugin rules live in a map only the config path writes

JS plugin rules are stored beside the builtin map (`config_builder.rs:36`):

```rust
// crates/oxc_linter/src/config/config_builder.rs:36
    pub(super) external_rules: FxHashMap<ExternalRuleId, (ExternalOptionsId, AllowWarnDeny)>,
```

The only writer is `OxlintRules::override_rules`,
called from `from_oxlintrc` (`config_builder.rs:318-327`) with the config file's `rules` block.
For a plugin name that is not builtin it resolves the id through the `ExternalPluginStore`
(`crates/oxc_linter/src/config/rules.rs:199-215`):

```rust
// crates/oxc_linter/src/config/rules.rs:201-208
                    if external_plugin_store.is_enabled() {
                        match external_plugin_store.lookup_rule_id(plugin_name, rule_name) {
                            Ok(external_rule_id) => {
                                // Add options to store and get options ID
                                let options_id = external_plugin_store
                                    .add_options(external_rule_id, &rule_config.config);

                                external_rules_for_override
```

`with_filter` has no `ExternalPluginStore` parameter,
so it could not perform this lookup even if it tried.
`build` (`config_builder.rs:514-519`) copies `external_rules` into the `Config`,
and `Config::new` keeps only warn/deny entries:

```rust
// crates/oxc_linter/src/config/config_store.rs:104
                    external_rules.retain(|(_, _, sev)| sev.is_warn_deny());
```

That single line is why config `'off'` works:
`override_rules` records `AllowWarnDeny::Allow` for the rule,
and `Config::new` drops it.
A CLI `--allow` never writes to `external_rules`,
so the `'error'` entry from the config survives,
`run_external_rules` (`crates/oxc_linter/src/lib.rs:534-544`) sees a non-empty list,
and every surviving id is handed to JS at `lib.rs:850`.

The same reasoning covers `-A all`:
`self.rules.clear()` empties the builtin map,
`external_rules` is untouched,
and `apps/oxlint/src/lint.rs:388-392` keeps the JS linter alive because the store is not empty.
The `85 rules` in the `-A all` trailer are exactly the JS plugin rules the shared config registers.

## Verification

Reproduced 2026-09-06 in a throwaway worktree of this repo at `a0d4f156d`
(`pnpm install --trust-lockfile`,
`mise trust --all --yes`,
one `mise run //package/mcp/mvm:lint:oxlint` so `ensureOxlintConfig()` builds the shared config).
`package/mcp/mvm` was the subject because it reports zero findings on its own.

Harness (a scratch `.ts` run with `node`;
`WT` is the worktree root):

```ts
// scratch allow-probe.ts (excerpt): run from the subject package the way the mise task does
const RULE = 'prefer-readonly-parameter-type/prefer-readonly-parameter-types';
copyFileSync(
  join(WT, 'package/test-fixture/oxlint-no-restricted-syntax/src/readonly-result-provenance-invalid.ts'),
  join(WT, 'package/mcp/mvm/src/readonly-result-provenance-invalid.ts'),
);
for (const extra of [[], ['--allow', RULE], [`--allow=${RULE}`], ['-A', RULE], ['-W', RULE], ['-A', 'all']]) {
  const result = spawnSync('node', [join(WT, 'package/monorepo-task/core-exec/src/oxlint-wrapper.ts'), ...extra], {
    cwd: join(WT, 'package/mcp/mvm'),
    env: { ...process.env, OXLINT_THREADS: '1', PATH: `${join(WT, 'node_modules/.bin')}:${process.env.PATH}` },
    encoding: 'utf8',
  });
  // count lines matching /^\s+x prefer-readonly-parameter-type\(prefer-readonly-parameter-types\)/
}
```

### Invocations that disable the rule

- Config `'off'`:
  root `oxlint.config.ts` rewritten to
  `defineConfig({ ...base, rules: { ...base.rules, [RULE]: 'off' } })`.
  Trailer `Finished in 566ms on 12 files with 484 rules using 1 threads.`,
  `Found 1207 warnings and 31 errors.`,
  0 rule findings.
- `--config /abs/path/oxlint-allow-override.config.ts`,
  a TS file that imports `./oxlint.config.ts` and sets the rule `'off'` (see "Verified workarounds").
  Trailer `484 rules`,
  `Found 1207 warnings and 31 errors.`,
  0 rule findings.
- Positive control:
  `-A all` drops the trailer from `485 rules` to `85 rules`,
  so the flag is parsed and applied to builtin rules.

### Invocations that leave the rule running

Each of these reports 11 `x prefer-readonly-parameter-type(prefer-readonly-parameter-types)` errors,
`485 rules`,
and `Found 1207 warnings and 42 errors.`:

- no flag (baseline)
- `--allow prefer-readonly-parameter-type/prefer-readonly-parameter-types`
- `--allow=prefer-readonly-parameter-type/prefer-readonly-parameter-types`
- `--allow=prefer-readonly-parameter-types`
- `-A prefer-readonly-parameter-type/prefer-readonly-parameter-types`
- `-W prefer-readonly-parameter-type/prefer-readonly-parameter-types` (still errors,
  not warnings)

`-A all` reports the same 11 errors at `85 rules`,
`Found 1174 warnings and 37 errors.`:
the builtin findings are gone,
the JS plugin findings are not.

Timings "with the rule allowed via `--allow`" in `doc/planning/issue-486-workspace-ts-source-imports.md`
measured the rule on;
that file already retracts them.

## Verified workarounds

### Set the rule to `off` in the config

```ts
// oxlint.config.ts (root; throwaway edit, do not commit)
const config: OxlintConfig = defineConfig({
  ...base,
  rules: {
    ...base.rules,
    'prefer-readonly-parameter-type/prefer-readonly-parameter-types': 'off',
  },
},);
```

Tradeoff:
a file edit,
not a flag.
The root `oxlint.config.ts` is generated by `file-enforcer` (`file-enforcer.config.ts:1870`),
so the edit is overwritten on the next run and must never be committed;
keep it in a throwaway worktree.

### Pass an override config file with `--config`

```ts
// oxlint-allow-override.config.ts (repo root, untracked)
import { defineConfig } from 'oxlint';
import root from './oxlint.config.ts';
export default defineConfig({
  ...root,
  rules: { ...root.rules, 'prefer-readonly-parameter-type/prefer-readonly-parameter-types': 'off' },
});
```

Run `oxlint --config /abs/path/oxlint-allow-override.config.ts` from the package directory
(or the wrapper with the same flag).
This is the only CLI-only path found:
it goes through `override_rules`,
which does write `external_rules`.

Tradeoffs:
`--config` disables nested config discovery (`apps/oxlint/src/lint.rs:244-248`),
so any package-level `.oxlintrc` is ignored;
the file must spread the root config or it replaces it entirely;
the `lint:oxlint` mise task forwards no arguments,
so this works only when the wrapper or `oxlint` is invoked directly.

## What does not work

- `--allow <plugin>/<rule>` (also `--allow=<plugin>/<rule>` and `-A <plugin>/<rule>`):
  `with_filter` matches `LintFilterKind::Rule` against `RuleEnum` only.
- `--allow=<rule>` (bare name):
  `LintFilterKind::Generic` also matches `RuleEnum` only.
- `-W <plugin>/<rule>` and `-D <plugin>/<rule>`:
  `upsert_where` iterates the builtin `RULES` table,
  so severity changes never reach JS plugin rules either (upstream issue #22730).
- `-A all`:
  clears the builtin map only;
  JS plugin rules keep running.
- `--disable-unicorn-plugin` and its siblings `--disable-oxc-plugin` and `--disable-typescript-plugin`
  (`apps/oxlint/src/command/lint.rs:383-410`):
  the only plugin toggles on the CLI,
  all builtin;
  `apps/oxlint/src/command/lint.rs` has no `jsPlugins` flag.
- Reading the rule cache as proof the flag worked:
  a whole-repo cold sweep under `--allow` still wrote
  `node_modules/.cache/prefer-readonly-parameter-type` and reported 22 findings;
  the rule executed,
  it was not merely reported.

## Upstream filing artifact

### Upstream filing decision

`.out-of-scope/` was read in full (eleven files):
none covers oxlint,
oxc,
or JS plugins,
so no exemption applies.

Duplicate search (`gh search issues` and `gh api search/issues`,
open and closed,
issues and pull requests,
terms combining `--allow` / `-A` / `-D` / `js plugin` / `jsPlugins` / `external rules` / `cli filter`):

- [oxc-project/oxc#22730](https://github.com/oxc-project/oxc/issues/22730),
  open,
  filed 2026-05-26 against oxlint 1.64.0,
  labels `A-linter` and `A-linter-plugins`,
  no comments:
  "CLI -D does not override severity for jsPlugins rules".
  Same defect,
  reported from the `-D` side with `eslint-plugin-import`.
- [oxc-project/oxc#23336](https://github.com/oxc-project/oxc/pull/23336),
  `fix(linter): apply CLI severity filters to JS plugin rules`,
  opened 2026-06-12,
  closed unmerged by its author on 2026-06-20 with no maintainer review
  ("Closing this for now while I step back to study the project more deeply").
  It names the same root cause
  ("the CLI filter path only updated built-in rules through `ConfigStoreBuilder::with_filter`")
  and added a `with_filters_and_external_rules(filters, &store)` method plus both call sites.

So the artifact is an additive comment on #22730,
not a new issue.
Walking the six constraints:

1. **Is it really upstream's fault?**
   Yes.
   The behaviour contradicts the flag's own help text ("Allow the rule or category (suppress the lint)"),
   and the config path proves the lookup is available at the point the filter is applied.
   Nothing in this repo's wrapper or config alters the flags.
2. **Can upstream fix it?**
   Yes.
   The prototype below is two files and one call-site-free change;
   no architectural constraint is involved.
3. **Are they supporting this use case?**
   Yes,
   with a caveat.
   The CLI docs describe `-A`/`-W`/`-D` as applying to "the rule or category" without excluding plugin rules,
   `--config` plus CLI filters is a tested combination (`apps/oxlint/src/lint.rs:1529-1534`),
   and the issue template asks for `oxlint.config.ts` contents.
   The caveat:
   the JS plugins page says "JS plugins are currently in alpha",
   so parity gaps are expected.
4. **Would the repo welcome our contribution?**
   Yes,
   with disclosure.
   `CONTRIBUTING.md:12-21` and `AGENTS.md:7-15` allow AI-assisted contributions when disclosed and reviewed,
   and warn that unreviewed content is closed.
   PR #23336 disclosed AI assistance and was withdrawn by its author,
   not rejected.
   No ban on outside contributors was found.
5. **Will they likely fix it?**
   Plausible,
   not confirmed.
   Maintainers labelled #22730 `A-linter-plugins` a week after filing,
   nobody has declined it,
   and `config_builder.rs` had twelve upstream commits between 2026-04-30 and 2026-08-17,
   two of them JS-plugin fixes.
   No documented won't-fix exists.
6. **Have we prototyped a minimal fix compatible with their architecture?**
   Yes,
   see "Prototype":
   two source files plus a unit test,
   red before and green after,
   saved as [`oxlint-allow-flag-js-plugin-rules.patch`](oxlint-allow-flag-js-plugin-rules.patch).

All six hold,
so the comment draft is fileable once its disclosure line is completed (constraint 4).
The workaround at our boundary stays the config-level `off`.

### Prototype

Disposable clone (never the read-only one):
`mktemp --directory "${HOME}/temp/agent/upstream-prototype.XXXXXXXX"`,
`gh repo clone oxc-project/oxc <dir> -- --depth 1 --branch apps_v1.81.0`,
origin `https://github.com/oxc-project/oxc.git`,
HEAD `0b4e2e67f4193e7ebfcc64982275eb583ae82c83`.
Removed after the run.

Design:
`with_filter` has no `ExternalPluginStore`,
while `build` already receives one (`config_builder.rs:490-493`) and already resolves `overrides` against it.
The patch records every non-category filter in a new `external_filters` field
and applies them to `external_rules` inside `build`,
mirroring the builtin arms:
`-A plugin/rule` removes,
`-W`/`-D plugin/rule` upserts (keeping configured options),
bare `-A rule` matches by name across plugins through a new `ExternalPluginStore::rule_ids_named`,
and `-A all` clears.
`-W all`/`-D all` are not applied to JS plugin rules:
they carry no category,
so "every rule except nursery" has no meaning for them.
No call site changes;
the nested-config path (`config_loader.rs:481`) and the language server get the fix through `build`.
PR #23336 instead added a second method taking the store and changed both call sites;
this shape also covers the bare-name and `all` cases the PR skipped.

Harness:
a unit test in `config_builder.rs`'s test module with a fake plugin registered straight into an
`ExternalPluginStore` (no JS runtime,
no upstream npm scripts),
run in a secret-free environment inside this session's 8 GB sandbox cgroup:

```bash
# in the disposable clone
env -i HOME="${HOME}" PATH="${HOME}/.cargo/bin:/usr/bin:/bin" RUSTUP_TOOLCHAIN=1.98.1 CARGO_INCREMENTAL=0 \
  cargo test -p oxc_linter --lib --jobs 1 config::config_builder::test::test_filter_external_rules
```

(`RUSTUP_TOOLCHAIN=1.98.1` stands in for the pinned `1.98.0` to avoid a download;
`--jobs 1` and `CARGO_INCREMENTAL=0` keep `rustc` under the 8 GB cap,
`--jobs 4` was SIGKILLed by the kernel OOM killer.)

Before the patch (test added,
source untouched):

```text
test config::config_builder::test::test_filter_external_rules ... FAILED
thread 'config::config_builder::test::test_filter_external_rules' panicked at
  crates/oxc_linter/src/config/config_builder.rs:1085:9:
assertion `left == right` failed: `-A plugin/rule` should remove a configured JS plugin rule
  left: [(0, 1, Warn), (1, 0, Deny)]
 right: [(1, 0, Deny)]
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1257 filtered out; finished in 0.00s
```

After the patch:

```text
test config::config_builder::test::test_filter_external_rules ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1257 filtered out; finished in 0.01s
```

Every `config::` test in the crate still passes after the patch,
same command with filter `config::`:

```text
test result: ok. 142 passed; 0 failed; 0 ignored; 0 measured; 1116 filtered out; finished in 0.04s
```

The build emitted no warnings.

Not run:
the `apps/oxlint` end-to-end fixtures,
which need the napi build plus `pnpm install` of upstream's JS workspace.
The unit test drives the same `with_filters(...).build(...)` sequence as `apps/oxlint/src/lint.rs:343-349`
and `apps/oxlint/src/config_loader.rs:481-483`;
argument parsing is the only untested surface,
and "Verification" shows it already reaches `with_filter`.

### Draft additive comment for oxc-project/oxc#22730

Diffed against the thread:
the issue covers `-D` on 1.64.0 with no source trace;
PR #23336 (closed) has a one-sentence cause and a different patch shape.
New here:
`-A` and `-W` and bare-name and `-A all` are affected too;
still present at 1.81.0;
the full trace;
a patch with no call-site changes;
the `--config` workaround.
No new issue;
not filed.

~~~md
Still present in oxlint 1.81.0 (`apps_v1.81.0`, `0b4e2e67`), and it is wider than `-D`: `-A`, `-W`,
`--allow=<rule>` without the plugin prefix, and even `-A all` leave `jsPlugins` rules running at their
configured severity, while `"off"` in the config file disables them.

Cause: CLI filters are applied by `ConfigStoreBuilder::with_filter`
(`crates/oxc_linter/src/config/config_builder.rs:404-435`), whose every arm mutates `self.rules`
(`FxHashMap<RuleEnum, AllowWarnDeny>`). JS plugin rules live in the separate `self.external_rules` map
(`config_builder.rs:36`), which only `OxlintRules::override_rules` writes
(`crates/oxc_linter/src/config/rules.rs:199-215`) by resolving the name through the `ExternalPluginStore`.
`with_filter` has no store to resolve against, so the config-file severity survives to `Config::new`
(`config_store.rs:104`).

Repro (oxlint 1.81.0, `@oxlint/plugins` 1.81.0, a local JS plugin rule configured as `"error"`):

```sh
oxlint -A my-plugin/my-rule .      # still reports the rule as errors, trailer still counts it
oxlint -W my-plugin/my-rule .      # still errors, not warnings
oxlint -A all .                    # builtin rules gone, JS plugin rules still reported
```

CLI-side workaround: `--config` with a TS config that spreads the real config and sets the rule `"off"`,
which goes through `override_rules`.

Prototype fix with no call-site changes: record non-category filters in the builder and apply them to
`external_rules` in `build`, where the `ExternalPluginStore` is already available (where `overrides` are
resolved too). `-A plugin/rule` removes, `-W`/`-D plugin/rule` upserts keeping options, bare `-A rule` matches
by name via a new `ExternalPluginStore::rule_ids_named`, `-A all` clears; `-W all`/`-D all` are left alone
since JS rules have no category. A unit test in `config_builder.rs` fails before and passes after, and the
existing `config::config_builder::test` cases still pass. Patch: <attach the .patch file, or open as a PR>.

Disclosure: reproduction, source trace, and prototype were prepared with AI assistance (Claude).
<filer: state here what you personally re-ran and reviewed before posting; delete this line otherwise>
~~~
