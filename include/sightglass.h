#ifndef sightglass_h
#define sightglass_h 1

#include <fcntl.h>
#include <stdlib.h>
#include <unistd.h>

/**
 * Call this function to indicate that recording should start. This call should be placed
 * immediately prior to the code to measure with sightglass-recorder. The attributes allow compilers
 * to generate the correct Wasm imports.
 */
__attribute__((import_module("bench")))
__attribute__((import_name("start")))
void bench_start();

/**
 * Call this function to indicate that recording should end. This call should be placed immediately
 * after the code to measure with sightglass-recorder. The attributes allow compilers to generate
 * the correct Wasm imports.
 */
__attribute__((import_module("bench")))
__attribute__((import_name("end")))
void bench_end();

/**
 * Call this function to prevent certain compiler-related optimizations related to knowing the value
 * of the passed variable.
 */
#ifndef black_box
static void _black_box(void *x)
{
    (void)x;
}
static void (*volatile black_box)(void *x) = _black_box;
#else
void black_box(void *x);
#endif
#define BLACK_BOX(X) black_box((void *)&(X))

/**
 * Read a decimal integer -- typically a workload size -- from the file at `path`, returning
 * `default_value` if the file is missing, empty, or does not start with a number.
 *
 * Benchmarks use this to keep their workload size in a sibling `*.input` file rather than in a
 * compile-time constant, so that the benchmark can be retuned by editing that file instead of
 * rebuilding the Wasm (see `benchmarks/README.md`). Sightglass preopens the directory containing
 * the `.wasm` as the working directory, so `path` is normally of the form `"./default.input"`.
 *
 * Call this *before* `bench_start()` so the I/O is not measured.
 */
static inline long bench_read_long(const char *path, long default_value)
{
    char    buf[64];
    size_t  n = 0;
    ssize_t nread;
    long    value;
    char   *end;
    int     fd;

    fd = open(path, O_RDONLY);
    if (fd < 0) {
        return default_value;
    }
    while (n + 1 < sizeof(buf)) {
        nread = read(fd, buf + n, sizeof(buf) - n - 1);
        if (nread <= 0) {
            break;
        }
        n += (size_t)nread;
    }
    close(fd);

    buf[n] = '\0';
    value = strtol(buf, &end, 10);
    if (end == buf) {
        return default_value;
    }
    return value;
}

#endif
