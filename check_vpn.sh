#!/bin/bash
#check_vpn.sh

# Configuration variables
CHECK_INTERVAL=60
ISP_TO_CHECK="Hutchison 3G UK Ltd"
VPN_LOST_ACTION="/sbin/shutdown -r now"
PING_CMD="/usr/bin/ping"
CURL_CMD="/usr/bin/curl"
JQ_CMD="/usr/bin/jq"
LOGGER_CMD="/usr/bin/logger"

# Source error logging script
source /usr/local/share/bin/check_vpn/error-logging/error-logging.sh

# Set log file and log level for the script
log_file="/var/log/check_vpn.log"
log_verbose=1  # ERROR level logging

# Flag to control the main loop
keep_running=true

# Trap signals to allow graceful shutdown
trap 'keep_running=false' SIGINT SIGTERM

# Check if jq is installed
if command -v jq >/dev/null 2>&1; then
    jq_available=true
else
    jq_available=false
    log_write 2 "jq not found, falling back to sed for JSON parsing"
fi

# Function to check internet connectivity
function check_vpn() {
    if $PING_CMD -c 1 -W 1 8.8.8.8 >/dev/null 2>&1; then
        get_isp
    else
        log_write 2 "Internet Down"
    fi
}

# Function to determine current ISP and take action if VPN is lost
function get_isp() {
    if ! response=$($CURL_CMD -s http://ip-api.com/json); then
        log_write 1 "Failed to retrieve location data (curl error)"
        return 1
    fi

    if [ -z "$response" ]; then
        log_write 1 "Empty response received from the API"
        return 1
    fi

    if [ "$jq_available" = true ]; then
        isp=$(echo "$response" | $JQ_CMD -r '.isp')
        log_write 2 "jq not found, falling back to sed for JSON parsing"
    else
        isp=$(echo "$response" | sed -n 's/.*"isp":"\([^"]*\)".*/\1/p')
    fi


    if [ -z "$isp" ]; then
        log_write 1 "Failed to parse ISP from response"
        return 1
    fi

    if [ "$isp" == "$ISP_TO_CHECK" ]; then
        log_write 1 "VPN Lost (ISP: $isp)"
        $LOGGER_CMD "VPN lost, taking action"
        $VPN_LOST_ACTION
    else
        log_write 3 "VPN active (ISP: $isp)"
    fi
}

# Main loop to continuously check VPN status
while $keep_running; do
    check_vpn
    sleep $CHECK_INTERVAL
done
