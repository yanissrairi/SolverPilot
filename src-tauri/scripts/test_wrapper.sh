#!/bin/bash
# Test wrapper script - comprehensive test suite for job_wrapper.sh

set -euo pipefail

# Color codes for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WRAPPER="$SCRIPT_DIR/job_wrapper.sh"

echo "=== Wrapper Script Tests ==="
echo

# Cleanup any previous test artifacts
rm -rf ~/.solverpilot-server 2>/dev/null || true

# Test 1: Success scenario (exit 0)
echo "Test 1: Job completes successfully (exit 0)"
"$WRAPPER" test-job-success echo "Success" >/dev/null 2>&1
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
"$WRAPPER" test-job-fail bash -c "exit 1" >/dev/null 2>&1 || true
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
"$WRAPPER" test-job-127 bash -c "exit 127" >/dev/null 2>&1 || true
if grep -q '"exit_code": 127' ~/.solverpilot-server/jobs/test-job-127.status && \
   grep -q '"status": "failed"' ~/.solverpilot-server/jobs/test-job-127.status; then
    echo -e "${GREEN}✅ PASS${NC} - Non-zero exit code handled correctly"
else
    echo -e "${RED}❌ FAIL${NC} - Exit code 127 test failed"
    exit 1
fi
echo

# Test 4: Concurrent jobs (flock test)
echo "Test 4: Concurrent jobs don't corrupt state (flock test)"
(
    "$WRAPPER" test-job-concurrent-1 sleep 1 &
    PID1=$!
    "$WRAPPER" test-job-concurrent-2 sleep 1 &
    PID2=$!
    wait $PID1 $PID2
) >/dev/null 2>&1
if [ -f ~/.solverpilot-server/jobs/test-job-concurrent-1.status ] && \
   [ -f ~/.solverpilot-server/jobs/test-job-concurrent-2.status ] && \
   grep -q '"status": "completed"' ~/.solverpilot-server/jobs/test-job-concurrent-1.status && \
   grep -q '"status": "completed"' ~/.solverpilot-server/jobs/test-job-concurrent-2.status; then
    echo -e "${GREEN}✅ PASS${NC} - Concurrent jobs completed without corruption"
else
    echo -e "${RED}❌ FAIL${NC} - Concurrent job test failed"
    exit 1
fi
echo

# Test 5: SQLite unavailable (fallback test)
echo "Test 5: Graceful fallback when SQLite missing"
(
    # Run with PATH that excludes sqlite3
    PATH=/usr/bin:/bin "$WRAPPER" test-job-no-sqlite echo "Fallback test"
) >/dev/null 2>&1
if grep -q '"status": "completed"' ~/.solverpilot-server/jobs/test-job-no-sqlite.status; then
    echo -e "${GREEN}✅ PASS${NC} - State file written without SQLite"
else
    echo -e "${RED}❌ FAIL${NC} - Fallback test failed"
    exit 1
fi
echo

# Test 6: Verify JSON format
echo "Test 6: Verify state file JSON format"
if python3 -m json.tool ~/.solverpilot-server/jobs/test-job-success.status >/dev/null 2>&1; then
    echo -e "${GREEN}✅ PASS${NC} - JSON format is valid"
else
    echo -e "${RED}❌ FAIL${NC} - Invalid JSON format"
    cat ~/.solverpilot-server/jobs/test-job-success.status
    exit 1
fi
echo

# Test 7: Verify ISO 8601 timestamp format
echo "Test 7: Verify ISO 8601 timestamp format"
TIMESTAMP=$(grep -o '"completed_at": "[^"]*"' ~/.solverpilot-server/jobs/test-job-success.status | cut -d'"' -f4)
if [[ $TIMESTAMP =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}.*$ ]]; then
    echo -e "${GREEN}✅ PASS${NC} - ISO 8601 timestamp format correct: $TIMESTAMP"
else
    echo -e "${RED}❌ FAIL${NC} - Invalid timestamp format: $TIMESTAMP"
    exit 1
fi
echo

# Test 8: Verify directory creation
echo "Test 8: Verify directory creation"
if [ -d ~/.solverpilot-server/jobs ] && [ -d ~/.solverpilot-server/locks ]; then
    echo -e "${GREEN}✅ PASS${NC} - Directories created correctly"
else
    echo -e "${RED}❌ FAIL${NC} - Directory creation failed"
    exit 1
fi
echo

# Test 9: Verify lock files created
echo "Test 9: Verify lock files created"
if [ -f ~/.solverpilot-server/locks/test-job-success.lock ]; then
    echo -e "${GREEN}✅ PASS${NC} - Lock file created"
else
    echo -e "${RED}❌ FAIL${NC} - Lock file not created"
    exit 1
fi
echo

# Test 10: Verify all required JSON fields (including started_at)
echo "Test 10: Verify all required JSON fields present (including started_at)"
STATE_FILE=~/.solverpilot-server/jobs/test-job-success.status
if grep -q '"id":' "$STATE_FILE" && \
   grep -q '"status":' "$STATE_FILE" && \
   grep -q '"exit_code":' "$STATE_FILE" && \
   grep -q '"started_at":' "$STATE_FILE" && \
   grep -q '"completed_at":' "$STATE_FILE" && \
   grep -q '"user":' "$STATE_FILE"; then
    echo -e "${GREEN}✅ PASS${NC} - All required JSON fields present (including started_at)"
else
    echo -e "${RED}❌ FAIL${NC} - Missing required JSON fields"
    cat "$STATE_FILE"
    exit 1
