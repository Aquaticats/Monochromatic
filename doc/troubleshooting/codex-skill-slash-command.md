# Codex 0.167.0 rejects `/troubleshooting-doc` in the prompt box

## Symptom

The user entered `/troubleshooting-doc` directly in the current Codex prompt box and received:

```text
# Reported prompt-box diagnostic.
Unrecognized command '/troubleshooting-doc'. Type "/" for a list of supported commands.
```

The user clarified:
"I input `/troubleshooting-doc` in this prompt box back then and it errored."
This was an interactive prompt submission,
not a shell invocation of an executable named `troubleshooting-doc`,
and not the prompt argument of `codex exec`.

The local skill exists at
[`.agents/skills/troubleshooting-doc/SKILL.md`](../../.agents/skills/troubleshooting-doc/SKILL.md).
Its frontmatter name is `troubleshooting-doc`.
Skill discovery and recognition as a slash command are separate mechanisms.

## Root cause

Source inspected at `rust-v0.167.0`,
commit `086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a`.

[`codex-rs/tui/src/bottom_pane/slash_commands.rs:135`][lookup]
looks for a built-in command and then an enabled service-tier command:

```rust
// openai/codex: codex-rs/tui/src/bottom_pane/slash_commands.rs:140
if let Some(cmd) = find_builtin_command(name, flags) {
    return Some(SlashCommandItem::Builtin(cmd));
}

let tiers_enabled = flags.service_tier_commands_enabled;
tiers_enabled
    .then(|| {
        service_tier_commands
            .iter()
            .find(|command| command.name == name)
            .cloned()
            .map(SlashCommandItem::ServiceTier)
    })
    .flatten()
```

That lookup does not consult the skill catalog.
In [`codex-rs/tui/src/bottom_pane/chat_composer/slash_input.rs:68`][validation],
an unrecognized slash name becomes `UnknownCommand`:

```rust
// openai/codex: codex-rs/tui/src/bottom_pane/chat_composer/slash_input.rs:82
if self.command(name).is_some() {
    SubmissionValidation::Valid
} else {
    SubmissionValidation::UnknownCommand(name.to_string())
}
```

[`codex-rs/tui/src/bottom_pane/chat_composer.rs:3121`][diagnostic]
formats the exact diagnostic,
restores the draft,
and returns `None` instead of submitting the prompt:

```rust
// openai/codex: codex-rs/tui/src/bottom_pane/chat_composer.rs:3126
let message = format!(
    r#"Unrecognized command '/{name}'. Type "/" for a list of supported commands."#
);
```

The skill mention parser uses a different prefix in
[`codex-rs/skills/src/mentions.rs:43`][sigil]:

```rust
// openai/codex: codex-rs/skills/src/mentions.rs:43
const TOOL_MENTION_SIGIL: char = '$';
```

The built-in `/skills` command has its own dispatch in
[`codex-rs/tui/src/chatwidget/slash_dispatch.rs:527`][menu]:

```rust
// openai/codex: codex-rs/tui/src/chatwidget/slash_dispatch.rs:527
SlashCommand::Skills => {
    self.open_skills_menu();
}
```

The [official skills documentation][skills-doc] likewise specifies `/skills` or `$` for explicit CLI skill invocation.
Installing a skill does not register `/<skill-name>` in this command lookup.
The observed error therefore does not establish that the skill is absent or invalid.

## Verification

The original prompt-box report did not include its own version banner.
The adjacent `codex exec` transcript identified `0.167.0`;
that alone would not establish which build handled the earlier prompt-box input.

The behavior was separately reproduced on 2026-11-09 in the installed `codex-cli 0.167.0` interactive terminal.
The reproduction used a fresh configuration home and working directory,
a local Responses provider,
and a fixture skill with this frontmatter:

```yaml
# Disposable .agents/skills/troubleshooting-doc/SKILL.md frontmatter.
name: troubleshooting-doc
description: Test skill for Codex invocation syntax.
```

The fixture skill had a distinct body marker so invocation could be distinguished from merely listing its name.
It did not run the repository's documentation workflow.

The failing input was:

```text
# Text submitted in the interactive Codex prompt box.
/troubleshooting-doc
```

