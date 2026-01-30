#!/bin/bash

# =============================================================================
# Hulunote 监控脚本
# 监控线上服务的运行状态、内存、CPU、网络连接等
# =============================================================================

# 配置变量
REMOTE_HOST="root@104.244.95.160"
REMOTE_APP_DIR="/root/app"
BINARY_NAME="hulunote-server"
SERVICE_PORT="6689"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color
BOLD='\033[1m'

# =============================================================================
# 辅助函数
# =============================================================================
print_header() {
    echo ""
    echo -e "${BOLD}${BLUE}══════════════════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}${BLUE}  $1${NC}"
    echo -e "${BOLD}${BLUE}══════════════════════════════════════════════════════════════${NC}"
}

print_section() {
    echo ""
    echo -e "${CYAN}▶ $1${NC}"
    echo -e "${CYAN}──────────────────────────────────────────${NC}"
}

# =============================================================================
# 基础状态检查
# =============================================================================
check_status() {
    print_header "🖥️  Hulunote 服务监控"
    echo -e "  ${PURPLE}服务器:${NC} 104.244.95.160"
    echo -e "  ${PURPLE}时间:${NC} $(date '+%Y-%m-%d %H:%M:%S')"
    
    print_section "服务状态"
    
    ssh "$REMOTE_HOST" << 'EOFSTATUS'
        # 检查进程是否运行
        PID=$(pgrep -f hulunote-server)
        if [ -n "$PID" ]; then
            echo -e "\033[0;32m✅ 服务运行中 (PID: $PID)\033[0m"
            
            # 获取进程启动时间
            START_TIME=$(ps -p $PID -o lstart= 2>/dev/null)
            if [ -n "$START_TIME" ]; then
                echo -e "   启动时间: $START_TIME"
            fi
            
            # 获取运行时长
            ELAPSED=$(ps -p $PID -o etime= 2>/dev/null | xargs)
            if [ -n "$ELAPSED" ]; then
                echo -e "   运行时长: $ELAPSED"
            fi
        else
            echo -e "\033[0;31m❌ 服务未运行\033[0m"
        fi
        
        # 检查端口监听
        if netstat -tlnp 2>/dev/null | grep -q ":6689 "; then
            echo -e "\033[0;32m✅ 端口 6689 正在监听\033[0m"
        else
            echo -e "\033[0;31m❌ 端口 6689 未监听\033[0m"
        fi
EOFSTATUS
}

# =============================================================================
# 资源使用情况
# =============================================================================
check_resources() {
    print_section "资源使用"
    
    ssh "$REMOTE_HOST" << 'EOFRES'
        PID=$(pgrep -f hulunote-server)
        if [ -n "$PID" ]; then
            echo "┌─────────────────────────────────────────────────────────┐"
            echo "│  Hulunote-Server 进程资源                               │"
            echo "├─────────────────────────────────────────────────────────┤"
            
            # 获取详细的进程信息
            PS_INFO=$(ps -p $PID -o pid=,pcpu=,pmem=,rss=,vsz= 2>/dev/null)
            if [ -n "$PS_INFO" ]; then
                read PID CPU MEM RSS VSZ <<< "$PS_INFO"
                RSS_MB=$(echo "scale=2; $RSS/1024" | bc)
                VSZ_MB=$(echo "scale=2; $VSZ/1024" | bc)
                
                printf "│  %-12s: %-40s │\n" "PID" "$PID"
                printf "│  %-12s: %-40s │\n" "CPU 使用率" "${CPU}%"
                printf "│  %-12s: %-40s │\n" "内存使用率" "${MEM}%"
                printf "│  %-12s: %-40s │\n" "物理内存" "${RSS_MB} MB (RSS)"
                printf "│  %-12s: %-40s │\n" "虚拟内存" "${VSZ_MB} MB (VSZ)"
            fi
            
            # 获取线程数
            THREADS=$(cat /proc/$PID/status 2>/dev/null | grep Threads | awk '{print $2}')
            if [ -n "$THREADS" ]; then
                printf "│  %-12s: %-40s │\n" "线程数" "$THREADS"
            fi
            
            # 获取文件描述符数量
            FD_COUNT=$(ls /proc/$PID/fd 2>/dev/null | wc -l)
            printf "│  %-12s: %-40s │\n" "打开文件数" "$FD_COUNT"
            
            echo "└─────────────────────────────────────────────────────────┘"
        else
            echo "进程未运行，无法获取资源信息"
        fi
EOFRES
}

