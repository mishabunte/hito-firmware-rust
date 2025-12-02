#include <random/rand32.h>
// on older Zephyr it might be:
// #include <random/rand32.h>

uint32_t hito_sys_rand32_get(void)
{
    return sys_rand32_get();
}
