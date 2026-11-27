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
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `earlier_unix_sockets_that_listen.rs`:
//!   whether a socket of an earlier release's unit listens,
//!   read from the table of unix sockets.
//! - `journald_native_socket_target.rs`:
//!   the address native journal entries are sent to.
//! - `journald_native_the_bounds_of_one_entry.rs`:
//!   the most characters of a message,
//!   of a field value and of a field name.
//! - `journald_native_the_words_of_one_entry.rs`:
//!   how serious an entry is,
//!   and the text of one entry cut and made one line.
//! - `journald_native_the_fields_of_one_entry.rs`:
//!   the fields of one entry,
//!   written as the protocol carries them.
//! - `journald_native_where_an_entry_goes.rs`:
//!   whether standard error is the journal,
//!   and the address a datagram is sent to.
//! - `journald_native_the_sink.rs`:
//!   where the entries of one process go,
//!   decided once.
//! - `journald_native_the_stream_writer.rs`:
//!   a write to the stream that never waits for room.
//! - `journald_native.rs`:
//!   the native journal protocol and the standard error fallback the service's entries go through.
//! - `client_journal.rs`:
//!   one entry from the wrapper a person ran to the journal,
//!   sent without waiting.
//! - `service_busy_answers_and_name_checks.rs`:
//!   the answers the landing board gives and the checks on the names it keeps.
//! - `service_client_interrupted.rs`:
//!   the signals a waiting commit answers by withdrawing its request,
//!   and the flag they set for the wait.
//! - `service_client_why_withdrawn.rs`:
//!   why a request a commit sent is given up,
//!   and the failure that says what became of the commit.
//! - `service_fault.rs`:
//!   test-only fault points of the commit service.
//! - `service_journal_folder_of_an_instance.rs`:
//!   the journal folder the unit gives an instance,
//!   and the state folder that holds it.
//! - `service_journal_restored_at_start.rs`:
//!   the plan of the pushes waiting for a new try and the repositories an earlier service was woken for,
//!   read at a service's start on a thread of its own.
//! - `service_log_lines_off_the_calling_thread.rs`:
//!   the service's log lines on standard error,
//!   written by a thread of their own so no landing waits for the stream.
//! - `service_plans_of_a_gone_repository.rs`:
//!   the pushes a service owes a repository that no longer exists:
//!   the retry plan of each branch whose Git directory is proven gone ends.
//! - `service_provision_slow_manager_notice.rs`:
//!   the line a commit prints when the service manager has not answered its provisioning within a moment.
//! - `service_provision_unit_file_ownership_and_writes.rs`:
//!   checks and writes on the unit files of the commit service.
//! - `service_push_programs_held_in_memory.rs`:
//!   the program search list and the transport settings of the command that asked for a push,
//!   held only in the memory of the service that runs it.
//! - `service_protocol_programs_of_a_request.rs`:
//!   the programs and transport settings one request or wake carries,
//!   read from the socket.
//! - `service_queue_received_on_the_runtime.rs`:
//!   the wait of a thread on a queue the runtime owns:
//!   an item,
//!   the end of the wait at its bound,
//!   or the end of the queue.
//! - `service_repository_scratch_sweep.rs`:
//!   the collection of the repository scratch directory a killed command left,
//!   beside the landing loop.
//! - `service_review_key.rs`:
//!   the standards-review credential:
//!   redacted when printed,
//!   read from the wrapper's environment,
//!   sent in the work message.
//! - `service_protocol_messages_and_name_checks.rs`:
//!   the protocol messages as typed values,
//!   and the checks on the names they carry.
//! - `service_protocol_decode_the_words_of_a_refused_line.rs`:
//!   the sentence a protocol line that cannot be read is refused with.
//! - `service_protocol_decode_service_answers.rs`:
//!   reads one line of the protocol as a service message,
//!   checking every number,
//!   name and figure of a landing.
//! - `service_protocol_decode_lines_to_messages.rs`:
//!   reads one line of the protocol as a client message or a service message,
//!   checking every field.
//! - `service_protocol_encode_the_fields_of_a_message.rs`:
//!   one field,
//!   one changed path,
//!   one sync,
//!   and the one object every message is written as.
//! - `service_protocol_encode_the_commit_line.rs`:
//!   the line for one commit request,
//!   and whether it fits.
//! - `service_protocol_encode_client_messages.rs`:
//!   a retire at a named version,
//!   and every message a client writes.
//! - `service_protocol_encode_service_messages.rs`:
//!   every message a service writes.
//! - `service_protocol_encode_messages_to_lines.rs`:
//!   writes protocol messages as one JSON object per line.
//! - `service_protocol.rs`:
//!   messages between the wrapper and the resident commit service.
//! - `service_journal_intent_text_and_parse.rs`:
//!   the text form of a batch intent:
//!   written and parsed.
//! - `service_protocol_forgetting_programs.rs`:
//!   reads protocol lines as messages,
//!   checking every field.
//!   The reader of the message that names the push records which can no longer run.
//! - `service_reply.rs`:
//!   the connection of one waiting client,
//!   answered by the landing thread and kept alive by a beat from the reading thread.
//! - `service_scratch_release_removal.rs`:
//!   the removal of a commit's scratch directory once the client that held it is gone,
//!   beside the landing loop.
//! - `service_scratch_trees_of_queued_requests.rs`:
//!   the commit scratch trees the requests this service holds still name,
//!   which a collection is not to remove by age.
//! - `service_server_socket_binding.rs`:
//!   gets the listening socket of the commit service.
//! - `service_shadow_pool_fill.rs`:
//!   the fill of the shadow pool a commit takes its private repository from,
//!   beside the landing loop.
//! - `service_shadow_draw.rs`:
//!   the service's stock of finished shadows drawn ahead of the commits that ask for one.
//! - `service_timing_records.rs`:
//!   what one request's timing is made of.
//! - `service_timing_wait_line.rs`:
//!   the line about the wait for the index lock,
//!   and the read of a step's duration from a line already written.
//! - `service_timing_answered_line.rs`:
//!   the one line each answered request gets.
//! - `service_timing_line_text.rs`:
//!   the prefix of every line,
//!   the wall clock it states,
//!   and the text of one line.
//! - `service_timing_life_lines.rs`:
//!   the lines about the service's own life.
//! - `service_timing_log_entry.rs`:
//!   writing one entry,
//!   and the three macros every call site uses.
//! - `service_timing.rs`:
//!   the lines the commit service writes about its life and about the time each request took,
//!   and the parts they are made of re-exported from here.
//! - `service_server_land_group_answer_log.rs`:
//!   what one batch's answer says about that batch:
//!   the times it shares,
//!   its figures,
//!   and the log line of each answer.
//! - `service_server_queue_types_and_send.rs`:
//!   the shapes the connection threads and the landing thread pass between them,
//!   and the one-message send.
//! - `service_signal.rs`:
//!   the line the commit service writes when a signal ends it.
//! - `service_warm_request_control_window.rs`:
//!   the answers service warm request control window holds.
//! - `systemd_dbus_wire_format.rs`:
//!   the D-Bus wire format of the user manager client:
//!   calls written,
//!   replies read.
//! - `systemd_dbus_manager_calls_and_error_names.rs`:
//!   the names the user manager is reached by on the session bus,
//!   and the error names a bus or the manager answers with.
//! - `systemd_dbus_failures.rs`:
//!   the failures of a bus exchange as the call's own error.
//! - `systemd_dbus_session_address.rs`:
//!   where the session bus of this user listens.
//! - `systemd_dbus.rs`:
//!   a minimal D-Bus client for the user session bus,
//!   for the provisioning's two calls.
//! - `service_provision_host_and_manager.rs`:
//!   what provisioning knows about the machine and the manager it asks,
//!   the earlier release's unit names and what a refused call is to the person.
//! - `service_files_of_a_stopped_instance_note_beside_socket.rs`:
//!   the note an instance's units leave beside their socket,
//!   and the two functions that write it.
//! - `service_instance.rs`:
//!   which commit service instance belongs to a worktree:
//!   its name,
//!   socket and unit instances.
//! - `service_own_socket_of_a_retire_that_served_nothing.rs`:
//!   the socket a service stops when it was retired before it served anything.
//! - `wait_ceilings.rs`:
//!   the longest the wrapper waits for each thing it waits for,
//!   with the measurement beside each number.
//! - `service_is_gone_after_a_stop.rs`:
//!   whether a stopped service's processes are gone,
//!   which is what lets the journal it writes be removed.
//! - `service_journal_record_age.rs`:
//!   how old a finished batch's record is,
//!   read from the clocks the record itself carries.
//! - `service_journal_file_names.rs`:
//!   what the files of the commit service's journal are called.
//! - `service_journal_records_read_from_the_journal.rs`:
//!   what the records in the journal say,
//!   and which of them a journal may forget.
//! - `service_journal.rs`:
//!   the commit service's journal and its restart decisions.
//! - `service_busy.rs`:
//!   what the one landing thread is doing,
//!   shared with the threads that talk to waiting clients.
//! - `service_busy_words_of_a_stuck_write.rs`:
//!   the words a waiting client reads about the write the landing thread is inside.
//! - `service_busy_withdrawals_beside_a_stuck_write.rs`:
//!   a withdrawal answered on the connection thread while the landing thread is stuck,
//!   and the ceiling that bounds what the record remembers.
//! - `service_busy_step_record_on_this_thread.rs`:
//!   the step record a thread keeps while it is inside a write that is never ended.
//! - `service_files_of_a_stopped_instance_journal_entries.rs`:
//!   the entries of a stopped instance's journal,
//!   and what each name in it says about keeping the journal.
//! - `service_files_of_a_stopped_instance.rs`:
//!   the files an instance of the commit service keeps between runs:
//!   the note beside its socket,
//!   the note and socket file that say it has units,
//!   and the journal that remembers its batches.
//! - `service_provision_loaded_unit_texts.rs`:
//!   the record of the unit texts the service manager last read for one instance,
//!   which tells an installation it runs from files a kill left behind.
//! - `service_review.rs`:
//!   the service's handler for a review work item,
//!   bounded by a call deadline and a total budget.
//! - `service_runtime_build_report.rs`:
//!   how the service's own runtime was built,
//!   and the words a task that cannot get a worker ends the service with.
//! - `service_server_connection_handling_what_every_connection_shares.rs`:
//!   the queues,
//!   the board and the drawn shadows one service run hands every connection.
//! - `service_server_connection_handling_the_note_of_a_stuck_landing.rs`:
//!   the note a beat sends about what the landing is stuck in.
//! - `service_server_connection_handling_whose_message_it_is.rs`:
//!   whether a message is this instance's own,
//!   and the words for the ones it is not.
//! - `service_server_connection_handling_the_wait_of_a_commit.rs`:
//!   a queued commit answered with beats,
//!   and a withdrawal answered the same way.
//! - `service_server_connection_handling_one_message.rs`:
//!   what one decoded line of a connection comes to.
//! - `service_server_connection_handling.rs`:
//!   handles one client connection of the commit service.
//! - `service_server_accept_connections_as_tasks.rs`:
//!   the acceptor of connections:
//!   one task that starts a task per connection.
//! - `service_server_land_group_owed_pushes.rs`:
//!   what a batch of landings owes,
//!   and takes before its branch moves.
//! - `service_server_land_group_the_batches_left_open.rs`:
//!   the batches an earlier service left open,
//!   settled before this one lands.
//! - `service_server_land_group_the_words_a_client_waits_with.rs`:
//!   what the landing waits for,
//!   and the place each request of the batch has.
//! - `service_server_land_group_the_answers_to_a_batch.rs`:
//!   what each request of a landed batch came to,
//!   and the note that names the repository to the next service.
//! - `service_server_land_group.rs`:
//!   lands one group of queued requests as a batch and answers each client.
//! - `service_server_shutdown_after_a_stop.rs`:
//!   the service's own shutdown after a stop:
//!   it tells the waiters,
//!   waits a bounded time for the Gits and the landing in flight,
//!   and ends the process by the signal.
//! - `service_server_work_loop_awaiting_storage_steps.rs`:
//!   the storage steps the after-commit loop makes as jobs of the blocking pool:
//!   the notes a restart looks at and the plan of the pushes a service owes.
//! - `service_server_work_loop_awaiting.rs`:
//!   the same loop as a task of the runtime,
//!   with the two storage steps only it makes.
//! - `service_server_work_loop.rs`:
//!   the loop of the commit service's after-commit work asked by a thread outside the runtime.
//! - `service_unit_folders_and_the_worktree_of_an_instance.rs`:
//!   the two folders every question about an instance's units is asked of,
//!   which worktree a unit instance belongs to,
//!   and whether that worktree is certainly gone.
//! - `service_instances_with_units_and_their_survey.rs`:
//!   which instances have a unit or a journal,
//!   and the survey that sorts them by whether their worktree is gone.
//! - `service_own_worktree_gone_and_the_stop_of_its_socket.rs`:
//!   what a service knows about its own worktree,
//!   and the stop of its own socket when that worktree is gone.
//! - `service_units_of_removed_worktrees.rs`:
//!   the commit service units of worktrees that no longer exist:
//!   which worktree a unit belongs to,
//!   whether it is gone,
//!   and the stop of its units through the manager.
//! - `service_server_batch_and_idle_limits.rs`:
//!   the numbers one service run is held to:
//!   the batch cap,
//!   the idle exit,
//!   and how often the run asks whether its own worktree is still there.
//! - `service_server_serve_settings.rs`:
//!   what one service run is configured with.
//! - `service_server_scratch_trees_of_a_group.rs`:
//!   the scratch trees one group's requests name,
//!   and their removal once the landing that read them is over.
//! - `service_server_batch_read.rs`:
//!   one arrived batch of requests read:
//!   its withdrawals answered,
//!   its commits grouped by branch.
//! - `service_server_after_commit_work_start.rs`:
//!   the start of the after-commit work,
//!   and the end of the service when no thread can be had for it.
//! - `service_server_landing_one_group.rs`:
//!   one group of a batch,
//!   landed and answered.
//! - `service_server_own_worktree_question.rs`:
//!   the question the serving loop asks about the worktree whose units are its own,
//!   asked before the loop takes the next batch.
//! - `service_server.rs`:
//!   the commit service's loop,
//!   and the parts it is made of re-exported so every caller keeps naming them through it.
//! - `service_address_of_a_worktree.rs`:
//!   the address one worktree's service listens at:
//!   a socket file under the runtime directory when that path fits one socket address,
//!   and an abstract name when it does not.
//! - `service_client_connect_within_a_bound.rs`:
//!   the connect itself,
//!   waited for with a bound because a socket whose queue of unaccepted connections is full takes no connection until
//!   something accepts.
//! - `service_client_listen_and_ask_to_retire.rs`:
//!   connecting to the service,
//!   reading one line from it,
//!   writing a request within a bound,
//!   and asking an older service to retire.
//! - `service_client_shadow_hand_out.rs`:
//!   the question a commit asks its service for the finished shadow drawn ahead of it.
//! - `service_main.rs`:
//!   the service as a mode of the one executable:
//!   the words that select it and the start of the service.
//! - `service_provision_earlier_service_retirement.rs`:
//!   the retirement of the single service of the earlier releases.
//! - `service_provision_instance_unloaded.rs`:
//!   dropping the service manager's copy of one worktree's units,
//!   so the next start reads the unit files as they are.
//! - `service_provision_refusals_before_the_wait.rs`:
//!   the states in which provisioning already knows no service can answer this command,
//!   so the command says why instead of waiting out the start ceiling.
//! - `service_provision_the_unit_texts.rs`:
//!   the two unit files for one host,
//!   with every placeholder filled in.
//! - `service_provision_the_links_the_units_run.rs`:
//!   the links the unit files run,
//!   and the one write both come to.
//! - `service_provision_what_is_already_in_place.rs`:
//!   whether there is anything left to do,
//!   read without touching the session bus.
//! - `service_provision_the_calls_to_the_manager.rs`:
//!   the reset,
//!   the start,
//!   and the stop of the earlier release's socket.
//! - `service_provision.rs`:
//!   the wrapper's own provisioning of the commit service's socket activation.
//! - `service_client_settings_and_watches.rs`:
//!   the settings of one client run and the watch on the service manager's answer.
//! - `service_client_reach_service_the_waits_own_steps.rs`:
//!   the wait for a service that speaks this release:
//!   the start it asks the manager for,
//!   what it says,
//!   and how it ends.
//! - `service_client_reach_service.rs`:
//!   reaching the commit service,
//!   starting it when nothing answers.
//! - `service_client_waiting_words.rs`:
//!   the words a waiting command is told about its own wait.
//! - `service_client_wake.rs`:
//!   the client's one-way message that work items were written for a repository.
//! - `service_client_what_a_request_came_to.rs`:
//!   what a commit request came to:
//!   landed,
//!   kept,
//!   prepared again,
//!   or not made.
//! - `service_client_the_failure_a_person_reads.rs`:
//!   the plain failure a person reads for each code the service answers with.
//! - `service_client_withdraw_a_request.rs`:
//!   asks the service to drop a request this command sent,
//!   and turns its answer into the command's end.
//! - `service_client_scratch_renewal.rs`:
//!   the renewal of the command's scratch directory while a commit waits.
//! - `service_client_wait_for_the_answer.rs`:
//!   the wait for the answer to a request that was written,
//!   with what it remembers between two messages.
//! - `service_client_send_and_wait.rs`:
//!   the sends,
//!   the one identity each carries,
//!   and the tries a service that went silent or closed its connection is given.
//! - `service_client.rs`:
//!   the wrapper's side of a commit through the commit service.
//! - `service_warm_request.rs`:
//!   the client's one-way request to a running service to compile a repository's rules.
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

