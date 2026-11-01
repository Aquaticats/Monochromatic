# Git 2.56.0 URL rewrite cleanup leaves a credential-bearing key active

## Symptom

The frozen file-enforcer generator writes a Git URL rewrite containing a build-secret value,
then attempts to remove a different key.
With the invented value `research-synthetic-not-a-credential`,
the exact cleanup command exits successfully while the private Git configuration still contains that value
and `git ls-remote --get-url` still selects the credential-bearing URL.
No real credential was used.

Application source is `e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`.
The tested generator is
`package/file-enforcer/core/generator/Containerfile`,
SHA-256 `16c174c71e32066baa7e8cd9de77c774fe89f32ec8b450533ad00ca94e43f7ef`.
The [compressed receipt][receipt] retains the full input,
exact task body,
command arguments,
exit statuses,
configuration snapshots,
and exact-version Git source.

## Root cause

The first-party write at `Containerfile:48` puts the token-bearing URL in the subsection name.
Cleanup at `Containerfile:72` names a subsection without that token.
The comments describe a credential helper and an extra header,
but the actual command configures `url.<base>.insteadOf`.
The excerpt is frozen source,
not a proposed implementation:

```dockerfile
# package/file-enforcer/core/generator/Containerfile:45
RUN --mount=type=secret,id=github_token \
    if [ -f /run/secrets/github_token ] && [ -s /run/secrets/github_token ]; then \
        TOKEN=$(cat /run/secrets/github_token) \
        && git config --global url."https://${TOKEN}@github.com/".insteadOf "https://github.com/" ; \
    fi

# package/file-enforcer/core/generator/Containerfile:72
RUN git config --global --unset-all url."https://github.com/".insteadOf 2>/dev/null || true
```

[Git `v2.56.0` source][git-source],
commit `a018953688f1b10bddf91bff8747068f5f4746a4`,
establishes the deciding path.
At `remote.c:470`,
Git parses the exact URL subsection into the rewrite base:

```c
// Git v2.56.0: remote.c:470
	if (parse_config_key(key, "url", &name, &namelen, &subkey) >= 0) {
		struct rewrite *rewrite;
		if (!name)
			return 0;
		if (!strcmp(subkey, "insteadof")) {
			if (!value)
				return config_error_nonbool(key);
			rewrite = make_rewrite(&remote_state->rewrites, name,
					       namelen);
			add_instead_of(rewrite, xstrdup(value));
```

At `remote.c:49`,
`alias_url` searches matching prefixes;
its return at `remote.c:73` combines the selected subsection base with the unmatched URL suffix:

```c
// Git v2.56.0: remote.c:70
		return NULL;

	return xstrfmt("%s%s", r->rewrite[longest_i]->base, url + longest->len);
}

```

At `builtin/config.c:1578`,
the legacy `--unset-all` command passes the supplied key and a null value to the configuration writer:

```c
// Git v2.56.0: builtin/config.c:1578
	else if (actions == ACTION_UNSET_ALL) {
		check_write(&location_opts.source);
		check_argc(argc, 1, 2);
		ret = repo_config_set_multivar_in_file_gently(the_repository, location_opts.source.file,
							      argv[0], NULL, argv[1],
							      NULL, flags | CONFIG_FLAGS_MULTI_REPLACE);
	}
```

At `config.c:3151`,
no matching key produces `CONFIG_NOTHING_SET`.
The exact release defines that status as `5` in `config.h:33`:

```c
// Git v2.56.0: config.c:3150
		/* if nothing to unset, or too many matches, error out */
		if ((store.seen_nr == 0 && value == NULL) ||
		    (store.seen_nr > 1 && !store.multi_replace)) {
			ret = CONFIG_NOTHING_SET;
			goto out_free;
		}

```

```c
// Git v2.56.0: config.h:33
#define CONFIG_NOTHING_SET 5
```

