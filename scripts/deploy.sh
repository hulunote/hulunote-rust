#!/bin/bash

# =============================================================================
# Hulunote 部署脚本
# 部署前端(ClojureScript)和后端(Rust)到远程服务器
# =============================================================================

# for cljs compile
export PATH="/usr/local/opt/openjdk@8/bin:$PATH"
export CPPFLAGS="-I/usr/local/opt/openjdk@8/include"

set -e  # 遇到错误立即退出

# 配置变量
REMOTE_HOST="root@104.244.95.160"
REMOTE_APP_DIR="/root/app"
LOCAL_FRONTEND_DIR="/Users/xlisp/CljPro/hulunote"
LOCAL_BACKEND_DIR="/Users/xlisp/CljPro/hulunote-rust"
BINARY_NAME="hulunote-server"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# =============================================================================
# 步骤 1: 编译前端 (ClojureScript with shadow-cljs)
# =============================================================================
build_frontend() {
    log_info "开始编译前端..."
    cd "$LOCAL_FRONTEND_DIR"
    
    # 检查 node_modules 是否存在
    if [ ! -d "node_modules" ]; then
        log_info "安装前端依赖..."
        yarn install
    fi
    
    # 使用 shadow-cljs 编译 release 版本
    log_info "执行 shadow-cljs release 编译..."
    npx shadow-cljs release hulunote
    
    log_success "前端编译完成！"
}

# =============================================================================
# 步骤 2: 编译后端 (Rust - 交叉编译到 Linux)
# =============================================================================
build_backend() {
    log_info "开始编译后端 (Rust)..."
    cd "$LOCAL_BACKEND_DIR"
    
    # 检查是否安装了 Linux 目标
    if ! rustup target list --installed | grep -q "x86_64-unknown-linux-musl"; then
        log_info "安装 Linux MUSL 目标..."
        rustup target add x86_64-unknown-linux-musl
    fi
    
    # 检查是否安装了 musl-cross (macOS 交叉编译工具链)
    if ! command -v x86_64-linux-musl-gcc &> /dev/null; then
        log_warning "未找到 musl-cross 工具链，尝试安装..."
        log_info "正在通过 Homebrew 安装 musl-cross..."
        brew install FiloSottile/musl-cross/musl-cross || {
            log_error "无法安装 musl-cross。请手动安装: brew install FiloSottile/musl-cross/musl-cross"
            log_info "或者选择在服务器上编译..."
            return 1
        }
    fi
    
    # 设置交叉编译环境变量
    export CC_x86_64_unknown_linux_musl=x86_64-linux-musl-gcc
    export CXX_x86_64_unknown_linux_musl=x86_64-linux-musl-g++
    export AR_x86_64_unknown_linux_musl=x86_64-linux-musl-ar
    export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=x86_64-linux-musl-gcc
    
    # 编译 release 版本
    log_info "执行 cargo build --release (目标: x86_64-unknown-linux-musl)..."
    cargo build --release --target x86_64-unknown-linux-musl
    
    log_success "后端编译完成！"
    log_info "二进制文件位置: target/x86_64-unknown-linux-musl/release/$BINARY_NAME"
}

# apt install postgresql && apt install rsync
# =============================================================================
# 步骤 2b: 在服务器上编译后端 (备选方案)
# =============================================================================
build_backend_on_server() {
    log_info "在服务器上编译后端..."
    
    # 首先上传源代码
    log_info "上传 Rust 源代码到服务器..."
    ssh "$REMOTE_HOST" "mkdir -p $REMOTE_APP_DIR/build-src"
    
    rsync -avz --progress \
        --exclude 'target' \
        --exclude '.git' \
        "$LOCAL_BACKEND_DIR/" "$REMOTE_HOST:$REMOTE_APP_DIR/build-src/"
    
    # 在服务器上编译
    log_info "在服务器上执行 cargo build..."
    ssh "$REMOTE_HOST" << 'EOF'
        cd /root/app/build-src
        
        # 检查 Rust 是否安装
        if ! command -v cargo &> /dev/null; then
            echo "安装 Rust..."
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
            source $HOME/.cargo/env
        fi
        
        # 编译
        source $HOME/.cargo/env
        cargo build --release
        
        # 复制二进制文件
        cp target/release/hulunote-server /root/app/
EOF
    
    log_success "服务器端编译完成！"
}

