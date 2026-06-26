# Local Demo Profiles

Local demo profiles let the OSS Local Control Plane and Local Dashboard demonstrate Windows,
Linux, and macOS capability states from one development machine without pretending that the
current host actually has those OS-native controls.

Demo profiles are disabled by default.

```powershell
$env:POLLEK_ENABLE_DEMO_PROFILES = "1"
```

```bash
export POLLEK_ENABLE_DEMO_PROFILES=1
```

Then request a fixture snapshot explicitly:

```text
GET /v1/tenants/local/devices/local/capability-snapshot-v2?mode=desktop_advanced&demo_os=windows&demo_profile=ready
```

Supported `demo_os` values:

- `windows`
- `linux`
- `macos`

Supported `demo_profile` values:

- `ready` - methods are marked available and warm-check passed.
- `observe_only` - methods are visible but not enforceable.
- `needs_setup` - methods show install/setup gaps.

Isolation rules:

- No demo profile is returned unless `POLLEK_ENABLE_DEMO_PROFILES=1` and `demo_os` is present.
- Demo snapshots use device IDs such as `demo_windows_ready`.
- Demo snapshots set `contract.reason_code` to `demo_fixture`.
- Demo snapshots add method limitations saying they are fixture data.
- Demo snapshot reads do not replace the latest real capability snapshot stored by the Local Control Plane.

Production cleanup:

- Leave `POLLEK_ENABLE_DEMO_PROFILES` unset in production-like environments.
- Remove demo support by deleting the `DemoProfile` helpers in `crates/local-control-plane/src/policy_first_api.rs`
  and the `e2e_demo_profiles` test.
- The core real-host detection path remains separate from demo profile generation.