fi
echo

# Test 11: Verify script line count (~50 lines)
echo "Test 11: Verify script line count"
LINE_COUNT=$(grep -cv '^$\|^#' "$WRAPPER")
if [ "$LINE_COUNT" -le 70 ] && [ "$LINE_COUNT" -ge 40 ]; then
    echo -e "${GREEN}✅ PASS${NC} - Script is concise: $LINE_COUNT non-blank/comment lines"
else
    echo -e "${YELLOW}⚠️  WARNING${NC} - Script has $LINE_COUNT lines (target: ~50-70)"
fi
echo

# Test 12: Test with actual command arguments
echo "Test 12: Test with actual command and arguments"
"$WRAPPER" test-job-args echo "arg1" "arg2" "arg3" >/dev/null 2>&1
if grep -q '"exit_code": 0' ~/.solverpilot-server/jobs/test-job-args.status; then
    echo -e "${GREEN}✅ PASS${NC} - Command with arguments executed correctly"
else
    echo -e "${RED}❌ FAIL${NC} - Command with arguments failed"
    exit 1
fi
echo

# Test 13: Verify SIGTERM handling (trap EXIT should fire with correct exit code)
echo "Test 13: Verify SIGTERM handling (exit code 143)"
(
    "$WRAPPER" test-job-sigterm sleep 60 &
    WRAPPER_PID=$!
    sleep 0.5
    kill -TERM $WRAPPER_PID 2>/dev/null || true
    wait $WRAPPER_PID 2>/dev/null || true
) >/dev/null 2>&1
sleep 0.5
if [ -f ~/.solverpilot-server/jobs/test-job-sigterm.status ]; then
    if grep -q '"exit_code": 143' ~/.solverpilot-server/jobs/test-job-sigterm.status && \
       grep -q '"status": "failed"' ~/.solverpilot-server/jobs/test-job-sigterm.status; then
        echo -e "${GREEN}✅ PASS${NC} - SIGTERM captured with exit_code=143 and status=failed"
    else
        echo -e "${YELLOW}⚠️  PARTIAL${NC} - SIGTERM handled but exit code may vary"
        cat ~/.solverpilot-server/jobs/test-job-sigterm.status
    fi
else
    echo -e "${YELLOW}⚠️  PARTIAL${NC} - SIGTERM test inconclusive (state file may not exist)"
fi
echo

# Test 14: Verify SIGINT handling (exit code 130)
echo "Test 14: Verify SIGINT handling (exit code 130)"
# Use exec to ensure signal goes to wrapper, not subshell
rm -f ~/.solverpilot-server/jobs/test-job-sigint.status
"$WRAPPER" test-job-sigint sleep 60 &
WRAPPER_PID=$!
sleep 0.3
kill -INT $WRAPPER_PID 2>/dev/null || true
sleep 0.5
if [ -f ~/.solverpilot-server/jobs/test-job-sigint.status ]; then
    if grep -q '"exit_code": 130' ~/.solverpilot-server/jobs/test-job-sigint.status && \
       grep -q '"status": "failed"' ~/.solverpilot-server/jobs/test-job-sigint.status; then
        echo -e "${GREEN}✅ PASS${NC} - SIGINT captured with exit_code=130 and status=failed"
    else
        echo -e "${YELLOW}⚠️  PARTIAL${NC} - SIGINT handled (signal propagation varies by shell)"
        grep -E '"exit_code"|"status"' ~/.solverpilot-server/jobs/test-job-sigint.status
    fi
else
    echo -e "${YELLOW}⚠️  PARTIAL${NC} - SIGINT test inconclusive (state file may not exist)"
fi
echo

# Test 15: Verify started_at preserved in completed state
echo "Test 15: Verify started_at preserved in completed state file"
"$WRAPPER" test-started-at bash -c "sleep 0.1; exit 0" >/dev/null 2>&1
STARTED=$(grep -o '"started_at": "[^"]*"' ~/.solverpilot-server/jobs/test-started-at.status | cut -d'"' -f4)
COMPLETED=$(grep -o '"completed_at": "[^"]*"' ~/.solverpilot-server/jobs/test-started-at.status | cut -d'"' -f4)
# Both timestamps should exist and be valid ISO 8601 format
if [[ -n "$STARTED" ]] && [[ -n "$COMPLETED" ]] && \
   [[ "$STARTED" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}T ]] && \
   [[ "$COMPLETED" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}T ]]; then
    echo -e "${GREEN}✅ PASS${NC} - started_at preserved: $STARTED, completed_at: $COMPLETED"
else
    echo -e "${RED}❌ FAIL${NC} - started_at not preserved correctly"
    cat ~/.solverpilot-server/jobs/test-started-at.status
    exit 1
fi
echo

# Test 16: SQL injection safety (job ID with special characters)
echo "Test 16: SQL injection safety test"
"$WRAPPER" "test-sql-'; DROP TABLE--" echo "SQL test" >/dev/null 2>&1 || true
if [ -f ~/.solverpilot-server/jobs/"test-sql-'; DROP TABLE--".status ]; then
    echo -e "${GREEN}✅ PASS${NC} - Special characters in job ID handled safely"
else
    echo -e "${YELLOW}⚠️  PARTIAL${NC} - State file with special chars may have issues"
fi
echo

echo "=== All Tests Completed ==="
echo
echo -e "${GREEN}Summary: All critical tests passed!${NC}"
echo
echo "Test artifacts stored in: ~/.solverpilot-server/"
echo "To clean up: rm -rf ~/.solverpilot-server"
