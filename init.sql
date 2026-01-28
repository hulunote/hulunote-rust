-- =====================================================
-- Hulunote Database Initialization Script
-- Generated: 2026-01-29
-- 
-- This script creates all necessary tables and inserts
-- test data for the Hulunote application.
-- 
-- Test User: chanshunli@gmail.com / 123456
-- =====================================================

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- =====================================================
-- TABLE: accounts
-- =====================================================
CREATE TABLE IF NOT EXISTS accounts (
    id BIGSERIAL PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    nickname TEXT,
    password TEXT,
    mail TEXT UNIQUE,
    address TEXT,
    introduction TEXT,
    avatar TEXT,
    info TEXT,
    need_update_password BOOLEAN DEFAULT false,
    invitation_code TEXT UNIQUE,
    state TEXT,
    cell_number TEXT UNIQUE,
    show_link BOOLEAN DEFAULT false,
    is_new_user BOOLEAN NOT NULL DEFAULT true,
    oauth_key TEXT,
    created_at TIMESTAMP(6) WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP(6) WITH TIME ZONE NOT NULL DEFAULT now()
);

-- =====================================================
-- TABLE: hulunote_databases
-- =====================================================
CREATE TABLE IF NOT EXISTS hulunote_databases (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v1mc(),
    name TEXT NOT NULL,
    description TEXT,
    is_delete BOOLEAN NOT NULL DEFAULT false,
    is_public BOOLEAN NOT NULL DEFAULT false,
    is_offline BOOLEAN NOT NULL DEFAULT false,
    is_default BOOLEAN NOT NULL DEFAULT false,
    bot_group_platform TEXT NOT NULL DEFAULT '',
    account_id BIGSERIAL NOT NULL,
    "favorite-notes" TEXT NOT NULL DEFAULT '[]',
    setting TEXT NOT NULL DEFAULT '{}',
    created_at TIMESTAMP(6) WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP(6) WITH TIME ZONE NOT NULL DEFAULT now()
);

-- =====================================================
-- TABLE: hulunote_notes
-- =====================================================
CREATE TABLE IF NOT EXISTS hulunote_notes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v1mc(),
    title TEXT NOT NULL,
    database_id VARCHAR(36) NOT NULL,
    root_nav_id VARCHAR(36) NOT NULL,
    is_delete BOOLEAN NOT NULL DEFAULT false,
    is_public BOOLEAN NOT NULL DEFAULT false,
    is_shortcut BOOLEAN NOT NULL DEFAULT false,
    current_updater VARCHAR(36),
    account_id BIGSERIAL NOT NULL,
    created_at TIMESTAMP(6) WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP(6) WITH TIME ZONE NOT NULL DEFAULT now(),
    import_uuid VARCHAR(36),
    merged_to VARCHAR(36),
    pv BIGINT NOT NULL DEFAULT 0,
    catalog_id VARCHAR(36),
    UNIQUE(database_id, title)
);

-- =====================================================
-- TABLE: hulunote_navs
-- =====================================================
CREATE TABLE IF NOT EXISTS hulunote_navs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v1mc(),
    parid VARCHAR(36) NOT NULL,
    same_deep_order REAL NOT NULL,
    content TEXT NOT NULL,
    account_id BIGSERIAL NOT NULL,
    note_id VARCHAR(36) NOT NULL,
    database_id VARCHAR(36) NOT NULL,
    is_display BOOLEAN NOT NULL DEFAULT true,
    is_public BOOLEAN NOT NULL DEFAULT false,
    is_delete BOOLEAN NOT NULL DEFAULT false,
    current_updater VARCHAR(36),
    properties TEXT NOT NULL DEFAULT '',
    extra_id TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMP(6) WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP(6) WITH TIME ZONE NOT NULL DEFAULT now()
);

