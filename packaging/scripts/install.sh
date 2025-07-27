#!/usr/bin/env bash

set -euo pipefail  # Exit on error, undefined vars, pipe failures

# Script configuration
readonly SCRIPT_NAME="${0##*/}"
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Project files
readonly UDEV_RULE='99-af-pro-display.rules'
readonly SYSTEMD_UNIT="af-pro-display.service"
readonly BIN_NAME="af-pro-display"

# Project directories (go up 3 levels from script location)
readonly PROJECT_PATH="$(cd "${SCRIPT_DIR}/../.." && pwd)"
readonly PACKAGING_PATH="${PROJECT_PATH}/packaging"

# Source file paths
readonly UDEV_RULE_SRC="${PACKAGING_PATH}/udev/${UDEV_RULE}"
readonly SYSTEMD_UNIT_SRC="${PACKAGING_PATH}/scripts/${SYSTEMD_UNIT}"
readonly BIN_SRC="${PROJECT_PATH}/target/release/${BIN_NAME}"

# Installation target paths
readonly UDEV_RULE_DEST="/etc/udev/rules.d/${UDEV_RULE}"
readonly SYSTEMD_UNIT_DEST="${HOME}/.config/systemd/user/${SYSTEMD_UNIT}"
readonly BIN_DEST="${HOME}/.local/bin/${BIN_NAME}"

# Color codes for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

log_warn() {
    echo -e "${YELLOW}[WARNING]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*" >&2
}

# Function to display usage information
usage() {
    cat << EOF
Usage: ${SCRIPT_NAME} [OPTIONS]

Install af-pro-display application with udev rules and systemd service.

OPTIONS:
    -h, --help      Show this help message
    -v, --verbose   Enable verbose output
    -f, --force     Force reinstallation of existing files
    --build-only    Only build the binary, don't install
    --no-build      Skip building, assume binary exists

DESCRIPTION:
    This script installs the af-pro-display application by:
    1. Building the Rust binary (if needed)
    2. Installing udev rules (requires sudo)
    3. Installing systemd user service
    4. Installing the binary to ~/.local/bin
    5. Enabling and starting the systemd service

EOF
}

# Function to validate prerequisites
validate_environment() {
    log_info "Validating environment..."
    
    # Check if we're in the right directory structure
    if [[ ! -d "${PROJECT_PATH}" ]]; then
        log_error "Project path not found: ${PROJECT_PATH}"
        return 1
    fi
    
    if [[ ! -d "${PACKAGING_PATH}" ]]; then
        log_error "Packaging path not found: ${PACKAGING_PATH}"
        return 1
    fi
    
    # Check for required source files
    local missing_files=()
    
    [[ ! -f "${UDEV_RULE_SRC}" ]] && missing_files+=("${UDEV_RULE_SRC}")
    [[ ! -f "${SYSTEMD_UNIT_SRC}" ]] && missing_files+=("${SYSTEMD_UNIT_SRC}")
    
    if [[ ${#missing_files[@]} -gt 0 ]]; then
        log_error "Missing required source files:"
        printf '\t%s\n' "${missing_files[@]}"
        return 1
    fi
    
    # Check for cargo if we need to build
    if [[ "${BUILD_ONLY:-false}" == "true" || "${NO_BUILD:-false}" == "false" ]]; then
        if ! command -v cargo >/dev/null 2>&1; then
            log_error "cargo command not found. Please install Rust toolchain."
            return 1
        fi
    fi
    
    log_success "Environment validation passed"
}

# Function to display current configuration
show_environment() {
    if [[ "${VERBOSE:-false}" == "true" ]]; then
        log_info "Configuration:"
        cat << EOF
	Project path:     ${PROJECT_PATH}
	Packaging path:   ${PACKAGING_PATH}
	
	Source files:
	  udev rule:      ${UDEV_RULE_SRC}
	  systemd unit:   ${SYSTEMD_UNIT_SRC}
	  binary:         ${BIN_SRC}
	
	Installation targets:
	  udev rule:      ${UDEV_RULE_DEST}
	  systemd unit:   ${SYSTEMD_UNIT_DEST}
	  binary:         ${BIN_DEST}
EOF
    fi
}

# Function to build the binary
build_binary() {
    if [[ "${NO_BUILD:-false}" == "true" ]]; then
        log_info "Skipping build as requested"
        return 0
    fi
    
    if [[ -f "${BIN_SRC}" && "${FORCE:-false}" == "false" ]]; then
        log_info "Binary already exists: ${BIN_SRC}"
        return 0
    fi
    
    log_info "Building binary..."
    
    if ! (cd "${PROJECT_PATH}" && cargo build --release); then
        log_error "Failed to build binary"
        return 1
    fi
    
    if [[ ! -f "${BIN_SRC}" ]]; then
        log_error "Binary not found after build: ${BIN_SRC}"
        return 1
    fi
    
    log_success "Binary built successfully"
}

# Function to prompt for yes/no confirmation
confirm() {
    local prompt="$1"
    local response
    
    while true; do
        read -rp "${prompt} (y/n): " response
        case "${response,,}" in
            y|yes) return 0 ;;
            n|no) return 1 ;;
            *) echo "Please answer yes or no." ;;
        esac
    done
}

