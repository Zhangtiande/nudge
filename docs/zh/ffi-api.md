# FFI API

[English](../en/ffi-api.md) | [中文](ffi-api.md)

将 Nudge 作为 C 兼容库使用，以便在自定义工具中嵌入补全功能。

## 解决的问题

- 从原生代码中复用 Nudge 的补全和安全检查逻辑
- 无需直接调用 CLI 子进程即可集成命令建议流程

## 构建产物

```bash
cargo build --release
```

头文件：

- `include/nudge.h`

库路径可通过以下命令获取：

```bash
nudge info --field lib_path
```

## 最小 C 示例

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

## API 接口

核心生命周期：

- `nudge_init`
- `nudge_complete`
- `nudge_get_error`
- `nudge_free`
- `nudge_version`

Auto 模式辅助函数（FFI 路径）：

- `nudge_auto_start`
- `nudge_auto_update_buffer`
- `nudge_auto_trigger`
- `nudge_auto_get_suggestion`
- `nudge_auto_stop`
- `nudge_auto_is_active`
- `nudge_auto_get_delay_ms`

## 安全注意事项

- 字符串仅在回调生命周期内有效；如需保留请自行拷贝
- 所有输入字符串必须是有效的 UTF-8 C 字符串
- `nudge_complete` 的回调在函数返回前同步执行

## 边界

- FFI API 未暴露所有 CLI 诊断功能
- Shell 特定的快捷键绑定行为仍保留在 shell 集成脚本中
