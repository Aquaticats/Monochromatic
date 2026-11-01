# Translation repair documentation redaction timing

## Standing instruction

Documentation redaction is deferred while this package remains unfinished and not production-ready.
Preserve corpus,
review,
calibration,
and takeover evidence needed to finish the package instead of redacting it preemptively.

Redaction becomes a required readiness gate before the package may be declared finished or production-ready.
That gate must review package-local and repository translation-repair documentation together.

This standing instruction supersedes older package README statements that corpus wording must not be pasted anywhere or
that corpus and reviewer wording are not retained.
Treat those statements as historical workflow descriptions,
not as instructions to redact unfinished-package documentation before the readiness gate.

## Reading and committing during development

Owner instruction,
2026-09-01:
"You can read any content and commit anything.
Sanitization will be done after the project is finished."

While the package is unfinished,
agents may read any retained run artifact,
candidate page,
review,
raw provider response,
or corpus text,
and may commit working evidence into the repository without pre-emptive masking.
Do not refuse or defer package work on redaction grounds;
redaction is the readiness gate at the end,
not a constraint during development.

## Materials that remain private

The timing deferral does not authorize committing credentials,
API keys,
image bytes,
or raw provider requests or responses.
Keep those materials private throughout development and after release.
