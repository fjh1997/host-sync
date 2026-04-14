// C header for hostsync-core FFI
// Auto-generate with: cbindgen --crate hostsync-core --output hostsync_core.h --lang c

#ifndef HOSTSYNC_CORE_H
#define HOSTSYNC_CORE_H

#include <stdint.h>

char* hostsync_load_servers_json(void);
int32_t hostsync_save_servers_json(const char* json);
char* hostsync_parse_ssh_config(const char* config);
char* hostsync_generate_ssh_config(void);
int32_t hostsync_is_logged_in(void);
char* hostsync_get_github_username(void);
void hostsync_free_string(char* s);

#endif
