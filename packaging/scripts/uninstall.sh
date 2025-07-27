#!/usr/bin/env bash

set -euo pipefail  # Exit on error, undefined vars, pipe failures

# Script configuration
readonly SCRIPT_NAME="${0##*/}"

# Project files
readonly UDEV_RULE='99-af-pro-display.rules'
readonly SYSTEMD_UNIT="af-pro-display.service"
readonly BIN_NAME="af-pro-display"

# Installation paths (same as install script)
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

Uninstall af-pro-display application, systemd service, and udev rules.

OPTIONS:
    -h, --help      Show this help message
    -v, --verbose   Enable verbose output
    -y, --yes       Skip confirmation prompt
    --keep-config   Keep configuration files (only remove binary)

DESCRIPTION:
    This script removes the af-pro-display installation by:
    1. Stopping and disabling the systemd service
    2. Removing the systemd user service file
    3. Removing the binary from ~/.local/bin
    4. Removing udev rules (requires sudo)

EOF
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

# Function to stop and disable systemd service
stop_service() {
    if ! systemctl --user is-enabled --quiet "${SYSTEMD_UNIT}" 2>/dev/null; then
        log_info "Service is not enabled, skipping service operations"
        return 0
    fi
    
    log_info "Stopping and disabling systemd service..."
    
    # Stop the service if it's running
    if systemctl --user is-active --quiet "${SYSTEMD_UNIT}" 2>/dev/null; then
        if systemctl --user stop "${SYSTEMD_UNIT}"; then
            log_success "Service stopped"
        else
            log_error "Failed to stop service"
            return 1
        fi
    else
        log_info "Service is not running"
    fi
    
    # Disable the service
    if systemctl --user disable "${SYSTEMD_UNIT}"; then
        log_success "Service disabled"
    else
        log_error "Failed to disable service"
        return 1
    fi
}

# Function to remove systemd unit file
remove_systemd_unit() {
    if [[ ! -f "${SYSTEMD_UNIT_DEST}" ]]; then
        log_info "systemd unit file not found: ${SYSTEMD_UNIT_DEST}"
        return 0
    fi
    
    log_info "Removing systemd unit file..."
    
    if rm -f "${SYSTEMD_UNIT_DEST}"; then
        log_success "systemd unit file removed"
    else
        log_error "Failed to remove systemd unit file"
        return 1
    fi
    
    # Reload daemon
    if systemctl --user daemon-reload; then
        [[ "${VERBOSE:-false}" == "true" ]] && log_info "systemd daemon reloaded"
    else
        log_warn "Failed to reload systemd daemon"
    fi
}

# Function to remove binary
remove_binary() {
    if [[ ! -f "${BIN_DEST}" ]]; then
        log_info "Binary not found: ${BIN_DEST}"
        return 0
    fi
    
    log_info "Removing binary..."
    
    if rm -f "${BIN_DEST}"; then
        log_success "Binary removed"
    else
        log_error "Failed to remove binary"
        return 1
    fi
}

# Function to remove udev rule
remove_udev_rule() {
    if [[ ! -f "${UDEV_RULE_DEST}" ]]; then
        log_info "udev rule not found: ${UDEV_RULE_DEST}"
        return 0
    fi
    
    log_info "Removing udev rule (requires sudo)..."
    
    if sudo rm -f "${UDEV_RULE_DEST}"; then
        log_success "udev rule removed"
    else
        log_error "Failed to remove udev rule"
        return 1
    fi
    
    # Reload udev rules
    if sudo udevadm control --reload-rules; then
        [[ "${VERBOSE:-false}" == "true" ]] && log_info "udev rules reloaded"
    else
        log_warn "Failed to reload udev rules"
    fi
}

# Function to check what's currently installed
check_installation() {
    log_info "Checking current installation..."
    
    local found_files=()
    local missing_files=()
    
    # Check each component
    if [[ -f "${UDEV_RULE_DEST}" ]]; then
        found_files+=("udev rule: ${UDEV_RULE_DEST}")
    else
        missing_files+=("udev rule")
    fi
    
    if [[ -f "${SYSTEMD_UNIT_DEST}" ]]; then
        found_files+=("systemd unit: ${SYSTEMD_UNIT_DEST}")
    else
        missing_files+=("systemd unit")
    fi
    
    if [[ -f "${BIN_DEST}" ]]; then
        found_files+=("binary: ${BIN_DEST}")
    else
        missing_files+=("binary")
    fi
    
    # Check service status
    local service_status="not installed"
    if systemctl --user list-unit-files --quiet "${SYSTEMD_UNIT}" 2>/dev/null | grep -q "${SYSTEMD_UNIT}"; then
        if systemctl --user is-enabled --quiet "${SYSTEMD_UNIT}" 2>/dev/null; then
            if systemctl --user is-active --quiet "${SYSTEMD_UNIT}" 2>/dev/null; then
                service_status="enabled and running"
            else
                service_status="enabled but not running"
            fi
        else
            service_status="installed but disabled"
        fi
    fi
    
    if [[ ${#found_files[@]} -eq 0 ]]; then
        log_info "No af-pro-display components found"
        return 1
    fi
    
    log_info "Found components:"
    printf '\t✓ %s\n' "${found_files[@]}"
    
    if [[ ${#missing_files[@]} -gt 0 ]]; then
        log_info "Missing components:"
        printf '\t✗ %s\n' "${missing_files[@]}"
    fi
    
    log_info "Service status: ${service_status}"
    
    return 0
}

# Main uninstall function
main() {
    local skip_confirmation=false
    local verbose=false
    local keep_config=false
    
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
            -y|--yes)
                skip_confirmation=true
                shift
                ;;
            --keep-config)
                keep_config=true
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
    
    log_info "af-pro-display uninstaller"
    
    # Check what's installed
    if ! check_installation; then
        log_info "Nothing to uninstall"
        exit 0
    fi
    
    # Confirmation prompt
    if [[ "${skip_confirmation}" != "true" ]]; then
        echo
        if [[ "${keep_config}" == "true" ]]; then
            if ! confirm "Remove af-pro-display binary only (keep config files)?"; then
                log_info "Uninstallation cancelled"
                exit 0
            fi
        else
            if ! confirm "Remove all af-pro-display components?"; then
                log_info "Uninstallation cancelled"
                exit 0
            fi
        fi
    fi
    
    echo
    log_info "Starting uninstallation..."
    
    # Stop and disable service
    if [[ "${keep_config}" != "true" ]]; then
        stop_service || true  # Continue even if this fails
    fi
    
    # Remove components
    if [[ "${keep_config}" != "true" ]]; then
        remove_systemd_unit || true
    fi
    
    remove_binary || true
    
    if [[ "${keep_config}" != "true" ]]; then
        remove_udev_rule || true
    fi
    
    echo
    if [[ "${keep_config}" == "true" ]]; then
        log_success "Binary removed successfully!"
        log_info "Configuration files were preserved"
    else
        log_success "Uninstallation completed!"
        
        log_info "Removed components:"
        echo -e "\t✓ systemd service stopped and disabled"
        echo -e "\t✓ systemd unit file removed"
        echo -e "\t✓ Binary removed"
        echo -e "\t✓ udev rule removed"
    fi
    
    # Check for any remaining files
    echo
    log_info "Verifying removal..."
    if ! check_installation 2>/dev/null; then
        log_success "All components successfully removed"
    else
        log_warn "Some components may still be present"
    fi
}

# Run main function with all arguments
main "$@"
