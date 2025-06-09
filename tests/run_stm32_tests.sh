#!/bin/bash

# STM32 GDB Debug Test Runner
# Comprehensive test suite for STM32 debugging functionality

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TEST_CONFIG="$SCRIPT_DIR/fixtures/stm32_test_config.json"

# Function to print colored output
print_header() {
    echo -e "${PURPLE}========================================${NC}"
    echo -e "${PURPLE}$1${NC}"
    echo -e "${PURPLE}========================================${NC}"
}

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    local issues=0
    
    # Check if we're in the right directory
    if [ ! -f "$PROJECT_ROOT/Cargo.toml" ]; then
        print_error "Not in project root directory"
        issues=$((issues + 1))
    fi
    
    # Check if Rust/Cargo is available
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo not found - Rust toolchain required"
        issues=$((issues + 1))
    fi
    
    # Check if test configuration exists
    if [ ! -f "$TEST_CONFIG" ]; then
        print_error "Test configuration not found: $TEST_CONFIG"
        issues=$((issues + 1))
    fi
    
    # Check if ARM GDB is available
    if ! command -v arm-none-eabi-gdb &> /dev/null; then
        print_warning "ARM GDB not found - hardware tests will be skipped"
        print_status "Install with: brew install arm-none-eabi-gdb"
    fi
    
    # Check if ST-Link utilities are available
    if ! command -v st-util &> /dev/null; then
        print_warning "ST-Link utilities not found - hardware tests will be skipped"
        print_status "Install with: brew install stlink"
    fi
    
    # Check if STM32 hardware is connected
    if command -v st-info &> /dev/null; then
        if st-info --probe &> /dev/null; then
            print_success "STM32 hardware detected"
            export STM32_HARDWARE_AVAILABLE=1
        else
            print_warning "STM32 hardware not detected - hardware tests will be skipped"
            export STM32_HARDWARE_AVAILABLE=0
        fi
    else
        export STM32_HARDWARE_AVAILABLE=0
    fi
    
    # Check if STM32 project is built
    local elf_file="$PROJECT_ROOT/tests/stm32-f0-disco/stm32-f429/Debug/stm32-f429.elf"
    if [ ! -f "$elf_file" ]; then
        print_warning "STM32 ELF file not found - some tests may fail"
        print_status "Build STM32 project first if you want full test coverage"
    fi
    
    if [ $issues -gt 0 ]; then
        print_error "Prerequisites check failed with $issues issues"
        return 1
    fi
    
    print_success "Prerequisites check passed"
    return 0
}

# Function to build the project
build_project() {
    print_status "Building MCP GDB server..."
    
    cd "$PROJECT_ROOT"
    
    if cargo build --release; then
        print_success "Build successful"
    else
        print_error "Build failed"
        return 1
    fi
    
    # Check if binary exists
    if [ -f "$PROJECT_ROOT/target/release/mcp-server-gdb" ]; then
        print_success "MCP server binary available"
    else
        print_error "MCP server binary not found after build"
        return 1
    fi
}

# Function to run specific test category
run_test_category() {
    local category=$1
    local description=$2
    local requires_hardware=$3
    
    print_header "Running $category tests: $description"
    
    # Skip hardware tests if hardware not available
    if [ "$requires_hardware" = "true" ] && [ "${STM32_HARDWARE_AVAILABLE:-0}" = "0" ]; then
        print_warning "Skipping $category tests - STM32 hardware not available"
        return 0
    fi
    
    cd "$PROJECT_ROOT"
    
    # Run tests with timeout
    local test_pattern="test_stm32"
    if [ "$category" = "unit" ]; then
        test_pattern="tests/unit"
    elif [ "$category" = "integration" ]; then
        test_pattern="tests/integration"
    elif [ "$category" = "hardware" ]; then
        test_pattern="tests/hardware"
    elif [ "$category" = "e2e" ]; then
        test_pattern="tests/e2e"
    elif [ "$category" = "performance" ]; then
        test_pattern="tests/performance"
    fi
    
    print_status "Running tests matching pattern: $test_pattern"
    
    if timeout 300 cargo test --test "*" -- --test-threads=1 --nocapture; then
        print_success "$category tests completed successfully"
        return 0
    else
        print_error "$category tests failed or timed out"
        return 1
    fi
}

