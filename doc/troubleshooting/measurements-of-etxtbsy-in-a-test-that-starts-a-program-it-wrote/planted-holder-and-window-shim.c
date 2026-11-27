/*
 * What:     A preloaded library (`LD_PRELOAD`) for test binaries, with three switches.
 *           `ETXTBSY_SHIM_HOLD_US` makes it plant the holder the real failure needs:
 *           when a threaded process is about to close a descriptor open for writing on a
 *           file that starts with an ELF header or a `#!` line, it forks a child that
 *           holds everything it inherits for that many microseconds and then leaves.
 *           `ETXTBSY_SHIM_DELAY_US` holds the descriptor itself open that long instead.
 *           `ETXTBSY_SHIM_PREFIX` limits both to paths under that prefix.
 * Why:      The window in which a program file is open for writing in a test process is a
 *           few microseconds wide, so a threaded run meets `ETXTBSY` only once in many
 *           runs, and holding the window open does not widen it either (the child's own
 *           hold is what the start has to overlap).
 *           A planted holder makes the failure certain, so one run shows which tests write
 *           a program in the process that starts it.
 * Gotcha:   It is a measurement instrument for test binaries, never something to run a real
 *           program under.
 *
 * Build and run, from a shell that can reach the test binary:
 * ```sh
 * gcc -shared -fPIC -O2 -o libetxtbsy_window.so planted-holder-and-window-shim.c -ldl
 * LD_PRELOAD=$PWD/libetxtbsy_window.so \
 *   ETXTBSY_SHIM_HOLD_US=300000 \
 *   ETXTBSY_SHIM_PREFIX=/path/to/the/build/directory/ \
 *   ./the-test-binary
 * ```
 *
 * In TS you'd write (pseudocode):
 * ```ts
 * interceptClose((fd) => {
 *   if (isWriteDescriptor(fd) && isProgram(pathOf(fd))) plantHolder();
 * });
 * ```
 */

/* What:     `_GNU_SOURCE` turns on GNU extensions in the C library headers. */
/* Why:      `RTLD_NEXT` and `dlsym` need it, and so does `gettid`. */
#define _GNU_SOURCE

/* What:     `#include <dlfcn.h>` pastes in the header that declares `dlsym`, the
 *           call that looks up the address of a function by name in a library. */
/* Why:      This file must call the real `close` of the C library, which it can only
 *           name through `dlsym`. */
#include <dlfcn.h>

/* What:     `#include <fcntl.h>` declares `fcntl` and its `F_GETFL` command, which
 *           asks a descriptor what mode it was opened with. */
/* Why:      Only a descriptor open for writing can make a start fail. */
#include <fcntl.h>

/* What:     `#include <unistd.h>` declares the POSIX system calls this file wraps:
 *           `close`, `readlink` and `syscall`. */
/* Why:      The wrapper replaces `close` and needs the other two. */
#include <unistd.h>

/* What:     `#include <stdlib.h>` declares `getenv`, the call that reads an
 *           environment variable. */
/* Why:      The prefix and the delay come from the environment. */
#include <stdlib.h>

/* What:     `#include <stdio.h>` declares `snprintf` and `fprintf`. */
/* Why:      Each delayed close prints one line so a run can be read afterwards. */
#include <stdio.h>

/* What:     `#include <string.h>` declares `strncmp`, the prefix comparison. */
/* Why:      Only paths under the test fixtures are delayed. */
#include <string.h>

/* What:     `#include <dirent.h>` declares `opendir`, `readdir` and `closedir`. */
/* Why:      The holder scan walks `/proc` and `/proc/<pid>/fd`. */
#include <dirent.h>

/* What:     `#include <ctype.h>` declares `isdigit`. */
/* Why:      Only a `/proc` entry whose name starts with a digit is a process. */
#include <ctype.h>

/* What:     `#include <time.h>` declares `clock_gettime`. */
/* Why:      A line must be placed on the same clock the trace of the run uses. */
#include <time.h>

/* What:     `#include <sys/wait.h>` declares `waitpid`, which collects a planted holder. */
/* Why:      Each holder must be reaped so it does not stay a zombie. */
#include <sys/wait.h>

/* What:     `#include <sys/syscall.h>` declares the `SYS_gettid` number. */
/* Why:      A line must name the thread, since another thread's child is the holder
 *           this file exists to widen the window for. */