/// The service's stock of finished shadows drawn ahead of the commits that ask for one.
pub mod service_shadow_draw;

/// The question a commit asks its service for the finished shadow drawn ahead of it.
pub mod service_client_shadow_hand_out;

/// Whether a socket of an earlier release's unit listens,
/// read from the table of unix sockets.
pub mod earlier_unix_sockets_that_listen;

/// Messages between the wrapper and the resident commit service.
pub mod service_protocol;

/// The protocol messages as typed values,
/// and the checks on the names they carry.
pub mod service_protocol_messages_and_name_checks;

/// The fields one protocol message is written from.
pub mod service_protocol_encode_the_fields_of_a_message;

/// The line for one commit request,
/// and whether it fits.
pub mod service_protocol_encode_the_commit_line;

/// A retire at a named version,
/// and every message a client writes.
pub mod service_protocol_encode_client_messages;

/// Every message a service writes.
pub mod service_protocol_encode_service_messages;

/// Writes protocol messages as one JSON object per line.
pub mod service_protocol_encode_messages_to_lines;

/// Reads protocol lines as messages,
/// checking every field.
/// The reader of the message that names the push records which can no longer run.
pub mod service_protocol_forgetting_programs;

/// The sentence a protocol line that cannot be read is refused with.
pub mod service_protocol_decode_the_words_of_a_refused_line;

