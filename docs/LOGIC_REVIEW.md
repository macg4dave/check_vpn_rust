# Program logic review

Reviewed 12 November 2026. The findings below describe the original implementation. The fixes listed here have now been implemented.

## Implementation status

All twelve findings have corresponding changes:

| Findings | Implemented changes |
| --- | --- |
| 1 | Packaged and generated XML use repeated list fields; both are loaded and roundtripped in regression tests. |
| 2 | Explicit normal-ISP aliases accommodate fallback naming; empty identities are rejected and fall through to the next provider. Exact matching remains the rule. |
| 3 | Daemon lookup and action errors follow the same `exit_on_error` policy. A subprocess regression verifies continued monitoring after failure. |
| 4 | Startup pins an absolute source path; disappearance is an error. Reloads prepare replacements before committing; strict mode rejects semantic errors too. Discovery performs no writes. |
| 5 | DNS failures do not abort independent candidates; resolver waits include a timeout. |
| 6 | Providers run first. TCP probes are bounded diagnostics after lookup failure, never a recovery gate. Offline status alone does not trigger recovery. |
| 7 | Recovery attempts, including failures, have a configurable cooldown after completion (default 60 seconds). |
| 8 | URLs, JSON keys, aliases, command quoting, executable tokens, timeouts and platform-specific actions are validated before use. |
| 9 | HTTP requests, provider attempts, the whole provider chain, connectivity diagnostics and recovery processes have bounds. Retry/interval waits and recovery processes observe cancellation; Linux termination signals are handled. |
| 10 | Exit definitions are centralised: success 0, runtime/lookup 1, configuration/CLI 2, recovery 3. |
| 11 | Defaults and the wizard use the portable typed `log` action. |
| 12 | Packages include recovery permission instructions; the optional polkit example grants only restart of one configured VPN unit. Installer/service paths and obsolete flags are corrected. |

The HTTP implementations now share one transport and reuse clients. Legacy custom-provider options normalize into an ordered provider specification list. Timing-only reloads retain clients. Action arguments are prepared when configuration changes. Redundant startup/error handling and `include!` module wrappers are gone. Unused timer, metrics and JSON filesystem APIs, duplicate tests, the alternate XML backend, and their dependencies/features were removed. These removals change the library API and are listed in CHANGELOG.md.

Operational limits: cancellation waits for an in-flight HTTP request (at most five seconds with the built-in client). The operating system's resolver worker may finish after its caller times out; it cannot run recovery. Process timeout terminates the immediate recovery process; scripts must own their descendants. Linux/macOS service execution and authorization require validation on those platforms; shell syntax is checked locally. CI now runs formatting, Clippy, default-feature tests and all-feature tests on Linux, Windows and macOS.

## Fix validation

On Windows, `cargo test --all-targets --no-fail-fast` passed 115 tests and `cargo test --all-targets --all-features --no-fail-fast` passed 116 tests. Both runs intentionally ignored two tests requiring external network conditions. Formatting, `cargo clippy --all-targets --all-features -- -D warnings`, `git diff --check`, and syntax checks for every changed shell script passed. Native Linux/macOS recovery and authorization were not executed locally; WSL Debian was waiting for initial setup. CI has been expanded to all three operating systems.

## Intended detection rule

The user confirmed that an exact match to the configured normal ISP means VPN lost. Any other successfully obtained ISP means VPN active, because the VPN's ISP is not known in advance. Keep this rule. Lookup failures are a separate condition.

## Current flow

```text
Parse CLI -> initialise logging
  -> init wizard? generate configuration and return
  -> completions? generate completions and return
  -> load XML from explicit path or automatic discovery, otherwise defaults
  -> merge CLI > file > defaults -> validate
  -> run once, or install signal handler and enter loop
       -> rediscover/reload configuration (continuous mode only)
       -> apply changed configuration if valid
       -> construct all provider clients
       -> TCP connectivity probes
            offline/error -> log; fail only for run-once/exit-on-error
            online -> query providers until first success
                 lookup failure -> log; fail only for run-once/exit-on-error
                 ISP differs -> log VPN active
                 ISP matches -> run action; ANY action failure exits
       -> sleep interval, checking shutdown flag each second
```

The interval is a delay after work completes, so check duration adds to the time between checks. There is no saved VPN state or cooldown: a matching ISP triggers an action on every iteration.

## Findings, ordered by practical impact

