# gtd-tui 项目规划

## 1. 项目概述

### 1.1 目标

创建 Things 3 的开源 TUI 版本，核心特点是**完全可自定义的键盘快捷键**，实现纯键盘操作的任务管理应用。

### 1.2 技术栈

- **核心语言**: Rust
- **TUI 框架**: ratatui (Rust TUI 库)
- **数据存储**: SQLite + JSON 导出/导入
- **后续支持**: 
  - macOS App (AppKit/SwiftUI)
  - iOS App (SwiftUI)
  - 通过共享数据层支持多端

### 1.3 核心设计原则

1. **数据优先**: 数据层与 UI 层完全分离，支持多端共享
2. **键盘优先**: 所有操作均可通过快捷键完成
3. **GTD 友好**: 遵循 Getting Things Done 方法论
4. **云端同步**: 数据可导出至云盘，支持跨设备恢复

---

## 2. Things 3 功能调研

### 2.1 组织结构

| 层级 | 说明 | 示例 |
|------|------|------|
| Inbox | 快速收集箱 | 临时任务 |
| Today | 今日待办 | 当天计划 |
| Upcoming | 即将到来 | 未来7天计划 |
| Anytime | 随时可做 | 放入其中等待时机 |
| Someday | 有朝一日 | 未来可能做的事情 |
| Area | 领域/分类 | 工作、生活、学习 |
| Project | 项目 | 具体的目标 |
| Heading | 标题 | 项目内的任务分组 |
| Task | 任务 | 最小执行单元 |
| Checklist | 检查清单 | 任务的子任务 |

### 2.2 任务属性

- **标题** (title) - 必填
- **备注** (notes) - 详细描述、链接
- **标签** (tags) - 多标签支持
- **开始日期** (start_date) - 计划开始日期
- **截止日期** (due_date) - 硬性截止
- **提醒** (reminder) - 时间提醒
- **重复** (recurrence) - 重复规则
- **检查清单** (checklist) - 子任务列表
- **状态** (status) - 待办/进行中/已完成

### 2.3 核心视图

1. **Today View**: 当日任务，支持拖拽排序
2. **Upcoming View**: 按日期分组显示未来任务
3. **Anytime View**: 按 Area/Project 分类显示
4. **Someday View**: 未来可能的任务
5. **All Projects**: 所有项目列表
6. **Search**: 全局搜索 + 标签过滤

### 2.4 Things 3 交互特色

- Quick Entry: 全局快速添加任务
- 自然语言输入: "next month", "every monday"
- 键盘快捷键 (Mac):
  - `⌘N` - 新建任务
  - `⌘⇧N` - 快速添加
  - `⌘S` - 移至 Someday
  - `⌘T` - 移至 Today
  - `⌘.` - 完成任务
  - `Space` - 查看详情

---

## 3. 系统架构

### 3.1 分层设计

```
┌─────────────────────────────────────────┐
│            UI Layer (多端实现)           │
│  ┌──────────┐  ┌──────────┐  ┌────────┐ │
│  │  TUI    │  │ iOS App  │  │ Others │ │
│  └────┬─────┘  └────┬─────┘  └───┬────┘ │
└───────┼─────────────┼────────────┼──────┘
        │             │            │
        └─────────────┼────────────┘
                      ▼
┌─────────────────────────────────────────┐
│           Data Access Layer             │
│  ┌─────────────────────────────────────┐ │
│  │     gtd-core (核心库/Rust)          │ │
│  │  - 业务逻辑                          │ │
│  │  - 数据验证                          │ │
│  │  - 快捷键管理                       │ │
│  └────────────────┬────────────────────┘ │
└───────────────────┼──────────────────────┘
                    ▼
┌─────────────────────────────────────────┐
│            Storage Layer                 │
│  ┌──────────┐  ┌──────────────────────┐ │
│  │ SQLite   │  │ JSON Export/Import   │ │
│  │ (本地)   │  │ (云盘同步)            │ │
│  └──────────┘  └──────────────────────┘ │
└─────────────────────────────────────────┘
```

### 3.2 核心模块 (gtd-core)

