# Clip Verse 开发文档

## 项目概述

Clip Verse 是一个基于 Tauri + React + Rust 的剪贴板监控系统，实时监控并记录系统剪贴板内容，支持多种数据类型和灵活的存储策略。

## 技术栈

### 前端
- **React 18** - UI 框架
- **TypeScript** - 类型安全
- **Vite** - 构建工具
- **Chakra UI** - UI 组件库
- **Emotion** - CSS-in-JS 样式库 (Chakra UI 依赖)


### 后端 (Rust)
- **Tauri** - 桌面应用框架
- **rusqlite** - SQLite 数据库
- **AES 加密** - 可选加密功能

## 核心功能

### 1. 剪贴板监控

**自动监控机制**：程序启动后会自动后台运行，实时监控剪贴板变化，无需任何手动操作。当用户复制任何内容时，系统会自动检测并保存到数据库。

#### 工作流程
1. 程序启动时自动启动监控线程
2. 定期轮询（默认 500ms）检查剪贴板变化
3. 通过内容哈希值去重，避免重复保存相同内容
4. 检测到变化后自动保存到数据库
5. 图片文件自动保存到指定目录
6. 可选：生成缩略图、加密保存等

#### 支持的数据类型

### 2. 数据存储

#### 时区设置
项目统一使用 **中国时区 (Asia/Shanghai, UTC+8)** 存储所有时间，无需前端转换。

#### SQLite 数据库结构

```sql
-- 剪贴板记录表
CREATE TABLE clipboard_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content_type TEXT NOT NULL,  -- 'text', 'image', 'file', 'rich_text'
    timestamp INTEGER NOT NULL,  -- Unix 时间戳(毫秒，中国时区)
    created_at TEXT NOT NULL,    -- 中国时区 ISO 8601: 2024-01-15T18:30:45.123+08:00
    updated_at TEXT NOT NULL,    -- 最后更新时间(中国时区)
    preview TEXT,                -- 文本预览或文件名
    content_size INTEGER,        -- 内容大小(字节)
    is_encrypted BOOLEAN DEFAULT 0,
    is_favorite BOOLEAN DEFAULT 0
);

-- 文本内容表
CREATE TABLE text_contents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    record_id INTEGER NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,  -- 中国时区
    updated_at TEXT NOT NULL,  -- 中国时区
    FOREIGN KEY (record_id) REFERENCES clipboard_records(id) ON DELETE CASCADE
);

-- 图片引用表
CREATE TABLE image_contents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    record_id INTEGER NOT NULL,
    file_path TEXT NOT NULL,        -- 实际文件路径
    thumbnail_path TEXT,            -- 缩略图路径
    width INTEGER,
    height INTEGER,
    format TEXT,                    -- 'png', 'bmp', 'jpeg'
    is_encrypted BOOLEAN DEFAULT 0,
    created_at TEXT NOT NULL,      -- 中国时区
    updated_at TEXT NOT NULL,      -- 中国时区
    FOREIGN KEY (record_id) REFERENCES clipboard_records(id) ON DELETE CASCADE
);

-- 文件内容表
CREATE TABLE file_contents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    record_id INTEGER NOT NULL,
    file_paths TEXT NOT NULL,       -- JSON 数组格式: ["path1", "path2"]
    total_size INTEGER,
    file_count INTEGER,
    created_at TEXT NOT NULL,      -- 中国时区
    updated_at TEXT NOT NULL,      -- 中国时区
    FOREIGN KEY (record_id) REFERENCES clipboard_records(id) ON DELETE CASCADE
);

-- 富文本内容表
CREATE TABLE rich_text_contents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    record_id INTEGER NOT NULL,
    html_content TEXT,
    rtf_content TEXT,
    created_at TEXT NOT NULL,      -- 中国时区
    updated_at TEXT NOT NULL,      -- 中国时区
    FOREIGN KEY (record_id) REFERENCES clipboard_records(id) ON DELETE CASCADE
);

-- 配置表
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL  -- 中国时区
);

-- 索引
CREATE INDEX idx_clipboard_records_timestamp ON clipboard_records(timestamp DESC);
CREATE INDEX idx_clipboard_records_content_type ON clipboard_records(content_type);
CREATE INDEX idx_clipboard_records_created_at ON clipboard_records(created_at);
CREATE INDEX idx_text_contents_record_id ON text_contents(record_id);
CREATE INDEX idx_image_contents_record_id ON image_contents(record_id);
CREATE INDEX idx_file_contents_record_id ON file_contents(record_id);
CREATE INDEX idx_rich_text_contents_record_id ON rich_text_contents(record_id);
```

