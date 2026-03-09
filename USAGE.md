# 操作说明

本项目当前为 TUI 最小可用版本，支持任务列表、内联编辑与 SQLite 持久化。

## 启动

```bash
cargo run -p gtd-tui
```

默认数据库路径：`~/.local/share/gtd-tui/gtd.db`

可通过环境变量覆盖：

```bash
GTD_TUI_DB_PATH=/path/to/gtd.db cargo run -p gtd-tui
```

## 基本操作（Normal 模式）

- `q` / `Esc`：退出
- `j` / `k` / `↑` / `↓`：移动选中任务
- `n`：新建任务（进入内联编辑）
- `e`：编辑选中任务（进入内联编辑）
- `x`：切换完成状态
- `r`：刷新任务列表
- 视图切换：`1~5` 或 `i/t/u/a/s`

## 内联编辑（Editing 模式）

编辑器位于任务列表中，包含 Title / Notes / Due / Checklist。

- `Tab` / `Shift+Tab`：切换字段焦点
- `Ctrl+S`：保存
- `Esc`：取消

### Title / Notes

- 单行输入，`Backspace` 删除字符

### Due（日期）

- `←/→`：前后一天
- `↑/↓`：前后 7 天
- `p` / `n`：上月 / 下月
- `t`：今天
- `m`：明天
- `Enter`：确认日期
- `Backspace`：清空日期

### Checklist

- `Enter`：新增条目
- `Backspace`：若当前条目为空则删除

## 说明

- 当前仅支持单个日期（类似 Things 3 的“安排到某天”），不含开始/结束日期。
- Notes 为单行。
- Checklist 仅支持文本条目，未实现勾选与排序。
