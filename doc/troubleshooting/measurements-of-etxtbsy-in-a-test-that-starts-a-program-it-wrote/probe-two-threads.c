/*
 * What:     A probe of `ETXTBSY` across threads: one thread holds a program file open for
 *           writing while another thread starts programs of its own, and a child forked
 *           right after the close starts the file.
 * Why:      The threaded test run failed rarely, so this probe measures the mechanism in a
 *           controlled setting: whether a child forked by another thread still holds the
 *           write descriptor when the file is started, and how often that answers
 *           `ETXTBSY`.
 *
 * In TS you'd write (pseudocode):
 * ```ts
 * for (const attempt of range(20)) {
 *   const fd = open(program, 'w');
 *   const spawner = thread(() => { for (...) spawnSync('/bin/true'); });
 *   await sleep(50);
 *   close(fd);
 *   const answer = spawnSync(program);
 *   counts[answer.kind]++;
 *   await spawner.join();
 * }
 * ```
 */

/* What:     `_GNU_SOURCE` turns on the GNU declarations of the C library. */
/* Why:      `posix_spawn` and `environ` come from them. */
#define _GNU_SOURCE

/* What:     `#include <spawn.h>` declares `posix_spawn`, which is what a Rust
 *           `Command::output()` uses when it has no `pre_exec`. */
/* Why:      The probe must fork the way the test binary does. */
#include <spawn.h>

/* What:     `#include <pthread.h>` declares `pthread_create` and `pthread_join`. */
/* Why:      The holder and the spawner must be two threads of one process. */
#include <pthread.h>

/* What:     `#include <fcntl.h>`, `<unistd.h>`, `<sys/wait.h>` and `<sys/stat.h>` declare
 *           the file and process calls the probe makes. */
/* Why:      It writes a program, starts it, and reaps what it started. */
#include <fcntl.h>
#include <sys/stat.h>
#include <sys/wait.h>
#include <unistd.h>

/* What:     `#include <stdio.h>` and `<string.h>` print the counts. */
/* Why:      The result is one line per arm. */
#include <stdio.h>
#include <string.h>

/* What:     `#include <errno.h>` declares `errno` and `ETXTBSY`. */
/* Why:      The refused start is read from it. */
#include <errno.h>

/* What:     `#include <time.h>` declares `nanosleep`. */
/* Why:      The window is a length of time. */
#include <time.h>

/* What:     The program the probe writes and starts. */
/* Why:      The start must succeed when nothing holds the file. */
static const char *PROGRAM_TEXT = "#!/bin/sh\nprintf 'started\\n'\nexit 0\n";

/* What:     The path both threads use,
 *           taken from the first argument or from `threaded-probe-program` beside the run. */
/* Why:      One inode is what the kernel's write count belongs to,
 *           and nothing in this file may name a directory of the run that measured it. */
static const char *PROGRAM_PATH = "threaded-probe-program";

/* What:     How long the spawner keeps forking inside the holder's window. */
/* Why:      Long enough for a few dozen forks to overlap the close. */
static const long WINDOW_MICROSECONDS = 50000;

/* What:     How many attempts the probe makes. */
/* Why:      The failure is a race, so one attempt says nothing. */
static const int ATTEMPTS = 20;

/* What:     A flag the spawner reads, set by the main thread when the window is over. */
/* Why:      It bounds the spawner's loop without a timeout. */
static volatile int stop_spawning = 0;

/* What:     `sleep_microseconds(us)` waits without ending early. */
/* Why:      `usleep` returns early when a signal arrives, and the window must be exact. */
static void sleep_microseconds(long microseconds) {
    struct timespec length;
    length.tv_sec = microseconds / 1000000;
    length.tv_nsec = (microseconds % 1000000) * 1000;
    nanosleep(&length, NULL);
}

/* What:     `spawner()` starts `/bin/true` in a loop the way a test binary starts its
 *           fixtures, until the main thread sets the flag. */
/* Why:      Each start is a fork whose child holds whatever this process held open at that
 *           moment, which is the descriptor the main thread is about to close. */
static void *spawner(void *ignored) {
    (void)ignored;
    char *const arguments[] = {(char *)"/bin/true", NULL};
    while (stop_spawning == 0) {
        pid_t child = 0;
        int started = posix_spawn(&child, "/bin/true", NULL, NULL, arguments, environ);
        if (started != 0) {
            fprintf(stderr, "spawn failed: %s\n", strerror(started));
            return NULL;
        }
        int status = 0;
        waitpid(child, &status, 0);
    }
    return NULL;
}

/* What:     `start_through_a_child()` starts the program in a child forked now and answers
 *           what the start said. */
/* Why:      The child is forked after the close, so the only holder left is one another
 *           thread's fork made. */
static int start_through_a_child(void) {
    pid_t child = fork();
    if (child == 0) {
        char *const arguments[] = {(char *)PROGRAM_PATH, NULL};
        execv(PROGRAM_PATH, arguments);
        _exit(errno == ETXTBSY ? 26 : 27);
    }
    int status = 0;
    waitpid(child, &status, 0);
    if (WIFEXITED(status) && WEXITSTATUS(status) == 0) {
        return 0;
    }
    return WEXITSTATUS(status);
}

int main(int argc, char *argv[]) {
    if (argc > 1) {
        PROGRAM_PATH = argv[1];
    }
    FILE *program = fopen(PROGRAM_PATH, "w");
    if (program == NULL) {
        perror("write the program");
        return 2;
    }
    fputs(PROGRAM_TEXT, program);
    fclose(program);
    chmod(PROGRAM_PATH, 0755);

    int busy = 0;
    int started = 0;
    for (int attempt = 0; attempt < ATTEMPTS; attempt++) {
        int held = open(PROGRAM_PATH, O_WRONLY | O_CLOEXEC);
        if (held < 0) {
            perror("open for writing");
            return 2;
        }
        stop_spawning = 0;
        pthread_t other;
        pthread_create(&other, NULL, spawner, NULL);
        sleep_microseconds(WINDOW_MICROSECONDS);
        /* The close, and at once the start: nothing in this process holds the file now,
         * and only a child another thread forked a moment ago can. */
        close(held);
        int answer = start_through_a_child();
        if (answer == 26) {
            busy++;
        } else if (answer == 0) {
            started++;
        } else {
            fprintf(stderr, "attempt %d answered unexpectedly: %d\n", attempt, answer);
        }
        stop_spawning = 1;
        pthread_join(other, NULL);
    }
    fprintf(stderr, "attempts=%d started=%d refused-with-ETXTBSY=%d\n", ATTEMPTS, started, busy);
    unlink(PROGRAM_PATH);
    return 0;
}