### 3. 文件存储策略

#### 文件目录结构
```
~/.clip_verse/
├── database/
│   └── clipboard.db              # SQLite 数据库
├── images/
│   ├── raw/                      # 原始图片文件
│   │   ├── YYYY/
│   │   │   └── MM/
│   │   │       └── image_{uuid}.png
│   └── thumbnails/               # 缩略图文件
│       ├── YYYY/
│       │   └── MM/
│       │       └── thumb_{uuid}.png
├── encrypted/                    # 加密文件存储区
│   ├── images/
│   │   └── encrypted_{uuid}.enc
│   └── config.json
└── logs/
    └── app.log
```

#### 文件命名规则
- 原始图片: `image_{uuid}_{timestamp}.png`
- 缩略图: `thumb_{uuid}_{timestamp}.png`
- 加密文件: `encrypted_{uuid}.enc`

### 4. 配置选项

```json
{
  "security": {
    "enable_image_encryption": false,
    "safe_mode": false,
    "max_image_size_mb": 5,
    "encryption_key": null
  },
  "storage": {
    "max_records": 10000,
    "auto_cleanup_days": 30,
    "thumbnail_enabled": true,
    "thumbnail_size": [200, 200]
  },
  "monitoring": {
    "enable_text_monitoring": true,
    "enable_image_monitoring": true,
    "enable_file_monitoring": true,
    "debounce_interval_ms": 500
  },
  "paths": {
    "data_dir": "~/.clip_verse",
    "custom_image_dir": null,
    "custom_encrypted_dir": null
  }
}
```

## 核心功能模块

### 1. 剪贴板监控器 (src-tauri/src/clipboard/monitor.rs)

**自动监控实现**：监控器在后台线程中持续运行，无需用户干预。

```rust
use clipboard::{ClipboardContext, ClipboardProvider};
use tokio::time::{interval, sleep};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct ClipboardMonitor {
    interval: Duration,              // 检测间隔（默认 500ms）
    last_hash: Option<u64>,         // 上次内容哈希值
    is_running: bool,               // 运行状态
}

pub enum ClipboardEvent {
    Text(String),
    Image(Vec<u8>, String),  // bytes, format
    Files(Vec<String>),
    RichText { html: Option<String>, rtf: Option<String> },
}

impl ClipboardMonitor {
    pub fn new(interval: Duration) -> Self {
        ClipboardMonitor {
            interval,
            last_hash: None,
            is_running: false,
        }
    }

    /// 启动自动监控循环
    pub async fn start(&mut self) -> Result<(), Error> {
        self.is_running = true;

        while self.is_running {
            if let Some(event) = self.check_clipboard()? {
                // 自动保存到数据库
                self.save_event(event).await?;
            }

            // 等待下次检测
            sleep(self.interval).await;
        }

        Ok(())
    }

    /// 停止监控
    pub fn stop(&mut self) {
        self.is_running = false;
    }

    /// 检测剪贴板变化
    fn check_clipboard(&mut self) -> Option<ClipboardEvent> {
        let ctx: ClipboardContext = ClipboardProvider::new().ok()?;

        // 获取当前剪贴板内容
        let content = ctx.get_contents().ok()?;
        let current_hash = self.calculate_hash(&content);

        // 通过哈希值去重
        if Some(current_hash) == self.last_hash {
            return None;  // 内容未变化
        }

        self.last_hash = Some(current_hash);

        // 检测内容类型并返回事件
        self.detect_content_type(&content)
    }

    /// 计算内容哈希值（用于去重）
    fn calculate_hash(&self, content: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }

    /// 保存事件到数据库（自动执行）
    async fn save_event(&self, event: ClipboardEvent) -> Result<(), Error> {
        // 调用数据库管理器保存
        database::save_clipboard_event(event).await
    }
}
```

**使用示例**：

```rust
#[tokio::main]
async fn main() {
    let mut monitor = ClipboardMonitor::new(Duration::from_millis(500));

    // 启动监控（程序启动时自动执行）
    tokio::spawn(async move {
        monitor.start().await.unwrap();
    });

    // 主程序继续运行...
}
```

