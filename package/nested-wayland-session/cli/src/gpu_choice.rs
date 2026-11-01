//! Which GPU a headless session renders on:
//!  the `--gpu` value,
//!  the order of devices `auto` tries,
//!  and the environment the hosted child needs to land on the same device.
//!
//! Everything here is pure data and decisions,
//!  so the fallback order and the failure text are unit-tested without a GPU.
//! Opening a device,
//!  through EGL and sysfs,
//!  is in `gpu_device.rs`.

/// Display for the attempt and failure text.
use std::fmt;
/// Owned render node paths.
use std::path::PathBuf;

/// The `--gpu` value.
///
/// What:
///      `pub enum GpuChoice { Auto, Software, RenderNode(PathBuf) }`.
///  `Auto` is the default and keeps the behavior headless had before the option existed.
/// Why:
///       Headless has no parent compositor to inherit a GPU from,
///  so a person who has several GPUs,
///  or none,
///  names the one to use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GpuChoice = { kind: 'auto' } | { kind: 'software' } | { kind: 'render-node'; path: string };
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum GpuChoice {
    /// Mesa's default render node,
    /// then the other render nodes,
    /// then software rendering.
    #[default]
    Auto,
    /// Mesa's CPU rasterizer (`llvmpipe`);
    /// no GPU is opened.
    Software,
    /// One DRM render node,
    /// such as `/dev/dri/renderD128`.
    RenderNode(
        /// The absolute path the person passed.
        PathBuf,
    ),
}

/// Parsing and describing the choice.
impl GpuChoice {
    /// Parse a `--gpu` value.
    ///
    /// What:
    ///      `pub fn parse(text: &str) -> Result<Self, String>`.
    ///  `auto` and `software` are words;
    ///  anything else must be an absolute path.
    /// Why:
    ///       A bare word such as `amd` or a relative path would need a guess about which device is meant,
    ///  so the message names the three accepted forms and the listing option instead.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// parse('auto') // => { kind: 'auto' }
    /// parse('/dev/dri/renderD128') // => { kind: 'render-node', path: '/dev/dri/renderD128' }
    /// ```
    pub fn parse(text: &str) -> Result<Self, String> {
        if text == "auto" {
            return Ok(Self::Auto)
        }
        if text == "software" {
            return Ok(Self::Software)
        }
        if text.starts_with('/') {
            return Ok(Self::RenderNode(PathBuf::from(text)))
        }
        return Err(format!(
            "--gpu takes auto, software, or the absolute path of a DRM render node such as /dev/dri/renderD128, got: {text}. \
             Run the tool with --list-gpus to see the render nodes and the software renderer this host offers"
        ))
    }

    /// Whether the person named a device,
    /// as opposed to leaving the choice to `auto`.
    ///
    /// What:
    ///      `pub fn is_explicit(&self) -> bool`.
    /// Why:
    ///       An explicit choice never falls back,
    ///  and the hosted child's environment is pinned to it.
    pub fn is_explicit(&self) -> bool {
        return !matches!(self, Self::Auto)
    }

    /// The text of the choice as the person would type it.
    ///
    /// What:
    ///      `pub fn label(&self) -> String`.
    /// Why:
    ///       Logs,
    ///  the `gpu` control answer,
    ///  and error messages quote it.
    pub fn label(&self) -> String {
        return match self {
            Self::Auto => "auto".to_string(),
            Self::Software => "software".to_string(),
            Self::RenderNode(path) => path.to_string_lossy().into_owned(),
        }
    }
}

/// One way of obtaining a renderer.
///
/// What:
///      `pub enum Attempt { MesaDefault, RenderNode(PathBuf), Software }`.
/// Why:
///       `auto` is an ordered list of these;
///  each failure is reported with the attempt it belongs to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Attempt {
    /// The device Mesa's surfaceless platform picks on its own.
    MesaDefault,
    /// One render node,
    /// opened through `EGL_EXT_platform_device`.
    RenderNode(
        /// The render node path.
        PathBuf,
    ),
    /// The CPU rasterizer device (`EGL_MESA_device_software`).
    Software,
}

/// Names the attempt in a failure list.
impl fmt::Display for Attempt {
    /// Write the attempt as the failure list names it.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self {
            Self::MesaDefault => write!(formatter, "Mesa's default render node"),
            Self::RenderNode(path) => write!(formatter, "render node {}", path.display()),
            Self::Software => write!(formatter, "the software renderer (llvmpipe)"),
        }
    }
}

