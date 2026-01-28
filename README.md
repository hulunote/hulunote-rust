# Hulunote - Rust Backend

这是 Hulunote 大纲笔记应用的 Rust 后端实现，替换原有的 Clojure 后端。

## 项目结构

```
hulunote-rust/
├── Cargo.toml              # Rust 依赖配置
├── src/
│   ├── main.rs             # 入口文件
│   ├── config.rs           # 配置管理
│   ├── db.rs               # 数据库连接池
│   ├── error.rs            # 错误处理
│   ├── middleware.rs       # JWT 认证中间件
│   ├── models.rs           # 数据模型
│   ├── routes.rs           # 路由配置
│   └── handlers/           # API 处理函数
│       ├── mod.rs
│       ├── auth.rs         # 登录/注册
│       ├── database.rs     # 笔记库操作
│       ├── note.rs         # 笔记操作
│       └── nav.rs          # 大纲节点操作
├── resources/
│   └── public/             # 静态文件 (前端 CLJS 编译输出)
└── .env.example            # 环境变量示例
```

## 技术栈

- **Web Framework**: Axum 0.7
- **Database**: PostgreSQL + SQLx
- **Authentication**: JWT (jsonwebtoken)
- **Password Hashing**: bcrypt
- **Async Runtime**: Tokio

## 快速开始

### 1. 环境准备

```bash
# 复制环境配置
cp .env.example .env

# 编辑 .env 文件，配置数据库连接
vim .env
```

### 2. 数据库初始化

```bash
createdb -U postgres hulunote_open
psql -U postgres -d hulunote_open -f init.sql
```

### 3. 编译运行

```bash
# 开发模式
cargo run

# 发布模式
cargo build --release
./target/release/hulunote-server
```

### 4. 前端构建

前端仍然使用 ClojureScript，需要在原项目中构建：

```bash
cd ../hulunote
npx shadow-cljs release hulunote

# 复制编译输出到 Rust 项目
cp -r resources/public/hulunote ../hulunote-rust/resources/public/
```

## API 接口

### 认证接口 (无需登录)

- `POST /login/web-login` - 登录
- `POST /login/web-signup` - 注册
- `POST /login/send-ack-msg` - 发送验证码

### 笔记库接口 (需要登录)

- `POST /hulunote/new-database` - 创建笔记库
- `POST /hulunote/get-database-list` - 获取笔记库列表
- `POST /hulunote/update-database` - 更新笔记库

### 笔记接口 (需要登录)

- `POST /hulunote/new-note` - 创建笔记
- `POST /hulunote/get-note-list` - 分页获取笔记
- `POST /hulunote/get-all-note-list` - 获取所有笔记
- `POST /hulunote/update-hulunote-note` - 更新笔记

### 大纲节点接口 (需要登录)

- `POST /hulunote/create-or-update-nav` - 创建/更新节点
- `POST /hulunote/get-note-navs` - 获取笔记的节点
- `POST /hulunote/get-all-nav-by-page` - 分页获取所有节点
- `POST /hulunote/get-all-navs` - 获取所有节点

## 与原 Clojure 后端的兼容性

此 Rust 后端实现了与原 Clojure 后端相同的 API 接口和数据格式，可以：

1. 使用相同的 PostgreSQL 数据库
2. 使用相同的前端 ClojureScript 代码
3. 返回相同格式的 JSON 响应 (kebab-case 字段名)

## 性能优势

- 更低的内存占用
- 更快的启动速度
- 更高的并发处理能力
- 零 GC 停顿