### 2. 数据库管理器 (src-tauri/src/database/mod.rs)

```rust
use rusqlite::{Connection, params};

pub struct DatabaseManager {
    conn: Connection,
}

impl DatabaseManager {
    pub fn new(path: &Path) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        conn.execute_batch(INIT_SQL)?;
        Ok(DatabaseManager { conn })
    }

    pub fn save_text_record(&self, text: &str) -> Result<i64, Error> {
        // 保存文本记录
    }

    pub fn save_image_record(
        &self,
        file_path: &str,
        thumbnail_path: Option<&str>,
        width: i32,
        height: i32,
    ) -> Result<i64, Error> {
        // 保存图片引用记录
    }

    pub fn save_file_record(&self, files: &[String]) -> Result<i64, Error> {
        // 保存文件记录
    }

    pub fn get_records(&self, limit: usize, offset: usize) -> Result<Vec<Record>, Error> {
        // 获取记录列表
    }
}
```

### 3. 文件存储管理器 (src-tauri/src/storage/mod.rs)

```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};

pub struct StorageManager {
    base_dir: PathBuf,
    encryption_key: Option<[u8; 32]>,
}

impl StorageManager {
    pub fn save_image(&self, data: &[u8], format: &str) -> Result<String, Error> {
        // 保存原始图片
    }

    pub fn generate_thumbnail(&self, source: &Path) -> Result<String, Error> {
        // 生成缩略图
    }

    pub fn encrypt_file(&self, source: &Path) -> Result<String, Error> {
        // 加密文件
    }

    pub fn decrypt_file(&self, encrypted: &Path) -> Result<Vec<u8>, Error> {
        // 解密文件
    }

    pub fn cleanup_old_files(&self, days: i32) -> Result<u64, Error> {
        // 清理旧文件
    }
}
```

### 4. 安全模块 (src-tauri/src/security/mod.rs)

```rust
pub struct SecurityManager {
    settings: SecuritySettings,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    pub enable_image_encryption: bool,
    pub safe_mode: bool,
    pub max_image_size: usize,
}

impl SecurityManager {
    pub fn should_store_image(&self, size: usize) -> bool {
        // 判断是否应该存储图片
        if self.safe_mode {
            return false;
        }
        if size > self.max_image_size {
            return false;
        }
        true
    }

    pub fn encrypt_data(&self, data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, Error> {
        // 加密数据
    }

    pub fn generate_encryption_key() -> [u8; 32] {
        // 生成加密密钥
    }
}
```

## API 接口设计

### Tauri Commands

```rust
#[tauri::command]
async fn save_clipboard_event(event: ClipboardEvent) -> Result<i64, String> {
    // 保存剪贴板事件
}

#[tauri::command]
async fn get_records(limit: usize, offset: usize) -> Result<Vec<RecordDto>, String> {
    // 获取记录列表
}

#[tauri::command]
async fn search_records(query: String) -> Result<Vec<RecordDto>, String> {
    // 搜索记录
}

#[tauri::command]
async fn delete_record(id: i64) -> Result<(), String> {
    // 删除记录
}

#[tauri::command]
async fn get_settings() -> Result<Settings, String> {
    // 获取设置
}

#[tauri::command]
async fn update_settings(settings: Settings) -> Result<(), String> {
    // 更新设置
}

#[tauri::command]
async fn cleanup_old_images(days: i32) -> Result<u64, String> {
    // 清理旧图片
}

#[tauri::command]
async fn get_image_thumbnail(id: i64) -> Result<String, String> {
    // 获取图片缩略图路径
}

#[tauri::command]
async fn decrypt_image(id: i64, password: String) -> Result<Vec<u8>, String> {
    // 解密图片
}
```

## 前端组件设计

### 主要组件

#### 1. App 组件
- 应用入口
- 状态管理
- 路由配置

#### 2. Dashboard 组件
- 记录列表展示（使用 `ChakraProvider`, `Box`, `Stack`）
- 搜索过滤（使用 `InputGroup`, `InputLeftElement`, `Input`）
- 分页（使用 `Pagination`）
- 响应式布局（使用 `useBreakpointValue`）

#### 3. RecordItem 组件
- 单条记录展示（使用 `Card`, `CardBody`, `CardHeader`）
- 文本/图片/文件内容预览（使用 `Image`, `Text`, `Code`）
- 操作按钮（使用 `IconButton`, `Button`, `Menu`, `MenuItem`）
- 收藏功能（使用 `Icon`, `Tooltip`）

