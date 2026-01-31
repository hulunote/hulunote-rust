#!/bin/bash

# =====================================================
# Registration Code Generator
# =====================================================
# Usage:
#   ./scripts/generate_registration_code.sh [VALIDITY_PERIOD]
#
# VALIDITY_PERIOD options:
#   6months  - 6 months (180 days)
#   1year    - 1 year (365 days)
#   2years   - 2 years (730 days)
#   custom   - Custom number of days (will prompt)
#
# Example:
#   ./scripts/generate_registration_code.sh 6months
#   ./scripts/generate_registration_code.sh 1year
# =====================================================

set -e

# Default database URL (can be overridden with environment variable)
DATABASE_URL=${DATABASE_URL:-"postgresql://localhost/hulunote_open"}

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to generate random code
generate_code() {
    # Generate a random code using openssl
    # Format: XXXX-XXXX-XXXX-XXXX (16 characters + 3 hyphens)
    local part1=$(openssl rand -hex 2 | tr '[:lower:]' '[:upper:]')
    local part2=$(openssl rand -hex 2 | tr '[:lower:]' '[:upper:]')
    local part3=$(openssl rand -hex 2 | tr '[:lower:]' '[:upper:]')
    local part4=$(openssl rand -hex 2 | tr '[:lower:]' '[:upper:]')
    echo "${part1}-${part2}-${part3}-${part4}"
}

# Parse validity period argument
if [ -z "$1" ]; then
    echo -e "${YELLOW}Usage: $0 [6months|1year|2years|custom]${NC}"
    echo ""
    echo "Examples:"
    echo "  $0 6months   # Generate code valid for 6 months"
    echo "  $0 1year     # Generate code valid for 1 year"
    echo "  $0 2years    # Generate code valid for 2 years"
    echo "  $0 custom    # Generate code with custom validity"
    exit 1
fi

case "$1" in
    6months|6m)
        VALIDITY_DAYS=180
        VALIDITY_DESC="6 months"
        ;;
    1year|1y)
        VALIDITY_DAYS=365
        VALIDITY_DESC="1 year"
        ;;
    2years|2y)
        VALIDITY_DAYS=730
        VALIDITY_DESC="2 years"
        ;;
    custom)
        echo -n "Enter number of days: "
        read VALIDITY_DAYS
        VALIDITY_DESC="$VALIDITY_DAYS days"
        ;;
    *)
        echo -e "${YELLOW}Error: Invalid validity period '$1'${NC}"
        echo "Valid options: 6months, 1year, 2years, custom"
        exit 1
        ;;
esac

# Validate validity days
if ! [[ "$VALIDITY_DAYS" =~ ^[0-9]+$ ]] || [ "$VALIDITY_DAYS" -lt 1 ]; then
    echo -e "${YELLOW}Error: Invalid number of days${NC}"
    exit 1
fi

# Generate registration code
CODE=$(generate_code)

echo ""
echo -e "${BLUE}==================================================${NC}"
echo -e "${BLUE}  Generating Registration Code${NC}"
echo -e "${BLUE}==================================================${NC}"
echo ""
echo -e "Code:          ${GREEN}$CODE${NC}"
echo -e "Validity:      ${GREEN}$VALIDITY_DESC ($VALIDITY_DAYS days)${NC}"
echo ""

# Insert into database
psql "$DATABASE_URL" <<EOF
INSERT INTO registration_codes (code, validity_days, is_used)
VALUES ('$CODE', $VALIDITY_DAYS, false);
EOF

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Registration code created successfully!${NC}"
    echo ""
    echo -e "${BLUE}==================================================${NC}"
    echo -e "Share this code with users for registration:"
    echo ""
    echo -e "  ${GREEN}$CODE${NC}"
    echo ""
    echo -e "${BLUE}==================================================${NC}"
else
    echo -e "${YELLOW}✗ Failed to create registration code${NC}"
    exit 1
fi