/// An attempt that did not produce a renderer,
/// with the reason.
///
/// What:
///      `pub struct Failure { attempt, reason }`.
/// Why:
///       The log line and the final error list every one,
///  so a person sees why the GPU was not used.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failure {
    /// What was tried.
    pub attempt: Attempt,
    /// Why it did not work,
    /// as the failing step reported it.
    pub reason: String,
}

/// The attempt that worked,
/// and what failed before it.
///
/// What:
///      `pub struct Selected<T> { attempt, value, failures }`.
/// Why:
///       `failures` is empty when the first attempt worked;
///  otherwise the session fell back and says so.
#[derive(Debug, PartialEq, Eq)]
pub struct Selected<T> {
    /// The attempt that succeeded.
    pub attempt: Attempt,
    /// What the successful attempt produced.
    pub value: T,
    /// Earlier attempts that failed,
    /// in the order they were tried.
    pub failures: Vec<Failure>,
}

/// The attempts to make for a choice,
/// in order.
///
/// What:
///      `pub fn plan(choice: &GpuChoice, hardware_nodes: &[PathBuf]) -> Vec<Attempt>`.
///  `Auto` is Mesa's default,
///  then every listed render node in path order,
///  then software.
///  An explicit choice is one attempt.
/// Why:
///       Only `auto` falls back.
///  `hardware_nodes` are the render nodes EGL enumerates,
///  passed in so the order is testable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plan({ kind: 'auto' }, ['/dev/dri/renderD129', '/dev/dri/renderD128'])
/// // => [mesaDefault, node(D128), node(D129), software]
/// ```
pub fn plan(choice: &GpuChoice, hardware_nodes: &[PathBuf]) -> Vec<Attempt> {
    match choice {
        GpuChoice::Software => return vec![Attempt::Software],
        GpuChoice::RenderNode(path) => return vec![Attempt::RenderNode(path.clone())],
        GpuChoice::Auto => {}
    }
    let mut nodes = hardware_nodes.to_vec();
    nodes.sort();
    nodes.dedup();
    let mut attempts = vec![Attempt::MesaDefault];
    attempts.extend(nodes.into_iter().map(Attempt::RenderNode));
    attempts.push(Attempt::Software);
    return attempts
}

/// The reason Mesa's default device is not accepted as the GPU.
///
/// What:
///      `pub fn default_device_rejection(is_software: bool) -> Option<String>`.
///  `Some(reason)` when Mesa's surfaceless platform answered with the CPU rasterizer.
/// Why:
///       Mesa's surfaceless platform quietly opens `llvmpipe` when no render node can be used.
///  Taking that as success would stop `auto` before it tried the other GPUs,
///  and would hide that no GPU is rendering.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// defaultDeviceRejection(true) // => "Mesa chose the software rasterizer ..."
/// ```
pub fn default_device_rejection(is_software: bool) -> Option<String> {
    if is_software {
        return Some(
            "Mesa chose the software rasterizer for its default device, which means it found no render node it can use"
                .to_string(),
        )
    }
    return None
}

/// Try the attempts in order and stop at the first that works.
///
/// What:
///      `pub fn select<T>(attempts: &[Attempt], open: impl FnMut(&Attempt) -> Result<T, String>)
///           -> Result<Selected<T>, Vec<Failure>>`.
///  `Err` carries every failure when none worked.
/// Why:
///       The decision is separate from opening devices,
///  so tests feed it per-device outcomes.
///  Later attempts are never opened once one works.
pub fn select<T>(
    attempts: &[Attempt],
    mut open: impl FnMut(&Attempt) -> Result<T, String>,
) -> Result<Selected<T>, Vec<Failure>> {
    let mut failures = Vec::new();
    for attempt in attempts {
        match open(attempt) {
            Ok(value) => return Ok(Selected { attempt: attempt.clone(), value, failures }),
            Err(reason) => failures.push(Failure { attempt: attempt.clone(), reason }),
        }
    }
    return Err(failures)
}