The authored shell clause suppresses stderr and uses `|| true`.
The measured direct unset returns `5`;
the measured exact shell clause returns `0` while preserving the live rewrite.
This is a first-party key mismatch.
The probe does not establish a Git defect.

## Verification

The tested binary reported `git version 2.56.0`.
Its SHA-256 was `20739384fd97e904e8545b39753c3e42a0cd7a8016d382f6ec1aadc910f8c872`.
The exact tagged Git source is retained with per-file hashes.
The command fixture redirected `HOME`,
`XDG_CONFIG_HOME`,
and `GIT_CONFIG_GLOBAL` to an owned private directory,
disabled system configuration,
cleared inherited Git-specific environment settings,
and ran outside a repository.
No real user-global configuration was read or modified by these commands.

The executed Mise task `probe-file-enforcer-git-config` and its complete Node body are in the receipt.
The URL expansion used `git ls-remote --get-url`.
The exact release's `Documentation/git-ls-remote.adoc:56` states that this option
expands URL rewrites and exits without talking to the remote.
No packet-level network isolation claim is made.

The working catalog is:

- Empty configuration resolves the ordinary GitHub URL.
- Setting the exact credential-bearing key changes the resolved URL to the invented-token URL.
- Unsetting that exact key returns `0`,
  restores the ordinary URL,
  and leaves an empty configuration file in this fixture.
- Removing the exact credential-bearing section independently returns `0`,
  restores the ordinary URL,
  and leaves an empty configuration file.

The failing catalog is:

- Unsetting the token-free key returns `5` and preserves both the token-bearing configuration and URL.
- Running the exact frozen cleanup shell clause returns `0` and preserves the same configuration and URL.

Both positive removal controls followed an active rewrite,
so an unchanged result could not pass because the setup failed to install the rewrite.
The complete generator image was not built,
no actual exported image or layer was inspected,
and no real secret exposure was demonstrated.

## Verified workarounds

For the measured live-configuration boundary,
unset the exact key that was set:

`git config --global --unset-all url.https://research-synthetic-not-a-credential@github.com/.insteadOf`

That command is the actual successful positive control.
The tradeoff is that automation must retain the precise subsection identity and avoid disclosing its real value
through command logging.
It removes every value under that exact key,
including values unrelated to one clone if the caller shared that subsection.

The independently measured section control is:

`git config --global --remove-section url.https://research-synthetic-not-a-credential@github.com/`

Its tradeoff is broader removal:
it removes every setting under the exact section.
Neither live-configuration control proves removal from an already-created container layer.
A production image change needs an actual exported-image consumer and secret-retention checks.
No source fix or production image change was made in this research.

## What does not work

- Removing the token-free subsection while the active rewrite uses a token-bearing subsection.
- Treating the exact cleanup shell's successful exit as evidence that the rewrite disappeared.
- Treating comments about credential helpers or extra headers as descriptions of the executed URL-rewrite command.
- Crediting this local Git control as an exported-image or layer inspection.

## Upstream filing decision

Nothing to add to Git upstream:
the observed mismatch is in this repository's generator.
No Git issue,
comment,
or patch was drafted or posted.
No duplicate search or contribution-policy inference is needed for a report that is not proposed.

- Upstream fault:
  not established;
  Git applies and removes the exact supplied key in the measured controls.
- Fixability:
  existing Git commands already satisfy the live-configuration requirement.
- Supported use case:
  URL rewriting and exact-key removal are documented in the exact released source.
- Contribution acceptance:
  unassessed;
  no external contribution is proposed.
- Likelihood of an upstream fix:
  unassessed;
  no upstream fix is required by this evidence.
- Prototype:
  no Git patch was produced;
  both consumer-side configuration controls were executed against an active rewrite.

[receipt]: ../planning/depot-complete-coverage-file-enforcer-git-config.json.gz
[git-source]: https://github.com/git/git/tree/v2.56.0