```
gtd-tui/
├── gtd-core/           # 核心库 (被各端引用)
│   ├── src/
│   │   ├── models/     # 数据模型
│   │   ├── storage/    # 存储抽象
│   │   ├── services/   # 业务逻辑
│   │   └── hotkey/    # 快捷键管理
│   └── Cargo.toml
│
├── gtd-tui/            # TUI 应用
│   ├── src/
│   │   ├── ui/         # TUI 界面
│   │   ├── commands/   # 命令实现
│   │   └── main.rs
│   └── Cargo.toml
│
└── ios/                # iOS 应用 (后续)
    └── ...
```

---

## 4. 数据模型设计

### 4.1 核心实体

```rust
// 领域/分类
struct Area {
    id: Uuid,
    name: String,
    color: Option<String>,
    sort_order: i32,
    created_at: DateTime,
    updated_at: DateTime,
}

// 项目
struct Project {
    id: Uuid,
    area_id: Option<Uuid>,      // 所属 Area
    title: String,
    notes: Option<String>,
    status: ProjectStatus,     // active, completed, dropped
    start_date: Option<Date>,
    due_date: Option<Date>,
    sort_order: i32,
    created_at: DateTime,
    updated_at: DateTime,
}

// 标题/分组
struct Heading {
    id: Uuid,
    project_id: Uuid,
    title: String,
    sort_order: i32,
}

// 任务
struct Task {
    id: Uuid,
    project_id: Option<Uuid>,  // 所属 Project
    heading_id: Option<Uuid>,  // 所属 Heading
    area_id: Option<Uuid>,     // 直接在 Area 下
    title: String,
    notes: Option<String>,
    status: TaskStatus,        // pending, completed, cancelled
    start_date: Option<Date>,
    due_date: Option<Date>,
    is_today: bool,            // 在 Today 视图中
    is_someday: bool,          // 在 Someday 视图中
    sort_order: i32,
    created_at: DateTime,
    updated_at: DateTime,
}

// 标签
struct Tag {
    id: Uuid,
    name: String,
    color: Option<String>,
}

// 检查清单项
struct ChecklistItem {
    id: Uuid,
    task_id: Uuid,
    title: String,
    is_checked: bool,
    sort_order: i32,
}

// 重复规则
struct RecurrenceRule {
    frequency: Frequency,      // daily, weekly, monthly, yearly
    interval: i32,             // 间隔
    days_of_week: Option<Vec<DayOfWeek>>,
    day_of_month: Option<i32>,
    end_date: Option<Date>,
}

// 快捷键配置
struct HotkeyConfig {
    id: Uuid,
    action: String,            // action 名称
    key: String,               // 按键描述 "ctrl+c", "j"
    modifiers: Vec<Modifier>, // ctrl, shift, alt, etc.
}
```

### 4.2 视图模型

```rust
// Today 视图任务
struct TodayTask {
    task: Task,
    tags: Vec<Tag>,
    checklist: Vec<ChecklistItem>,
    project_title: Option<String>,
    area_title: Option<String>,
}

// 项目视图
struct ProjectView {
    project: Project,
    headings: Vec<Heading>,
    tasks: Vec<Task>,
    area: Option<Area>,
}
```

### 4.3 存储抽象

```rust
trait Storage: Send + Sync {
    // Area
    fn get_areas(&self) -> Result<Vec<Area>>;
    fn create_area(&self, area: &Area) -> Result<()>;
    fn update_area(&self, area: &Area) -> Result<()>;
    fn delete_area(&self, id: Uuid) -> Result<()>;

    // Project
    fn get_projects(&self, area_id: Option<Uuid>) -> Result<Vec<Project>>;
    fn create_project(&self, project: &Project) -> Result<()>;
    fn update_project(&self, project: &Project) -> Result<()>;
    fn delete_project(&self, id: Uuid) -> Result<()>;

    // Task
    fn get_tasks(&self, filter: TaskFilter) -> Result<Vec<Task>>;
    fn create_task(&self, task: &Task) -> Result<()>;
    fn update_task(&self, task: &Task) -> Result<()>;
    fn delete_task(&self, id: Uuid) -> Result<()>;

    // Tags
    fn get_tags(&self) -> Result<Vec<Tag>>;
    fn create_tag(&self, tag: &Tag) -> Result<()>;
    fn delete_tag(&self, id: Uuid) -> Result<()>;

    // Checklist
    fn get_checklist(&self, task_id: Uuid) -> Result<Vec<ChecklistItem>>;
    fn create_checklist_item(&self, item: &ChecklistItem) -> Result<()>;
    fn update_checklist_item(&self, item: &ChecklistItem) -> Result<()>;
    fn delete_checklist_item(&self, id: Uuid) -> Result<()>;

    // Hotkey
    fn get_hotkeys(&self) -> Result<Vec<HotkeyConfig>>;
    fn save_hotkey(&self, config: &HotkeyConfig) -> Result<()>;
}
```