/// Reads one line of the protocol as a service message,
/// checking every number,
/// name and figure of a landing.
pub mod service_protocol_decode_service_answers;

pub mod service_protocol_decode_lines_to_messages;

/// The longest the wrapper waits for each thing it waits for,
/// with the measurement beside each number.
pub mod wait_ceilings;

/// What the one landing thread is doing,
/// shared with the threads that talk to waiting clients.
pub mod service_busy;

/// The words a waiting client reads about the write the landing thread is inside.
pub mod service_busy_words_of_a_stuck_write;

/// A withdrawal answered on the connection thread while the landing thread is stuck,
/// and the ceiling that bounds what the record remembers.
pub mod service_busy_withdrawals_beside_a_stuck_write;

/// The answers the landing board gives and the checks on the names it keeps.
pub mod service_busy_answers_and_name_checks;

/// The step record a thread keeps while it is inside a write that is never ended.
pub mod service_busy_step_record_on_this_thread;

/// The connection of one waiting client,
/// answered by the landing thread and kept alive by a beat from the reading thread.
pub mod service_reply;

/// What the files of the commit service's journal are called.
pub mod service_journal_file_names;

/// What the records in the journal say,
/// and which of them a journal may forget.
pub mod service_journal_records_read_from_the_journal;

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

/// The bounds one native journal entry is held to.
pub mod journald_native_the_bounds_of_one_entry;