1. **Packaged configuration cannot load with the default XML backend.** `contrib/config.xml` uses nested `<connectivity_endpoints><endpoint>…` and `<connectivity_ports><port>…` structures, but `Config` declares plain vectors without wrapper structs. Reproduced with `target/debug/check_vpn.exe --config contrib/config.xml --run-once --dry-run`: configuration parsing fails with `unexpected Event::Start("endpoint")`. `Cargo.toml` installs this file into `/etc/check_vpn/config.xml`, so this affects a fresh package installation. Pick one list schema, make the example and wizard use it, and test loading the exact packaged file. `tests/xml_io_tests.rs` explicitly avoids vectors in its basic configuration test.

2. **Provider fallback can change the meaning of the exact ISP comparison.** `IpApiProvider` returns `isp`; `IfconfigCoProvider` can return `asn_org`; generic providers try `isp`, `asn_org`, and `org`. These fields need not contain identical strings for the same connection. If the first provider fails and the fallback names the normal ISP differently, the normal connection is classified as VPN active. This does not invalidate the requested detection rule: configuration must represent the normal ISP as reported by each supported source. Consider provider-specific expected strings, or an explicit list of normal-ISP aliases. An empty or whitespace-only identity should be rejected so it triggers fallback rather than being treated as an active VPN.

3. **A failed recovery action stops monitoring regardless of `exit_on_error`.** `src/app/check.rs` returns `ActionFailed` unconditionally, and `src/app.rs` propagates it with `?`. A transient command failure therefore kills a standalone daemon even with `exit_on_error=false`. The supplied systemd unit restarts the process, but other launch methods may not. Handle action failure through the same explicit daemon error policy as lookup/connectivity failures.

4. **Automatic config discovery can silently replace live settings with defaults.** Every iteration calls `Config::load()` again. If a previously discovered file is removed and no other file is found, `load()` returns defaults successfully. The application accepts those settings; even `--strict-config` cannot detect the disappearance as an error. Explicit CLI/environment paths do fail on disappearance. Resolve the source once at startup and reload that same source. Treat 'started intentionally with defaults' as a separate source. Also decide whether strict mode should reject semantic validation failures: currently it only exits for read/parse failures, while invalid merged values retain the previous config.

5. **DNS failure on one connectivity candidate aborts all remaining candidates.** `src/networking/mod.rs` immediately returns an error from `try_connect`. A broken first hostname prevents trying a reachable later endpoint and bypasses remaining retries. Continue through independent candidates and retain the error for reporting if none succeeds.

6. **The connectivity gate can suppress otherwise working provider checks and all recovery.** A blocked probe port is not proof that provider HTTPS is unavailable. When the gate fails, no provider or recovery action runs. With a VPN kill switch, losing the tunnel can therefore produce an endless offline loop. This is a policy choice, not an instruction to reboot on every outage. A simpler option is to query providers first and use connectivity probes only to explain lookup failure. If offline recovery is desired, define it separately with a failure threshold.

7. **There is no recovery grace period.** A restart request can succeed before the VPN reconnects. The next matching check starts another restart; a short interval can interfere with reconnection. Consider a recovery cooldown and explicit recovery state. Keep this distinct from changing the normal-ISP comparison rule.

8. **Configuration validation accepts values that cannot work later.** Provider URL constructors build HTTP clients without parsing/validating the URL, and availability validation only counts configured entries. An empty custom URL can satisfy 'at least one provider'. Commands with unmatched quoting pass validation and fail only when recovery is needed. Validate URLs, nonempty keys and identities, and parse command arguments when preparing the configuration. Platform-incompatible actions can also be rejected early, such as `restart-unit` on Windows.

9. **Shutdown is only responsive while sleeping.** Blocking DNS resolution, sequential TCP/HTTP requests, retry sleeps and child command `.status()` calls do not observe the shutdown flag. DNS resolution is outside the TCP connect timeout, and recovery commands have no timeout. The wizard cancellation variant only checks cancellation between providers. Define a bounded check/action duration if prompt service shutdown matters.

10. **Exit-code definitions disagree.** `src/config/mod.rs` advertises DNS=3, connectivity=4, ISP=5. Actual lookup/connectivity failures become `RuntimeError` and exit 1; `error_types.rs` uses 3 for action failure and 4 for argument failure. Existing binary tests assert connectivity exit 1. Choose a single documented contract and remove or replace obsolete constants.

11. **Default Windows recovery command fails.** Defaults and the wizard use `echo "vpn lost"`, but the runner starts programs directly without a shell. Windows `echo` is a shell built-in. A matching ISP can therefore cause an action failure with the default configuration. A typed log-only action would provide a portable harmless default.

