#!/bin/bash

# Test script for Chronos API endpoints
# Currently tests existing user endpoints and shows structure for future auth endpoints

BASE_URL="http://127.0.0.1:3000"
API_BASE="$BASE_URL/api"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_test() {
    echo -e "${BLUE}=== $1 ===${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

# Function to test HTTP endpoint
test_endpoint() {
    local method=$1
    local url=$2
    local data=$3
    local expected_status=$4
    local description=$5

    echo "Testing: $description"
    echo "Request: $method $url"

    if [ -n "$data" ]; then
        echo "Data: $data"
        response=$(curl -s -w "\n%{http_code}" -X "$method" \
            -H "Content-Type: application/json" \
            -d "$data" "$url")
    else
        response=$(curl -s -w "\n%{http_code}" -X "$method" "$url")
    fi

    # Extract status code and body
    status_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n -1)

    echo "Status: $status_code"
    echo "Response: $body"

    if [ "$status_code" = "$expected_status" ]; then
        print_success "Test passed"
    else
        print_error "Test failed - Expected: $expected_status, Got: $status_code"
    fi
    echo
}

# Check if server is running
print_test "Checking if server is running"
if ! curl -s "$BASE_URL" > /dev/null; then
    print_error "Server is not running at $BASE_URL"
    print_warning "Please start the server with: cargo run"
    exit 1
fi
print_success "Server is running"
echo

# Test existing user endpoints
print_test "Testing existing user endpoints"

# Test 1: Get all users
test_endpoint "GET" "$API_BASE/users" "" "200" "Get all users"

# Test 2: Get user by email (this user was created during auth testing)
test_endpoint "GET" "$API_BASE/users/email/nonexistent@example.com" "" "404" "Get user by email (non-existent)"

# Test 3: Get user by ID (using a random UUID)
random_uuid="550e8400-e29b-41d4-a716-446655440000"
test_endpoint "GET" "$API_BASE/users/$random_uuid" "" "404" "Get user by ID (non-existent)"

# Test authentication endpoints
print_test "Testing authentication endpoints"

# Test register endpoint with valid data
test_endpoint "POST" "$API_BASE/auth/register" \
    '{"name": "New Test User", "email": "newuser@example.com", "password": "ValidPass123@"}' \
    "201" "Register endpoint with valid data"

# Test register endpoint with duplicate email (should fail)
test_endpoint "POST" "$API_BASE/auth/register" \
    '{"name": "Duplicate User", "email": "newuser@example.com", "password": "ValidPass123@"}' \
    "409" "Register endpoint with duplicate email"

# Test register endpoint with invalid password
test_endpoint "POST" "$API_BASE/auth/register" \
    '{"name": "Invalid Pass User", "email": "invalid@example.com", "password": "weak"}' \
    "400" "Register endpoint with invalid password"

# Test login endpoint with valid credentials
test_endpoint "POST" "$API_BASE/auth/login" \
    '{"email": "newuser@example.com", "password": "ValidPass123@"}' \
    "200" "Login endpoint with valid credentials"

# Test login endpoint with invalid credentials
test_endpoint "POST" "$API_BASE/auth/login" \
    '{"email": "newuser@example.com", "password": "wrongpassword"}' \
    "401" "Login endpoint with invalid credentials"

# Test login endpoint with non-existent user
test_endpoint "POST" "$API_BASE/auth/login" \
    '{"email": "nonexistent@example.com", "password": "ValidPass123@"}' \
    "401" "Login endpoint with non-existent user"

