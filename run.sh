#!/bin/bash

# 设置环境变量（如果没有 .env 文件）
##export DATABASE_URL="${DATABASE_URL:-postgres://postgres:password@localhost:5432/hulunote}"
export JWT_SECRET="${JWT_SECRET:-hulunote-secret-key}"
export PORT="${PORT:-6689}"
export RUST_LOG="${RUST_LOG:-hulunote_server=debug,tower_http=debug}"

# 检查是否需要复制前端文件
if [ ! -d "resources/public/hulunote" ]; then
    echo "Copying frontend files from original project..."
    mkdir -p resources/public
    if [ -d "../hulunote/resources/public/hulunote" ]; then
        cp -r ../hulunote/resources/public/hulunote resources/public/
        echo "Frontend files copied successfully."
    else
        echo "Warning: Frontend files not found. Please build them first:"
        echo "  cd ../hulunote && npx shadow-cljs release hulunote"
    fi
fi

# 运行服务器
cargo run
