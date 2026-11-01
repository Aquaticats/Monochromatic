# GitHub workflow inline scripts need an independent size boundary

## Symptom and deciding source

The first prepared current-host research workflow at `334a5b75f376eb467e30b426a14369690bc2795f`
contained a `run` field of 114,331 characters.
It was measured before dispatch and caused no cloud allocation.
This is not a claim of an observed GitHub rejection.
The current [official workflow syntax](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax)
says `jobs.<job_id>.steps[*].run` runs command-line programs
that do not exceed 21,000 characters.
The captured primary HTML and measured fields are retained in the current-host admission evidence.

## Verified construction

The research bootstrap emits the shared task function once,
then derives the bounded task definitions at runtime.
A separate step writes the immutable source configuration.
The corrected deployed harness is `aa501f4a5c0e74a0a993b668c98b00fe3be2da19`;
its largest actual `run` field is 12,005 characters.
The actual bootstrap executes locally,
its generated task bytes match the expected task file,
Mise parses the generated task, and Node checks its module syntax.
Application assertions remain frozen.

## Separate evidence-handoff failures

Passing this script-size and bootstrap check did not qualify task logging or artifact discovery.
Public run `39141147245` exposed a separate
[log-filename boundary](node-task-log-filenames.md)
and a separately verified [hidden-root upload boundary](github-artifact-hidden-sdk-resources.md).
Correcting one does not prove the others.
The public recovery is `39144499822`; its completed Mac artifact restores the actual owner output.

## Upstream filing decision

No upstream defect was established.
The documented limit was detected before dispatch and the integration was changed to respect it.
No issue or external comment was sent.