### 4.4 数据导出/导入 (云同步)

```rust
#[derive(Serialize, Deserialize)]
struct ExportData {
    version: String,
    exported_at: DateTime,
    areas: Vec<Area>,
    projects: Vec<Project>,
    headings: Vec<Heading>,
    tasks: Vec<Task>,
    tags: Vec<Tag>,
    checklist_items: Vec<ChecklistItem>,
    hotkey_config: Vec<HotkeyConfig>,
}
```

---

## 5. TUI 功能规划

### 5.1 主界面布局

```
┌────────────────────────────────────────────────────────┐
│  [Logo] gtd-tui              🔍 Search    ⚙️ Settings  │
├─────────────┬──────────────────────────────────────────┤
│             │                                          │
│  INBOX      │   ┌──────────────────────────────────┐   │
│  ─────────  │   │ Task Title                   ☐   │   │
│  Today (3)  │   │   Tags: @work @urgent          │   │
│  Upcoming   │   │   Due: tomorrow                │   │
│  Anytime    │   └──────────────────────────────────┘   │
│  Someday    │                                          │
│             │   ┌──────────────────────────────────┐   │
│  AREAS      │   │ Another Task                 ☐   │   │
│  ─────────  │   └──────────────────────────────────┘   │
│  > Work     │                                          │
│  > Life     │                                          │
│  > Learning │                                          │
│             │                                          │
│  + New View │                                          │
├─────────────┴──────────────────────────────────────────┤
│  ↑↓ Navigate  Enter Select  n New  e Edit  d Delete   │
└────────────────────────────────────────────────────────┘
```

### 5.2 快捷键系统 (核心功能)

#### 5.2.1 默认快捷键

```toml
# 默认快捷键配置
[hotkeys]
# 导航
"j" = "cursor_down"
"k" = "cursor_up"
"h" = "collapse"
"l" = "expand"
"gg" = "goto_top"
"G" = "goto_bottom"

# 操作
"n" = "new_task"
"N" = "new_project"
"e" = "edit"
"d" = "delete"
"x" = "toggle_complete"
"Space" = "toggle_select"

# 视图切换
"t" = "goto_today"
"u" = "goto_upcoming"
"a" = "goto_anytime"
"s" = "goto_someday"
"i" = "goto_inbox"
"p" = "goto_projects"

# 任务操作
"T" = "move_to_today"
"S" = "move_to_someday"
"+" = "add_tag"
"-" = "remove_tag"

# 搜索与过滤
"/" = "search"
"f" = "filter_by_tag"

# 系统
"?" = "show_help"
"q" = "quit"
"Ctrl+s" = "save"
"Ctrl+r" = "refresh"
```

#### 5.2.2 快捷键自定义

用户可通过以下方式自定义快捷键：

1. **配置文件**: `~/.gtd-tui/hotkeys.toml`
2. **TUI 内设置**: `Ctrl+,` 打开设置面板
3. **重置**: 可恢复默认快捷键

```toml
# 自定义快捷键示例
[hotkeys]
# 覆盖默认
"j" = "cursor_down"
"n" = "new_task"

# 添加自定义动作
"Ctrl+Shift+N" = "quick_add"  # 全局快速添加
"Ctrl+[" = "escape"

# 自定义新动作
[custom_actions]
"my_action" = { command = "move_to_project", args = { project_id = "xxx" } }
```

