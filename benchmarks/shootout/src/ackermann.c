#include <assert.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include "sightglass.h"

int ackermann(int M, int N)
{
    if (M == 0)
    {
        return N + 1;
    }
    if (N == 0)
    {
        return ackermann(M - 1, 1);
    }
    return ackermann(M - 1, ackermann(M, (N - 1)));
}

int main()
{
    int M = (int)bench_read_long("./shootout-ackermann.m.input", 3);
    int N = (int)bench_read_long("./shootout-ackermann.n.input", 7);
    /* `A(3, n)` grows by ~4x with each step of `n`, so no single `(M, N)` lands near the ~100M
       instruction target; repeat the call instead. */
    int repeat = (int) bench_read_long("./shootout-ackermann.repeat.input", 11);
    printf("[ackermann] running with M = %d and N = %d\n", M, N);

    int result = 0;
    int i;
    bench_start();
    for (i = 0; i < repeat; i++)
    {
        /* Keep the compiler from hoisting this pure call out of the loop. */
        BLACK_BOX(M);
        BLACK_BOX(N);
        result = ackermann(M, N);
        BLACK_BOX(result);
    }
    bench_end();

    printf("[ackermann] returned %d\n", result);
}
