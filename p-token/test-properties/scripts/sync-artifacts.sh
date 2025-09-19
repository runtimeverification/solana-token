#!/bin/bash

# Sync artifacts from remote server to local machine
# Usage: ./scripts/sync-artifacts.sh

# Set colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Remote server details
REMOTE_HOST="zhaoji"
REMOTE_PATH="/home/zhaoji/solana-token/p-token/test-properties/artefacts/"

# Local destination path (relative to project root)
LOCAL_PATH="./artefacts/"

echo -e "${GREEN}Starting artifact sync from remote server...${NC}"
echo -e "${YELLOW}Remote: ${REMOTE_HOST}:${REMOTE_PATH}${NC}"
echo -e "${YELLOW}Local: ${LOCAL_PATH}${NC}"

# Create local artefacts directory if it doesn't exist
mkdir -p "$LOCAL_PATH"

# Perform the sync
rsync -avz --progress "${REMOTE_HOST}:${REMOTE_PATH}" "${LOCAL_PATH}"

# Check if rsync was successful
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Sync completed successfully!${NC}"
else
    echo -e "${RED}✗ Sync failed. Please check your connection and paths.${NC}"
    exit 1
fi