# Function to run all tests
run_all_tests() {
    print_header "STM32 GDB Debug Test Suite"
    
    local total_categories=0
    local passed_categories=0
    local skipped_categories=0
    
    # Test categories with their requirements
    declare -A test_categories=(
        ["unit"]="Unit tests for MCP GDB server functions|false"
        ["integration"]="Integration tests for STM32 debugging sessions|true"
        ["hardware"]="Hardware-specific debugging tests|true"
        ["e2e"]="End-to-end debugging workflow tests|true"
        ["performance"]="Performance and timing tests|true"
    )
    
    for category in "${!test_categories[@]}"; do
        IFS='|' read -r description requires_hardware <<< "${test_categories[$category]}"
        total_categories=$((total_categories + 1))
        
        if run_test_category "$category" "$description" "$requires_hardware"; then
            passed_categories=$((passed_categories + 1))
        elif [ "$requires_hardware" = "true" ] && [ "${STM32_HARDWARE_AVAILABLE:-0}" = "0" ]; then
            skipped_categories=$((skipped_categories + 1))
        fi
        
        echo
    done
    
    # Summary
    print_header "Test Summary"
    echo "Total categories: $total_categories"
    echo "Passed: $passed_categories"
    echo "Skipped: $skipped_categories"
    echo "Failed: $((total_categories - passed_categories - skipped_categories))"
    
    if [ $passed_categories -eq $total_categories ]; then
        print_success "All tests passed!"
        return 0
    elif [ $((passed_categories + skipped_categories)) -eq $total_categories ]; then
        print_success "All available tests passed (some skipped due to missing hardware)"
        return 0
    else
        print_error "Some tests failed"
        return 1
    fi
}

# Function to show usage
show_usage() {
    echo "STM32 GDB Debug Test Runner"
    echo "Usage: $0 [COMMAND]"
    echo
    echo "Commands:"
    echo "  all              Run all test categories"
    echo "  unit             Run unit tests only"
    echo "  integration      Run integration tests"
    echo "  hardware         Run hardware tests"
    echo "  e2e              Run end-to-end tests"
    echo "  performance      Run performance tests"
    echo "  check            Check prerequisites only"
    echo "  build            Build project only"
    echo "  help             Show this help message"
    echo
    echo "Environment Variables:"
    echo "  STM32_HARDWARE_AVAILABLE  Set to 1 to force hardware tests (auto-detected)"
    echo "  RUST_LOG                  Set log level (debug, info, warn, error)"
    echo
    echo "Examples:"
    echo "  $0 all                    # Run all tests"
    echo "  $0 unit                   # Run unit tests only"
    echo "  $0 check                  # Check if environment is ready"
    echo "  RUST_LOG=debug $0 unit    # Run unit tests with debug logging"
}

# Main script logic
main() {
    case "${1:-all}" in
        "all")
            check_prerequisites || exit 1
            build_project || exit 1
            run_all_tests
            ;;
        "unit")
            check_prerequisites || exit 1
            build_project || exit 1
            run_test_category "unit" "Unit tests for MCP GDB server functions" "false"
            ;;
        "integration")
            check_prerequisites || exit 1
            build_project || exit 1
            run_test_category "integration" "Integration tests for STM32 debugging sessions" "true"
            ;;
        "hardware")
            check_prerequisites || exit 1
            build_project || exit 1
            run_test_category "hardware" "Hardware-specific debugging tests" "true"
            ;;
        "e2e")
            check_prerequisites || exit 1
            build_project || exit 1
            run_test_category "e2e" "End-to-end debugging workflow tests" "true"
            ;;
        "performance")
            check_prerequisites || exit 1
            build_project || exit 1
            run_test_category "performance" "Performance and timing tests" "true"
            ;;
        "check")
            check_prerequisites
            ;;
        "build")
            build_project
            ;;
        "help"|*)
            show_usage
            ;;
    esac
}

# Run main function with all arguments
main "$@"
