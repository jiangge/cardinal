#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

void init_sdk(void);

void get_events(const int8_t *context, void (*callback)(const int8_t*, const int8_t*));
