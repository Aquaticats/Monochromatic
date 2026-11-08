/**
 Defects the planted-control run applies, one per invariant family.
 */
import { dependentVersionPlants } from './planted-dependent-version.ts';
import { markdownPlants } from './planted-markdown.ts';

/**
 One defect: its name, the file it edits and the edits, each of which must match exactly once.
 */
export type Plant = {
  readonly name: string;
  readonly file: string;
  readonly edits: readonly {
    readonly from: string;
    readonly to: string;
  }[];
};

/**
 Defects with observable behavior, one per invariant family.
 Each `from` must occur exactly once so a control cannot silently stop applying.
 Removing a flag from a mutation list alone changes nothing, because unlisted flags already
 require configuration; the first plant therefore also lists the flag as presentation.
 */
export const plants: readonly Plant[] = [
  {
    name: 'PID file text keeps trailing whitespace before the pid prefix is read',
    file: 'package/git-policy/cli/src/native/lock/index_lock_evidence.rs',
    edits: [{
      from: 'let trimmed: &str = crate::js_text::trim_javascript_end(text);',
      to: 'let trimmed: &str = text;'
    }],
  },
  {
    name: 'start ticks past the safe integers are accepted',
    file: 'package/git-policy/cli/src/native/process/process_start.rs',
    edits: [{
      from: 'if ticks > crate::json_record::MAX_SAFE_INTEGER {',
      to: 'if ticks > i64::MAX {'
    }],
  },
  {
    name: 'branch -d is treated as a presentation letter',
    file: 'package/git-policy/cli/src/native/config/config_loading.rs',
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
    file: 'package/git-policy/cli/src/native/config/config_loading.rs',
    edits: [{
      from: 'if layout.outcome == GlobalOutcome::NoCommand {\n        return ConfigLoading::Required;',
      to: 'if layout.outcome == GlobalOutcome::NoCommand {\n        return ConfigLoading::Skip;',
    }],
  },
  {
    name: 'a separated global value is not consumed',
    file: 'package/git-policy/cli/src/native/invocation/global_arguments.rs',
    edits: [{
      from: 'is_value = true;',
      to: 'is_value = false;'
    }],
  },
  {
    name: 'warn is read as error',
    file: 'package/git-policy/cli/src/native/policy/policy_registry.rs',
    edits: [{
      from: 'if name == "warn" {\n        return Some(Severity::Warn);',
      to: 'if name == "warn" {\n        return Some(Severity::Error);',
    }],
  },
  {
    name: 'the landing section is rejected as unknown',
    file: 'package/git-policy/cli/src/native/config/config_parse.rs',
    edits: [{
      from: 'key == "hooks" || key == "indexLock" || key == "landing"',
      to: 'key == "hooks" || key == "indexLock"',
    }],
  },
  {
    name: 'a control spelling is removed anywhere in a region read without a table',
    file: 'package/git-policy/cli/src/native/invocation/wrapper_invocation.rs',
    edits: [{
      from: 'break;',
      to: 'continue;'
    }],
  },
  {
    name: 'keep-going is removed without being recorded',
    file: 'package/git-policy/cli/src/native/invocation/wrapper_controls.rs',
    edits: [{
      from: 'ControlMeaning::KeepGoing => controls.keep_going = true,',
      to: 'ControlMeaning::KeepGoing => controls.keep_going = false,',
    }],
  },
  {
    name: 'a publishing push skips the manual-push gate',
    file: 'package/git-policy/cli/src/native/invocation/wrapped_command.rs',
    edits: [{
      from: 'if any_policy_applies(&gate_request, gated.as_slice()) {',
      to: 'if false {',
    }],
  },
  {
    name: 'object content past its declared size is accepted',
    file: 'package/git-policy/cli/src/native/candidate/candidate_batch.rs',
    edits: [{
      from: String.raw`if terminator[0] != b'\n' {`,
      to: 'if false {'
    }],
  },
  {
    name: 'an object reply for another object is accepted',
    file: 'package/git-policy/cli/src/native/candidate/candidate_batch.rs',
    edits: [{
      from: 'if requested.is_some() && requested.as_ref() != Some(&object) {',
      to: 'if false {',
    }],
  },
  {
    name: 'a path whose index records changed is left out of the staged delta',
    file: 'package/git-policy/cli/src/native/candidate/candidate_stage.rs',
    edits: [{
      from: 'if later != Some(*records) {',
      to: 'if later.is_none() {'
    }],
  },
  {
    name: 'a rules file named with .. is accepted',
    file: 'package/git-policy/cli/src/native/config/config_rules_file.rs',
    edits: [{
      from: 'if component == "." || component == ".." {',
      to: 'if component == "." {',
    }],
  },
  {
    name: 'an owner token naming PID 0 is accepted',
    file: 'package/git-policy/cli/src/native/worktree_copying/worktree_copy_owner.rs',
    edits: [{
      from: 'if pid < 1 {',
      to: 'if false {',
    }],
  },
  {
    name: 'the start time is read from the field before it',
    file: 'package/git-policy/cli/src/native/process/process_identity.rs',
    edits: [{
      from: 'const LINUX_START_FIELD_INDEX: usize = 19;',
      to: 'const LINUX_START_FIELD_INDEX: usize = 18;'
    }],
  },
  {
    name: 'extra final line feeds are kept',
    file: 'package/git-policy/cli/src/native/policy/policy_final_newline.rs',
    edits: [{
      from: 'if content_end + 1 == bytes.len() {',
      to: 'if content_end < bytes.len() {',
    }],
  },
  ...dependentVersionPlants,
  ...markdownPlants,
];
