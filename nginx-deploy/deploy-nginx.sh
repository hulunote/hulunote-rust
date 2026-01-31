#!/bin/bash

# ============================================
# Nginx 部署脚本 - www.hulunote.top
# 转发到 104.244.95.160:6689
# 包含 Let's Encrypt 免费 SSL 证书
# ============================================

set -e

DOMAIN="www.hulunote.top"
DOMAIN_ALT="hulunote.top"
BACKEND="104.244.95.160:6689"
EMAIL="admin@hulunote.top"  # 修改为你的邮箱

echo "=========================================="
echo "开始部署 Nginx 反向代理"
echo "域名: $DOMAIN"
echo "后端: $BACKEND"
echo "=========================================="

# 检查是否以 root 运行
if [ "$EUID" -ne 0 ]; then
    echo "请使用 root 权限运行此脚本"
    echo "sudo ./deploy-nginx.sh"
    exit 1
fi

# 1. 安装 Nginx 和 Certbot
echo "[1/5] 安装 Nginx 和 Certbot..."
if command -v apt-get &> /dev/null; then
    # Debian/Ubuntu
    apt-get update
    apt-get install -y nginx certbot python3-certbot-nginx
elif command -v yum &> /dev/null; then
    # CentOS/RHEL
    yum install -y epel-release
    yum install -y nginx certbot python3-certbot-nginx
elif command -v dnf &> /dev/null; then
    # Fedora
    dnf install -y nginx certbot python3-certbot-nginx
else
    echo "不支持的包管理器，请手动安装 nginx 和 certbot"
    exit 1
fi

# 2. 创建 Nginx 配置文件（HTTP 版本，用于获取证书）
echo "[2/5] 创建 Nginx 配置文件..."
cat > /etc/nginx/sites-available/hulunote.conf << 'NGINX_CONF'
# HTTP 配置 - 用于证书验证和重定向
server {
    listen 80;
    listen [::]:80;
    server_name www.hulunote.top hulunote.top;

    # Let's Encrypt 验证路径
    location /.well-known/acme-challenge/ {
        root /var/www/certbot;
    }

    # 其他请求重定向到 HTTPS
    location / {
        return 301 https://$host$request_uri;
    }
}
NGINX_CONF

# 创建 certbot webroot 目录
mkdir -p /var/www/certbot

# 如果 sites-enabled 目录不存在，创建它
mkdir -p /etc/nginx/sites-enabled

# 检查 nginx.conf 是否包含 sites-enabled
if ! grep -q "sites-enabled" /etc/nginx/nginx.conf; then
    # 在 http 块中添加 include
    sed -i '/http {/a \    include /etc/nginx/sites-enabled/*.conf;' /etc/nginx/nginx.conf
fi

# 启用配置
ln -sf /etc/nginx/sites-available/hulunote.conf /etc/nginx/sites-enabled/

# 测试 Nginx 配置
nginx -t

# 重启 Nginx
systemctl restart nginx
systemctl enable nginx

# 3. 获取 SSL 证书
echo "[3/5] 获取 Let's Encrypt SSL 证书..."
certbot certonly --nginx \
    -d $DOMAIN \
    -d $DOMAIN_ALT \
    --non-interactive \
    --agree-tos \
    --email $EMAIL \
    --redirect

# 4. 创建完整的 HTTPS Nginx 配置
echo "[4/5] 配置 HTTPS 反向代理..."
cat > /etc/nginx/sites-available/hulunote.conf << 'NGINX_CONF'
# HTTP - 重定向到 HTTPS
server {
    listen 80;
    listen [::]:80;
    server_name www.hulunote.top hulunote.top;

    # Let's Encrypt 验证路径
    location /.well-known/acme-challenge/ {
        root /var/www/certbot;
    }

    # 所有其他请求重定向到 HTTPS
    location / {
        return 301 https://www.hulunote.top$request_uri;
    }
}

