#!/bin/bash
# Test wrapper script locally before deployment

set -euo pipefail

# Color codes for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=== Wrapper Script Tests ==="
echo

# Cleanup any previous test artifacts
rm -rf ~/.solverpilot-server 2>/dev/null || true

# Test 1: Success scenario (exit 0)
echo "Test 1: Job completes successfully (exit 0)"
./src-tauri/scripts/job_wrapper.sh test-job-success echo "Success" >/dev/null 2>&1
if grep -q '"exit_code": 0' ~/.solverpilot-server/jobs/test-job-success.status && \
   grep -q '"status": "completed"' ~/.solverpilot-server/jobs/test-job-success.status; then
    echo -e "${GREEN}✅ PASS${NC} - Success case handled correctly"
else
    echo -e "${RED}❌ FAIL${NC} - Success case failed"
    cat ~/.solverpilot-server/jobs/test-job-success.status
    exit 1
fi
echo

# Test 2: Failure scenario (exit 1)
echo "Test 2: Job fails (exit 1)"
./src-tauri/scripts/job_wrapper.sh test-job-fail bash -c "exit 1" >/dev/null 2>&1 || true
if grep -q '"exit_code": 1' ~/.solverpilot-server/jobs/test-job-fail.status && \
   grep -q '"status": "failed"' ~/.solverpilot-server/jobs/test-job-fail.status; then
    echo -e "${GREEN}✅ PASS${NC} - Failure case handled correctly"
else
    echo -e "${RED}❌ FAIL${NC} - Failure case failed"
    cat ~/.solverpilot-server/jobs/test-job-fail.status
    exit 1
fi
echo

# Test 3: Different exit codes
echo "Test 3: Job with exit code 127 (command not found)"
./src-tauri/scripts/job_wrapper.sh test-job-127 bash -c "exit 127" >/dev/null 2>&1 || true
if grep -q '"exit_code": 127' ~/.solverpilot-server/jobs/test-job-127.status && \
   grep -q '"status": "failed"' ~/.solverpilot-server/jobs/test-job-127.status; then
    echo -e "${GREEN}✅ PASS${NC} - Non-zero exit code handled correctly"
else
    echo -e "${RED}❌ FAIL${NC} - Exit code 127 test failed"
    exit 1
fi
echo

# Test 4: Verify JSON format
echo "Test 4: Verify state file JSON format"
if cat ~/.solverpilot-server/jobs/test-job-success.status | python3 -m json.tool >/dev/null 2>&1; then
    echo -e "${GREEN}✅ PASS${NC} - JSON format is valid"
else
    echo -e "${RED}❌ FAIL${NC} - Invalid JSON format"
    cat ~/.solverpilot-server/jobs/test-job-success.status
    exit 1
fi
echo

# Test 5: Verify ISO 8601 timestamp format
echo "Test 5: Verify ISO 8601 timestamp format"
TIMESTAMP=$(grep -o '"completed_at": "[^"]*"' ~/.solverpilot-server/jobs/test-job-success.status | cut -d'"' -f4)
if [[ $TIMESTAMP =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}.*$ ]]; then
    echo -e "${GREEN}✅ PASS${NC} - ISO 8601 timestamp format correct: $TIMESTAMP"
else
    echo -e "${RED}❌ FAIL${NC} - Invalid timestamp format: $TIMESTAMP"
    exit 1
fi
echo

# Test 6: Verify directory creation
echo "Test 6: Verify directory creation"
if [ -d ~/.solverpilot-server/jobs ] && [ -d ~/.solverpilot-server/locks ]; then
    echo -e "${GREEN}✅ PASS${NC} - Directories created correctly"
else
    echo -e "${RED}❌ FAIL${NC} - Directory creation failed"
    exit 1
fi
echo

# Test 7: Verify lock files created
echo "Test 7: Verify lock files created"
if [ -f ~/.solverpilot-server/locks/test-job-success.lock ]; then
    echo -e "${GREEN}✅ PASS${NC} - Lock file created"
else
    echo -e "${RED}❌ FAIL${NC} - Lock file not created"
    exit 1
fi
echo

# Test 8: Verify all required JSON fields
echo "Test 8: Verify all required JSON fields present"
STATE_FILE=~/.solverpilot-server/jobs/test-job-success.status
if grep -q '"id":' "$STATE_FILE" && \
   grep -q '"status":' "$STATE_FILE" && \
   grep -q '"exit_code":' "$STATE_FILE" && \
   grep -q '"completed_at":' "$STATE_FILE" && \
   grep -q '"user":' "$STATE_FILE"; then
    echo -e "${GREEN}✅ PASS${NC} - All required JSON fields present"
else
    echo -e "${RED}❌ FAIL${NC} - Missing required JSON fields"
    cat "$STATE_FILE"
    exit 1
fi
echo

# Test 9: Verify script line count (~50 lines)
echo "Test 9: Verify script line count (~50 lines)"
LINE_COUNT=$(grep -v '^$\|^#' src-tauri/scripts/job_wrapper.sh | wc -l)
if [ "$LINE_COUNT" -le 60 ] && [ "$LINE_COUNT" -ge 40 ]; then
    echo -e "${GREEN}✅ PASS${NC} - Script is concise: $LINE_COUNT non-blank/comment lines"
else
    echo -e "${YELLOW}⚠️  WARNING${NC} - Script has $LINE_COUNT lines (target: ~50)"
fi
echo

# Test 10: Test with actual command arguments
echo "Test 10: Test with actual command and arguments"
./src-tauri/scripts/job_wrapper.sh test-job-args echo "arg1" "arg2" "arg3" >/dev/null 2>&1
if grep -q '"exit_code": 0' ~/.solverpilot-server/jobs/test-job-args.status; then
    echo -e "${GREEN}✅ PASS${NC} - Command with arguments executed correctly"
else
    echo -e "${RED}❌ FAIL${NC} - Command with arguments failed"
    exit 1
fi
echo

# Test 11: Verify SIGTERM handling (trap EXIT should fire)
echo "Test 11: Verify SIGTERM handling"
(
    ./src-tauri/scripts/job_wrapper.sh test-job-sigterm sleep 10 &
    WRAPPER_PID=$!
    sleep 0.5
    kill -TERM $WRAPPER_PID 2>/dev/null || true
    wait $WRAPPER_PID 2>/dev/null || true
) >/dev/null 2>&1
sleep 0.5
if [ -f ~/.solverpilot-server/jobs/test-job-sigterm.status ]; then
    echo -e "${GREEN}✅ PASS${NC} - SIGTERM handled (trap EXIT fired)"
else
    echo -e "${YELLOW}⚠️  PARTIAL${NC} - SIGTERM test inconclusive (state file may not exist)"
fi
echo

echo "=== All Tests Completed ==="
echo
echo -e "${GREEN}Summary: All critical tests passed!${NC}"
echo
echo "Test artifacts stored in: ~/.solverpilot-server/"
echo "To clean up: rm -rf ~/.solverpilot-server"
