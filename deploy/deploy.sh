#!/bin/bash

# GenXLink Production Deployment Script
# This script deploys GenXLink to a production server

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
DEPLOY_DIR="/opt/genxlink"
BACKUP_DIR="/opt/genxlink-backups"
SERVICE_NAME="genxlink"

echo -e "${GREEN}Starting GenXLink Production Deployment${NC}"

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Please run as root (use sudo)${NC}"
    exit 1
fi

# Create directories
echo -e "${YELLOW}Creating directories...${NC}"
mkdir -p $DEPLOY_DIR
mkdir -p $BACKUP_DIR
mkdir -p $DEPLOY_DIR/logs
mkdir -p $DEPLOY_DIR/ssl

# Backup existing deployment
if [ -d "$DEPLOY_DIR/docker-compose.yml" ]; then
    echo -e "${YELLOW}Backing up existing deployment...${NC}"
    cp -r $DEPLOY_DIR $BACKUP_DIR/genxlink-$(date +%Y%m%d-%H%M%S)
fi

# Stop existing services
echo -e "${YELLOW}Stopping existing services...${NC}"
cd $DEPLOY_DIR
docker-compose -f docker-compose.prod.yml down || true

# Update system packages
echo -e "${YELLOW}Updating system packages...${NC}"
apt update && apt upgrade -y

# Install Docker and Docker Compose if not present
if ! command -v docker &> /dev/null; then
    echo -e "${YELLOW}Installing Docker...${NC}"
    apt install -y apt-transport-https ca-certificates curl gnupg lsb-release
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg
    echo "deb [arch=amd64 signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null
    apt update
    apt install -y docker-ce docker-ce-cli containerd.io
    systemctl enable docker
    systemctl start docker
fi

if ! command -v docker-compose &> /dev/null; then
    echo -e "${YELLOW}Installing Docker Compose...${NC}"
    curl -L "https://github.com/docker/compose/releases/download/v2.20.0/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
    chmod +x /usr/local/bin/docker-compose
fi

# Copy files
echo -e "${YELLOW}Copying deployment files...${NC}"
cp docker-compose.prod.yml $DEPLOY_DIR/
cp .env.example $DEPLOY_DIR/.env

# Setup SSL certificates (Let's Encrypt)
echo -e "${YELLOW}Setting up SSL certificates...${NC}"
if [ ! -f "$DEPLOY_DIR/ssl/cert.pem" ]; then
    echo -e "${YELLOW}Installing Certbot for SSL certificates...${NC}"
    apt install -y certbot python3-certbot-nginx
    
    echo -e "${YELLOW}Please enter your domain name for SSL certificate: ${NC}"
    read DOMAIN
    
    if [ ! -z "$DOMAIN" ]; then
        certbot --nginx -d $DOMAIN --non-interactive --agree-tos --email admin@$DOMAIN || {
            echo -e "${YELLOW}Certbot failed, generating self-signed certificate...${NC}"
            openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
                -keyout $DEPLOY_DIR/ssl/key.pem \
                -out $DEPLOY_DIR/ssl/cert.pem \
                -subj "/C=US/ST=State/L=City/O=Organization/CN=$DOMAIN"
        }
        
        # Update nginx configuration with domain
        sed -i "s/your-domain.com/$DOMAIN/g" $DEPLOY_DIR/../deployment/nginx/nginx.conf
    fi
fi

# Generate secure passwords and secrets
echo -e "${YELLOW}Generating secure configuration...${NC}"
POSTGRES_PASSWORD=$(openssl rand -base64 32)
REDIS_PASSWORD=$(openssl rand -base64 32)
JWT_SECRET=$(openssl rand -base64 32)
API_KEY=$(openssl rand -base64 32)
GRAFANA_PASSWORD=$(openssl rand -base64 32)

