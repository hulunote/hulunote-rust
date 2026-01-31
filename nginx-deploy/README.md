# Nginx 部署脚本 - www.hulunote.top

## 概述

这套脚本用于部署 Nginx 反向代理，将 `www.hulunote.top` 转发到后端服务 `104.244.95.160:6689`，并配置免费的 Let's Encrypt SSL 证书及自动更新。

## 文件说明

| 文件 | 说明 |
|------|------|
| `deploy-nginx.sh` | 一键部署脚本（安装 + 配置 + 证书） |
| `hulunote.nginx.txt` | Nginx 配置文件（部署时复制为 .conf） |
| `renew-ssl.sh` | SSL 证书自动更新脚本 |

## 快速开始

### 1. 前提条件

- 一台 Linux 服务器（Ubuntu/Debian/CentOS）
- 域名 `www.hulunote.top` 和 `hulunote.top` 已解析到该服务器
- 服务器 80 和 443 端口已开放
- root 权限

### 2. 一键部署

```bash
# 修改脚本中的邮箱地址
vim deploy-nginx.sh
# 找到 EMAIL="admin@hulunote.top" 修改为你的邮箱

# 执行部署
sudo chmod +x deploy-nginx.sh
sudo ./deploy-nginx.sh
```

### 3. 手动部署（可选）

如果一键脚本不适用，可以手动执行：

```bash
# 1. 安装软件
sudo apt update
sudo apt install -y nginx certbot python3-certbot-nginx

# 2. 复制配置文件
sudo cp hulunote.nginx.txt /etc/nginx/sites-available/hulunote.conf
sudo ln -s /etc/nginx/sites-available/hulunote.conf /etc/nginx/sites-enabled/

# 3. 获取 SSL 证书
sudo certbot --nginx -d www.hulunote.top -d hulunote.top

# 4. 设置自动更新
sudo cp renew-ssl.sh /usr/local/bin/
sudo chmod +x /usr/local/bin/renew-ssl.sh
echo "30 2,14 * * * /usr/local/bin/renew-ssl.sh" | sudo crontab -
```

## 配置说明

### Nginx 配置特性

- ✅ HTTP 自动重定向到 HTTPS
- ✅ TLS 1.2/1.3 现代加密配置
- ✅ WebSocket 支持
- ✅ 传递真实客户端 IP
- ✅ Gzip 压缩
- ✅ 安全头配置
- ✅ OCSP Stapling

### 证书自动更新

- 每天 02:30 和 14:30 自动检查证书
- 证书有效期少于 30 天时自动更新
- 更新后自动重载 Nginx
- 日志位置: `/var/log/certbot-renew.log`

## 常用命令

```bash
# 测试 Nginx 配置
sudo nginx -t

# 重载 Nginx 配置
sudo systemctl reload nginx

# 重启 Nginx
sudo systemctl restart nginx

# 查看 Nginx 状态
sudo systemctl status nginx

# 查看证书信息
sudo certbot certificates

# 手动测试证书更新（不实际更新）
sudo certbot renew --dry-run

# 强制更新证书
sudo certbot renew --force-renewal

# 查看更新日志
sudo tail -f /var/log/certbot-renew.log

# 查看访问日志
sudo tail -f /var/log/nginx/hulunote.access.log

# 查看错误日志
sudo tail -f /var/log/nginx/hulunote.error.log
```

## 故障排除

### 1. 证书获取失败

```bash
# 检查域名解析
dig www.hulunote.top
dig hulunote.top

# 确保 80 端口可访问
curl -I http://www.hulunote.top

# 检查防火墙
sudo ufw status
sudo ufw allow 80
sudo ufw allow 443
```

### 2. 502 Bad Gateway

```bash
# 检查后端服务是否运行
curl http://104.244.95.160:6689

# 检查 Nginx 错误日志
sudo tail -100 /var/log/nginx/hulunote.error.log
```

### 3. SSL 证书问题

```bash
# 检查证书文件
ls -la /etc/letsencrypt/live/www.hulunote.top/

# 检查证书有效期
sudo openssl x509 -enddate -noout -in /etc/letsencrypt/live/www.hulunote.top/fullchain.pem

# 重新获取证书
sudo certbot --nginx -d www.hulunote.top -d hulunote.top --force-renewal
```

## 配置修改

### 修改后端地址

编辑 `/etc/nginx/sites-available/hulunote.conf`，找到：
```nginx
proxy_pass http://104.244.95.160:6689;
```
修改为新的后端地址，然后：
```bash
sudo nginx -t && sudo systemctl reload nginx
```

### 启用 HSTS

编辑配置文件，取消注释：
```nginx
add_header Strict-Transport-Security "max-age=63072000" always;
```

### 调整上传大小限制

修改 `client_max_body_size` 值：
```nginx
client_max_body_size 200M;  # 改为 200MB
```

## 安全建议

1. 启用 HSTS（确保网站稳定后）
2. 定期检查 Nginx 和系统更新
3. 配置防火墙仅开放必要端口
4. 监控证书更新日志
5. 定期备份配置文件

## 相关链接

- [Let's Encrypt 文档](https://letsencrypt.org/docs/)
- [Certbot 文档](https://certbot.eff.org/docs/)
- [Nginx 文档](https://nginx.org/en/docs/)
- [SSL Labs 测试](https://www.ssllabs.com/ssltest/)
