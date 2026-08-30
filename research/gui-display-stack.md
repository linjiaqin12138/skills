# How GUI Display Works: X11, Wayland, Windows, and SSH Forwarding

Notes from a discussion on Ubuntu 24.04 (GNOME 46, Wayland session).

## 1. Display protocols: X11 vs Wayland

A display protocol defines how applications draw windows and receive keyboard/mouse input.

### X11 (X.Org) — the old guard (1984)

- A separate **X server** process sits between apps and hardware; apps are "clients" sending drawing requests over a protocol.
- The protocol works over the network — the origin of X forwarding over SSH.
- Problems: predates GPUs/HiDPI/touch; no app isolation (any app can keylog or read another's windows); development nearly frozen.

### Wayland — the modern replacement (2008–)

- **No separate server**: the desktop's compositor *is* the display server. Apps render into buffers; the compositor assembles the final frame.
- Better security (apps isolated), tear-free rendering, proper mixed-DPI multi-monitor.
- Old X11-only apps run via **XWayland** (a compatibility X server inside Wayland).
- Default on Ubuntu 24.04.

## 2. Where GNOME fits

GNOME is a **desktop environment** sitting *on top of* the display protocol:

```
You (keyboard, mouse, eyes)
   ↕
GNOME Shell (compositor = Mutter, window manager, top bar, Activities)
   ↕
Display protocol: Wayland  (or X11)
   ↕
Kernel / GPU drivers → hardware
```

- On GNOME/Wayland, GNOME Shell (via Mutter) is the Wayland compositor.
- GNOME is protocol-agnostic: same desktop, different plumbing ("Ubuntu" vs "Ubuntu on Xorg" at the GDM login gear menu).

## 3. The Windows display stack (for comparison)

Windows has no pluggable "display protocol" — one fixed, proprietary stack:

- **USER / GDI (win32k.sys)** — windowing, input messages, 2D drawing; largely in the kernel since NT 4.
- **DWM (Desktop Window Manager)** — mandatory compositor since Vista (2007). Apps render off-screen; DWM composites. Same idea as a Wayland compositor.
- **DirectX / DirectComposition** — GPU rendering APIs.
- **WDDM** — GPU driver model (scheduling, hang recovery, sharing).
- **RDP** — the closest thing to a "display protocol" in the X11 sense, but a separate remote-access feature, not the foundation of local display.

### Comparison table

| Aspect | X11 | Wayland | Windows (DWM stack) |
|---|---|---|---|
| Age | 1984 | 2008– | compositor since 2007 |
| Architecture | Separate server; apps are network clients | Compositor **is** the server | Compositor always on; parts in kernel |
| Who provides it | Interchangeable (X.Org, XWayland…) | Each DE ships its own (Mutter, KWin…) | Only Microsoft's; not replaceable |
| Network transparency | Built-in (X forwarding) | Excluded by design (use RDP/VNC/PipeWire) | Not built-in; RDP separate |
| App isolation | None | Strong | Strong-ish |
| Rendering model | Server can draw (legacy); modern apps render client-side | Always client-side; compositor composites | Client-side buffers; DWM composites |
| Customization | Highly extensible | Compositor-specific protocols | Closed |

### Takeaways

- **Wayland converged on the Windows model** (mandatory compositing, client-side rendering) — Windows forced the switch in 2007; Linux needed a protocol redesign.
- **X11's network transparency became its burden**: almost nobody uses it, but everyone pays the complexity cost.
- **Monolithic vs replaceable**: Windows gets uniform, early features but no alternatives; Linux gets choice and fragmentation.

## 4. X11 forwarding over SSH

The mind-bending part: **roles feel backwards** — the X *server* runs on YOUR laptop (it owns the display/input), the X *client* is the remote application doing the computing.

```
        Your laptop (local)                          Remote server
┌────────────────────────────────────┐      ┌──────────────────────────────────────────┐
│   ┌──────────────┐                 │      │   ┌────────────────┐                       │
│   │  X server    │  draws windows, │      │   │  X client app  │  the PROGRAM runs     │
│   │  (X.Org or   │◀──┐ reads your  │      │   │  (e.g. gedit,  │  here — CPU, disk,    │
│   │  XWayland)   │   │ keyboard &  │      │   │  matlab, xterm)│  files all here       │
│   └──────▲───────┘   │ mouse       │      │   └───────┬────────┘                       │
│          │ X11 protocol commands   │      │  "open a window", "draw this line",        │
│   ┌──────┴───────┐                 │      │  carried inside the SSH channel            │
│   │  ssh client  │◀════════════════╪══════╪══▶│  sshd: sets    │                       │
│   └──────────────┘  one encrypted  │      │   │  DISPLAY=      │                       │
│                     SSH connection │      │   │  localhost:10  │                       │
└────────────────────────────────────┘      └──────────────────────────────────────────┘
```

### How it works, step by step

1. Connect with `ssh -X user@remote` (`-X` = untrusted/restricted, safer; `-Y` = trusted/full access).
2. Remote `sshd` listens on a virtual display (e.g. TCP 6010) and sets `DISPLAY=localhost:10.0`.
3. An auth cookie is planted in remote `~/.Xauthority` (via `xauth`) so other users can't inject windows.
4. Launching a GUI app remotely: it reads `DISPLAY`, connects to `localhost:6010` → actually `sshd`.
5. `sshd` funnels X11 traffic inside the encrypted SSH channel to your local `ssh` client → your local X server (or XWayland).
6. Bidirectional: app → you ("draw window"), you → app ("key pressed", "mouse clicked").

The app must be launched **inside** the `-X` session — `DISPLAY` is only set there. Otherwise: "Can't open display".

### Why it can feel slow

X11 is **chatty** (many round trips per window). Fine on LAN, painful over high-latency links. Alternatives: VNC/RDP (compressed pixels), VS Code Remote (UI local, files remote), Xpra (per-window with compression).

### On Wayland systems

`ssh -X` still works because **XWayland** plays the local X server role; forwarded apps appear as Wayland windows. Server side needs `xauth` installed and `X11Forwarding yes` in `/etc/ssh/sshd_config`.

## 5. XWayland

A real X server (X.Org codebase) that uses a Wayland compositor as its "display hardware":

```
X11 app ──X11 protocol──▶ XWayland ──Wayland protocol──▶ GNOME Shell (Mutter) ──▶ GPU/screen
```

- X11 apps can't tell the difference.
- Caveat: Wayland's app isolation doesn't extend across XWayland — X11 apps can still snoop on each other.

## 6. Local X11 transport: does it use the network stack?

**No — usually a Unix domain socket.** The `DISPLAY` value selects the transport:

| `DISPLAY` value | Transport | Network stack? |
|---|---|---|
| `:0` (typical local) | Unix socket `/tmp/.X11-unix/X0` | No — kernel IPC |
| `localhost:0` | TCP loopback, port 6000 | Yes (loopback only) |
| `somehost:0` | TCP to host, port 6000 | Yes, real network (insecure, obsolete) |
| `localhost:10` (SSH forwarding) | TCP loopback, port 6010 → SSH pipe | Yes (loopback, then encrypted SSH) |

- Port math: display `N` → TCP port `6000 + N` (hence `:10` → 6010).
- Modern X.Org runs with `-nolisten tcp`: Unix socket only for local clients.
- SSH forwarding is hybrid: Unix socket on the local end, loopback TCP on the remote end, encrypted SSH in the middle.
- X11 is transport-agnostic (TCP, Unix sockets, even shared memory via MIT-SHM locally): the client-server *design* enables network transparency without mandating it.

### Inspect it

```bash
ls /tmp/.X11-unix/     # socket files, e.g. X0 for display :0
ss -x | grep X11       # local apps connected via Unix socket
```
