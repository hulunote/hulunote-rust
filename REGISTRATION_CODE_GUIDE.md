# 注册码系统使用指南

## 概述

本系统已从邮箱验证码注册改为注册码注册。注册码决定用户账号的过期时间。

## 主要变更

### 1. 数据库变更

- 新增 `registration_codes` 表，用于存储注册码
- `accounts` 表新增字段:
  - `expires_at`: 账号过期时间
  - `registration_code`: 使用的注册码

### 2. 注册流程变更

**旧流程**:
- 用户输入邮箱
- 发送验证码到邮箱
- 用户输入 `ack_number` (验证码)
- 创建账号

**新流程**:
- 用户输入邮箱
- 用户输入 `registration_code` (注册码)
- 系统验证注册码是否有效
- 根据注册码的 `validity_days` 设置账号过期时间
- 创建账号并标记注册码为已使用

### 3. 登录流程变更

登录时会检查账号是否过期,如果过期则返回错误信息。

## 生成注册码

使用提供的脚本生成注册码:

### 基本用法

```bash
# 生成6个月有效期的注册码
./scripts/generate_registration_code.sh 6months

# 生成1年有效期的注册码
./scripts/generate_registration_code.sh 1year

# 生成2年有效期的注册码
./scripts/generate_registration_code.sh 2years

# 生成自定义天数的注册码
./scripts/generate_registration_code.sh custom
# 然后按提示输入天数
```

### 示例输出

```
==================================================
  Generating Registration Code
==================================================

Code:          FA8E-AF6E-4578-9347
Validity:      6 months (180 days)

✓ Registration code created successfully!

==================================================
Share this code with users for registration:

  FA8E-AF6E-4578-9347

==================================================
```

## 注册码格式

- 格式: `XXXX-XXXX-XXXX-XXXX`
- 总长度: 19 个字符(包含3个连字符)
- 16个十六进制字符(大写)
- 示例: `FA8E-AF6E-4578-9347`

## 注册码特性

1. **唯一性**: 每个注册码只能使用一次
2. **有效期控制**: 通过 `validity_days` 字段控制用户账号的有效期
3. **使用追踪**: 记录使用注册码的用户ID和使用时间
4. **过期检查**: 登录时自动检查账号是否过期

## API 变更

### 注册接口

**端点**: `POST /api/web-signup`

**请求体变更**:

```json
{
  "username": "optional_username",
  "email": "user@example.com",
  "password": "password123",
  "registration_code": "FA8E-AF6E-4578-9347"  // 改为使用注册码,不再使用 ack_number
}
```

**响应**:

成功:
```json
{
  "token": "jwt_token_here",
  "hulunote": {
    "accounts/id": 1,
    "accounts/username": "user@example.com",
    ...
  },
  "database": "user-1234",
  "region": null
}
```

错误示例:
```json
{
  "error": "Invalid registration code"
}
```

```json
{
  "error": "Registration code has already been used"
}
```

### 登录接口

**端点**: `POST /api/web-login`

新增过期检查,如果账号过期会返回:

```json
{
  "error": "Account has expired"
}
```

## 数据库查询

### 查看所有注册码

```sql
SELECT code, validity_days, is_used, used_by_account_id, used_at, created_at
FROM registration_codes
ORDER BY created_at DESC;
```

### 查看未使用的注册码

```sql
SELECT code, validity_days, created_at
FROM registration_codes
WHERE is_used = false
ORDER BY created_at DESC;
```

### 查看已过期的账号

```sql
SELECT id, username, mail, expires_at
FROM accounts
WHERE expires_at IS NOT NULL AND expires_at < NOW()
ORDER BY expires_at DESC;
```

### 查看即将过期的账号(30天内)

```sql
SELECT id, username, mail, expires_at,
       EXTRACT(DAY FROM (expires_at - NOW())) as days_remaining
FROM accounts
WHERE expires_at IS NOT NULL
  AND expires_at > NOW()
  AND expires_at < NOW() + INTERVAL '30 days'
ORDER BY expires_at;
```

## 环境变量

脚本使用的数据库连接可以通过环境变量配置:

```bash
# 设置数据库连接
export DATABASE_URL="postgresql://user:password@localhost/hulunote_open"

# 然后生成注册码
./scripts/generate_registration_code.sh 1year
```

默认值: `postgresql://localhost/hulunote_open`

## 注意事项

1. **备份注册码**: 生成的注册码需要妥善保管,发给用户使用
2. **一次性使用**: 每个注册码只能使用一次,请确保发给正确的用户
3. **过期管理**: 定期检查即将过期的账号,及时通知用户续期
4. **安全性**: 注册码使用随机生成,确保安全性

## 迁移说明

如果你有现有的用户数据需要迁移:

1. 运行 migration: `psql hulunote_open -f migrations/001_add_registration_codes.sql`
2. 现有用户的 `expires_at` 字段为 `NULL`,表示永不过期
3. 可以通过 SQL 手动设置现有用户的过期时间:

```sql
-- 为现有用户设置1年后过期
UPDATE accounts
SET expires_at = NOW() + INTERVAL '365 days'
WHERE expires_at IS NULL;
```

## 故障排查

### 问题: 注册时提示"Invalid registration code"

解决方案:
1. 检查注册码是否正确输入(区分大小写)
2. 确认注册码存在于数据库中
3. 检查注册码是否已被使用

### 问题: 脚本无法连接数据库

解决方案:
1. 检查 PostgreSQL 是否运行
2. 验证 `DATABASE_URL` 环境变量设置是否正确
3. 确认数据库 `hulunote_open` 存在

### 问题: 登录时提示"Account has expired"

解决方案:
1. 生成新的注册码
2. 管理员可以手动延长用户的过期时间:

```sql
UPDATE accounts
SET expires_at = NOW() + INTERVAL '365 days'
WHERE id = <user_id>;
```
