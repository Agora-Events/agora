#!/bin/bash

# Health Check Endpoints Test Script
# This script tests all health check endpoints with various scenarios

BASE_URL="http://localhost:3001/api/v1"

echo "=========================================="
echo "Testing Health Check Endpoints"
echo "=========================================="
echo ""

# Test 1: Basic health check
echo "1. Testing GET /api/v1/health (Basic API health check)"
echo "Expected: 200 OK"
curl -i -X GET "$BASE_URL/health"
echo ""
echo ""

# Test 2: Database health check (with database running)
echo "2. Testing GET /api/v1/health/blockchain (Soroban RPC reachability check)"
echo "Expected: 200 OK with blockchain: soroban"
curl -i -X GET "$BASE_URL/health/blockchain"
echo ""
echo ""

# Test 3: Database health check (with database running)
echo "3. Testing GET /api/v1/health/db (Database health check - DB running)"
echo "Expected: 200 OK with database: connected"
curl -i -X GET "$BASE_URL/health/db"
echo ""
echo ""

# Test 4: Readiness check (with both healthy)
echo "4. Testing GET /api/v1/health/ready (Readiness check - all healthy)"
echo "Expected: 200 OK with api: ok, database: ok"
curl -i -X GET "$BASE_URL/health/ready"
echo ""
echo ""

echo "=========================================="
echo "Manual Test Instructions"
echo "=========================================="
echo ""
echo "To test failure scenarios:"
echo ""
echo "1. Stop the database:"
echo "   docker-compose down (or stop your PostgreSQL service)"
echo ""
echo "2. Test /api/v1/health/db endpoint (should return 503):"
echo "   curl -i -X GET $BASE_URL/health/db"
echo ""
echo "3. Test /api/v1/health/ready endpoint (should return 503):"
echo "   curl -i -X GET $BASE_URL/health/ready"
echo ""
echo "4. Test /api/v1/health endpoint (should still return 503 in the current implementation because it also checks database connectivity):"
echo "   curl -i -X GET $BASE_URL/health"
echo ""
echo "=========================================="