#### 5.2.3 快捷键引擎设计

```rust
// 动作定义
enum Action {
    // 导航
    CursorDown,
    CursorUp,
    GotoTop,
    GotoBottom,
    
    // 视图
    GotoToday,
    GotoInbox,
    GotoProject { project_id: Uuid },
    
    // 任务操作
    NewTask { target: TaskTarget },
    EditTask,
    DeleteTask,
    ToggleComplete,
    MoveToToday,
    MoveToSomeday,
    
    // 标签
    AddTag,
    RemoveTag,
    
    // 其他
    Search,
    Help,
    Quit,
    Custom { name: String, args: HashMap<String, String> },
}

// 快捷键绑定
struct HotkeyBinding {
    key: Key,
    modifiers: KeyModifiers,
    action: Action,
}

// 快捷键管理器
trait HotkeyManager {
    fn bind(&mut self, key: Key, modifiers: KeyModifiers, action: Action);
    fn unbind(&mut self, key: Key, modifiers: KeyModifiers);
    fn get_action(&self, key: Key, modifiers: KeyModifiers) -> Option<&Action>;
    fn handle_key_event(&self, event: KeyEvent) -> Option<Action>;
    fn load_config(&mut self, config: &HotkeyConfig) -> Result<()>;
    fn save_config(&self) -> Result<HotkeyConfig>;
    fn reset_to_default(&mut self);
}
```

### 5.3 视图实现

#### 5.3.1 Inbox 视图
- 显示所有未分类的任务
- 支持快速添加 (`n` 键)
- 支持拖拽排序

#### 5.3.2 Today 视图
- 显示今日计划任务
- 按计划时间排序
- 允许拖拽调整顺序
- 显示已完成任务（可选）

#### 5.3.3 Upcoming 视图
- 按日期分组显示
- 支持周/月视图切换
- 显示日历事件（可选集成）

#### 5.3.4 Anytime 视图
- 按 Area > Project 树形显示
- 支持折叠/展开
- 显示任务计数

#### 5.3.5 Someday 视图
- 归档未来可能的任务
- 支持标签过滤

#### 5.3.6 Project 详情视图
- 显示项目信息
- 标题分组
- 任务列表
- 进度统计

### 5.4 命令面板

支持命令模式（类似 Vim），按 `:` 激活：

```
:new task
:new project Work
:add tag @urgent
:move to Today
:search meeting
:filter @work
:export json
:import backup.json
:shortcuts
```

---

## 6. 功能优先级

### Phase 1: MVP (核心功能)

- [ ] 基础 CRUD 操作 (Task, Project, Area)
- [ ] 视图切换 (Inbox, Today, Upcoming, Anytime, Someday)
- [ ] 默认快捷键系统
- [ ] 任务完成/删除
- [ ] 数据持久化 (SQLite)
- [ ] 搜索功能

### Phase 2: 完善功能

- [ ] 标签系统
- [ ] 重复任务
- [ ] 快捷键自定义
- [ ] 检查清单
- [ ] 任务备注
- [ ] 日期/提醒设置

### Phase 3: 高级功能

- [ ] 命令面板
- [ ] 数据导出/导入 (JSON)
- [ ] 云盘同步支持
- [ ] 主题自定义
- [ ] 插件系统

---

## 7. 数据存储

### 7.1 SQLite 表结构