# HTTPS - 主配置
server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name www.hulunote.top;

    # SSL 证书配置
    ssl_certificate /etc/letsencrypt/live/www.hulunote.top/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/www.hulunote.top/privkey.pem;
    
    # SSL 安全配置
    ssl_session_timeout 1d;
    ssl_session_cache shared:SSL:50m;
    ssl_session_tickets off;
    
    # 现代 SSL 配置
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305:DHE-RSA-AES128-GCM-SHA256:DHE-RSA-AES256-GCM-SHA384;
    ssl_prefer_server_ciphers off;
    
    # HSTS (可选，取消注释启用)
    # add_header Strict-Transport-Security "max-age=63072000" always;
    
    # OCSP Stapling
    ssl_stapling on;
    ssl_stapling_verify on;
    ssl_trusted_certificate /etc/letsencrypt/live/www.hulunote.top/chain.pem;
    resolver 8.8.8.8 8.8.4.4 valid=300s;
    resolver_timeout 5s;

    # 日志配置
    access_log /var/log/nginx/hulunote.access.log;
    error_log /var/log/nginx/hulunote.error.log;

    # 客户端配置
    client_max_body_size 100M;
    
    # 反向代理到后端服务
    location / {
        proxy_pass http://104.244.95.160:6689;
        proxy_http_version 1.1;
        
        # WebSocket 支持
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        
        # 传递真实 IP
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header X-Forwarded-Host $host;
        proxy_set_header X-Forwarded-Port $server_port;
        
        # 超时配置
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
        
        # 缓冲配置
        proxy_buffering on;
        proxy_buffer_size 4k;
        proxy_buffers 8 4k;
    }

    # 健康检查端点（可选）
    location /nginx-health {
        return 200 'OK';
        add_header Content-Type text/plain;
    }
}

# hulunote.top 重定向到 www.hulunote.top
server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name hulunote.top;

    ssl_certificate /etc/letsencrypt/live/www.hulunote.top/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/www.hulunote.top/privkey.pem;

    return 301 https://www.hulunote.top$request_uri;
}
NGINX_CONF

# 测试并重新加载 Nginx
nginx -t
systemctl reload nginx

# 5. 设置证书自动更新
echo "[5/5] 配置证书自动更新..."

# 创建更新脚本
cat > /usr/local/bin/renew-ssl.sh << 'RENEW_SCRIPT'
#!/bin/bash
# SSL 证书自动更新脚本

LOG_FILE="/var/log/certbot-renew.log"
DATE=$(date '+%Y-%m-%d %H:%M:%S')

echo "[$DATE] 开始检查证书更新..." >> $LOG_FILE

# 更新证书
certbot renew --quiet --deploy-hook "systemctl reload nginx"

if [ $? -eq 0 ]; then
    echo "[$DATE] 证书检查/更新完成" >> $LOG_FILE
else
    echo "[$DATE] 证书更新失败" >> $LOG_FILE
fi
RENEW_SCRIPT

chmod +x /usr/local/bin/renew-ssl.sh

# 添加 cron 任务（每天凌晨 2:30 和 14:30 检查更新）
(crontab -l 2>/dev/null | grep -v "renew-ssl.sh"; echo "30 2,14 * * * /usr/local/bin/renew-ssl.sh") | crontab -

# 或者使用 systemd timer（如果系统支持）
if command -v systemctl &> /dev/null; then
    cat > /etc/systemd/system/certbot-renewal.service << 'SERVICE'
[Unit]
Description=Certbot Renewal

[Service]
Type=oneshot
ExecStart=/usr/bin/certbot renew --quiet --deploy-hook "systemctl reload nginx"
SERVICE

    cat > /etc/systemd/system/certbot-renewal.timer << 'TIMER'
[Unit]
Description=Run certbot renewal twice daily

[Timer]
OnCalendar=*-*-* 02,14:30:00
RandomizedDelaySec=3600
Persistent=true

[Install]
WantedBy=timers.target
TIMER

    systemctl daemon-reload
    systemctl enable certbot-renewal.timer
    systemctl start certbot-renewal.timer
fi

echo ""
echo "=========================================="
echo "✅ 部署完成！"
echo "=========================================="
echo ""
echo "配置信息："
echo "  - 域名: https://$DOMAIN"
echo "  - 后端: $BACKEND"
echo "  - Nginx 配置: /etc/nginx/sites-available/hulunote.conf"
echo "  - SSL 证书: /etc/letsencrypt/live/$DOMAIN/"
echo "  - 访问日志: /var/log/nginx/hulunote.access.log"
echo "  - 错误日志: /var/log/nginx/hulunote.error.log"
echo ""
echo "常用命令："
echo "  - 测试配置: nginx -t"
echo "  - 重载配置: systemctl reload nginx"
echo "  - 查看状态: systemctl status nginx"
echo "  - 手动更新证书: certbot renew --dry-run"
echo "  - 查看证书信息: certbot certificates"
echo ""
echo "证书自动更新已配置（每天两次检查）"
echo "=========================================="