/// How serious an entry is,
/// and the text of one entry cut and made one line.
pub mod journald_native_the_words_of_one_entry;

/// The fields of one entry,
/// written as the protocol carries them.
pub mod journald_native_the_fields_of_one_entry;

/// Whether standard error is the journal,
/// and the address a datagram is sent to.
pub mod journald_native_where_an_entry_goes;

/// Where the entries of one process go,
/// decided once.
pub mod journald_native_the_sink;

/// A write to the journal's stream that never waits for room.
pub mod journald_native_the_stream_writer;

/// The native journal protocol and the standard error fallback the service's entries go through.
pub mod journald_native;

/// The address native journal entries are sent to.
pub mod journald_native_socket_target;

/// The service's log lines on standard error,
/// written by a thread of their own so no landing waits for the stream.
pub mod service_log_lines_off_the_calling_thread;

/// What one request's timing is made of.
pub mod service_timing_records;

/// The line about the wait for the index lock,
/// and the read of a step's duration from a line already written.
pub mod service_timing_wait_line;

/// The one line each answered request gets.
pub mod service_timing_answered_line;

/// The prefix of every line,
/// the wall clock it states,
/// and the text of one line.
pub mod service_timing_line_text;

/// The lines about the service's own life.
pub mod service_timing_life_lines;