# =============================================================================
# 系统整体资源
# =============================================================================
check_system() {
    print_section "系统资源"
    
    ssh "$REMOTE_HOST" << 'EOFSYS'
        echo "┌─────────────────────────────────────────────────────────┐"
        echo "│  服务器系统资源                                         │"
        echo "├─────────────────────────────────────────────────────────┤"
        
        # CPU 信息
        CPU_CORES=$(nproc)
        LOAD=$(cat /proc/loadavg | awk '{print $1, $2, $3}')
        printf "│  %-12s: %-40s │\n" "CPU 核心" "$CPU_CORES"
        printf "│  %-12s: %-40s │\n" "系统负载" "$LOAD (1/5/15 min)"
        
        # 内存信息
        MEM_INFO=$(free -m | grep Mem)
        MEM_TOTAL=$(echo $MEM_INFO | awk '{print $2}')
        MEM_USED=$(echo $MEM_INFO | awk '{print $3}')
        MEM_FREE=$(echo $MEM_INFO | awk '{print $4}')
        MEM_AVAIL=$(echo $MEM_INFO | awk '{print $7}')
        MEM_PERCENT=$(echo "scale=1; $MEM_USED * 100 / $MEM_TOTAL" | bc)
        
        printf "│  %-12s: %-40s │\n" "总内存" "${MEM_TOTAL} MB"
        printf "│  %-12s: %-40s │\n" "已使用" "${MEM_USED} MB (${MEM_PERCENT}%)"
        printf "│  %-12s: %-40s │\n" "可用" "${MEM_AVAIL} MB"
        
        # 磁盘信息
        DISK_INFO=$(df -h /root | tail -1)
        DISK_SIZE=$(echo $DISK_INFO | awk '{print $2}')
        DISK_USED=$(echo $DISK_INFO | awk '{print $3}')
        DISK_AVAIL=$(echo $DISK_INFO | awk '{print $4}')
        DISK_PERCENT=$(echo $DISK_INFO | awk '{print $5}')
        
        printf "│  %-12s: %-40s │\n" "磁盘总量" "$DISK_SIZE"
        printf "│  %-12s: %-40s │\n" "磁盘已用" "$DISK_USED ($DISK_PERCENT)"
        printf "│  %-12s: %-40s │\n" "磁盘可用" "$DISK_AVAIL"
        
        # 运行时间
        UPTIME=$(uptime -p)
        printf "│  %-12s: %-40s │\n" "系统运行" "$UPTIME"
        
        echo "└─────────────────────────────────────────────────────────┘"
EOFSYS
}

# =============================================================================
# 网络连接
# =============================================================================
check_network() {
    print_section "网络连接"
    
    ssh "$REMOTE_HOST" << 'EOFNET'
        echo "活跃连接数 (端口 6689):"
        
        # 统计连接状态
        ESTABLISHED=$(netstat -an | grep ":6689 " | grep ESTABLISHED | wc -l)
        TIME_WAIT=$(netstat -an | grep ":6689 " | grep TIME_WAIT | wc -l)
        CLOSE_WAIT=$(netstat -an | grep ":6689 " | grep CLOSE_WAIT | wc -l)
        
        echo "  ESTABLISHED: $ESTABLISHED"
        echo "  TIME_WAIT:   $TIME_WAIT"
        echo "  CLOSE_WAIT:  $CLOSE_WAIT"
        echo ""
        
        # 显示当前连接的客户端 IP (去重)
        echo "连接的客户端 IP:"
        netstat -an | grep ":6689 " | grep ESTABLISHED | awk '{print $5}' | cut -d: -f1 | sort | uniq -c | sort -rn | head -10
EOFNET
}