12. **The packaged service needs an explicit recovery permission setup.** It runs as `checkvpn`, while restart/reboot need authorization. An example polkit rule exists under `contrib/polkit`, but the Debian asset list does not install it. The installer should explain or provide narrowly scoped permission for the configured recovery action. This was inspected in source, not executed on Linux.

## What can be merged or removed

| Area | Suggested simplification |
| --- | --- |
| HTTP clients | Share one configured `reqwest::blocking::Client`; clone its handle into providers. Rebuild only when relevant configuration changes. Currently every provider gets a new client on every iteration. |
| HTTP response handling | Share status handling, bounded body reading, response limits and retry policy. All three providers repeat most of this logic, but only ip-api implements retries. Retain provider-specific extraction. |
| Provider configuration | One ordered list of provider specifications with optional JSON key can replace `custom_json_server` plus `provider_urls`. Retain old CLI/XML fields temporarily as aliases if compatibility matters. |
| Provider iteration | Merge the cancelled/non-cancelled fallback loops into one implementation with an optional cancellation check. Use the same provider-chain builder in the wizard and runtime. |
| Defaults | Define defaults once. They are repeated in `Config::default`, `merge_with_args`, and the wizard; the wizard already uses a different port list. |
| Config loading | Share startup/reload source resolution and error context. The two startup error branches in `lib.rs` are substantially identical. |
| Actions | Parse and validate once into a typed action; store it with prepared runtime configuration. `app.rs` currently parses once merely to log it, then `perform_check` parses it again on each match. |
| Error logging | Log errors at the boundary that handles them. Action failures are currently logged by the runner, action wrapper, check function and binary. |
| Timer | `timer.rs` is not used by the application loop. Remove it if external library consumers do not need it, or choose it as the single scheduler. Do not keep two scheduling implementations solely for internal tests. |
| Metrics | Remove the permanently `None` handle and unreachable join in `app.rs`. `metrics.rs` is unused by the application and only closes connections; it does not serve HTTP metrics. Remove it or implement an actual supported feature. |
| JSON filesystem API | `json_io` is not used by the runtime. Remove it and `json_pretty` if this is an application rather than a general utility library. Parsing provider JSON is still needed. |
| XML backends/features | Consider keeping only quick-xml unless a real consumer needs the alternate backend. Multiple serializers increase the schema/feature combinations to maintain. Keep serialization and validation separate. |
| Module declarations | Replace legacy `include!` wrappers with ordinary Rust modules. Clean up comments about unfinished moves and obsolete metrics behaviour. |
| Tests | Consolidate repeated match/non-match/action tests spread across `actions_tests`, `app_tests`, `app_perform_check_tests`, and `perform_check_tests`. Replace source-text clone checks with behaviour tests. Preserve distinct error paths and platform checks. |
| Service/docs | Consolidate duplicated systemd hardening directives. Correct README `--config-file` examples to `--config`, remove unsupported metrics claims, and reconcile its CI description with the actual workflow. |

Public modules and re-exports may be used outside this repository. Runtime-unused does not automatically mean safe to delete from a published library.

## Proposed flow after refactoring

```text
CLI -> special command, or resolve config source once
    -> load + merge defaults + validate
    -> prepare typed action and persistent provider chain
    -> loop:
         reload same source; prepare replacement completely before applying
         obtain identity (optional connectivity diagnostics on failure)
         exact normal-ISP match -> recovery if cooldown allows
         other nonempty identity -> VPN active
         lookup/action error -> apply one consistent error policy
         interruptible wait
    -> one boundary maps typed errors to documented exit codes
```

Keep `Config` and effective/prepared configuration separate: a partial file and fully resolved runtime settings serve different purposes. Keep provider extraction, network access and action execution independently testable. Return a meaningful check outcome rather than `Ok(())` for healthy, offline and failed lookup alike; this makes policy and tests clearer without changing the requested matching rule.

Suggested order: repair packaged XML and its regression coverage; fix reload source handling and daemon action-error policy; address provider identity consistency; consolidate defaults/HTTP/prepared configuration; remove unused APIs and duplicate tests last.

## Validation

The packaged configuration parse failure was reproduced with dry-run enabled. `cargo test --all-targets --no-fail-fast` produced passing summaries for all test targets on Windows: 130 passed, 3 ignored. This existing suite does not catch the packaged configuration bug. Linux D-Bus/polkit behaviour and Unix-only path handling were inspected but not executed. Existing non-UTF8 tests only assert failure for malformed contents, so they do not prove valid non-UTF8 paths load: `load_from_path` currently changes those paths using `to_string_lossy()` before opening them.
