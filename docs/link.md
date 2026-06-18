# niri-link

`niri-link` is a built-in niri feature for creating a shared scrollable workspace across trusted LAN peers. When link mode is enabled, all participating niri instances form one global scrollable workspace. Each physical machine remains the owner of its local Wayland clients, but all windows appear as tiles in a continuous global strip.

---

## Architecture overview

```
Node A (Asahi)                    Node B (ThinkPad)
┌──────────────────────────┐      ┌──────────────────────────┐
│  niri compositor          │      │  niri compositor          │
│  ┌────────────────────┐  │      │  ┌────────────────────┐  │
│  │  Local tiles (wl)  │  │      │  │  Local tiles (wl)  │  │
│  │  RemoteTile(B→A)   │  │      │  │  RemoteTile(A→B)   │  │
│  └────────────────────┘  │      │  └────────────────────┘  │
│         ↓ TCP               │ TCP  │         ↓ TCP               │
│  LinkManager              │◄────►│  LinkManager              │
│  - session state          │      │  - session state          │
│  - layout sync            │      │  - layout sync            │
│  - transport              │      │  - transport              │
└──────────────────────────┘      └──────────────────────────┘
         Viewport A                        Viewport B
         global_x: 0                       global_x: 1920
```

The global workspace is a single continuous horizontal strip. Each node's physical outputs map to viewport segments. Tiles can visually span viewport boundaries.

---

## How it works

### Session lifecycle

1. User enables link via `niri msg action link-enable` or config.
2. niri starts a TCP listener and optionally broadcasts via mDNS (`_niri-link._tcp.local.`).
3. Peer joins via `niri msg action link-join --addr HOST:PORT` or auto-discovery.
4. Handshake: `Hello` → `JoinRequest` → `JoinAccept`.
5. Session forms: one node becomes leader (lowest UUID wins deterministically).
6. Each node syncs its tiles and viewports; the leader broadcasts ordered `LayoutOp` messages.
7. Remote tiles are represented as `RemoteTile` objects inside the compositor.
8. When disabled: session is persisted, remote tiles are removed, local workspaces reconstructed.

### Leader election

- Leader is the participant with the lexicographically smallest `node_id` (UUID).
- On leader disconnect, followers elect a new leader deterministically from alive participants.
- New leader broadcasts a `GlobalSnapshot` to restore consistent state.

### Layout operations

Every layout mutation is expressed as a `LayoutOp` with a monotonic sequence number:

```
InsertTile | RemoveTile | MoveTile | MoveColumn | ResizeColumn
FocusTile | FocusColumn | ScrollViewport | SetViewport
ChangeTileState | ChangeTileMetadata | SetFullscreen | SetFloating
EnterOverview | LeaveOverview | EnableLink | DisableLink
```

Operations flow: **follower → leader → broadcast → all nodes apply in order**.

If a follower detects a sequence gap, it requests a `GlobalSnapshot` from the leader.

### Remote tiles

`RemoteTile` is a first-class compositor object that:
- implements the `LayoutElement` trait
- is not a `wl_surface` (returns false for surface hit checks)
- renders the latest received video frame, or a placeholder/stale indicator
- routes input to the tile's owner node

Frame streaming is frame-buffers over TCP. The design allows codec negotiation (`av1`, `h264`, `raw`). The initial implementation uses raw RGBA frames.

---

## Configuration

```kdl
link {
    enable false
    listen-address "0.0.0.0:0"
    discovery true
    pairing-mode false
    trusted-node "uuid-or-fingerprint"

    transport {
        prefer "quic"
        fallback "tcp-tls"
    }

    streaming {
        preferred-codec "av1,h264,raw"
        max-fps 60
        max-bitrate-mbps 80
        idle-fps 5
        stream-only-visible true
        stale-frame-timeout-ms 500
    }

    layout {
        mode "continuous-horizontal"
        restore-last-session true
        unlink-policy "keep-owned-tiles"
        remote-tile-placeholder true
        animate-link-transition true
    }

    input {
        forward-keyboard true
        forward-pointer true
        forward-scroll true
        forward-touch false
        forward-tablet false
        remote-focus-follows-pointer true
    }
}
```

| Option | Default | Description |
|---|---|---|
| `enable` | `false` | Enable link mode on startup |
| `listen-address` | `"0.0.0.0:0"` | TCP bind address (`:0` = random port) |
| `discovery` | `true` | Enable mDNS peer discovery |
| `pairing-mode` | `false` | Accept unknown peers (enable temporarily for first pairing) |
| `trusted-node` | — | Pre-trust a node by UUID or fingerprint |

---

## IPC

### Actions

