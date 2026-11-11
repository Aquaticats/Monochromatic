//! What:
//!  The resident commit service:
//!  protocol,
//!  server,
//!  client,
//!  journal,
//!  provisioning and logging.
//! Why:
//!  The service is a mode of the one executable with a life of its own:
//! the operating system starts it on demand.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // service/index.ts: one `export * from './file'` per module below.
//! ```

/// The removal of a commit's scratch directory once the client that held it is gone,
/// beside the landing loop.
pub mod service_scratch_release_removal;

/// The fill of the shadow pool a commit takes its private repository from,
/// beside the landing loop.
pub mod service_shadow_pool_fill;

/// Whether a socket of an earlier release's unit listens,
/// read from the table of unix sockets.
pub mod earlier_unix_sockets_that_listen;

/// Messages between the wrapper and the resident commit service.
pub mod service_protocol;

/// The protocol messages as typed values,
/// and the checks on the names they carry.
pub mod service_protocol_messages_and_name_checks;

/// Writes protocol messages as one JSON object per line.
pub mod service_protocol_encode_messages_to_lines;

/// Reads protocol lines as messages,
/// checking every field.
pub mod service_protocol_decode_lines_to_messages;

/// The longest the wrapper waits for each thing it waits for,
/// with the measurement beside each number.
pub mod wait_ceilings;

/// What the one landing thread is doing,
/// shared with the threads that talk to waiting clients.
pub mod service_busy;

/// The answers the landing board gives and the checks on the names it keeps.
pub mod service_busy_answers_and_name_checks;

/// The step record a thread keeps while it is inside a write that is never ended.
pub mod service_busy_step_record_on_this_thread;

/// The connection of one waiting client,
/// answered by the landing thread and kept alive by a beat from the reading thread.
pub mod service_reply;

/// The commit service's journal and its restart decisions.
pub mod service_journal;

/// The text form of a batch intent:
/// written and parsed.
pub mod service_journal_intent_text_and_parse;

/// Test-only fault points of the commit service.
pub mod service_fault;

/// The native journal protocol and the standard error fallback the service's entries go through.
pub mod journald_native;

/// The address native journal entries are sent to.
pub mod journald_native_socket_target;

/// The lines the commit service writes about its life and about the time each request took.
pub mod service_timing;

/// The line the commit service writes when a signal ends it.
pub mod service_signal;

/// The service's own shutdown after a stop:
/// it tells the waiters,
/// waits a bounded time for the Gits and the landing in flight,
/// and ends the process by the signal.
pub mod service_server_shutdown_after_a_stop;

/// The plan of the pushes waiting for a new try and the repositories an earlier service was woken for,
/// read at a service's start on a thread of its own.
pub mod service_journal_restored_at_start;

/// The commit service's loop.
pub mod service_server;

/// The loop of the commit service's after-commit work asked by a thread outside the runtime.
pub mod service_server_work_loop;

/// The same loop as a task of the runtime,
/// with the two storage steps only it makes.
pub mod service_server_work_loop_awaiting;

/// The wait of a thread on a queue the runtime owns:
/// an item,
/// the end of the wait at its bound,
/// or the end of the queue.
pub mod service_queue_received_on_the_runtime;

/// The shapes the connection threads and the landing thread pass between them,
/// and the one-message send.
pub mod service_server_queue_types_and_send;

/// Lands one group of queued requests as a batch and answers each client.
pub mod service_server_land_group;

/// What one batch's answer says about that batch:
/// the times it shares,
/// its figures,
/// and the log line of each answer.
pub mod service_server_land_group_answer_log;
/// What a batch of landings owes,
/// and takes before its branch moves.
pub mod service_server_land_group_owed_pushes;

/// Handles one client connection of the commit service.
pub mod service_server_connection_handling;

/// The acceptor of connections:
/// one task that starts a task per connection.
pub mod service_server_accept_connections_as_tasks;

/// Gets the listening socket of the commit service.
pub mod service_server_socket_binding;

/// The service as a mode of the one executable:
/// the words that select it and the start of the service.
pub mod service_main;

/// The wrapper's side of a commit through the commit service.
pub mod service_client;

/// The words a waiting command is told about its own wait.
pub mod service_client_waiting_words;

/// The client's one-way request to a running service to compile a repository's rules.
pub mod service_warm_request;

/// Connecting to the service,
/// reading one line from it,
/// and asking an older service to retire.
pub mod service_client_listen_and_ask_to_retire;

/// The settings of one client run and the watch on the service manager's answer.
pub mod service_client_settings_and_watches;

/// Reaching the commit service,
/// starting it when nothing answers.
pub mod service_client_reach_service;

/// What provisioning knows about the machine and the manager it asks,
/// the earlier release's unit names and what a refused call is to the person.
pub mod service_provision_host_and_manager;

/// The retirement of the single service of the earlier releases.
pub mod service_provision_earlier_service_retirement;

/// The wrapper's own provisioning of the commit service's socket activation.
pub mod service_provision;

/// The states in which provisioning already knows no service can answer this command,
/// so the command says why instead of waiting out the start ceiling.
pub mod service_provision_refusals_before_the_wait;

/// The record of the unit texts the service manager was last told about,
/// which tells an installation it runs from files a kill left behind.
pub mod service_provision_loaded_unit_texts;

/// Checks and writes on the unit files of the commit service.
pub mod service_provision_unit_file_ownership_and_writes;

/// A minimal D-Bus client for the user session bus,
/// for the provisioning's two calls.
pub mod systemd_dbus;

/// The D-Bus wire format of the user manager client:
/// calls written,
/// replies read.
pub mod systemd_dbus_wire_format;

/// Which commit service instance belongs to a worktree:
/// its name,
/// socket and unit instances.
pub mod service_instance;

/// The files an instance of the commit service keeps between runs:
/// the note beside its socket,
/// the note and socket file that say it has units,
/// and the journal that remembers its batches.
pub mod service_files_of_a_stopped_instance;

/// The commit service units of worktrees that no longer exist:
/// which worktree a unit belongs to,
/// whether it is gone,
/// and the stop of its units through the manager.
pub mod service_units_of_removed_worktrees;

/// The journal folder the unit gives an instance,
/// and the state folder that holds it.
pub mod service_journal_folder_of_an_instance;

/// A manager that records the stops it is asked for,
/// for the controls of the units of removed worktrees.
#[cfg(test)]
pub mod service_unit_manager_recorder_test_support;

/// One disposable machine for the controls of the units of removed worktrees and of the files an instance keeps.
#[cfg(test)]
pub mod service_units_fixture_test_support;

/// One entry from the wrapper a person ran to the journal,
/// sent without waiting.
#[cfg(target_os = "linux")]
pub mod client_journal;

/// The service's handler for a review work item,
/// bounded by a call deadline and a total budget.
pub mod service_review;

/// The standards-review credential:
/// redacted when printed,
/// read from the wrapper's environment,
/// sent in the work message.
pub mod service_review_key;
