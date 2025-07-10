# Deployment Guide

This guide covers deploying the Localhost HTTP Server in production environments.

## Production Deployment

### System Requirements

- **Operating System**: Linux (Ubuntu 20.04+, CentOS 8+, or similar)
- **Architecture**: x86_64 or ARM64
- **Memory**: Minimum 512MB RAM, recommended 2GB+
- **Storage**: 100MB for server binary, additional space for content
- **Network**: Open ports for HTTP/HTTPS traffic

### Pre-deployment Checklist

- [ ] Server binary built with `--release` flag
- [ ] Configuration file tested and validated
- [ ] SSL certificates obtained (if using HTTPS)
- [ ] Firewall rules configured
- [ ] User account created for server process
- [ ] Log rotation configured
- [ ] Monitoring setup completed
- [ ] Backup strategy implemented

## Installation

### 1. Build the Server

```bash
# On the build machine
git clone https://github.com/mohalnassery/localhost.git
cd localhost
cargo build --release

# Copy binary to production server
scp target/release/localhost-server user@server:/opt/localhost-http/
```

### 2. Create System User

```bash
# Create dedicated user for the server
sudo useradd --system --shell /bin/false --home /opt/localhost-http localhost-http
sudo mkdir -p /opt/localhost-http
sudo chown localhost-http:localhost-http /opt/localhost-http
```

### 3. Directory Structure

```bash
# Create directory structure
sudo mkdir -p /opt/localhost-http/{bin,config,www,logs,cgi-bin}
sudo mkdir -p /var/log/localhost-http

# Set permissions
sudo chown -R localhost-http:localhost-http /opt/localhost-http
sudo chown -R localhost-http:localhost-http /var/log/localhost-http
sudo chmod 755 /opt/localhost-http/bin/localhost-server
sudo chmod 644 /opt/localhost-http/config/*
sudo chmod 755 /opt/localhost-http/cgi-bin/*
```

### 4. Configuration

Create production configuration at `/opt/localhost-http/config/production.conf`:

```nginx
server {
    server_name example.com www.example.com
    listen 80
    max_body_size 10485760  # 10MB
    
    # Static content
    route / {
        methods GET
        root /opt/localhost-http/www
        index index.html
        directory_listing off
    }
    
    # CGI scripts
    route /cgi-bin {
        methods GET POST
        root /opt/localhost-http/cgi-bin
        cgi python3
    }
    
    # API endpoints
    route /api {
        methods GET POST PUT DELETE
        root /opt/localhost-http/cgi-bin/api
        cgi python3
    }
    
    # Error pages
    error_page 404 /errors/404.html
    error_page 500 /errors/500.html
}
```

## Service Management

### Systemd Service

Create `/etc/systemd/system/localhost-http.service`:

```ini
[Unit]
Description=Localhost HTTP Server
After=network.target
Wants=network.target

[Service]
Type=simple
User=localhost-http
Group=localhost-http
WorkingDirectory=/opt/localhost-http
ExecStart=/opt/localhost-http/bin/localhost-server /opt/localhost-http/config/production.conf
ExecReload=/bin/kill -HUP $MAINPID
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal
SyslogIdentifier=localhost-http

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/localhost-http /var/log/localhost-http

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096

[Install]
WantedBy=multi-user.target
```

### Service Commands

```bash
# Enable and start service
sudo systemctl enable localhost-http
sudo systemctl start localhost-http

# Check status
sudo systemctl status localhost-http

# View logs
sudo journalctl -u localhost-http -f

# Restart service
sudo systemctl restart localhost-http

# Stop service
sudo systemctl stop localhost-http
```

## Reverse Proxy Setup

### Nginx Reverse Proxy

For HTTPS termination and load balancing, use nginx as a reverse proxy:

```nginx
# /etc/nginx/sites-available/example.com
server {
    listen 80;
    server_name example.com www.example.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name example.com www.example.com;
    
    # SSL configuration
    ssl_certificate /etc/ssl/certs/example.com.crt;
    ssl_certificate_key /etc/ssl/private/example.com.key;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512;
    ssl_prefer_server_ciphers off;
    
    # Security headers
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;
    add_header X-XSS-Protection "1; mode=block";
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains";
    
    # Proxy to localhost-http server
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Timeouts
        proxy_connect_timeout 5s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }
    
    # Static files (optional optimization)
    location /static/ {
        alias /opt/localhost-http/www/static/;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
```

### Apache Reverse Proxy

```apache
# /etc/apache2/sites-available/example.com.conf
<VirtualHost *:80>
    ServerName example.com
    ServerAlias www.example.com
    Redirect permanent / https://example.com/
</VirtualHost>

<VirtualHost *:443>
    ServerName example.com
    ServerAlias www.example.com
    
    # SSL configuration
    SSLEngine on
    SSLCertificateFile /etc/ssl/certs/example.com.crt
    SSLCertificateKeyFile /etc/ssl/private/example.com.key
    
    # Security headers
    Header always set X-Frame-Options DENY
    Header always set X-Content-Type-Options nosniff
    Header always set X-XSS-Protection "1; mode=block"
    Header always set Strict-Transport-Security "max-age=31536000; includeSubDomains"
    
    # Proxy configuration
    ProxyPreserveHost On
    ProxyPass / http://127.0.0.1:8080/
    ProxyPassReverse / http://127.0.0.1:8080/
    
    # Optional: serve static files directly
    Alias /static /opt/localhost-http/www/static
    <Directory "/opt/localhost-http/www/static">
        Require all granted
        ExpiresActive On
        ExpiresDefault "access plus 1 year"
    </Directory>
</VirtualHost>
```