```bash
# Enable/disable
niri msg action link-enable
niri msg action link-disable
niri msg action link-toggle

# Connect to a specific peer
niri msg action link-join --addr 192.168.1.10:7890

# Leave session (disconnect + persist)
niri msg action link-leave

# Pairing and trust
niri msg action link-pair                          # enable pairing mode
niri msg action link-pair --node uuid              # trust specific node
niri msg action link-unpair --node uuid
niri msg action link-trust-node --node uuid
niri msg action link-forget-node --node uuid

# Session management
niri msg action link-restore-session --session UUID
niri msg action link-set-leader --node UUID

# Status
niri msg action link-status
```

### Requests

```bash
niri msg link-status           # current link state
niri msg link-peers            # connected peers
niri msg link-sessions         # persisted session list
niri msg link-global-workspace # global layout snapshot
niri msg link-remote-tiles     # remote tile list
```

### Events (event-stream)

```
LinkEnabled          { session_id }
LinkDisabled         { session_id }
LinkPeerDiscovered   { peer }
LinkPeerJoined       { peer }
LinkPeerLeft         { peer }
LinkLeaderChanged    { leader_node_id, generation }
LinkGlobalLayoutChanged { session_id, generation, operation_seq }
LinkRemoteTileCreated   { tile }
LinkRemoteTileUpdated   { tile }
LinkRemoteTileClosed    { tile_id }
LinkStreamStarted    { tile_id }
LinkStreamStopped    { tile_id }
LinkStreamDegraded   { tile_id }
LinkInputForwarded   { tile_id, owner_node_id }
LinkSessionPersisted { session_id }
LinkError            { message }
```

---

## Security

niri-link is designed for trusted LAN use only.

- Default bind address: `0.0.0.0:0` (listens on all interfaces, random port)
- First connection requires explicit pairing
- Trusted node fingerprints are persisted at:  
  `~/.local/state/niri-link/trusted-nodes.json`
- Unknown nodes are rejected unless `pairing-mode` is active
- Node identity persisted at:  
  `~/.local/state/niri-link/local-node-id`

**Do not expose niri-link to the public internet.**

---

## Storage

| Path | Contents |
|---|---|
| `~/.local/state/niri-link/local-node-id` | This node's persistent UUID |
| `~/.local/state/niri-link/trusted-nodes.json` | Trusted peer fingerprints |
| `~/.local/state/niri-link/sessions/*.json` | Persisted global session snapshots |

---

## Protocol

All messages are length-prefixed binary (bincode serialized) sent over TCP. Each `Envelope` contains:

```rust
struct Envelope {
    protocol_version: u16,  // current: 1
    session_id: Uuid,
    sender_node_id: Uuid,
    nonce: u64,
    timestamp_millis: u64,
    kind: MessageKind,
}
```

Message types: `Hello`, `CapabilityExchange`, `JoinRequest`, `JoinAccept`, `JoinReject`, `PeerList`, `LeaderElection`, `Heartbeat`, `GlobalSnapshot`, `LayoutOp`, `LayoutAck`, `ViewportUpdate`, `TileMetadataUpdate`, `TileClosed`, `FocusUpdate`, `InputEvent`, `FrameRequest`, `FrameBegin`, `FrameChunk`, `FrameEnd`, `StreamControl`, `ClipboardOffer`, `DisableLink`, `PersistSession`, `RestoreSession`, `Error`.

---

## Discovery

mDNS service name: `_niri-link._tcp.local.`

Properties advertised:
- `node_id`: this node's UUID
- `fingerprint`: trust fingerprint

Manual join (bypasses mDNS): `niri msg action link-join --addr HOST:PORT`

---

## Failure handling

| Failure | Behavior |
|---|---|
| Peer disconnects | Remote tiles become stale placeholders (configurable timeout) |
| Leader disconnects | Followers elect new leader; request snapshot |
| Stream fails | Layout preserved; stale indicator shown |
| Protocol mismatch | Peer rejected with error message |
| Compositor crash | Session persisted on next clean disable; local workspaces always preserved |

---

## Quick start

### Two machines on the same LAN

**Machine A (Asahi):**
```bash
# Enable link and pairing mode
niri msg action link-enable
niri msg action link-pair

# Note the TCP port
niri msg link-status
```

**Machine B (ThinkPad):**
```bash
# Join Machine A (or let mDNS discovery handle it)
niri msg action link-enable
niri msg action link-join --addr 192.168.x.x:PORT
```

After joining, both machines share one scrollable workspace. Scroll right on Machine A to see Machine B's windows, or vice versa.

---

## Limitations (current implementation)

- Frame streaming uses raw RGBA (no codec compression yet)
- QUIC transport not yet implemented (TCP only)
- Input forwarding: pointer and keyboard only (no touch/tablet)
- Clipboard sharing disabled by default
- IME forwarding not implemented
- Single-session (no multi-session simultaneous links)