```sql
-- Areas
CREATE TABLE areas (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    color TEXT,
    sort_order INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Projects
CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    area_id TEXT REFERENCES areas(id),
    title TEXT NOT NULL,
    notes TEXT,
    status TEXT DEFAULT 'active',
    start_date TEXT,
    due_date TEXT,
    sort_order INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Headings
CREATE TABLE headings (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id),
    title TEXT NOT NULL,
    sort_order INTEGER DEFAULT 0
);

-- Tasks
CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    project_id TEXT REFERENCES projects(id),
    heading_id TEXT REFERENCES headings(id),
    area_id TEXT REFERENCES areas(id),
    title TEXT NOT NULL,
    notes TEXT,
    status TEXT DEFAULT 'pending',
    start_date TEXT,
    due_date TEXT,
    is_today INTEGER DEFAULT 0,
    is_someday INTEGER DEFAULT 0,
    sort_order INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Tags
CREATE TABLE tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    color TEXT
);

-- Task Tags
CREATE TABLE task_tags (
    task_id TEXT NOT NULL REFERENCES tasks(id),
    tag_id TEXT NOT NULL REFERENCES tags(id),
    PRIMARY KEY (task_id, tag_id)
);

-- Checklist Items
CREATE TABLE checklist_items (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL REFERENCES tasks(id),
    title TEXT NOT NULL,
    is_checked INTEGER DEFAULT 0,
    sort_order INTEGER DEFAULT 0
);

-- Recurrence Rules
CREATE TABLE recurrence_rules (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL REFERENCES tasks(id),
    frequency TEXT NOT NULL,
    interval INTEGER DEFAULT 1,
    days_of_week TEXT,
    day_of_month INTEGER,
    end_date TEXT
);

-- Hotkey Config
CREATE TABLE hotkey_config (
    id TEXT PRIMARY KEY,
    action TEXT NOT NULL UNIQUE,
    key TEXT NOT NULL,
    modifiers TEXT NOT NULL
);

-- Settings
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

### 7.2 同步策略

1. **本地优先**: 所有操作先写本地 SQLite
2. **导出同步**: 手动导出 JSON 到云盘目录
3. **导入恢复**: 从 JSON 文件恢复数据
4. **冲突处理**: 保留最新修改（按 updated_at）

---

## 8. iOS 集成规划

### 8.1 数据共享

```
┌─────────────────────────────────────────────┐
│              共享数据层 (gtd-core)           │
│  - Rust 编译为静态库                         │
│  - 通过 FFI 或 UniFFI 供 Swift 调用          │
│  - SQLite 数据库文件共享                      │
└─────────────────────────────────────────────┘
```

### 8.2 iOS 实现方案

1. **方案 A: Rust + SwiftUI**
   - gtd-core 编译为静态库
   - Swift 通过 UniFFI 调用 Rust
   - SwiftUI 构建 UI

2. **方案 B: 纯 Swift**
   - 共享 SQLite + JSON 数据格式
   - Swift 端重新实现业务逻辑

推荐方案 A，减少重复开发。

### 8.3 数据文件位置

- **macOS/Linux**: `~/.local/share/gtd-tui/`
- **iOS**: App Documents 目录
- **云盘**: 用户可选目录

---

## 9. 项目结构

```
gtd-tui/
├── gtd-core/              # 核心库 (60%)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── models/        # 数据模型
│   │   │   ├── mod.rs
│   │   │   ├── area.rs
│   │   │   ├── project.rs
│   │   │   ├── task.rs
│   │   │   ├── tag.rs
│   │   │   └── hotkey.rs
│   │   ├── storage/       # 存储层
│   │   │   ├── mod.rs
│   │   │   ├── sqlite.rs
│   │   │   └── json.rs
│   │   ├── services/      # 业务逻辑
│   │   │   ├── mod.rs
│   │   │   ├── task_service.rs
│   │   │   ├── project_service.rs
│   │   │   ├── sync_service.rs
│   │   │   └── import_service.rs  # 导入服务 (含 Reminders)
│   │   └── hotkey/       # 快捷键引擎
│   │       ├── mod.rs
│   │       ├── binding.rs
│   │       └── action.rs
│   └── tests/
│
├── gtd-tui/              # TUI 应用 (30%)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── app.rs
│   │   ├── ui/
│   │   │   ├── mod.rs
│   │   │   ├── layout.rs
│   │   │   ├── views/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── inbox.rs
│   │   │   │   ├── today.rs
│   │   │   │   ├── upcoming.rs
│   │   │   │   ├── anytime.rs
│   │   │   │   └── project.rs
│   │   │   └── components/
│   │   │       ├── task.rs
│   │   │       └── sidebar.rs
│   │   ├── commands/     # 命令实现
│   │   │   ├── mod.rs
│   │   │   └── task_commands.rs
│   │   └── config.rs
│   ├── default_hotkeys.toml
│   └── config/
│
├── gtd-macos/            # macOS 应用 (后续)
│   ├── Package.swift
│   ├── Sources/
│   │   └── GtdMacos/
│   │       ├── App/
│   │       ├── Views/
│   │       ├── Bridge/
│   │       └── Resources/
│   └── gtd-macos.entitlements
│
├── docs/                  # 文档
│   ├── SPEC.md
│   ├── HOTKEYS.md
│   ├── DATA_MODEL.md
│   └── IMPORT_GUIDE.md
│
└── README.md
```

---

## 10. 后续工作

### Phase 1: TUI 版本 (当前)
1. 初始化 Rust 项目结构
2. 实现核心数据模型
3. 实现 SQLite 存储
4. 实现基础 TUI 框架
5. 实现视图切换
6. 实现任务 CRUD
7. 实现快捷键系统
8. 添加数据导出/导入

### Phase 2: 数据导入
9. 实现 macOS Reminders 数据导入
   - 通过 ` remindersd ` CLI 或 EventKit 访问 Reminders 数据
   - 支持映射 Reminders 列表到 Area/Project
   - 处理标签、日期、重复等属性

### Phase 3: macOS 原生版本
10. 创建 `gtd-macos` 模块
    - 复用 `gtd-core` 核心库
    - 使用 SwiftUI + AppKit
    - 支持 Touch Bar (如果有)
    - 支持系统快捷键
    - 支持 Native Notifications

---

## 11. macOS Reminders 导入

### 11.1 导入方式

```rust
// Reminders 数据结构映射
struct ReminderImport {
    // Reminders → gtd-tui
    title → title
    notes → notes
    dueDate → due_date
    priority → tag (@priority-high/medium/low)
    listId → area_id 或 project_id
    isCompleted → status
    recurrenceRule → recurrence_rule
    alarm → reminder
}
```

### 11.2 实现方案

1. **EventKit 框架** (需要 macOS 权限)
   - 通过 `EKReminder` API 读取
   - 需要用户授权 Reminders 访问

2. **命令行工具** (备选)
   - 使用 `remindersd` (macOS 14+)
   - 或导出为 .ics 文件再解析

### 11.3 导入流程

```
1. 用户执行 :import reminders
2. 请求 Reminders 访问权限 (首次)
3. 获取所有 Reminders 列表
4. 显示映射配置界面:
   - 选择 Reminders 列表 → Area/Project
   - 导入标签映射
