# Configuration: Link

`niri-link` is configured with a top-level `link {}` section.

Example:

```kdl
link {
    enable false
    listen-address "127.0.0.1:0"
    discovery true
    pairing-mode false
    trusted-node "peer-fingerprint-or-uuid"

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

Notes:

- `enable` controls initial runtime state. Link mode can still be toggled later through IPC.
- `listen-address` defaults to loopback to avoid exposing the listener unintentionally.
- `trusted-node` entries are appended to the runtime trust list.
- `restore-last-session` enables reuse of the best matching persisted session snapshot.
- touch, tablet, and clipboard forwarding are intentionally conservative in the first implementation.
