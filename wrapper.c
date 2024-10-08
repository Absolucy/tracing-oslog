#include "wrapper.h"

void wrapped_os_log_with_type(os_log_t log, os_log_type_t type, const char* message) {
    os_log_with_type(log, type, "%{public}s", message);
}
os_log_t wrapped_os_log_default(void) {
    return OS_LOG_DEFAULT;
}