-- =====================================================
-- TEST DATA: Test User
-- Email: chanshunli@gmail.com
-- Password: 123456 (bcrypt hashed)
-- =====================================================
INSERT INTO accounts (
    username, nickname, password, mail, invitation_code, cell_number, is_new_user
) VALUES (
    'chanshunli@gmail.com',
    'Chan Shunli',
    '$2b$12$AoglaYw8qk.D0TuNoNYzxebWSxfw7Q.EeX4wJIKMjrQADh/1zRYRW',
    'chanshunli@gmail.com',
    'TEST8888',
    'test-cell-001',
    false
) ON CONFLICT (username) DO NOTHING;

-- =====================================================
-- TEST DATA: Test Database
-- =====================================================
INSERT INTO hulunote_databases (
    id, name, description, is_delete, is_public, is_offline, is_default, account_id, setting
) VALUES (
    'a1b2c3d4-e5f6-7890-abcd-ef1234567890'::uuid,
    'Test Knowledge Base',
    '测试知识库 - A test database for chanshunli@gmail.com',
    false, false, false, true,
    (SELECT id FROM accounts WHERE mail = 'chanshunli@gmail.com'),
    '{}'
) ON CONFLICT (id) DO NOTHING;

-- =====================================================
-- TEST DATA: Welcome Note
-- =====================================================
INSERT INTO hulunote_notes (
    id, title, database_id, root_nav_id, is_delete, is_public, is_shortcut, account_id, pv
) VALUES (
    '11111111-2222-3333-4444-555555555555'::uuid,
    '我的测试笔记 - Welcome Note',
    'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
    'aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee',
    false, false, false,
    (SELECT id FROM accounts WHERE mail = 'chanshunli@gmail.com'),
    0
) ON CONFLICT (id) DO NOTHING;

-- =====================================================
-- TEST DATA: Diary Note (2026-01-29)
-- =====================================================
INSERT INTO hulunote_notes (
    id, title, database_id, root_nav_id, is_delete, is_public, is_shortcut, account_id, pv
) VALUES (
    '22222222-3333-4444-5555-666666666666'::uuid,
    '2026-01-29',
    'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
    'bbbbbbbb-cccc-dddd-eeee-ffffffffffff',
    false, false, false,
    (SELECT id FROM accounts WHERE mail = 'chanshunli@gmail.com'),
    0
) ON CONFLICT (id) DO NOTHING;

-- =====================================================
-- TEST DATA: Nav nodes for Welcome Note
-- =====================================================
-- Root nav
INSERT INTO hulunote_navs (id, parid, same_deep_order, content, account_id, note_id, database_id, properties, extra_id)
SELECT 'aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee'::uuid, '00000000-0000-0000-0000-000000000000', 0, 'ROOT',
       id, '11111111-2222-3333-4444-555555555555', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', '', ''
FROM accounts WHERE mail = 'chanshunli@gmail.com'
ON CONFLICT (id) DO NOTHING;

-- Level 1: 欢迎
INSERT INTO hulunote_navs (id, parid, same_deep_order, content, account_id, note_id, database_id, properties, extra_id)
SELECT '00000001-0001-0001-0001-000000000001'::uuid, 'aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee', 1.0,
       '欢迎使用 Hulunote! 这是一个支持大纲结构的笔记应用 🎉',
       id, '11111111-2222-3333-4444-555555555555', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', '', ''
FROM accounts WHERE mail = 'chanshunli@gmail.com'
ON CONFLICT (id) DO NOTHING;

-- Level 1: 功能介绍
INSERT INTO hulunote_navs (id, parid, same_deep_order, content, account_id, note_id, database_id, properties, extra_id)
SELECT '00000001-0001-0001-0001-000000000002'::uuid, 'aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee', 2.0,
       '主要功能介绍',
       id, '11111111-2222-3333-4444-555555555555', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', '', ''
FROM accounts WHERE mail = 'chanshunli@gmail.com'
ON CONFLICT (id) DO NOTHING;

-- Level 2: 大纲结构 (under 功能介绍)
INSERT INTO hulunote_navs (id, parid, same_deep_order, content, account_id, note_id, database_id, properties, extra_id)
SELECT '00000001-0001-0001-0001-000000000003'::uuid, '00000001-0001-0001-0001-000000000002', 1.0,
       '📝 支持无限层级的大纲结构',
       id, '11111111-2222-3333-4444-555555555555', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', '', ''
