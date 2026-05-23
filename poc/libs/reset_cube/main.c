#include "ui_world.h"

#define HEAP_SIZE 65536
static uint8_t g_heap[HEAP_SIZE];
static unsigned long g_heap_idx = 0;

void *memcpy(void *dest, const void *src, unsigned long n) {
    uint8_t *d = (uint8_t *)dest;
    const uint8_t *s = (const uint8_t *)src;
    for (unsigned long i = 0; i < n; i++) {
        d[i] = s[i];
    }
    return dest;
}

void *malloc(unsigned long size) {
    size = (size + 7) & ~7;
    if (g_heap_idx + size > HEAP_SIZE) {
        return (void*)0;
    }
    void *ptr = &g_heap[g_heap_idx];
    g_heap_idx += size;
    return ptr;
}

void free(void *ptr) {
    (void)ptr;
}

void *realloc(void *ptr, unsigned long new_size) {
    if (!ptr) return malloc(new_size);
    void *new_ptr = malloc(new_size);
    if (!new_ptr) return (void*)0;
    uint8_t *src = (uint8_t *)ptr;
    uint8_t *dst = (uint8_t *)new_ptr;
    for (unsigned long i = 0; i < new_size; i++) {
        dst[i] = src[i];
    }
    return new_ptr;
}

void abort(void) {
    while(1);
}

unsigned long strlen(const char *s) {
    unsigned long len = 0;
    while (s[len]) len++;
    return len;
}

int custom_strncmp(const uint8_t *s1, const char *s2, unsigned long n) {
    while (n && *s1 && (*s1 == (uint8_t)*s2)) {
        s1++;
        s2++;
        n--;
    }
    if (n == 0) return 0;
    return *s1 - (uint8_t)*s2;
}

__attribute__((export_name("ui-world#init")))
void ui_world_init(void) {}

__attribute__((export_name("ui-world#get-commands")))
void exports_ui_world_get_commands(ui_world_list_command_desc_t *ret) {
    ui_world_command_desc_t *cmds = (ui_world_command_desc_t *)malloc(sizeof(ui_world_command_desc_t) * 2);

    cmds[0].module.ptr = (uint8_t *)"cbot";
    cmds[0].module.len = 4;
    cmds[0].name.ptr = (uint8_t *)"c_ping";
    cmds[0].name.len = 6;
    cmds[0].options.ptr = (uint8_t *)"";
    cmds[0].options.len = 0;
    cmds[0].help.ptr = (uint8_t *)"A ping from a 100% C module.";
    cmds[0].help.len = 28;

    cmds[1].module.ptr = (uint8_t *)"cbot";
    cmds[1].module.len = 4;
    cmds[1].name.ptr = (uint8_t *)"reset_cube";
    cmds[1].name.len = 10;
    cmds[1].options.ptr = (uint8_t *)"";
    cmds[1].options.len = 0;
    cmds[1].help.ptr = (uint8_t *)"Teleport Cube at 200 200.";
    cmds[1].help.len = 26;

    ret->ptr = cmds;
    ret->len = 2;
}

__attribute__((export_name("ui-world#run-command")))
void exports_ui_world_run_command(ui_world_string_t *cmd, ui_world_list_string_t *args, ui_world_response_command_t *ret) {
    (void)args;
    if (cmd->len == 6 && custom_strncmp(cmd->ptr, "c_ping", 6) == 0) {
        ui_world_string_t event_name = { .ptr = (uint8_t *)"console_log", .len = 11 };
        ui_world_string_t payload = { .ptr = (uint8_t *)"Pong! C module compiled via Zig + WASM!", .len = 42 };

        local_zappy_host_api_emit_event(&event_name, &payload);
        ret->tag = 0;
    } else if (cmd->len == 10 && custom_strncmp(cmd->ptr, "reset_cube", 10) == 0) {
        ui_world_string_t log_event = { .ptr = (uint8_t *)"teleport_cube", .len = 13 };
        ui_world_string_t log_msg = { .ptr = (uint8_t *)"reset", .len = 5 };

        local_zappy_host_api_emit_event(&log_event, &log_msg);
        ret->tag = 0;
    } else {
        ret->tag = 2;
    }
}

__attribute__((export_name("ui-world#handle-event")))
void exports_ui_world_handle_event(ui_world_string_t *event_name, ui_world_string_t *payload) {
    if (event_name->len == 13 && custom_strncmp(event_name->ptr, "teleport_cube", 13) == 0) {
        if (payload->len == 5 && custom_strncmp(payload->ptr, "reset", 5) == 0) {
            ui_world_string_t log_event = { .ptr = (uint8_t *)"console_log", .len = 11 };
            ui_world_string_t log_msg = { .ptr = (uint8_t *)"[CBot] Cube reset detected!", .len = 27 };
            local_zappy_host_api_emit_event(&log_event, &log_msg);
        }
    }
}

__attribute__((export_name("ui-world#handle-input")))
void exports_ui_world_handle_input(ui_world_input_state_t *state) {
    (void)state;
}

__attribute__((export_name("ui-world#update-module")))
void exports_ui_world_update_module(float time, float dt, float w, float h, ui_world_list_render_command_t *ret) {
    (void)time; (void)dt; (void)w; (void)h;
    ret->ptr = (void*)0;
    ret->len = 0;
}

__attribute__((export_name("ui-world#accept-log")))
void exports_ui_world_accept_log(ui_world_list_text_segment_t *segments) {
    (void)segments;
}

__attribute__((export_name("ui-world#serialize")))
void exports_ui_world_serialize(ui_world_list_u8_t *ret) {
    ret->ptr = (void*)0;
    ret->len = 0;
}

__attribute__((export_name("ui-world#deserialize")))
void exports_ui_world_deserialize(ui_world_list_u8_t *state) {
    (void)state;
}
