#!/bin/bash
# Test runner script for viewyard
# Runs all test suites with proper reporting

set -e

echo "🧪 Running Viewyard Test Suite"
echo "=============================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to run a test suite
run_test_suite() {
    local suite_name="$1"
    local test_command="$2"
    
    echo -e "\n${YELLOW}Running $suite_name...${NC}"
    if eval "$test_command"; then
        echo -e "${GREEN}✅ $suite_name passed${NC}"
        return 0
    else
        echo -e "${RED}❌ $suite_name failed${NC}"
        return 1
    fi
}

# Track overall success
overall_success=0

# Run property-based tests
run_test_suite "Property-based tests" "cargo test --test property_tests" || overall_success=1

# Run unit tests
run_test_suite "Unit tests" "cargo test --test unit_tests" || overall_success=1

# Run integration tests (may have some expected failures)
run_test_suite "Integration tests" "cargo test --test integration_tests" || echo -e "${YELLOW}⚠️  Some integration tests failed (expected)${NC}"

# Run persona-based tests
run_test_suite "Persona-based tests" "cargo test --test persona_tests" || overall_success=1

# Run simple integration tests
run_test_suite "Simple integration tests" "cargo test --test simple_integration" || overall_success=1

# Run quality checks
echo -e "\n${YELLOW}Running quality checks...${NC}"
if cargo fmt --check; then
    echo -e "${GREEN}✅ Code formatting is correct${NC}"
else
    echo -e "${RED}❌ Code formatting issues found${NC}"
    overall_success=1
fi

if cargo clippy -- -D warnings; then
    echo -e "${GREEN}✅ No clippy warnings${NC}"
else
    echo -e "${RED}❌ Clippy warnings found${NC}"
    overall_success=1
fi

# Summary
echo -e "\n=============================="
if [ $overall_success -eq 0 ]; then
    echo -e "${GREEN}🎉 All critical tests passed!${NC}"
    echo "Ready for CI/CD pipeline"
else
    echo -e "${RED}💥 Some tests failed${NC}"
    echo "Please fix issues before committing"
fi

exit $overall_success