#include <sys/syscall.h>

/* What:     The signature of the real `close`: takes a descriptor number, returns 0
 *           or -1. */
/* Why:      The wrapper calls it after the delay. */
/* In TS you'd write (pseudocode): */
/* ```ts */
/* type Close = (fd: number) => number; */
/* ``` */
typedef int (*close_fn)(int);

/* What:     The signature of the real `execve`: the path, the argument list and the
 *           environment to start it with, each an array of strings that ends at a NULL
 *           pointer. */
/* Why:      The wrapper reports the write descriptors a forked child inherited, then
 *           holds them a moment longer so they outlive the parent's start. */
/* In TS you'd write (pseudocode): */
/* ```ts */
/* type Execve = (path: string, argv: string[], env: string[]) => number; */
/* ``` */
typedef int (*execve_fn)(const char *, char *const[], char *const[]);

/* What:     The address of the real `execve`. */
/* Why:      The wrapper calls it after the report and the delay. */
static execve_fn real_execve = NULL;

/* What:     The delay in microseconds a forked child holds its inherited descriptors
 *           before it starts its own program, `0` meaning "not read yet". */
/* Why:      An inherited write descriptor lives from the fork to the child's own start,
 *           about a millisecond; holding it longer makes the window it creates as wide
 *           as the parent's. */
static long exec_delay_us = 0;

/* What:     `static` gives this variable one copy for the whole loaded library. */
/* Why:      The address of the real `close` is looked up once, not on every call. */
static close_fn real_close = NULL;

/* What:     The delay in microseconds, `0` meaning "not read yet". */
/* Why:      `getenv` on every close would be slow and could allocate. */
static long delay_us = 0;

/* What:     The prefix every delayed path must start with, `NULL` meaning "not read
 *           yet". */
/* Why:      A fixture path under the build directory, so the real program's own files
 *           are untouched. */
static const char *prefix = NULL;

/* What:     One thread's depth inside this wrapper, `static __thread` giving each
 *           thread its own copy. */
/* Why:      `fprintf` and `readlink` may themselves call `close`; without this the
 *           wrapper would recurse and never return. */
static __thread int inside = 0;

/* What:     How long a child planted by this library holds what it inherits, `0` meaning
 *           "plant nothing". */
/* Why:      A child forked by another thread of a test process is the holder the real
 *           failure needs; planting it removes the coincidence, so one run shows which
 *           tests write a program in the process that starts it. */
static long plant_hold_us = 0;

/* What:     Whether the holder scan is on, `0` meaning "not read yet". */
/* Why:      Walking every process's descriptors costs tens of milliseconds, so it is only
 *           turned on for the measurement that needs it. */
static int scan_holders = -1;

/* What:     The pids of the holders this library planted, so it can reap them without
 *           touching a child of the test. */
/* Why:      An unreaped holder stays a zombie, and a run that plants hundreds of them runs
 *           out of the process limit the test's own scope allows. */
static pid_t planted[256];
static int planted_count = 0;

/* What:     `reap_planted()` collects the holders that have ended. */
/* Why:      It keeps the process count where the test left it. */
static void reap_planted(void) {
    int kept = 0;
    for (int index = 0; index < planted_count; index++) {
        pid_t answer = waitpid(planted[index], NULL, WNOHANG);
        if (answer == 0) {
            planted[kept] = planted[index];
            kept++;
        }
    }
    planted_count = kept;
}

/* What:     `looks_like_a_program(path)` answers whether the file starts with an ELF header
 *           or a `#!` line. */
/* Why:      Only a file that is started can be refused, so only such a file gets a planted
 *           holder; a plan, a lock or a record never does. */
/* In TS you'd write (pseudocode): */
/* ```ts */
/* const head = readFirst4Bytes(path); return head.startsWith('\x7fELF') || head.startsWith('#!'); */
/* ``` */
static int looks_like_a_program(const char *path) {
    int reading = open(path, O_RDONLY | O_CLOEXEC);
    if (reading < 0) {
        return 0;
    }
    unsigned char head[4];
    ssize_t got = read(reading, head, sizeof(head));
    close(reading);
    if (got < 3) {
        return 0;
    }
    int elf = (got == 4) && (head[0] == 0x7f) && (head[1] == 'E') && (head[2] == 'L') && (head[3] == 'F');
    int script = (head[0] == '#') && (head[1] == '!');
    return elf || script;
}