/// Writing one entry,
/// the sink it goes to,
/// and the three macros every call site uses.
pub mod service_timing_log_entry;

/// The lines the commit service writes about its life and about the time each request took,
/// and the parts they are made of re-exported from here.
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

/// The numbers one service run is held to:
/// the batch cap,
/// the batch whose cost still grows as its size does,
/// the idle exit,
/// and how often the run asks whether its own worktree is still there.
pub mod service_server_batch_and_idle_limits;

/// What one service run is configured with.
pub mod service_server_serve_settings;

/// The scratch trees one group's requests name,
/// and their removal once the landing that read them is over.
pub mod service_server_scratch_trees_of_a_group;

/// One arrived batch of requests read:
/// its withdrawals answered,
/// its commits grouped by branch.
pub mod service_server_batch_read;

/// The start of the after-commit work,
/// and the end of the service when no thread can be had for it.
pub mod service_server_after_commit_work_start;

/// One group of a batch,
/// landed and answered.
pub mod service_server_landing_one_group;

/// The commit service's loop.
pub mod service_server;

/// The question the serving loop asks about the worktree whose units are its own.
pub mod service_server_own_worktree_question;

/// The loop of the commit service's after-commit work asked by a thread outside the runtime.
pub mod service_server_work_loop;

/// The storage steps the after-commit loop makes as jobs of the blocking pool:
/// the notes a restart looks at and the plan of the pushes a service owes.
pub mod service_server_work_loop_awaiting_storage_steps;

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
/// The batches an earlier service left open,
/// settled before this one lands.
pub mod service_server_land_group_the_batches_left_open;