FROM accounts WHERE mail = 'chanshunli@gmail.com'
ON CONFLICT (id) DO NOTHING;

-- Level 2: 多数据库 (under 功能介绍)
INSERT INTO hulunote_navs (id, parid, same_deep_order, content, account_id, note_id, database_id, properties, extra_id)
SELECT '00000001-0001-0001-0001-000000000004'::uuid, '00000001-0001-0001-0001-000000000002', 2.0,
       '📚 支持多数据库管理，每个数据库可以有多个笔记',
       id, '11111111-2222-3333-4444-555555555555', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', '', ''
FROM accounts WHERE mail = 'chanshunli@gmail.com'
ON CONFLICT (id) DO NOTHING;

-- Level 2: 实时同步 (under 功能介绍)
INSERT INTO hulunote_navs (id, parid, same_deep_order, content, account_id, note_id, database_id, properties, extra_id)
SELECT '00000001-0001-0001-0001-000000000005'::uuid, '00000001-0001-0001-0001-000000000002', 3.0,
       '🔄 支持实时同步和离线编辑',
       id, '11111111-2222-3333-4444-555555555555', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', '', ''
FROM accounts WHERE mail = 'chanshunli@gmail.com'
ON CONFLICT (id) DO NOTHING;

-- Level 1: 使用技巧
INSERT INTO hulunote_navs (id, parid, same_deep_order, content, account_id, note_id, database_id, properties, extra_id)
SELECT '00000001-0001-0001-0001-000000000006'::uuid, 'aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee', 3.0,
       '使用技巧',
       id, '11111111-2222-3333-4444-555555555555', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', '', ''
FROM accounts WHERE mail = 'chanshunli@gmail.com'
ON CONFLICT (id) DO NOTHING;

-- Level 2: Tab键 (under 使用技巧)
INSERT INTO hulunote_navs (id, parid, same_deep_order, content, account_id, note_id, database_id, properties, extra_id)
SELECT '00000001-0001-0001-0001-000000000007'::uuid, '00000001-0001-0001-0001-000000000006', 1.0,
       '⌨️ 使用 Tab 键可以快速缩进创建子节点',
       id, '11111111-2222-3333-4444-555555555555', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', '', ''
FROM accounts WHERE mail = 'chanshunli@gmail.com'
ON CONFLICT (id) DO NOTHING;

-- Level 2: 双向链接 (under 使用技巧)
INSERT INTO hulunote_navs (id, parid, same_deep_order, content, account_id, note_id, database_id, properties, extra_id)
SELECT '00000001-0001-0001-0001-000000000008'::uuid, '00000001-0001-0001-0001-000000000006', 2.0,
       '🔗 使用 [[]] 语法可以创建双向链接',
       id, '11111111-2222-3333-4444-555555555555', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', '', ''
FROM accounts WHERE mail = 'chanshunli@gmail.com'
ON CONFLICT (id) DO NOTHING;

-- Level 3: 链接示例 (under 双向链接)
INSERT INTO hulunote_navs (id, parid, same_deep_order, content, account_id, note_id, database_id, properties, extra_id)
SELECT '00000001-0001-0001-0001-000000000009'::uuid, '00000001-0001-0001-0001-000000000008', 1.0,
       '例如: [[另一篇笔记]] 会自动链接到对应的笔记',
       id, '11111111-2222-3333-4444-555555555555', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', '', ''
FROM accounts WHERE mail = 'chanshunli@gmail.com'
ON CONFLICT (id) DO NOTHING;

-- Level 1: 开始使用
INSERT INTO hulunote_navs (id, parid, same_deep_order, content, account_id, note_id, database_id, properties, extra_id)
SELECT '00000001-0001-0001-0001-000000000010'::uuid, 'aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee', 4.0,
       '🚀 开始创建你的第一篇笔记吧!',
       id, '11111111-2222-3333-4444-555555555555', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', '', ''
FROM accounts WHERE mail = 'chanshunli@gmail.com'
ON CONFLICT (id) DO NOTHING;

