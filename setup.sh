#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Setting up JSON Store Frontend...${NC}"

# Check if npm is installed
if ! command -v npm &> /dev/null; then
    echo "npm is not installed. Please install Node.js and npm first."
    exit 1
fi

# Install dependencies
echo -e "${BLUE}Installing dependencies...${NC}"
npm install

# Create .env.local file
echo -e "${BLUE}Creating environment configuration...${NC}"

# Prompt for configuration values
echo -e "${GREEN}Please enter the following configuration values:${NC}"
read -p "Treasury Address (xion1...): " treasury_address
read -p "Contract Address (xion1...): " contract_address
read -p "RPC URL [https://rpc.xion-testnet-2.burnt.com:443]: " rpc_url
rpc_url=${rpc_url:-https://rpc.xion-testnet-2.burnt.com:443}
read -p "REST URL [https://api.xion-testnet-2.burnt.com]: " rest_url
rest_url=${rest_url:-https://api.xion-testnet-2.burnt.com}

# Create .env.local file
cat > .env.local << EOF
NEXT_PUBLIC_TREASURY_ADDRESS=${treasury_address}
NEXT_PUBLIC_CONTRACT_ADDRESS=${contract_address}
NEXT_PUBLIC_RPC_URL=${rpc_url}
NEXT_PUBLIC_REST_URL=${rest_url}
EOF

echo -e "${BLUE}Configuration saved to .env.local${NC}"

# Build the application
echo -e "${BLUE}Building the application...${NC}"
npm run build

echo -e "${GREEN}Setup complete!${NC}"
echo -e "${BLUE}To start the application in development mode, run:${NC} npm run dev"
echo -e "${BLUE}To start the application in production mode, run:${NC} npm start" 