# 数据与存储

## 数据目录

默认：

```
~/.prompt-saviour/
├── drafts.db          # SQLite 主库
├── drafts.db-wal      # WAL（运行时）
├── drafts.db-shm
├── config.json        # 用户配置
└── inject.json        # 临时：inject 命令写入，daemon 消费后删除
```

覆盖：`export PROMPT_SAVIOUR_HOME=/path/to/dir`

## config.json

```json
{
  "ax_poll_ms": 400,
  "debounce_ms": 500,
  "retention_days": 30,
  "max_drafts": 500,
  "excluded_bundle_ids": []
}
```

| 字段 | 默认 | 说明 |
|------|------|------|
| `ax_poll_ms` | 400 | AX 轮询间隔 |
| `debounce_ms` | 500 | 落盘 debounce |
| `retention_days` | 30 | 自动删除早于 N 天的行 |
| `max_drafts` | 500 | 全局最多保留条数 |
| `excluded_bundle_ids` | [] | 不捕获的 bundle id |

首次 `AppConfig::load()` 若文件不存在则创建默认 config。

## drafts 表

```sql
CREATE TABLE drafts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slot_key TEXT NOT NULL,
    content TEXT NOT NULL,
    source TEXT NOT NULL,       -- accessibility | keystroke | clipboard | merged
    app_name TEXT NOT NULL,
    bundle_id TEXT NOT NULL,
    window_title TEXT NOT NULL,
    char_count INTEGER NOT NULL,
    updated_at TEXT NOT NULL    -- RFC3339 UTC
);
```

索引：`slot_key`、`updated_at DESC`。

## Slot Key

```
SHA256(bundle_id + ":" + pid + ":" + window_title)
```

同一窗口会话稳定；
换窗口 / 换 pid → 新 slot。

## 写入策略

1. Merge 产出 snapshot，debounce idle 后调用 `upsert_snapshot`
2. content trim 后 **< 8 字符** → 跳过
3. 与同 slot 最新行 content 相同 → 跳过（去重）
4. 否则 INSERT 新行
5. 每 slot 保留最近 **5** 版，更旧删除
6. `prune_old`：按 `retention_days` 与 `max_drafts` 全局 prune

## 读取

- `list_recent(limit)` - 按 `updated_at DESC`
- `get_latest()` - 最新一条
- `get_by_id(id)` - 按主键

## GUI Current Prompt（规划）

GUI **不**单独存表。
实时展示来自 daemon 内存中的 merge 结果；
落盘后与 `drafts` 表一致。
「当前 Prompt」页订阅 daemon 事件，历史页读 SQLite。

## 备份与迁移

- 复制整个 `~/.prompt-saviour/` 即可备份
- 无加密；明文 UTF-8
- 跨机器迁移：复制目录后在新机器授权 App 即可

## 隐私

- 捕获**所有**前台输入文本（无 allowlist 硬编码）
- 可通过 `excluded_bundle_ids` 排除特定 App
- 数据不出本机