-- =====================================================
-- TEST DATA: Nav nodes for Diary Note (2026-01-29)
-- =====================================================
-- Root nav
INSERT INTO hulunote_navs (id, parid, same_deep_order, content, account_id, note_id, database_id, properties, extra_id)
SELECT 'bbbbbbbb-cccc-dddd-eeee-ffffffffffff'::uuid, '00000000-0000-0000-0000-000000000000', 0, 'ROOT',
       id, '22222222-3333-4444-5555-666666666666', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', '', ''
FROM accounts WHERE mail = 'chanshunli@gmail.com'
ON CONFLICT (id) DO NOTHING;

-- Level 1: 今日待办
INSERT INTO hulunote_navs (id, parid, same_deep_order, content, account_id, note_id, database_id, properties, extra_id)
SELECT '00000002-0001-0001-0001-000000000001'::uuid, 'bbbbbbbb-cccc-dddd-eeee-ffffffffffff', 1.0,
       '今日待办',
       id, '22222222-3333-4444-5555-666666666666', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', '', ''
FROM accounts WHERE mail = 'chanshunli@gmail.com'
ON CONFLICT (id) DO NOTHING;

-- Level 2: 测试后端 (under 今日待办)
INSERT INTO hulunote_navs (id, parid, same_deep_order, content, account_id, note_id, database_id, properties, extra_id)
SELECT '00000002-0001-0001-0001-000000000002'::uuid, '00000002-0001-0001-0001-000000000001', 1.0,
       '✅ 测试 Hulunote Rust 后端',
       id, '22222222-3333-4444-5555-666666666666', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', '', ''
FROM accounts WHERE mail = 'chanshunli@gmail.com'
ON CONFLICT (id) DO NOTHING;

-- Level 2: 完善功能 (under 今日待办)
INSERT INTO hulunote_navs (id, parid, same_deep_order, content, account_id, note_id, database_id, properties, extra_id)
SELECT '00000002-0001-0001-0001-000000000003'::uuid, '00000002-0001-0001-0001-000000000001', 2.0,
       '⬜ 完善笔记功能',
       id, '22222222-3333-4444-5555-666666666666', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', '', ''
FROM accounts WHERE mail = 'chanshunli@gmail.com'
ON CONFLICT (id) DO NOTHING;

-- Level 1: 今日笔记
INSERT INTO hulunote_navs (id, parid, same_deep_order, content, account_id, note_id, database_id, properties, extra_id)
SELECT '00000002-0001-0001-0001-000000000004'::uuid, 'bbbbbbbb-cccc-dddd-eeee-ffffffffffff', 2.0,
       '今日笔记',
       id, '22222222-3333-4444-5555-666666666666', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', '', ''
FROM accounts WHERE mail = 'chanshunli@gmail.com'
ON CONFLICT (id) DO NOTHING;

-- Level 2: 记录想法 (under 今日笔记)
INSERT INTO hulunote_navs (id, parid, same_deep_order, content, account_id, note_id, database_id, properties, extra_id)
SELECT '00000002-0001-0001-0001-000000000005'::uuid, '00000002-0001-0001-0001-000000000004', 1.0,
       '开始使用 Hulunote 记录每日工作和想法 📝',
       id, '22222222-3333-4444-5555-666666666666', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', '', ''
FROM accounts WHERE mail = 'chanshunli@gmail.com'
ON CONFLICT (id) DO NOTHING;

-- Level 2: 大纲日记 (under 今日笔记)
INSERT INTO hulunote_navs (id, parid, same_deep_order, content, account_id, note_id, database_id, properties, extra_id)
SELECT '00000002-0001-0001-0001-000000000006'::uuid, '00000002-0001-0001-0001-000000000004', 2.0,
       '这是一个支持大纲结构的日记系统 🎯',
       id, '22222222-3333-4444-5555-666666666666', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890', '', ''
FROM accounts WHERE mail = 'chanshunli@gmail.com'
ON CONFLICT (id) DO NOTHING;

-- =====================================================
-- END OF INITIALIZATION SCRIPT
-- =====================================================