# =============================================================================
# 查看日志
# =============================================================================
check_logs() {
    local LINES=${1:-30}
    print_section "最近 $LINES 行日志"
    
    ssh "$REMOTE_HOST" "tail -n $LINES $REMOTE_APP_DIR/logs/hulunote.log 2>/dev/null || echo '日志文件不存在'"
}

# =============================================================================
# 实时日志
# =============================================================================
tail_logs() {
    print_section "实时日志 (Ctrl+C 退出)"
    ssh "$REMOTE_HOST" "tail -f $REMOTE_APP_DIR/logs/hulunote.log"
}

# =============================================================================
# 错误日志
# =============================================================================
check_errors() {
    local LINES=${1:-50}
    print_section "最近错误日志 (最近 $LINES 行中的错误)"
    
    ssh "$REMOTE_HOST" "tail -n $LINES $REMOTE_APP_DIR/logs/hulunote.log 2>/dev/null | grep -i -E 'error|err|fail|panic|exception' --color=always || echo '未发现错误日志'"
}

# =============================================================================
# HTTP 健康检查
# =============================================================================
check_health() {
    print_section "HTTP 健康检查"
    
    echo "检查首页响应..."
    HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" --connect-timeout 5 "http://104.244.95.160:6689/" 2>/dev/null)
    RESPONSE_TIME=$(curl -s -o /dev/null -w "%{time_total}" --connect-timeout 5 "http://104.244.95.160:6689/" 2>/dev/null)
    
    if [ "$HTTP_CODE" = "200" ]; then
        echo -e "${GREEN}✅ 首页正常 (HTTP $HTTP_CODE, 响应时间: ${RESPONSE_TIME}s)${NC}"
    else
        echo -e "${RED}❌ 首页异常 (HTTP $HTTP_CODE)${NC}"
    fi
    
    # 检查 API 端点
    echo ""
    echo "检查 API 端点..."
    API_CODE=$(curl -s -o /dev/null -w "%{http_code}" --connect-timeout 5 "http://104.244.95.160:6689/api/health" 2>/dev/null)
    if [ "$API_CODE" = "200" ] || [ "$API_CODE" = "404" ]; then
        echo -e "${GREEN}✅ API 可达 (HTTP $API_CODE)${NC}"
    else
        echo -e "${YELLOW}⚠️  API 状态: HTTP $API_CODE${NC}"
    fi
}

# =============================================================================
# 数据库状态
# =============================================================================
check_database() {
    print_section "数据库状态"
    
    ssh "$REMOTE_HOST" << 'EOFDB'
        # 检查 PostgreSQL 状态
        if systemctl is-active --quiet postgresql; then
            echo -e "\033[0;32m✅ PostgreSQL 服务运行中\033[0m"
        else
            echo -e "\033[0;31m❌ PostgreSQL 服务未运行\033[0m"
        fi
        
        # 检查数据库连接
        if command -v psql &> /dev/null; then
            DB_SIZE=$(sudo -u postgres psql -t -c "SELECT pg_size_pretty(pg_database_size('hulunote'));" 2>/dev/null | xargs)
            if [ -n "$DB_SIZE" ]; then
                echo "数据库大小: $DB_SIZE"
            fi
            
            # 活跃连接数
            CONNECTIONS=$(sudo -u postgres psql -t -c "SELECT count(*) FROM pg_stat_activity WHERE datname='hulunote';" 2>/dev/null | xargs)
            if [ -n "$CONNECTIONS" ]; then
                echo "活跃连接数: $CONNECTIONS"
            fi
        fi
EOFDB
}

