#!/bin/bash

# ============================================
# Let's Encrypt SSL 证书自动更新脚本
# www.hulunote.top
# ============================================

LOG_FILE="/var/log/certbot-renew.log"
DATE=$(date '+%Y-%m-%d %H:%M:%S')

log() {
    echo "[$DATE] $1" | tee -a $LOG_FILE
}

log "=========================================="
log "开始 SSL 证书更新检查"
log "=========================================="

# 检查证书过期时间
CERT_PATH="/etc/letsencrypt/live/www.hulunote.top/fullchain.pem"

if [ -f "$CERT_PATH" ]; then
    EXPIRY_DATE=$(openssl x509 -enddate -noout -in "$CERT_PATH" | cut -d= -f2)
    EXPIRY_EPOCH=$(date -d "$EXPIRY_DATE" +%s)
    NOW_EPOCH=$(date +%s)
    DAYS_LEFT=$(( ($EXPIRY_EPOCH - $NOW_EPOCH) / 86400 ))
    
    log "证书过期时间: $EXPIRY_DATE"
    log "剩余天数: $DAYS_LEFT 天"
    
    if [ $DAYS_LEFT -lt 30 ]; then
        log "证书即将过期，开始更新..."
    else
        log "证书有效期充足，检查是否需要更新..."
    fi
else
    log "警告: 证书文件不存在"
fi

# 执行更新
log "运行 certbot renew..."
certbot renew --quiet --deploy-hook "systemctl reload nginx" 2>&1 | tee -a $LOG_FILE

RESULT=$?

if [ $RESULT -eq 0 ]; then
    log "✅ 证书检查/更新完成"
    
    # 再次检查新的过期时间
    if [ -f "$CERT_PATH" ]; then
        NEW_EXPIRY=$(openssl x509 -enddate -noout -in "$CERT_PATH" | cut -d= -f2)
        log "当前证书过期时间: $NEW_EXPIRY"
    fi
else
    log "❌ 证书更新失败，退出码: $RESULT"
fi

log "=========================================="
log "更新检查结束"
log "=========================================="

exit $RESULT