# Function to install udev rule
install_udev_rule() {
    if [[ -f "${UDEV_RULE_DEST}" && "${FORCE:-false}" == "false" ]]; then
        log_info "udev rule already installed: ${UDEV_RULE_DEST}"
        return 0
    fi
    
    log_info "Installing udev rule (requires sudo)..."
    
    if ! sudo cp "${UDEV_RULE_SRC}" "${UDEV_RULE_DEST}"; then
        log_error "Failed to install udev rule"
        return 1
    fi
    
    # Reload udev rules
    if ! sudo udevadm control --reload-rules; then
        log_warn "Failed to reload udev rules"
    fi
    
    log_success "udev rule installed"
}

# Function to install systemd unit
install_systemd_unit() {
    if [[ -f "${SYSTEMD_UNIT_DEST}" && "${FORCE:-false}" == "false" ]]; then
        log_info "systemd unit already installed: ${SYSTEMD_UNIT_DEST}"
        return 0
    fi
    
    log_info "Installing systemd unit..."
    
    # Create directory if it doesn't exist
    local dest_dir
    dest_dir="$(dirname "${SYSTEMD_UNIT_DEST}")"
    if ! mkdir -p "${dest_dir}"; then
        log_error "Failed to create directory: ${dest_dir}"
        return 1
    fi
    
    # Copy and modify the systemd unit file
    if ! cp "${SYSTEMD_UNIT_SRC}" "${SYSTEMD_UNIT_DEST}"; then
        log_error "Failed to copy systemd unit"
        return 1
    fi
    
    log_success "systemd unit installed"
}

# Function to install binary
install_binary() {
    if [[ -f "${BIN_DEST}" && "${FORCE:-false}" == "false" ]]; then
        log_info "Binary already installed: ${BIN_DEST}"
        return 0
    fi
    
    log_info "Installing binary..."
    
    # Create directory if it doesn't exist
    local dest_dir
    dest_dir="$(dirname "${BIN_DEST}")"
    if ! mkdir -p "${dest_dir}"; then
        log_error "Failed to create directory: ${dest_dir}"
        return 1
    fi
    
    if ! cp "${BIN_SRC}" "${BIN_DEST}"; then
        log_error "Failed to install binary"
        return 1
    fi
    
    if ! chmod +x "${BIN_DEST}"; then
        log_error "Failed to make binary executable"
        return 1
    fi
    
    log_success "Binary installed"
}

# Function to enable and start systemd service
enable_service() {
    log_info "Enabling and starting systemd service..."
    
    if ! systemctl --user daemon-reload; then
        log_error "Failed to reload systemd daemon"
        return 1
    fi
    
    if ! systemctl --user enable "${SYSTEMD_UNIT}"; then
        log_error "Failed to enable service"
        return 1
    fi
    
    if ! systemctl --user start "${SYSTEMD_UNIT}"; then
        log_error "Failed to start service"
        return 1
    fi
    
    log_success "Service enabled and started"
}

# Function to check service status
check_service_status() {
    log_info "Checking service status..."
    
    if systemctl --user is-active --quiet "${SYSTEMD_UNIT}"; then
        log_success "Service is running"
    else
        log_warn "Service is not running"
        log_info "Service status:"
        systemctl --user status "${SYSTEMD_UNIT}" --no-pager || true
    fi
}

# Main installation function
main() {
    local build_only=false
    local no_build=false
    local force=false
    local verbose=false
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                usage
                exit 0
                ;;
            -v|--verbose)
                verbose=true
                shift
                ;;
            -f|--force)
                force=true
                shift
                ;;
            --build-only)
                build_only=true
                shift
                ;;
            --no-build)
                no_build=true
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done
    
    # Export flags for use in functions
    export VERBOSE="${verbose}"
    export FORCE="${force}"
    export BUILD_ONLY="${build_only}"
    export NO_BUILD="${no_build}"
    
    # Validate conflicting options
    if [[ "${build_only}" == "true" && "${no_build}" == "true" ]]; then
        log_error "Cannot use --build-only and --no-build together"
        exit 1
    fi
    
    log_info "Starting af-pro-display installation..."
    
    validate_environment || exit 1
    show_environment
    
    # Build phase
    if [[ "${build_only}" == "true" ]]; then
        build_binary || exit 1
        log_success "Build completed successfully"
        exit 0
    fi
    
    # Check if binary exists (build if needed)
    if [[ ! -f "${BIN_SRC}" && "${no_build}" == "false" ]]; then
        log_warn "Binary not found: ${BIN_SRC}"
        if confirm "Would you like to build it now?"; then
            build_binary || exit 1
        else
            log_error "Cannot proceed without binary"
            exit 1
        fi
    elif [[ "${no_build}" == "false" ]]; then
        build_binary || exit 1
    fi
    
    # Installation phase
    log_info "Starting installation..."
    
    install_udev_rule || exit 1
    install_systemd_unit || exit 1
    install_binary || exit 1
    enable_service || exit 1
    
    log_success "Installation completed successfully!"
    
    # Final status check
    check_service_status
    
    log_info "Installation summary:"
    echo -e "\t✓ udev rule installed to ${UDEV_RULE_DEST}"
    echo -e "\t✓ systemd unit installed to ${SYSTEMD_UNIT_DEST}"
    echo -e "\t✓ Binary installed to ${BIN_DEST}"
    echo -e "\t✓ Service enabled and started"
}

# Run main function with all arguments
main "$@"