# =============================================================================
# 步骤 3: 上传文件到服务器
# =============================================================================
upload_files() {
    log_info "开始上传文件到服务器..."
    
    # 创建远程目录结构
    ssh "$REMOTE_HOST" "mkdir -p $REMOTE_APP_DIR/resources/public"
    
    # 上传编译好的二进制文件 (如果存在)
    BINARY_PATH="$LOCAL_BACKEND_DIR/target/x86_64-unknown-linux-musl/release/$BINARY_NAME"
    if [ -f "$BINARY_PATH" ]; then
        log_info "上传二进制文件..."
        scp "$BINARY_PATH" "$REMOTE_HOST:$REMOTE_APP_DIR/"
    else
        log_warning "未找到交叉编译的二进制文件，将在服务器上编译..."
        build_backend_on_server
    fi
    
    # 上传静态资源 (前端文件)
    log_info "上传前端静态资源..."
    rsync -avz --progress \
        "$LOCAL_BACKEND_DIR/resources/public/" \
        "$REMOTE_HOST:$REMOTE_APP_DIR/resources/public/"
    
    # 上传配置文件
    log_info "上传配置文件..."
    if [ -f "$LOCAL_BACKEND_DIR/.env" ]; then
        scp "$LOCAL_BACKEND_DIR/.env" "$REMOTE_HOST:$REMOTE_APP_DIR/"
    fi
    
    # 上传 SQL 初始化文件
    if [ -f "$LOCAL_BACKEND_DIR/init.sql" ]; then
        scp "$LOCAL_BACKEND_DIR/init.sql" "$REMOTE_HOST:$REMOTE_APP_DIR/"
    fi
    
    log_success "文件上传完成！"
}

# =============================================================================
# 步骤 4: 创建服务器端启动脚本
# =============================================================================
create_server_scripts() {
    log_info "创建服务器端脚本..."
    
    # 创建启动脚本
    ssh "$REMOTE_HOST" "cat > $REMOTE_APP_DIR/start.sh << 'EOF'
#!/bin/bash
cd /root/app

# 加载环境变量
if [ -f .env ]; then
    export \$(cat .env | grep -v '^#' | xargs)
fi

# 设置默认值
export JWT_SECRET=\"\${JWT_SECRET:-hulunote-secret-key-production}\"
export PORT=\"\${PORT:-6689}\"
export RUST_LOG=\"\${RUST_LOG:-hulunote_server=info,tower_http=info}\"

# 启动服务
./hulunote-server
EOF"
    
    # 创建后台启动脚本
    ssh "$REMOTE_HOST" "cat > $REMOTE_APP_DIR/start-daemon.sh << 'EOF'
#!/bin/bash
cd /root/app

# 停止现有进程
./stop.sh 2>/dev/null || true

# 加载环境变量
if [ -f .env ]; then
    export \$(cat .env | grep -v '^#' | xargs)
fi

# 设置默认值
export JWT_SECRET=\"\${JWT_SECRET:-hulunote-secret-key-production}\"
export PORT=\"\${PORT:-6689}\"
export RUST_LOG=\"\${RUST_LOG:-hulunote_server=info,tower_http=info}\"

# 后台启动
nohup ./hulunote-server > logs/hulunote.log 2>&1 &
echo \$! > hulunote.pid
echo \"Hulunote server started with PID: \$(cat hulunote.pid)\"
EOF"
    
    # 创建停止脚本
    ssh "$REMOTE_HOST" "cat > $REMOTE_APP_DIR/stop.sh << 'EOF'
#!/bin/bash
cd /root/app

if [ -f hulunote.pid ]; then
    PID=\$(cat hulunote.pid)
    if kill -0 \$PID 2>/dev/null; then
        echo \"Stopping Hulunote server (PID: \$PID)...\"
        kill \$PID
        rm hulunote.pid
        echo \"Server stopped.\"
    else
        echo \"Process not running. Cleaning up PID file.\"
        rm hulunote.pid
    fi
else
    echo \"PID file not found. Trying to find process...\"
    pkill -f hulunote-server && echo \"Server stopped.\" || echo \"No process found.\"
fi
EOF"
    
    # 创建重启脚本
    ssh "$REMOTE_HOST" "cat > $REMOTE_APP_DIR/restart.sh << 'EOF'
#!/bin/bash
cd /root/app
./stop.sh
sleep 2
./start-daemon.sh
EOF"
    
    # 创建状态检查脚本
    ssh "$REMOTE_HOST" "cat > $REMOTE_APP_DIR/status.sh << 'EOF'
#!/bin/bash
cd /root/app

if [ -f hulunote.pid ]; then
    PID=\$(cat hulunote.pid)
    if kill -0 \$PID 2>/dev/null; then
        echo \"✅ Hulunote server is running (PID: \$PID)\"
        echo \"\"
        echo \"Recent logs:\"
        tail -20 logs/hulunote.log 2>/dev/null || echo \"No logs found.\"
    else
        echo \"❌ Server not running (stale PID file)\"
    fi
else
    echo \"❌ Server not running (no PID file)\"
fi
EOF"
    
    # 创建日志目录并设置权限
    ssh "$REMOTE_HOST" "mkdir -p $REMOTE_APP_DIR/logs && chmod +x $REMOTE_APP_DIR/*.sh"
    
    log_success "服务器端脚本创建完成！"
}