/* What:     `thread_count()` counts the threads of this process. */
/* Why:      A test process has several; `tee` and `cp`, which a fixed fixture starts to
 *           write the program, have one, and only a threaded process is the shape the real
 *           failure has. */
/* In TS you'd write (pseudocode): */
/* ```ts */
/* function threadCount(): number { return readdirSync('/proc/self/task').length; } */
/* ``` */
static int thread_count(void) {
    DIR *threads = opendir("/proc/self/task");
    if (threads == NULL) {
        return 1;
    }
    int count = 0;
    struct dirent *thread = NULL;
    while ((thread = readdir(threads)) != NULL) {
        if (isdigit((unsigned char)thread->d_name[0])) {
            count++;
        }
    }
    closedir(threads);
    return count;
}

/* What:     `plant_a_holder()` forks a child that holds everything it inherited for
 *           `plant_hold_us` and then leaves. */
/* Why:      The fork copies this process's descriptor table while the write descriptor is
 *           open, so the start that follows the close is refused for as long as the child
 *           lives: the mechanism with its coincidence removed. */
/* In TS you'd write (pseudocode): */
/* ```ts */
/* if (fork() === 0) { await sleep(holdMs); process.exit(0); } */
/* ``` */
static void plant_a_holder(const char *path) {
    reap_planted();
    if (thread_count() < 2) {
        return;
    }
    if (!looks_like_a_program(path)) {
        return;
    }
    pid_t holder = fork();
    if (holder < 0) {
        return;
    }
    if (holder == 0) {
        /* What:      The child waits without touching the copied heap, then leaves. */
        /* Why:       It exists only to keep the inherited descriptor open. */
        (void)usleep((useconds_t)plant_hold_us);
        _exit(0);
    }
    if (planted_count < (int)(sizeof(planted) / sizeof(planted[0]))) {
        planted[planted_count] = holder;
        planted_count++;
    }
    fprintf(stderr, "ETXTBSY-PLANTED pid=%d holder=%d hold-us=%ld path=%s\n",
            (int)getpid(), (int)holder, plant_hold_us, path);
}

/* What:     `report_holders(path)` prints every other process that holds `path` open for
 *           writing, one line each. */
/* Why:      This is the direct evidence for the mechanism: a child forked while the
 *           window was open keeps its copy of the descriptor, and the start is refused for
 *           as long as it does. */
/* In TS you'd write (pseudocode): */
/* ```ts */
/* for (const pid of readdirSync('/proc')) { */
/*   if (!/^\d+$/.test(pid)) continue; */
/*   for (const fd of readdirSync(`/proc/${pid}/fd`)) { */
/*     if (readlinkSync(`/proc/${pid}/fd/${fd}`) === path) report(pid, fd); */
/*   } */
/* } */
/* ``` */
static void report_holders(const char *path) {
    DIR *processes = opendir("/proc");
    if (processes == NULL) {
        return;
    }
    struct dirent *process = NULL;
    while ((process = readdir(processes)) != NULL) {
        if (!isdigit((unsigned char)process->d_name[0])) {
            continue;
        }
        char folder[64];
        (void)snprintf(folder, sizeof(folder), "/proc/%s/fd", process->d_name);
        DIR *descriptors = opendir(folder);
        if (descriptors == NULL) {
            continue;
        }
        struct dirent *descriptor = NULL;
        while ((descriptor = readdir(descriptors)) != NULL) {
            if (!isdigit((unsigned char)descriptor->d_name[0])) {
                continue;
            }
            char link[128];
            char target[4096];
            (void)snprintf(link, sizeof(link), "%s/%s", folder, descriptor->d_name);
            ssize_t length = readlink(link, target, sizeof(target) - 1);
            if (length <= 0) {
                continue;
            }
            target[length] = '\0';
            if (strcmp(target, path) != 0) {
                continue;
            }
            int holder = atoi(process->d_name);
            if (holder == (int)getpid()) {
                fprintf(stderr, "ETXTBSY-HOLDER pid=%d fd=%s path=%s (this process, the window's own descriptor)\n",
                        holder, descriptor->d_name, path);
            } else {
                fprintf(stderr, "ETXTBSY-HOLDER pid=%d fd=%s path=%s (another process)\n",
                        holder, descriptor->d_name, path);
            }
        }
        closedir(descriptors);
    }
    closedir(processes);
}

