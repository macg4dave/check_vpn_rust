# Recovery permissions

The packaged configuration uses `log`, which requires no special privilege and works on every platform. Configure your normal ISP before enabling a recovery action.

On Linux, `restart-unit` executes `systemctl --no-ask-password restart -- UNIT`. The packaged service runs as `checkvpn`; installing the binary does not authorize this account to manage system services.

To allow restart of one VPN unit:

1. Set `vpn_lost_action_type` to `restart-unit` and `vpn_lost_action_arg` to your actual VPN unit, such as `openvpn-client@myvpn.service`.
2. Copy `contrib/polkit/50-check_vpn.rules` to `/etc/polkit-1/rules.d/50-check_vpn.rules` as an administrator. Change its exact unit name to match your configuration. The example grants only the `restart` verb for that unit to `checkvpn`.
3. Test the same request as the service account: `sudo -u checkvpn systemctl --no-ask-password restart -- openvpn-client@myvpn.service`. This restarts the selected VPN, so run it during a suitable maintenance window.

Packages deliberately do not install an authorization rule automatically: only the administrator knows which VPN unit may be restarted. Reboot authorization is not granted by the example rule. Configure a separate, explicitly reviewed policy if reboot recovery is required.

Commands execute directly without a shell and run under the monitor's account. They must have the necessary permissions themselves. Windows users should use `log`, `reboot`, or a command pointing to a real executable; shell built-ins such as `echo` are not executables. `restart-unit` requires Linux (dry-run remains portable).

The default recovery cooldown is 60 seconds and the default command timeout is 30 seconds. Timeout or cancellation terminates and reaps the process started by the monitor. Recovery scripts should manage the lifetime of their own descendants and should not daemonize background work.