# =============================================================================
# 步骤 5: 创建 systemd 服务 (可选)
# =============================================================================
create_systemd_service() {
    log_info "创建 systemd 服务..."
    
    ssh "$REMOTE_HOST" "cat > /etc/systemd/system/hulunote.service << 'EOF'
[Unit]
Description=Hulunote Server
After=network.target postgresql.service

[Service]
Type=simple
User=root
WorkingDirectory=/root/app
EnvironmentFile=/root/app/.env
ExecStart=/root/app/hulunote-server
Restart=always
RestartSec=5
StandardOutput=append:/root/app/logs/hulunote.log
StandardError=append:/root/app/logs/hulunote-error.log

[Install]
WantedBy=multi-user.target
EOF"
    
    ssh "$REMOTE_HOST" "systemctl daemon-reload"
    
    log_success "systemd 服务创建完成！"
    log_info "可以使用以下命令管理服务:"
    log_info "  启动: systemctl start hulunote"
    log_info "  停止: systemctl stop hulunote"
    log_info "  重启: systemctl restart hulunote"
    log_info "  状态: systemctl status hulunote"
    log_info "  开机启动: systemctl enable hulunote"
}

# =============================================================================
# 步骤 6: 启动服务
# =============================================================================
start_service() {
    log_info "启动 Hulunote 服务..."
    
    ssh "$REMOTE_HOST" "cd $REMOTE_APP_DIR && chmod +x hulunote-server && ./start-daemon.sh"
    
    # 等待几秒后检查状态
    sleep 3
    ssh "$REMOTE_HOST" "cd $REMOTE_APP_DIR && ./status.sh"
    
    log_success "部署完成！"
    log_info "服务地址: http://104.244.95.160:6689"
}

# =============================================================================
# 主函数
# =============================================================================
main() {
    echo ""
    echo "=============================================="
    echo "       Hulunote 部署脚本"
    echo "=============================================="
    echo ""
    
    # 解析参数
    SKIP_FRONTEND=false
    SKIP_BACKEND=false
    ONLY_RESTART=false
    USE_SERVER_BUILD=false
    CREATE_SYSTEMD=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --skip-frontend)
                SKIP_FRONTEND=true
                shift
                ;;
            --skip-backend)
                SKIP_BACKEND=true
                shift
                ;;
            --restart)
                ONLY_RESTART=true
                shift
                ;;
            --server-build)
                USE_SERVER_BUILD=true
                shift
                ;;
            --systemd)
                CREATE_SYSTEMD=true
                shift
                ;;
            --help)
                echo "用法: $0 [选项]"
                echo ""
                echo "选项:"
                echo "  --skip-frontend    跳过前端编译"
                echo "  --skip-backend     跳过后端编译"
                echo "  --restart          仅重启服务 (不重新编译和上传)"
                echo "  --server-build     在服务器上编译 Rust (无需交叉编译)"
                echo "  --systemd          创建 systemd 服务"
                echo "  --help             显示帮助信息"
                exit 0
                ;;
            *)
                log_error "未知参数: $1"
                exit 1
                ;;
        esac
    done
    
    # 仅重启模式
    if [ "$ONLY_RESTART" = true ]; then
        log_info "仅重启服务..."
        ssh "$REMOTE_HOST" "cd $REMOTE_APP_DIR && ./restart.sh"
        exit 0
    fi
    
    # 编译前端
    if [ "$SKIP_FRONTEND" = false ]; then
        build_frontend
    else
        log_warning "跳过前端编译"
    fi
    
    # 编译后端
    if [ "$SKIP_BACKEND" = false ]; then
        if [ "$USE_SERVER_BUILD" = true ]; then
            log_info "将在服务器上编译后端..."
        else
            build_backend || {
                log_warning "本地交叉编译失败，将在服务器上编译..."
                USE_SERVER_BUILD=true
            }
        fi
    else
        log_warning "跳过后端编译"
    fi
    
    # 上传文件
    upload_files
    
    # 创建服务器端脚本
    create_server_scripts
    
    # 创建 systemd 服务 (可选)
    if [ "$CREATE_SYSTEMD" = true ]; then
        create_systemd_service
    fi
    
    # 启动服务
    start_service
    
    echo ""
    echo "=============================================="
    echo "       部署完成！"
    echo "=============================================="
    echo ""
    log_info "服务管理命令:"
    echo "  启动: ssh $REMOTE_HOST 'cd $REMOTE_APP_DIR && ./start-daemon.sh'"
    echo "  停止: ssh $REMOTE_HOST 'cd $REMOTE_APP_DIR && ./stop.sh'"
    echo "  重启: ssh $REMOTE_HOST 'cd $REMOTE_APP_DIR && ./restart.sh'"
    echo "  状态: ssh $REMOTE_HOST 'cd $REMOTE_APP_DIR && ./status.sh'"
    echo "  日志: ssh $REMOTE_HOST 'tail -f $REMOTE_APP_DIR/logs/hulunote.log'"
    echo ""
}

# 运行主函数
main "$@"