# =============================================================================
# 实时监控 (类似 top)
# =============================================================================
live_monitor() {
    print_section "实时监控 (每 2 秒刷新, Ctrl+C 退出)"
    
    while true; do
        clear
        echo -e "${BOLD}${BLUE}═══════════════════════════════════════════════════════════${NC}"
        echo -e "${BOLD}${BLUE}  Hulunote 实时监控 - $(date '+%Y-%m-%d %H:%M:%S')${NC}"
        echo -e "${BOLD}${BLUE}═══════════════════════════════════════════════════════════${NC}"
        
        ssh "$REMOTE_HOST" << 'EOFLIVE'
            PID=$(pgrep -f hulunote-server)
            
            if [ -n "$PID" ]; then
                echo -e "\033[0;32m● 服务运行中 (PID: $PID)\033[0m"
                echo ""
                
                # 进程资源
                PS_INFO=$(ps -p $PID -o pcpu=,pmem=,rss= 2>/dev/null)
                if [ -n "$PS_INFO" ]; then
                    read CPU MEM RSS <<< "$PS_INFO"
                    RSS_MB=$(echo "scale=2; $RSS/1024" | bc)
                    echo "CPU: ${CPU}%  |  内存: ${MEM}% (${RSS_MB} MB)"
                fi
                
                # 连接数
                CONN=$(netstat -an | grep ":6689 " | grep ESTABLISHED | wc -l)
                echo "活跃连接: $CONN"
                
                # 系统负载
                LOAD=$(cat /proc/loadavg | awk '{print $1, $2, $3}')
                echo "系统负载: $LOAD"
                
                # 内存
                MEM_INFO=$(free -m | grep Mem)
                MEM_USED=$(echo $MEM_INFO | awk '{print $3}')
                MEM_TOTAL=$(echo $MEM_INFO | awk '{print $2}')
                echo "系统内存: ${MEM_USED}/${MEM_TOTAL} MB"
            else
                echo -e "\033[0;31m● 服务未运行\033[0m"
            fi
EOFLIVE
        
        sleep 2
    done
}

# =============================================================================
# 完整报告
# =============================================================================
full_report() {
    check_status
    check_resources
    check_system
    check_network
    check_health
    check_database
    check_errors 20
    
    echo ""
    echo -e "${BOLD}${GREEN}══════════════════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}${GREEN}  监控报告生成完毕${NC}"
    echo -e "${BOLD}${GREEN}══════════════════════════════════════════════════════════════${NC}"
    echo ""
}

# =============================================================================
# 帮助信息
# =============================================================================
show_help() {
    echo ""
    echo -e "${BOLD}Hulunote 监控脚本${NC}"
    echo ""
    echo "用法: $0 [命令]"
    echo ""
    echo "命令:"
    echo "  status      检查服务状态 (默认)"
    echo "  resources   查看进程资源使用"
    echo "  system      查看系统资源"
    echo "  network     查看网络连接"
    echo "  health      HTTP 健康检查"
    echo "  database    数据库状态"
    echo "  logs [n]    查看最近 n 行日志 (默认 30)"
    echo "  errors [n]  查看错误日志"
    echo "  tail        实时查看日志"
    echo "  live        实时监控 (类似 top)"
    echo "  full        完整监控报告"
    echo "  help        显示帮助信息"
    echo ""
    echo "示例:"
    echo "  $0              # 查看服务状态"
    echo "  $0 full         # 完整报告"
    echo "  $0 logs 100     # 查看最近 100 行日志"
    echo "  $0 live         # 实时监控"
    echo ""
}

# =============================================================================
# 主函数
# =============================================================================
main() {
    case "${1:-status}" in
        status)
            check_status
            ;;
        resources|res)
            check_resources
            ;;
        system|sys)
            check_system
            ;;
        network|net)
            check_network
            ;;
        health|http)
            check_health
            ;;
        database|db)
            check_database
            ;;
        logs|log)
            check_logs "${2:-30}"
            ;;
        errors|err)
            check_errors "${2:-50}"
            ;;
        tail|follow)
            tail_logs
            ;;
        live|top|watch)
            live_monitor
            ;;
        full|all|report)
            full_report
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            echo "未知命令: $1"
            show_help
            exit 1
            ;;
    esac
}

# 运行主函数
main "$@"