/// The error text when no device could be opened.
///
/// What:
///      `pub fn failure_message(choice: &GpuChoice, failures: &[Failure], offered: &str) -> String`.
///  `offered` is the text of the devices this host lists,
///  so the person sees the valid choices beside the failure.
/// Why:
///       An explicit choice fails with its own reason;
///  `auto` lists everything it tried.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// failureMessage({ kind: 'software' }, [{ attempt: software, reason: 'no software device' }], listing)
/// ```
pub fn failure_message(choice: &GpuChoice, failures: &[Failure], offered: &str) -> String {
    let mut text = String::new();
    match choice {
        GpuChoice::Auto => {
            text.push_str("--headless could not open any GPU or the software renderer. Tried, in order:");
            for failure in failures {
                text.push_str(&format!("\n  {}: {}", failure.attempt, failure.reason));
            }
        }
        explicit => {
            let reason = failures.first().map_or("no device was tried", |failure| return failure.reason.as_str());
            text.push_str(&format!("--gpu {} cannot be used: {reason}", explicit.label()));
        }
    }
    text.push_str(&format!(
        "\nChoices: auto, software, or a render node. {offered}\nAn explicit --gpu choice never falls back to another device; only auto does"
    ));
    return text
}

/// What the hosted child's environment must say so it renders on the compositor's device.
///
/// What:
///      `pub struct ChildGpuEnvironment { software, pin_device }`.
/// Why:
///       Mesa reads these variables in the client:
///  `LIBGL_ALWAYS_SOFTWARE=1` keeps a client with no render node on its CPU rasterizer,
///  and `DRI_PRIME` otherwise moves a client's rendering to another GPU than the compositor's
///  (measured on this host's two AMD GPUs).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChildGpuEnvironment {
    /// The compositor renders in software,
    /// so the child must too.
    pub software: bool,
    /// The session pinned a device (an explicit choice or a fallback),
    /// so a `DRI_PRIME` inherited from the caller must not move the child.
    pub pin_device: bool,
}

/// Compute the child's environment needs from the final choice.
///
/// What:
///      `pub fn child_environment(choice: &GpuChoice, software: bool, fell_back: bool) -> ChildGpuEnvironment`.
///  `auto` with no fallback leaves the environment as the caller had it.
/// Why:
///       `auto` without a fallback is the behavior headless had before the option;
///  a pinned device must hold for the child too.
pub fn child_environment(choice: &GpuChoice, software: bool, fell_back: bool) -> ChildGpuEnvironment {
    return ChildGpuEnvironment {
        software,
        pin_device: choice.is_explicit() || fell_back || software,
    }
}

/// The variables to set (`Some`) or remove (`None`) on the child command.
///
/// What:
///      `pub fn child_variables(environment: ChildGpuEnvironment) -> Vec<(&'static str, Option<&'static str>)>`.
/// Why:
///       One list the spawn code applies and the tests read.
pub fn child_variables(environment: ChildGpuEnvironment) -> Vec<(&'static str, Option<&'static str>)> {
    let mut variables = Vec::new();
    if environment.software {
        variables.push(("LIBGL_ALWAYS_SOFTWARE", Some("1")));
    }
    if environment.pin_device {
        variables.push(("DRI_PRIME", None));
    }
    return variables
}

/// The device a session renders on,
/// as the `gpu` control command and the log report it.
///
/// What:
///      `pub struct GpuReport { choice, device, renderer, fell_back }`.
/// Why:
///       A caller that asked for `auto` must be able to see which device it got,
///  and whether the session fell back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuReport {
    /// The `--gpu` value as typed (`auto` by default).
    pub choice: String,
    /// The render node path,
    /// `software`,
    /// `parent` for the nested window,
    /// or `unknown` for a device with no render node path.
    pub device: String,
    /// The GL renderer string of the compositor's context.
    pub renderer: String,
    /// True when `auto` used a later attempt than the first.
    pub fell_back: bool,
}

/// The control answer text.
impl GpuReport {
    /// One line:
    ///  `device=D choice=C fallback=yes|no renderer=R`.
    ///
    /// What:
    ///      `pub fn answer(&self) -> String`.
    ///  The renderer comes last because it contains spaces.
    /// Why:
    ///       The `gpu` verb is read by scripts and by people.
    pub fn answer(&self) -> String {
        return format!(
            "device={} choice={} fallback={} renderer={}",
            self.device,
            self.choice,
            if self.fell_back { "yes" } else { "no" },
            self.renderer,
        )
    }
}

/// Verifies parsing,
///  the fallback order and decision,
///  the failure text,
///  and the child environment.
#[cfg(test)]
#[path = "gpu_choice_tests.rs"]
mod tests;