## Monitoring and Logging

### Log Configuration

Configure log rotation with logrotate:

```bash
# /etc/logrotate.d/localhost-http
/var/log/localhost-http/*.log {
    daily
    missingok
    rotate 52
    compress
    delaycompress
    notifempty
    create 644 localhost-http localhost-http
    postrotate
        systemctl reload localhost-http
    endscript
}
```

### Monitoring Scripts

Create monitoring script `/opt/localhost-http/bin/health-check.sh`:

```bash
#!/bin/bash

# Health check script for localhost-http server
URL="http://127.0.0.1:8080/"
TIMEOUT=5

# Check if server responds
if curl -f -s --max-time $TIMEOUT "$URL" > /dev/null; then
    echo "OK: Server is responding"
    exit 0
else
    echo "CRITICAL: Server is not responding"
    exit 2
fi
```

### Prometheus Metrics

Create metrics endpoint `/opt/localhost-http/cgi-bin/metrics.py`:

```python
#!/usr/bin/env python3
import os
import time

print("Content-Type: text/plain\n")

# Basic metrics (extend as needed)
print("# HELP http_requests_total Total HTTP requests")
print("# TYPE http_requests_total counter")
print("http_requests_total 1234")

print("# HELP http_request_duration_seconds HTTP request duration")
print("# TYPE http_request_duration_seconds histogram")
print("http_request_duration_seconds_sum 456.78")
print("http_request_duration_seconds_count 1234")

print("# HELP process_start_time_seconds Start time of the process")
print("# TYPE process_start_time_seconds gauge")
print(f"process_start_time_seconds {time.time()}")
```

## Security Hardening

### Firewall Configuration

```bash
# UFW (Ubuntu)
sudo ufw allow ssh
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw enable

# iptables
sudo iptables -A INPUT -p tcp --dport 22 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 80 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 443 -j ACCEPT
sudo iptables -A INPUT -j DROP
```

### File Permissions

```bash
# Secure file permissions
sudo chmod 750 /opt/localhost-http
sudo chmod 755 /opt/localhost-http/bin/localhost-server
sudo chmod 644 /opt/localhost-http/config/*
sudo chmod 755 /opt/localhost-http/cgi-bin/*
sudo chmod 644 /opt/localhost-http/www/*

# Prevent execution of uploaded files
sudo chmod 644 /opt/localhost-http/www/uploads/*
```

### SELinux Configuration

```bash
# Set SELinux contexts (if using SELinux)
sudo setsebool -P httpd_can_network_connect 1
sudo semanage fcontext -a -t httpd_exec_t "/opt/localhost-http/bin/localhost-server"
sudo restorecon -R /opt/localhost-http
```

## Performance Tuning

### System Limits

```bash
# /etc/security/limits.conf
localhost-http soft nofile 65536
localhost-http hard nofile 65536
localhost-http soft nproc 4096
localhost-http hard nproc 4096
```

### Kernel Parameters

```bash
# /etc/sysctl.d/99-localhost-http.conf
net.core.somaxconn = 1024
net.ipv4.tcp_max_syn_backlog = 1024
net.ipv4.ip_local_port_range = 1024 65535
net.ipv4.tcp_fin_timeout = 30
```

## Backup and Recovery

### Backup Script

```bash
#!/bin/bash
# /opt/localhost-http/bin/backup.sh

BACKUP_DIR="/backup/localhost-http"
DATE=$(date +%Y%m%d_%H%M%S)

mkdir -p "$BACKUP_DIR"

# Backup configuration and content
tar -czf "$BACKUP_DIR/localhost-http-$DATE.tar.gz" \
    /opt/localhost-http/config \
    /opt/localhost-http/www \
    /opt/localhost-http/cgi-bin

# Keep only last 30 days of backups
find "$BACKUP_DIR" -name "localhost-http-*.tar.gz" -mtime +30 -delete
```

### Recovery Procedure

1. Stop the service: `sudo systemctl stop localhost-http`
2. Restore files from backup
3. Verify configuration: `sudo -u localhost-http /opt/localhost-http/bin/localhost-server --test /opt/localhost-http/config/production.conf`
4. Start the service: `sudo systemctl start localhost-http`

## Troubleshooting

### Common Issues

1. **Service won't start**: Check configuration syntax and file permissions
2. **High memory usage**: Monitor for memory leaks, restart service if needed
3. **Slow responses**: Check system resources and network connectivity
4. **CGI errors**: Verify interpreter paths and script permissions

### Debug Mode

```bash
# Run in debug mode
sudo -u localhost-http RUST_LOG=debug /opt/localhost-http/bin/localhost-server /opt/localhost-http/config/production.conf
```

### Log Analysis

```bash
# View recent logs
sudo journalctl -u localhost-http --since "1 hour ago"

# Follow logs in real-time
sudo journalctl -u localhost-http -f

# Search for errors
sudo journalctl -u localhost-http | grep -i error
```
