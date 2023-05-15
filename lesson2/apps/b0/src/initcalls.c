/* Helper for getting init_calls_[start/end] */
#include <stdint.h>
extern void init_calls_start;
extern void init_calls_end;
#define _symval(p) ((uint64_t)((uintptr_t)(p)))

uint64_t initcalls_start() {
    /* Todo: fix it! */
    return _symval(&init_calls_start);
}

uint64_t initcalls_end() {
    /* Todo: fix it! */
    return _symval(&init_calls_end);
}
