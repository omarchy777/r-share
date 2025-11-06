#!/bin/bash
# Quick Start Script Linux version

echo "═══════════════════════════════════════════════════════════"
echo "  R-Share Docker Deployment - Quick Start"
echo "═══════════════════════════════════════════════════════════"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
# shellcheck disable=SC2034
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Check Docker
echo -e "${YELLOW}→ Checking Docker installation...${NC}"
if ! command -v docker &> /dev/null; then
    echo -e "${RED}  ✗ Docker not found! Please install Docker.${NC}"
    exit 1
fi
echo -e "${GREEN}  ✓ Docker found: $(docker --version)${NC}"

# Check Docker Compose
echo -e "${YELLOW}→ Checking Docker Compose...${NC}"
if ! command -v docker-compose &> /dev/null; then
    echo -e "${RED}  ✗ Docker Compose not found!${NC}"
    exit 1
fi
echo -e "${GREEN}  ✓ Docker Compose found: $(docker-compose --version)${NC}"

echo ""
echo -e "${YELLOW}→ Building R-Share server (this may take 5-10 minutes)...${NC}"
docker-compose build

if [ $? -ne 0 ]; then
    echo -e "${RED}  ✗ Build failed!${NC}"
    exit 1
fi

echo -e "${GREEN}  ✓ Build successful!${NC}"
echo ""

echo -e "${YELLOW}→ Starting R-Share server...${NC}"
docker-compose up -d

if [ $? -ne 0 ]; then
    echo -e "${RED}  ✗ Failed to start server!${NC}"
    exit 1
fi

echo -e "${GREEN}  ✓ Server started!${NC}"
echo ""

# Wait for health check
echo -e "${YELLOW}→ Waiting for server to be ready...${NC}"
sleep 10

for i in {1..12}; do
    if curl -f -s http://localhost:8080/actuator/health > /dev/null 2>&1; then
        echo -e "${GREEN}  ✓ Server is healthy!${NC}"
        HEALTHY=true
        break
    else
        echo -e "${YELLOW}  ⏳ Waiting... ($i/12)${NC}"
        sleep 5
    fi
done

if [ -z "$HEALTHY" ]; then
    echo -e "${RED}  ✗ Server health check failed!${NC}"
    echo ""
    echo -e "${YELLOW}Check logs with: docker-compose logs rshare-server${NC}"
    exit 1
fi