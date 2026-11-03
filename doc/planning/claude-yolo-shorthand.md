# Proposed `claude --yolo` shorthand

## Recommendation

For the user's interactive Bash command,
define a `claude` function in `${HOME}/.bashrc.d/claude.sh` that translates a leading `--yolo`
to Claude Code's supported `--dangerously-skip-permissions` option.
The inspected `${HOME}/.bashrc` already sources files from `${HOME}/.bashrc.d/`.
This is a proposal,
not an installed shell change.

```bash
# Proposed personal shell configuration: ${HOME}/.bashrc.d/claude.sh
claude() {
  if [[ ${1-} == --yolo ]]; then
    shift
    set -- --dangerously-skip-permissions "$@"
  fi
  command claude "$@"
}
```

`claude --yolo --continue` becomes `claude --dangerously-skip-permissions --continue`.
`claude --continue` forwards unchanged.
The bypass option skips permission prompts;
it does not add process isolation.

The shorthand must come first.
Translating arbitrary arguments would need Claude's option grammar to distinguish an option from
a literal `--yolo` prompt or an option value.
`command claude` invokes the executable found through `PATH` without recursively calling the function.
Quoted `"$@"` preserves argument boundaries,
and the last command's status becomes the function's status.

## Alternatives and ranking

- A Bash function supports the requested spelling and follows the existing executable on `PATH`.
  It applies only in shells where the function is loaded.
- An executable wrapper earlier on `PATH` also covers subprocess launches.
  It needs a distinct way to locate the real executable and avoid invoking itself,
  plus integration with the existing installation's update behavior.
- A separate alias such as `cyolo` is concise and needs no argument translation.
  It changes the requested command spelling.

Ranking:
Bash function > executable wrapper > separate alias for the stated interactive command.
The function achieves that command without managing another executable path;
the executable wrapper ranks ahead of the alias because it preserves `claude --yolo`.
If the required consumer is a program spawning `claude` directly,
the executable wrapper becomes the relevant design instead.

## Evidence and limits

[Verification](../troubleshooting/claude-code-yolo-option.md) covers the installed CLI,
argument preservation,
and exit status forwarding.
The existing user setting is `permissions.defaultMode = "auto"`;
the proposal makes bypass mode explicit per invocation.
Interactive onboarding,
policy restrictions,
and an authenticated model turn were not exercised.