/* What:     `read_the_settings()` reads the two environment variables and finds the
 *           real `close`. */
/* Why:      Doing it once makes every later `close` cheap and keeps the wrapper free
 *           of allocation on the hot path. */
/* In TS you'd write (pseudocode): */
/* ```ts */
/* function readSettings(): void { prefix = process.env.ETXTBSY_SHIM_PREFIX ?? ''; ... } */
/* ``` */
static void read_the_settings(void) {
    const char *delay = getenv("ETXTBSY_SHIM_DELAY_US");
    /* What:      `atoi` turns the variable's text into an integer; it answers 0 for
     *            text that is not a number. */
    /* Why:       The delay is a number of milliseconds from the command line, so the
     *            test run can widen the window however far it needs. */
    delay_us = (delay == NULL) ? 0 : atol(delay);
    /* What:      `getenv` again, for the prefix. */
    /* Why:       The same reason as the delay. */
    prefix = getenv("ETXTBSY_SHIM_PREFIX");
    if (prefix == NULL) {
        prefix = "";
    }
    /* What:      `dlsym(RTLD_NEXT, "close")` finds the `close` of the next library in
     *            the search order, which is the real one. */
    /* Why:       Without it the wrapper would call itself. */
    real_close = (close_fn)dlsym(RTLD_NEXT, "close");
    /* What:      The same lookup for `execve`, and the third environment variable. */
    /* Why:       The child's hold on an inherited descriptor is the other half of the
     *            window, so this file has to reach the child's own start too. */
    const char *plant = getenv("ETXTBSY_SHIM_HOLD_US");
    plant_hold_us = (plant == NULL) ? 0 : atol(plant);
    const char *exec_delay = getenv("ETXTBSY_SHIM_EXEC_DELAY_US");
    exec_delay_us = (exec_delay == NULL) ? 0 : atol(exec_delay);
    real_execve = (execve_fn)dlsym(RTLD_NEXT, "execve");
}

/* What:     `#include <time.h>` declares `clock_gettime`. */
/* Why:      A line must be placed on the same clock the trace of the run uses. */
#include <time.h>

/* What:     `now_microseconds()` reads the wall clock in microseconds. */
/* Why:      `strace -ttt` prints the same clock, so a window and a fork can be compared
 *           afterwards. */
static long now_microseconds(void) {
    struct timespec moment;
    (void)clock_gettime(CLOCK_REALTIME, &moment);
    return (long)moment.tv_sec * 1000000L + moment.tv_nsec / 1000L;
}

/* What:     `close(int fd)` replaces the C library's `close` for every caller in the
 *           process. */
/* Why:      The window this file widens opens at the write and ends at this call, so
 *           this is where the delay belongs. */
