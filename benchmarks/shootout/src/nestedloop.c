
#include <sightglass.h>
#include <stdio.h>
#include <stdlib.h>

/* Fallback for `./shootout-nestedloop.length.input`, tuned so that this benchmark
   executes ~100M Wasm instructions. */
#define LENGTH 15

int main()
{
    int n = (int) bench_read_long("./shootout-nestedloop.length.input", LENGTH);
    BLACK_BOX(n);
    int a, b, c, d, e, f;
    /* `x` is `volatile` so that the increment is a real load/store: otherwise the compiler
       closes the whole loop nest into a single multiply and the benchmark measures nothing. */
    volatile int x = 0;
    BLACK_BOX(x);

    printf("[nestedloop] running 6 nested loops with %d iterations each\n", n);
    bench_start();
    for (a = 0; a < n; a++) {
        for (b = 0; b < n; b++) {
            for (c = 0; c < n; c++) {
                for (d = 0; d < n; d++) {
                    for (e = 0; e < n; e++) {
                        for (f = 0; f < n; f++) {
                            x++;
                        }
                    }
                }
            }
        }
    }
    bench_end();

    BLACK_BOX(x);
    printf("[nestedloop] returned %d\n", x);
}