/// What the landing waits for,
/// and the place each request of the batch has.
pub mod service_server_land_group_the_words_a_client_waits_with;

/// What each request of a landed batch came to,
/// and the note that names the repository to the next service.
pub mod service_server_land_group_the_answers_to_a_batch;

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
/// What every connection of one service run shares:
/// the queues,
/// the board and the drawn shadows.
pub mod service_server_connection_handling_what_every_connection_shares;

/// The note a beat sends about what the landing is stuck in.
pub mod service_server_connection_handling_the_note_of_a_stuck_landing;

/// Whether a message is this instance's own,
/// and the words for the ones it is not.
pub mod service_server_connection_handling_whose_message_it_is;

/// A queued commit answered with beats,
/// and a withdrawal answered the same way.
pub mod service_server_connection_handling_the_wait_of_a_commit;

/// What one decoded line of a connection comes to.
pub mod service_server_connection_handling_one_message;

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

/// What a commit request came to:
/// landed,
/// kept,
/// prepared again,
/// or not made.
pub mod service_client_what_a_request_came_to;

/// The plain failure a person reads for each code the service answers with.
pub mod service_client_the_failure_a_person_reads;

/// Asks the service to drop a request this command sent,
/// and turns its answer into the command's end.
pub mod service_client_withdraw_a_request;

/// The renewal of the command's scratch directory while a commit waits.
pub mod service_client_scratch_renewal;

/// The wait for the answer to a request that was written,
/// with what it remembers between two messages.
pub mod service_client_wait_for_the_answer;

/// The sends,
/// the one identity each carries,
/// and the tries a service that went silent or closed its connection is given.
pub mod service_client_send_and_wait;

/// The wrapper's side of a commit through the commit service.
pub mod service_client;

/// The client's one-way message that work items were written for a repository.
pub mod service_client_wake;

/// The programs and transport settings one request or wake carries,
/// read from the socket.
pub mod service_protocol_programs_of_a_request;

/// The program search list and the transport settings of the command that asked for a push,
/// held only in the memory of the service that runs it.
pub mod service_push_programs_held_in_memory;

/// The words a waiting command is told about its own wait.
pub mod service_client_waiting_words;

