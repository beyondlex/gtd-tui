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
- `j` / `k` / `↓` / `↑`：移动选中任务
- `n`：新建任务（进入内联编辑）
- `l`（Inbox 视图）：编辑选中任务（进入内联编辑）
- `x`：切换完成状态
- `r`：刷新任务列表
- 视图切换：`1~5` 或 `i/t/u/a/s`

## 内联编辑（Editing 模式）

编辑器采用双层结构：
- **TaskItem 层**：Title → Notes → Due → Checklist
- **ChecklistItem 层**：checklist 条目列表

### TaskItem 层（Normal 模式）

- `j` / `k`：切换字段焦点 (Title → Notes → Due → Checklist → Title)
- `A`：直接进入 Title 编辑模式
- `l`：进入编辑模式（Title/Notes/Due）或进入 ChecklistItem 层（Checklist）
- `x`：切换 Checklist 条目完成状态

### TaskItem 层（编辑模式）

- `h` / `l`：前后一天（Due 日期）
- `k` / `j`：前后 7 天（Due 日期）
- `p` / `n`：上月 / 下月（Due 日期）
- `t`：今天
- `m`：明天
- `Enter`：确认日期并进入下一字段
- `Backspace`：删除字符
- `Esc`：退出编辑模式

### ChecklistItem 层

当光标在 Checklist 上按 `l` 进入此层。

- `j` / `k`：在 checklist 条目间移动
- `l`：进入当前条目的编辑模式
- `h`：返回父节点（Checklist）
- `x`：切换当前条目完成状态
- `Enter`：新增条目

### ChecklistItem 层（编辑模式）

- 输入字符：编辑条目内容
- `Backspace`：删除字符
- `Enter`：确认并新增下一条目

### 全局

- `Ctrl+S`：保存
- `Esc`：取消编辑/返回上层

## 配置文件

配置文件路径：`~/.config/gtd-tui/config.toml`（或 `$XDG_CONFIG_HOME/gtd-tui/config.toml`）。

示例：

```toml
[theme.calendar]
weekday = "bold"
weekend = "red bold"
today = "green bold"
selected = "blue bold"
bracket = "magenta"

[theme.editor]
checklist_edit = "lightyellow bold"
cursor = "yellow"

[keys]
quit = "q"
new_task = "n"
edit_title = "A"
select_next = "j"
select_prev = "k"
save_edit = "ctrl+s"
```

### 可配置快捷键

| 配置项 | 默认值 | 说明 |
|--------|--------|------|
| `quit` | `q` | 退出 |
| `view_inbox` | `1` | 切换到 Inbox |
| `view_today` | `2` | 切换到 Today |
| `view_upcoming` | `3` | 切换到 Upcoming |
| `view_anytime` | `4` | 切换到 Anytime |
| `view_someday` | `5` | 切换到 Someday |
| `select_next` | `j` | 选中下一个 |
| `select_prev` | `k` | 选中上一个 |
| `new_task` | `n` | 新建任务 |
| `edit_task` | `l` | 编辑任务 |
| `edit_title` | `A` | 进入 Title 编辑模式 |
| `toggle_task` | `x` | 切换完成状态 |
| `refresh` | `r` | 刷新列表 |
| `save_edit` | `ctrl+s` | 保存 |
| `cancel_edit` | `q` | 取消 |
| `next_focus` | `j` | 下一字段 |
| `prev_focus` | `k` | 上一字段 |
| `checklist_edit_toggle` | `ctrl+e` | 切换编辑模式 |
| `date_prev_day` | `h` | 日期减一天 |
| `date_next_day` | `l` | 日期加一天 |
| `date_prev_week` | `k` | 日期减一周 |
| `date_next_week` | `j` | 日期加一周 |
| `date_edit_mode` | `l` | 进入编辑/进入子层 |
| `date_prev_month` | `p` | 日期减一月 |
| `date_next_month` | `n` | 日期加一月 |
| `date_today` | `t` | 今天 |
| `date_tomorrow` | `m` | 明天 |
| `checklist_toggle` | `x` | 切换完成状态 |
| `checklist_add` | `l` | 新增条目 |
| `checklist_next` | `j` | 下一条目 |
| `checklist_prev` | `k` | 上一条目 |

快捷键格式支持 `ctrl+x` 形式。

### 主题颜色

可用颜色：black, red, green, yellow, blue, magenta, cyan, gray, darkgray, lightred, lightgreen, lightyellow, lightblue, lightmagenta, lightcyan, white, reset。

可配置的主题属性：

**[theme.editor]**
| 属性 | 默认值 | 说明 |
|------|--------|------|
| `checklist_edit` | `lightyellow bold` | 编辑模式下的文本样式 |
| `task_selected` | `blue bold` | 选中任务的样式 |
| `date_label` | `darkgray bold` | 日期标签样式 |
| `checklist_item_selected` | `blue bold` | 选中 checklist 项的样式 |
| `field_title` | `cyan bold` | Title 字段标签样式 |
| `field_notes` | `green bold` | Notes 字段标签样式 |
| `field_due` | `yellow bold` | Due 字段标签样式 |
| `field_checklist` | `magenta bold` | Checklist 字段标签样式 |
| `completed` | `darkgray` | 已完成任务的样式 |
| `cursor` | `yellow` | 编辑时光标的颜色 |

## 说明

- 当前仅支持单个日期（类似 Things 3 的"安排到某天"），不含开始/结束日期。
- Notes 为单行。
- Checklist 支持勾选与编辑。