/* In TS you'd write (pseudocode): */
/* ```ts */
/* export function close(fd: number): number { */
/*   if (isWrittenFixture(fd)) await delay(delayUs); */
/*   return realClose(fd); */
/* } */
/* ``` */
int close(int fd) {
    /* What:      The one-time setup, run by whichever thread closes first. */
    /* Why:       Two threads may reach this at once; a second lookup is harmless. */
    if (real_close == NULL) {
        read_the_settings();
    }
    if (inside != 0 || real_close == NULL) {
        return real_close(fd);
    }
    /* What:      The three switches, read once by the first caller. */
    /* Why:       Any one of them means this descriptor has to be looked at. */
    if (scan_holders < 0) {
        const char *scan = getenv("ETXTBSY_SHIM_SCAN_HOLDERS");
        scan_holders = (scan != NULL) && (scan[0] == '1');
    }
    if (delay_us <= 0 && plant_hold_us <= 0 && scan_holders != 1) {
        return real_close(fd);
    }
    inside = 1;
    /* What:      `fcntl(fd, F_GETFL)` asks the descriptor for its open mode, and
     *            `O_ACCMODE` masks out the two bits that say whether it is open for
     *            reading, writing or both. */
    /* Why:       A read-only descriptor cannot make a start fail. */
    int flags = fcntl(fd, F_GETFL);
    int is_writing = (flags >= 0) && ((flags & O_ACCMODE) != O_RDONLY);
    char path[4096];
    int matched = 0;
    if (is_writing) {
        /* What:      The buffer that receives the path the descriptor names, and the
         *            length `readlink` wrote into it. */
        /* Why:       `readlink("/proc/self/fd/<n>")` is how a descriptor's path is
         *            read without opening anything. */
        char link[64];
        (void)snprintf(link, sizeof(link), "/proc/self/fd/%d", fd);
        ssize_t length = readlink(link, path, sizeof(path) - 1);
        if (length > 0) {
            path[length] = '\0';
            size_t prefix_length = strlen(prefix);
            matched = (prefix_length == 0) || (strncmp(path, prefix, prefix_length) == 0);
        }
    }
    if (matched) {
        /* What:      The thread's own number, from the `gettid` system call. */
        /* Why:       The line is read beside a trace, which names threads as well. */
        long thread = syscall(SYS_gettid);
        /* What:      `fprintf` writes one line on standard error. */
        /* Why:       The harness shows a test's standard error beside its result. */
        fprintf(stderr, "ETXTBSY-WINDOW now=%ld pid=%d tid=%ld fd=%d open-us=%ld path=%s\n",
                now_microseconds(), (int)getpid(), thread, fd, delay_us, path);
        if (plant_hold_us > 0) {
            /* What:      The child is forked while this descriptor is open, so it inherits
             *            it. */
            /* Why:       The start after the close is what the real failure refuses. */
            plant_a_holder(path);
        }
        (void)usleep((useconds_t)delay_us);
        if (scan_holders == 1) {
            /* What:      The scan runs at the moment the descriptor is about to be closed. */
            /* Why:       A child forked inside the window still holds its copy then, and
             *            the start that follows is refused for as long as it does. */
            report_holders(path);
        }
        /* What:      A second line at the moment the descriptor is actually closed. */
        /* Why:       The two lines together are the interval in which a fork of another
         *            thread would copy this descriptor. */
        fprintf(stderr, "ETXTBSY-WINDOW-END now=%ld pid=%d tid=%ld fd=%d\n",
                now_microseconds(), (int)getpid(), thread, fd);
    }
    inside = 0;
    return real_close(fd);
}

/* What:     `execve(path, argv, envp)` replaces the process with the program at `path`,
 *           and it is the call `ETXTBSY` answers when that file is open for writing in
 *           any process of the machine. */
/* Why:      A child forked by another thread carries this thread's write descriptors
 *           until this call; printing them here is what shows the inheritance instead of
 *           leaving it a reading of the error. */
/* In TS you'd write (pseudocode): */
/* ```ts */
/* export function execve(path: string, argv: string[], env: string[]): number { */
/*   for (const fd of openDescriptors()) if (isWriteDescriptor(fd)) report(fd); */
/*   await delay(execDelayUs); */
/*   return realExecve(path, argv, env); */
/* } */
/* ``` */
int execve(const char *path, char *const argv[], char *const envp[]) {
    if (real_execve == NULL) {
        read_the_settings();
    }
    if (real_execve == NULL) {
        /* What:      The C library's own `execve` when the real address could not be
         *            found; `_exit` reports the failure without unwinding. */
        /* Why:       A wrapper that returned would make the caller think the start
         *            failed when it never ran. */
        _exit(127);
    }
    if (exec_delay_us > 0) {
        /* What:      `dupping` walks the descriptor table one number at a time. */
        /* Why:       `opendir` allocates and calls `close`, so the numbers are taken
         *            from `/proc/self/fd` with plain calls instead. */
        for (int fd = 3; fd < 256; fd++) {
            int flags = fcntl(fd, F_GETFL);
            if (flags < 0 || ((flags & O_ACCMODE) == O_RDONLY)) {
                continue;
            }
            char link[64];
            char inherited[4096];
            (void)snprintf(link, sizeof(link), "/proc/self/fd/%d", fd);
            ssize_t length = readlink(link, inherited, sizeof(inherited) - 1);
            if (length <= 0) {
                continue;
            }
            inherited[length] = '\0';
            /* What:      One line naming the child, the descriptor and the file it
             *            inherited. */
            /* Why:       The file is the one the parent is about to start, which is what
             *            `ETXTBSY` refuses for as long as this process holds it. */
            fprintf(stderr, "ETXTBSY-INHERITED pid=%d tid=%ld fd=%d path=%s exec=%s\n",
                    (int)getpid(), syscall(SYS_gettid), fd, inherited, path);
        }
        (void)usleep((useconds_t)exec_delay_us);
    }
    return real_execve(path, argv, envp);
}
