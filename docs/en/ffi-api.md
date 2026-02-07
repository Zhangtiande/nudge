# FFI API

[English](ffi-api.md) | [中文](../zh/ffi-api.md)

Use Nudge as a C-compatible library when embedding completion in custom tooling.

## What It Solves

- Reuse Nudge completion and safety logic from native code
- Integrate command suggestion flow without invoking CLI subprocesses directly

## Build Artifacts

```bash
cargo build --release
```

Headers:

- `include/nudge.h`

Library path can be discovered with:

```bash
nudge info --field lib_path
```

## Minimal C Example

```c
#include <stdio.h>
#include "nudge.h"

static void on_completion(const char* suggestion, const char* warning,
                          const char* error, void* user_data) {
    (void)user_data;
    if (error) {
        fprintf(stderr, "error: %s\n", error);
        return;
    }
    printf("suggestion: %s\n", suggestion ? suggestion : "");
    if (warning) {
        printf("warning: %s\n", warning);
    }
}

int main(void) {
    NudgeContext ctx = nudge_init(NULL);
    if (!ctx) {
        fprintf(stderr, "init failed: %s\n", nudge_get_error(NULL));
        return 1;
    }

    int rc = nudge_complete(ctx, "git st", 6, ".", "ffi-session", on_completion, NULL);
    if (rc != 0) {
        fprintf(stderr, "complete failed: %s\n", nudge_get_error(ctx));
    }

    nudge_free(ctx);
    return 0;
}
```

## API Surface

Core lifecycle:

- `nudge_init`
- `nudge_complete`
- `nudge_get_error`
- `nudge_free`
- `nudge_version`

Auto-mode helpers (FFI path):

- `nudge_auto_start`
- `nudge_auto_update_buffer`
- `nudge_auto_trigger`
- `nudge_auto_get_suggestion`
- `nudge_auto_stop`
- `nudge_auto_is_active`
- `nudge_auto_get_delay_ms`

## Safety Notes

- Strings are callback-lifetime only; copy if needed
- All input strings must be valid UTF-8 C strings
- `nudge_complete` callback is synchronous before function return

## Boundaries

- FFI API does not expose every CLI diagnostic capability
- Shell-specific keybinding behavior remains in shell integration scripts