Codex displayed the exact reported `Unrecognized command` diagnostic.
The successful menu input was:

```text
# Built-in command submitted in the same prompt box.
/skills
```

Codex opened the `Skills` menu,
including `List skills` and `Enable/Disable Skills`.
The menu itself displayed `Tip: press $ to open this list directly`.

The fixture also accepted `$troubleshooting-doc` as a skill selection in the composer.
The captured provider input contained the skill's full body inside a `<skill>` block,
including `fixture-skill-body-loaded`,
and the terminal displayed the provider's `fixture completed` response.
This verifies selection and delivery independently of whether a model follows the selected workflow.
Other Codex releases,
the IDE extension,
and the ChatGPT desktop composer were not exercised.

## Verified workarounds

Replace the unsupported slash name with a skill mention in the prompt box:

```text
# Prompt-box input, not a shell command.
$troubleshooting-doc Document the Codex behavior described in this conversation.
```

Alternatively,
use `/skills` and select the skill through the menu.
The tradeoff is that skill availability still depends on discovery and enabled state;
neither syntax creates a missing skill.
The selector also lets the user choose an explicit entry when names are ambiguous.

For this task,
the plain-language request "Write the troubleshooting-doc" successfully reached the assistant,
which opened the repository's `SKILL.md`.
That provides another observed route.
Its tradeoff is that it relies on the assistant recognizing the requested workflow.

## What does not work

- `/troubleshooting-doc` at the start of the prompt:
  the local reproduction rejects it even with a discovered skill of that name.
- Treating the error as proof that the skill must be reinstalled:
  the command lookup and skill mention parser are separate,
  and the fixture reproduced the error with the skill present.
- Testing `codex exec '/troubleshooting-doc'` as a reproduction of this report:
  that supplies prompt text through a different entry point and does not exercise the interactive composer.

## Upstream filing decision

The [Codex harness maintenance policy](../../.out-of-scope/codex-harness.md) was checked.
It does not prohibit investigation or issue tracking.

- Upstream fault:
  no broken documented skill-invocation route was established.
  The rejected input uses the slash-command prefix for a skill name.
- Fixability:
  additional aliases or a skill-specific diagnostic could be implemented,
  but that would be a separate interface change.
- Supported use case:
  explicit skills are supported through `/skills` and `$` in the cited documentation.
  That documentation does not promise a slash alias for each skill.
- Contribution policy:
  [`docs/contributing.md:3`][contributing] welcomes reports and analyses;
  line `5` says "We do not accept external code contributions or pull requests."
- Maintainer willingness:
  no maintainer commitment to adding arbitrary skill slash aliases was established.
- Minimal fix:
  none prepared because the supported invocation surface works in the local composer probe.

Tracker searches used `gh search issues --repo openai/codex` with
`"Unrecognized command" "skill"`
and `gh search prs --repo openai/codex` with `skills slash`.
The exact-error issue query returned no results;
the pull-request query returned a historical skills-documentation change.
These searches do not establish that no broader request for slash aliases exists.

Nothing to add as a demonstrated upstream defect.
No issue or comment was drafted or sent.
Latest-release comparisons,
fork-branch searches,
trust-system checks,
and a clean-runner publication were not performed;
no filing readiness is claimed.

[lookup]: https://github.com/openai/codex/blob/086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a/codex-rs/tui/src/bottom_pane/slash_commands.rs#L135-L154
[validation]: https://github.com/openai/codex/blob/086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a/codex-rs/tui/src/bottom_pane/chat_composer/slash_input.rs#L68-L87
[diagnostic]: https://github.com/openai/codex/blob/086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a/codex-rs/tui/src/bottom_pane/chat_composer.rs#L3121-L3143
[sigil]: https://github.com/openai/codex/blob/086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a/codex-rs/skills/src/mentions.rs#L38-L43
[menu]: https://github.com/openai/codex/blob/086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a/codex-rs/tui/src/chatwidget/slash_dispatch.rs#L524-L529
[skills-doc]: https://learn.chatgpt.com/docs/build-skills#:~:text=In%20Codex%20CLI%20or%20the%20IDE%20extension
[contributing]: https://github.com/openai/codex/blob/086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a/docs/contributing.md#L3-L5
