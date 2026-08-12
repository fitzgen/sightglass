
#include <sightglass.h>
#include <stdio.h>

static unsigned long
fib2(unsigned long n)
{
    if (n < 2) {
        return 1;
    }
    return fib2(n - 2) + fib2(n - 1);
}

int main()
{
    /* The fallback is tuned so that this benchmark executes ~100M Wasm instructions. */
    int n = (int) bench_read_long("./shootout-fib2.n.input", 32);
    printf("[fib2] finding fibonacci number of: %d\n", n);

    bench_start();
    int res = fib2(n);
    bench_end();

    printf("[fib2] returned: %d\n", res);
    BLACK_BOX(res);
}