#### 4. Settings 组件
- 配置管理界面（使用 `Tabs`, `TabList`, `TabPanel`）
- 安全设置（使用 `Switch`, `FormControl`, `FormLabel`）
- 存储设置（使用 `Slider`, `NumberInput`）
- 表单验证（使用 `useForm` 或 Chakra 表单组件）

#### 5. ImageViewer 组件
- 图片查看器（使用 `Modal`, `Image`, `Box`, `Flex`）
- 缩放、旋转（使用 `useDisclosure`, `Transform`）
- 原图查看（使用 `useLazyLoad`, `Spinner`）

#### 6. FileViewer 组件
- 文件列表展示（使用 `List`, `ListItem`, `ListIcon`）
- 文件操作（使用 `ButtonGroup`, `Menu`）

### Chakra UI 常用组件

- **布局**: `Box`, `Container`, `Stack`, `Flex`, `Grid`, `SimpleGrid`
- **排版**: `Text`, `Heading`, `Code`, `Link`, `Divider`
- **表单**: `Input`, `Textarea`, `Select`, `Checkbox`, `Radio`, `Switch`
- **按钮**: `Button`, `IconButton`, `ButtonGroup`
- **反馈**: `Alert`, `Spinner`, `Progress`, `Toast`
- **导航**: `Tabs`, `Breadcrumb`, `Menu`, `Dropdown`
- **数据展示**: `Table`, `Badge`, `Tag`, `Tooltip`
- **覆盖**: `Modal`, `Drawer`, `Popover`, `AlertDialog`
- **主题**: `useColorMode`, `useTheme`, `extendTheme`

## 开发流程

### 环境准备

1. 安装 Rust
2. 安装 Node.js


### 开发命令

```bash
# 安装依赖
npm install

# 开发模式
npm run tauri dev

# 构建
npm run tauri build

# 仅运行前端开发服务器
npm run dev
```

### 测试

```bash
# Rust 单元测试
cargo test

# Rust 集成测试
cargo test --test '*'

# 前端测试
npm test
```

## 安全注意事项

### 1. 敏感信息处理
- **密码二维码**: 建议开启加密或安全模式
- **私钥截图**: 必须加密存储
- **大文件**: 设置大小限制，避免磁盘占用过大

### 2. 加密策略
- 使用 AES-256-GCM 加密
- 密钥由用户设置或自动生成
- 加密文件单独存储在 `encrypted/` 目录

### 3. 权限控制
- 文件访问权限
- 数据库读写权限
- 加密目录访问限制

### 4. 数据清理
- 自动清理旧记录
- 用户手动清理
- 定期清理临时文件

## 性能优化

### 1. 数据库优化
- 添加索引
- 分页查询
- 连接池

### 2. 文件存储优化
- 异步保存
- 延迟生成缩略图
- 压缩存储

### 3. 内存优化
- 图片懒加载
- 虚拟滚动
- 缓存策略

## 部署

### 打包配置

修改 `src-tauri/tauri.conf.json`:

```json
{
  "productName": "Clip Verse",
  "version": "1.0.0",
  "identifier": "com.clipverse.app",
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:1420",
    "frontendDist": "../dist"
  }
}
```

### 构建

```bash
# 构建 Windows 版本
npm run tauri build --target x86_64-pc-windows-msvc

# 构建 Linux 版本
npm run tauri build --target x86_64-unknown-linux-gnu

# 构建 macOS 版本
npm run tauri build --target x86_64-apple-darwin
```

## 常见问题

### Q: 如何启用图片加密？
A: 在设置中开启 "图片加密" 选项，并设置加密密钥。

### Q: 如何清理旧图片？
A: 在设置中设置自动清理天数，或手动执行清理命令。

### Q: 自定义存储路径？
A: 在设置中配置 "自定义图片目录" 和 "自定义加密目录"。

### Q: 安全模式的区别？
A: 安全模式下不记录任何图片，仅记录文本和文件路径。

## 贡献指南

1. Fork 项目
2. 创建特性分支
3. 提交更改
4. 推送到分支
5. 创建 Pull Request

## 许可证

MIT License

## 联系方式

- GitHub: https://github.com/alexnirvana/clip_verse
- Issues: https://github.com/alexnirvana/clip_verse/issues
