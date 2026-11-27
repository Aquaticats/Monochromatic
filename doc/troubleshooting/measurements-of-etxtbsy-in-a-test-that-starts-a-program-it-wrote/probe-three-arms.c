/*
 * What:     A probe of `ETXTBSY`: open a program file for writing, then start it in
 *           two shapes and report what the start answered.
 * Why:      The threaded test runs failed only once in many, so before reading anything
 *           into that, this probe shows which descriptor holder actually makes a start
 *           fail on this machine.
 *
 * In TS you'd write (pseudocode):
 * ```ts
 * // arm 1: a forked child that never starts its own program holds the write descriptor
 * // arm 2: the same child starts a program, which closes the inherited descriptor
 * // arm 3: the parent closes the descriptor before it starts the program
 * ```
 */

/* What:     `_GNU_SOURCE` turns on the GNU declarations, among them `O_CLOEXEC`. */
/* Why:      The probe compares a descriptor that survives an exec with one that does not. */
#define _GNU_SOURCE

/* What:     `#include <fcntl.h>` declares `open` and the write flag. */
/* Why:      The write descriptor is the whole subject of the probe. */
#include <fcntl.h>

/* What:     `#include <stdio.h>` declares `perror` and `fprintf`. */
/* Why:      Each arm prints what the start answered. */
#include <stdio.h>

/* What:     `#include <stdlib.h>` declares `exit`. */
/* Why:      The probe ends after the first arm that starts the program. */
#include <stdlib.h>

/* What:     `#include <string.h>` declares `strerror`. */
/* Why:      An arm that fails prints the system's own reason. */
#include <string.h>

/* What:     `#include <sys/wait.h>` declares `waitpid`. */
/* Why:      An arm that forks a child reaps it. */
#include <sys/wait.h>

/* What:     `#include <unistd.h>` declares `fork`, `execv`, `sleep` and `unlink`. */
/* Why:      The three arms fork, sleep, start and clean up. */
#include <unistd.h>

/* What:     `#include <sys/stat.h>` declares `chmod`, which sets the program's mode
 *           without opening it. */
/* Why:      The start needs the execute bits. */
#include <sys/stat.h>

/* What:     `#include <errno.h>` declares `errno`. */
/* Why:      The start's own answer is read from `errno`. */
#include <errno.h>

/* What:     The program every arm writes and starts: a shell script that prints one
 *           line. */
/* Why:      It is the smallest thing the kernel will start. */
static const char *PROGRAM = "#!/bin/sh\nprintf 'started\\n'\n";

/* What:     The path the program is written at,
 *           taken from the first argument or from `probe-program` beside the run. */
/* Why:      Every arm must start the same file,
 *           and nothing in this file may name a directory of the run that measured it. */
/* In TS you'd write (pseudocode): */
/* ```ts */
/* const path = process.argv[2] ?? 'probe-program'; */
/* ``` */
static const char *path = "probe-program";

/* What:     `write_the_program()` creates the program file through this process's
 *           own descriptor, closes it, and marks it executable. */
/* Why:      Every arm starts from the same file, so only the holder differs. */
static void write_the_program(void) {
    /* What:      `open` with `O_CREAT|O_TRUNC|O_WRONLY` creates or empties the file for
     *            writing. */
    /* Why:       This is the descriptor whose inheritance the probe is about. */
    int fd = open(path, O_CREAT | O_TRUNC | O_WRONLY | O_CLOEXEC, 0644);
    if (fd < 0) {
        perror("open");
        exit(2);
    }
    /* What:      `write` copies the program text into the file. */
    /* Why:       An empty file cannot be started. */
    size_t length = strlen(PROGRAM);
    if (write(fd, PROGRAM, length) != (ssize_t)length) {
        perror("write");
        exit(2);
    }
    close(fd);
    if (chmod(path, 0755) != 0) {
        perror("chmod");
        exit(2);
    }
}

/* What:     `start_it()` runs the program in a child and prints the answer. */
/* Why:      `execv`'s answer is `ETXTBSY` when some process holds the file open for
 *           writing, which is the reading the probe has to confirm. */
static void start_it(void) {
    pid_t child = fork();
    if (child == 0) {
        char *const arguments[] = {(char *)path, NULL};
        execv(path, arguments);
        fprintf(stderr, "start answered: %s (errno %d)\n", strerror(errno), errno);
        _exit(errno == ETXTBSY ? 26 : 27);
    }
    int status = 0;
    waitpid(child, &status, 0);
    if (WIFEXITED(status) && WEXITSTATUS(status) == 0) {
        fprintf(stderr, "start succeeded\n");
        return;
    }
    fprintf(stderr, "start failed with status %d\n", WEXITSTATUS(status));
}

/* What:     `main` runs the three arms and prints one line each. */
/* Why:      The three lines together say whether an inherited write descriptor is what
 *           makes a start fail. */
/* In TS you'd write (pseudocode): */
/* ```ts */
/* const path = process.argv[2] ?? 'probe-program'; */
/* ``` */
int main(int argc, char *argv[]) {
    if (argc > 1) {
        path = argv[1];
    }
    write_the_program();

    /* Arm 1: the descriptor is open here, a forked child holds the same descriptor and
     * never starts a program of its own. */
    int fd = open(path, O_WRONLY | O_CLOEXEC);
    if (fd < 0) {
        perror("open for arm 1");
        return 2;
    }
    pid_t holder = fork();
    if (holder == 0) {
        /* The child keeps the inherited descriptor and starts nothing. */
        sleep(1);
        _exit(0);
    }
    sleep(0);
    fprintf(stderr, "arm 1 (a forked child that starts nothing holds it): ");
    start_it();
    waitpid(holder, NULL, 0);
    close(fd);

    /* Arm 2: the same child, but it starts its own program at once, which closes the
     * inherited descriptor because it was opened close-on-exec. */
    fd = open(path, O_WRONLY | O_CLOEXEC);
    if (fd < 0) {
        perror("open for arm 2");
        return 2;
    }
    holder = fork();
    if (holder == 0) {
        char *const arguments[] = {"/bin/true", NULL};
        execv("/bin/true", arguments);
        _exit(3);
    }
    waitpid(holder, NULL, 0);
    fprintf(stderr, "arm 2 (this process alone holds it): ");
    start_it();
    close(fd);

    /* Arm 3: no holder at all. */
    fprintf(stderr, "arm 3 (nothing holds it): ");
    start_it();
    unlink(path);
    return 0;
}
