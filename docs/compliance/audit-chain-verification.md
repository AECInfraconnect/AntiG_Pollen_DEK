# Audit Chain Verification

Pollen DEK uses a tamper-evident hash chain to protect the integrity of its audit logs (satisfying NIST AU-9). Each audit event references the cryptographic hash of the previous event.

## Verification Tool

You can verify an exported audit chain using the `dek-cli` verification utility.

```bash
dek-cli verify-audit --file audit_chain.json
```

## How It Works

Each event in the JSON array contains:

- `seq`: Monotonically increasing sequence number.
- `timestamp`: The ISO8601 time of the event.
- `event_type`: What occurred (e.g., `BundleReloaded`, `KeyRotated`).
- `prev_digest`: The SHA-256 hash of the immediate predecessor.
- `digest`: The SHA-256 hash of `(prev_digest + seq + timestamp + event_type + payload)`.

### Manual Verification

If an auditor requires manual verification without `dek-cli`, they can write a script to recompute the SHA-256 hashes sequentially:

```python
import hashlib
import json

with open("audit_chain.json") as f:
    events = json.load(f)

for i in range(1, len(events)):
    prev_event = events[i-1]
    curr_event = events[i]
    
    # Assert sequence
    assert curr_event["seq"] == prev_event["seq"] + 1
    
    # Assert digest linkage
    assert curr_event["prev_digest"] == prev_event["digest"]
    
    # Recompute current digest
    payload_str = json.dumps(curr_event["payload"], separators=(',', ':'))
    data_to_hash = f"{curr_event['prev_digest']}{curr_event['seq']}{curr_event['timestamp']}{curr_event['event_type']}{payload_str}"
    computed_digest = hashlib.sha256(data_to_hash.encode()).hexdigest()
    
    assert computed_digest == curr_event["digest"]

print("Audit chain is valid and untampered.")
```