5. 执行导入
6. 显示导入结果 (成功/失败数量)
```

---

## 12. macOS 版本规划 (后续)

### 12.1 技术方案

```
┌─────────────────────────────────────────────────┐
│              gtd-macos (SwiftUI)                │
├─────────────────────────────────────────────────┤
│  UI Layer                                        │
│  - SwiftUI Views                                 │
│  - AppKit Integration (NSWindow)                │
├─────────────────────────────────────────────────┤
│  Bridge Layer (UniFFI)                          │
│  - 调用 gtd-core 编译的 Rust 静态库              │
│  - 暴露 FFI 接口给 Swift                        │
├─────────────────────────────────────────────────┤
│  gtd-core (Rust 共享)                           │
│  - 业务逻辑                                      │
│  - 数据存储                                      │
└─────────────────────────────────────────────────┘
```

### 12.2 功能差异

| 功能 | TUI | macOS |
|------|-----|-------|
| 纯键盘操作 | ✓ 核心特性 | 支持 |
| 自定义快捷键 | ✓ 完整 | 有限 (系统快捷键) |
| 菜单栏 | - | ✓ |
| Touch Bar | - | ✓ |
| 通知 | - | ✓ |
| 系统集成 | - | ✓ (Reminders, Calendar) |

### 12.3 项目结构扩展

```
gtd-tui/
├── gtd-macos/              # macOS 应用
│   ├── Package.swift
│   ├── Sources/
│   │   └── GtdMacos/
│   │       ├── App/
│   │       ├── Views/
│   │       ├── Bridge/
│   │       └── Resources/
│   └── gtd-macos.entitlements
│
└── ...
```

---

*本文档将随项目进展持续更新*