/// The client's one-way request to a running service to compile a repository's rules.
pub mod service_warm_request;
/// The same re-export for the request's control point.
pub use service_warm_request::the_controls_window as service_warm_request_control_window;

/// The signals a waiting commit answers by withdrawing its request,
/// and the flag they set for the wait.
pub mod service_client_interrupted;

/// Why a request a commit sent is given up,
/// and the failure that says what became of the commit.
pub mod service_client_why_withdrawn;

/// Connecting to the service,
/// reading one line from it,
/// writing a request within a bound,
/// and asking an older service to retire.
pub mod service_client_listen_and_ask_to_retire;

/// The connect itself,
/// waited for with a bound because a socket whose queue of unaccepted connections is full takes no connection
/// until something accepts.
pub mod service_client_connect_within_a_bound;

/// The settings of one client run and the watch on the service manager's answer.
pub mod service_client_settings_and_watches;

/// The wait for a service that speaks this release:
/// the start it asks the manager for,
/// what it says,
/// and how it ends.
pub mod service_client_reach_service_the_waits_own_steps;

/// Reaching the commit service,
/// starting it when nothing answers.
pub mod service_client_reach_service;

/// What provisioning knows about the machine and the manager it asks,
/// the earlier release's unit names and what a refused call is to the person.
pub mod service_provision_host_and_manager;

/// The note an instance's units leave beside their socket,
/// and the two functions that write it.
pub mod service_files_of_a_stopped_instance_note_beside_socket;

/// The retirement of the single service of the earlier releases.
pub mod service_provision_earlier_service_retirement;

/// The two unit files for one host,
/// with every placeholder filled in.
pub mod service_provision_the_unit_texts;

/// The links the unit files run,
/// and the one write both come to.
pub mod service_provision_the_links_the_units_run;

/// Whether there is anything left to do,
/// read without touching the session bus.
pub mod service_provision_what_is_already_in_place;

/// The reset,
/// the start,
/// and the stop of the earlier release's socket.
pub mod service_provision_the_calls_to_the_manager;

/// The wrapper's own provisioning of the commit service's socket activation.
pub mod service_provision;

/// The socket a service stops when it was retired before it served anything.
pub mod service_own_socket_of_a_retire_that_served_nothing;

/// Dropping the service manager's copy of one worktree's units,
/// so the next start reads the unit files as they are.
pub mod service_provision_instance_unloaded;

/// The states in which provisioning already knows no service can answer this command,
/// so the command says why instead of waiting out the start ceiling.
pub mod service_provision_refusals_before_the_wait;

/// The record of the unit texts the service manager last read for one instance,
/// which tells an installation it runs from files a kill left behind.
pub mod service_provision_loaded_unit_texts;

/// The line a commit prints when the service manager has not answered its provisioning within a moment.
pub mod service_provision_slow_manager_notice;

/// Checks and writes on the unit files of the commit service.
pub mod service_provision_unit_file_ownership_and_writes;

/// The names the user manager is reached by on the session bus,
/// and the error names a bus or the manager answers with.
pub mod systemd_dbus_manager_calls_and_error_names;

/// The failures of a bus exchange as the call's own error.
pub mod systemd_dbus_failures;

/// Where the session bus of this user listens.
pub mod systemd_dbus_session_address;

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

/// The address one worktree's service listens at:
/// a socket file under the runtime directory when that path fits one socket address,
/// and an abstract name when it does not.
pub mod service_address_of_a_worktree;

/// The files an instance of the commit service keeps between runs:
/// the note beside its socket,
/// the note and socket file that say it has units,
/// and the journal that remembers its batches.
pub mod service_files_of_a_stopped_instance;

/// The entries of a stopped instance's journal,
/// and what each name in it says about keeping the journal.
pub mod service_files_of_a_stopped_instance_journal_entries;

/// The two folders every question about an instance's units is asked of,
/// which worktree a unit instance belongs to,
/// and whether that worktree is certainly gone.
pub mod service_unit_folders_and_the_worktree_of_an_instance;

/// Which instances have a unit or a journal,
/// and the survey that sorts them by whether their worktree is gone.
pub mod service_instances_with_units_and_their_survey;

/// What a service knows about its own worktree,
/// and the stop of its own socket when that worktree is gone.
pub mod service_own_worktree_gone_and_the_stop_of_its_socket;

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