# Update .env file
sed -i "s/your_secure_postgres_password_here/$POSTGRES_PASSWORD/g" $DEPLOY_DIR/.env
sed -i "s/your_secure_redis_password_here/$REDIS_PASSWORD/g" $DEPLOY_DIR/.env
sed -i "s/your_jwt_secret_minimum_32_characters_long/$JWT_SECRET/g" $DEPLOY_DIR/.env
sed -i "s/your_api_key_minimum_32_characters_long/$API_KEY/g" $DEPLOY_DIR/.env
sed -i "s/your_secure_grafana_password_here/$GRAFANA_PASSWORD/g" $DEPLOY_DIR/.env

# Build and start services
echo -e "${YELLOW}Building and starting services...${NC}"
cd $DEPLOY_DIR
docker-compose -f docker-compose.prod.yml build --no-cache
docker-compose -f docker-compose.prod.yml up -d

# Wait for services to be healthy
echo -e "${YELLOW}Waiting for services to be healthy...${NC}"
sleep 30

# Check service health
echo -e "${YELLOW}Checking service health...${NC}"
for service in postgres redis api-server signaling-server relay-server nginx; do
    if docker-compose -f docker-compose.prod.yml ps $service | grep -q "Up (healthy)"; then
        echo -e "${GREEN}✓ $service is healthy${NC}"
    else
        echo -e "${RED}✗ $service is not healthy${NC}"
        docker-compose -f docker-compose.prod.yml logs $service
    fi
done

# Setup automatic backups
echo -e "${YELLOW}Setting up automatic backups...${NC}"
cat > /etc/cron.daily/genxlink-backup << EOF
#!/bin/bash
cd $DEPLOY_DIR
docker-compose -f docker-compose.prod.yml exec postgres pg_dump -U genxlink genxlink > $BACKUP_DIR/genxlink-db-\$(date +\%Y\%m\%d).sql
find $BACKUP_DIR -name "*.sql" -mtime +7 -delete
EOF
chmod +x /etc/cron.daily/genxlink-backup

# Setup log rotation
echo -e "${YELLOW}Setting up log rotation...${NC}"
cat > /etc/logrotate.d/genxlink << EOF
$DEPLOY_DIR/logs/*.log {
    daily
    missingok
    rotate 30
    compress
    delaycompress
    notifempty
    create 644 root root
    postrotate
        docker-compose -f $DEPLOY_DIR/docker-compose.prod.yml restart nginx
    endscript
}
EOF

# Create systemd service for management
echo -e "${YELLOW}Creating systemd service...${NC}"
cat > /etc/systemd/system/genxlink.service << EOF
[Unit]
Description=GenXLink Remote Desktop Platform
Requires=docker.service
After=docker.service

[Service]
Type=oneshot
RemainAfterExit=yes
WorkingDirectory=$DEPLOY_DIR
ExecStart=/usr/local/bin/docker-compose -f docker-compose.prod.yml up -d
ExecStop=/usr/local/bin/docker-compose -f docker-compose.prod.yml down
TimeoutStartSec=0

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable genxlink

# Display deployment information
echo -e "${GREEN}Deployment completed successfully!${NC}"
echo ""
echo -e "${YELLOW}Service Information:${NC}"
echo "- API Server: https://localhost/api/"
echo "- Signaling Server: wss://localhost/ws/"
echo "- Relay Server: https://localhost/relay/"
echo "- Grafana Dashboard: https://localhost:3000 (admin/$GRAFANA_PASSWORD)"
echo "- Prometheus: https://localhost:9090"
echo ""
echo -e "${YELLOW}Management Commands:${NC}"
echo "- Start services: systemctl start genxlink"
echo "- Stop services: systemctl stop genxlink"
echo "- Restart services: systemctl restart genxlink"
echo "- View logs: docker-compose -f $DEPLOY_DIR/docker-compose.prod.yml logs -f"
echo "- Check status: docker-compose -f $DEPLOY_DIR/docker-compose.prod.yml ps"
echo ""
echo -e "${YELLOW}Configuration file: $DEPLOY_DIR/.env${NC}"
echo -e "${YELLOW}Backup directory: $BACKUP_DIR${NC}"
echo ""
echo -e "${GREEN}GenXLink is now running in production mode!${NC}"
