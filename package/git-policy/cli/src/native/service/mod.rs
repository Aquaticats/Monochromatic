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

/// The collection of the repository scratch directory a killed command left,
/// beside the landing loop.
pub mod service_repository_scratch_sweep;

/// The commit scratch trees the requests this service holds still name,
/// which a collection is not to remove by age.
pub mod service_scratch_trees_of_queued_requests;

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

/// How old a finished batch's record is,
/// read from the clocks the record itself carries.
pub mod service_journal_record_age;

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

/// Starts a worktree's commit service when nothing answers at its address,
/// one command at a time.
pub mod service_server_start_by_the_wrapper;

/// The enhancement: a transient scope through a service manager that is this account's own,
/// and the base's start whenever it is absent, slow or failing.
pub mod service_server_start_through_the_manager;

/// Starts the service's program as a process that outlives the command and the terminal.
pub mod service_server_start_spawns_a_detached_process;

/// The note that says which process is a worktree's commit service,
/// and how a reused process number is told apart from it.
pub mod service_server_pid_note_of_an_instance;

/// The file the service's output goes to,
/// its cap and its truncation at each start.
pub mod service_server_log_file_of_an_instance;

/// The socket folder and socket file,
/// enterable and openable by this account alone.
pub mod service_socket_directory_only_this_account_may_open;

/// The address and the note a running service owns,
/// made at its start and removed as it ends.
pub mod service_server_files_of_a_running_instance;

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

/// How the service's own runtime was built,
/// and the words a task that cannot get a worker ends the service with.
pub mod service_runtime_build_report;

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

/// The signals a waiting commit answers by withdrawing its request,
/// and the flag they set for the wait.
pub mod service_client_interrupted;

/// Why a request a commit sent is given up,
/// and the failure that says what became of the commit.
pub mod service_client_why_withdrawn;

/// Connecting to the service,
/// reading one line from it,
/// and asking an older service to retire.
pub mod service_client_listen_and_ask_to_retire;

/// The settings of one client run,
/// and what starts the server when nothing answers.
pub mod service_client_settings_and_watches;

/// Reaching the commit service,
/// starting it when nothing answers.
pub mod service_client_reach_service;

/// Which commit service instance belongs to a worktree:
/// its name,
/// its socket and its journal folder.
pub mod service_instance;

/// The files an instance of the commit service keeps between runs:
/// the note beside its socket that names its worktree,
/// and the journal that remembers its batches.
pub mod service_files_of_a_stopped_instance;

/// What one command knows about a worktree's service:
/// its folders,
/// its socket,
/// its journal and its program.
pub mod service_host_of_a_worktree;

/// Makes the worktree's server exist when a commit first needs it.
pub mod service_commit_starts_the_server;

/// The commit service units of worktrees that no longer exist:
/// which worktree a unit belongs to,
/// whether it is gone,
/// and the stop of its units through the manager.
pub mod service_units_of_removed_worktrees;

/// The pushes a service owes a repository that no longer exists:
/// the retry plan of each branch whose Git directory is proven gone ends.
pub mod service_plans_of_a_gone_repository;

/// The journal folder the unit gives an instance,
/// and the state folder that holds it.
pub mod service_journal_folder_of_an_instance;

/// Whether a stopped service's processes are gone,
/// which is what lets the journal it writes be removed.
pub mod service_is_gone_after_a_stop;

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
