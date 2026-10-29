/**
 Defects the planted-control run applies, one per invariant family.
 */
import { dependentVersionPlants } from './planted-dependent-version.mjs';
import { markdownPlants } from './planted-markdown.mjs';

/**
 One defect: its name, the file it edits and the edits, each of which must match exactly once.

 @typedef {{ name: string, file: string, edits: readonly { from: string, to: string }[] }} Plant
 */

/**
 Defects with observable behavior, one per invariant family.
 Each `from` must occur exactly once so a control cannot silently stop applying.
 Removing a flag from a mutation list alone changes nothing, because unlisted flags already
 require configuration; the first plant therefore also lists the flag as presentation.

 @type {readonly Plant[]}
 */
export const plants = [
  {
    name: 'PID file text keeps trailing whitespace before the pid prefix is read',
    file: 'package/git-policy/cli/src/native/index_lock_evidence.rs',
    edits: [{
      from: 'let trimmed: &str = super::js_text::trim_javascript_end(text);',
      to: 'let trimmed: &str = text;'
    }],
  },
  {
    name: 'start ticks past the safe integers are accepted',
    file: 'package/git-policy/cli/src/native/process_start.rs',
    edits: [{
      from: 'if ticks > super::json_record::MAX_SAFE_INTEGER {',
      to: 'if ticks > i64::MAX {'
    }],
  },
  {
    name: 'branch -d is treated as a presentation letter',
    file: 'package/git-policy/cli/src/native/config_loading.rs',
    edits: [
      {
        from: 'b"cCdDfmMut"',
        to: 'b"cCDfmMut"'
      },
      {
        from: 'b"vqrai"',
        to: 'b"vqraid"'
      },
    ],
  },
  {
    name: 'a bare git skips configuration',
    file: 'package/git-policy/cli/src/native/config_loading.rs',
    edits: [{
      from: 'if layout.outcome == GlobalOutcome::NoCommand {\n        return ConfigLoading::Required;',
      to: 'if layout.outcome == GlobalOutcome::NoCommand {\n        return ConfigLoading::Skip;',
    }],
  },
  {
    name: 'a separated global value is not consumed',
    file: 'package/git-policy/cli/src/native/global_arguments.rs',
    edits: [{
      from: 'is_value = true;',
      to: 'is_value = false;'
    }],
  },
  {
    name: 'warn is read as error',
    file: 'package/git-policy/cli/src/native/policy_registry.rs',
    edits: [{
      from: 'if name == "warn" {\n        return Some(Severity::Warn);',
      to: 'if name == "warn" {\n        return Some(Severity::Error);',
    }],
  },
  {
    name: 'the landing section is rejected as unknown',
    file: 'package/git-policy/cli/src/native/config_parse.rs',
    edits: [{
      from: 'key == "hooks" || key == "indexLock" || key == "landing"',
      to: 'key == "hooks" || key == "indexLock"',
    }],
  },
  {
    name: 'a control spelling is removed anywhere in a region read without a table',
    file: 'package/git-policy/cli/src/native/wrapper_invocation.rs',
    edits: [{
      from: 'break;',
      to: 'continue;'
    }],
  },
  {
    name: 'keep-going is removed without being recorded',
    file: 'package/git-policy/cli/src/native/wrapper_controls.rs',
    edits: [{
      from: 'ControlMeaning::KeepGoing => controls.keep_going = true,',
      to: 'ControlMeaning::KeepGoing => controls.keep_going = false,',
    }],
  },
  {
    name: 'a publishing push skips the manual-push gate',
    file: 'package/git-policy/cli/src/native/wrapped_command.rs',
    edits: [{
      from: 'if any_policy_applies(&gate_request, gated.as_slice()) {',
      to: 'if false {',
    }],
  },
  {
    name: 'object content past its declared size is accepted',
    file: 'package/git-policy/cli/src/native/candidate_batch.rs',
    edits: [{
      from: String.raw`if terminator[0] != b'\n' {`,
      to: 'if false {'
    }],
  },
  {
    name: 'an object reply for another object is accepted',
    file: 'package/git-policy/cli/src/native/candidate_batch.rs',
    edits: [{
      from: 'if requested.is_some() && requested.as_ref() != Some(&object) {',
      to: 'if false {',
    }],
  },
  {
    name: 'a path whose index records changed is left out of the staged delta',
    file: 'package/git-policy/cli/src/native/candidate_stage.rs',
    edits: [{
      from: 'if later != Some(*records) {',
      to: 'if later.is_none() {'
    }],
  },
  {
    name: 'a rules file named with .. is accepted',
    file: 'package/git-policy/cli/src/native/config_rules_file.rs',
    edits: [{
      from: 'if component == "." || component == ".." {',
      to: 'if component == "." {',
    }],
  },
  {
    name: 'an owner lock record naming PID 0 is accepted',
    file: 'package/git-policy/cli/src/native/owner_lock_record.rs',
    edits: [{
      from: 'if token.is_empty() || owner_pid < 1 || identity.is_empty() {',
      to: 'if token.is_empty() || identity.is_empty() {',
    }],
  },
  {
    name: 'the start time is read from the field before it',
    file: 'package/git-policy/cli/src/native/process_identity.rs',
    edits: [{
      from: 'const LINUX_START_FIELD_INDEX: usize = 19;',
      to: 'const LINUX_START_FIELD_INDEX: usize = 18;'
    }],
  },
  {
    name: 'a journal record of another schema version is accepted',
    file: 'package/git-policy/cli/src/native/transaction_journal_parse.rs',
    edits: [{
      from: 'if !number_equals(&map, "schemaVersion", JOURNAL_SCHEMA_VERSION)\n        || string_field',
      to: 'if string_field',
    }],
  },
  {
    name: 'a capture record with sequence 0 is accepted',
    file: 'package/git-policy/cli/src/native/capture_records.rs',
    edits: [{
      from: 'Some(number) if number >= 1 => return Ok(number),',
      to: 'Some(number) if number >= 0 => return Ok(number),'
    }],
  },
  {
    name: 'a sequence file with a leading zero is accepted',
    file: 'package/git-policy/cli/src/native/capture_store.rs',
    edits: [{
      from: "if digits.is_empty() || (digits.len() > 1 && digits[0] == b'0') {",
      to: 'if digits.is_empty() {',
    }],
  },
  {
    name: 'extra final line feeds are kept',
    file: 'package/git-policy/cli/src/native/policy_final_newline.rs',
    edits: [{
      from: 'if content_end + 1 == bytes.len() {',
      to: 'if content_end < bytes.len() {',
    }],
  },
  ...dependentVersionPlants,
  ...markdownPlants,
];
