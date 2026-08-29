# Linux Input Methods: IBus, Fcitx, Engines, and How Input Really Works

Notes from a discussion on Ubuntu 24.04 (GNOME 46, Wayland session), after setting up Chinese input (IBus + libpinyin). Companion to `gui-display-stack.md`.

## 1. Why input method frameworks exist

A keyboard has ~100 keys; Chinese has tens of thousands of characters, Japanese has three scripts, Korean composes syllable blocks. A keyboard layout alone can't solve this. An **input method (IM) framework** intercepts keystrokes, shows candidate characters, and only then sends the final text to the app.

## 2. IBus (Intelligent Input Bus)

One of the two dominant Linux IM frameworks (the other: **Fcitx**). "Bus" is literal — everything talks over D-Bus.

### Architecture

```
Physical keyboard
      ↓ key events
┌─────────────────────────────────────────────┐
│  ibus-daemon (the central hub, D-Bus based) │
│                                             │
│    ┌──────────┐   ┌──────────┐   ┌────────┐ │
│    │ engine:  │   │ engine:  │   │ engine:│ │
│    │libpinyin │   │  anthy   │   │ hangul │ │
│    │ (Chinese)│   │(Japanese)│   │(Korean)│ │
│    └──────────┘   └──────────┘   └────────┘ │
└───────▲──────────────────────┬──────────────┘
        │ key events           │ committed text
        │                      ↓
   Applications (GTK/Qt apps, via IM modules or Wayland protocol)
```

- **ibus-daemon** — central process: receives raw key events, routes them to the active engine, delivers output to the focused app.
- **Engines** — pluggable per-language backends, each a separate process (`ibus list-engine`).
- **Client side** — apps connect via IM modules (built into GTK/Qt) or, on Wayland, the compositor's input-method protocol.

### What happens when you type "nihao"

1. Key events go to the compositor (GNOME Shell), which forwards them to IBus.
2. IBus routes them to the libpinyin engine.
3. The engine maintains the pinyin buffer, computes candidates (你好, 尼浩, …), draws the candidate popup.
4. On selection, the engine **commits** `你好` back through IBus → compositor → app, as ordinary text.

Until commit, the app sees nothing — the in-progress pinyin lives only in the IM layer (hence Esc cancels mid-word).

## 3. Framework vs engine: the Linux model vs Windows

Windows bundles both layers into monolithic products ("Sogou", "Microsoft Pinyin" = plumbing + brain in one). Linux separates them:

```
framework (IBus / Fcitx)   ← plumbing: key routing, candidate popup, app integration
    + engine (libpinyin, Rime, ...)  ← brain: pinyin→characters, dictionaries, prediction
```

The "product variety" on Linux lives at the **engine** layer:

- **libpinyin** — sensible default, open source.
- **Rime (中州韻, `ibus-rime` / `fcitx5-rime`)** — enthusiast favorite; a platform with schemas (pinyin, double-pinyin, wubi, cangjie; community schemas like 雾凇拼音 rival commercial IMEs). Configured via YAML; powerful but a learning curve.
- **sunpinyin** — older statistical engine, mostly unmaintained.
- **Cloud Pinyin** — queries cloud services for candidates.
- Other languages: anthy/mozc (Japanese), hangul (Korean), table engines (wubi, zhengma).

**Sogou for Linux exists** — free, closed-source, built on Fcitx; default on Ubuntu Kylin / UOS / Deepin. Caveats: infrequent updates, occasional breakage on new distro releases, privacy (telemetry) concerns.

Why it *feels* like fewer choices: tiny Chinese Linux desktop market (community-driven, effort pooled into Rime/libpinyin), and the layer split (2 frameworks × several engines) looks like less variety than 10 monolithic brands.

## 4. Switching frameworks (IBus ↔ Fcitx)

Fully supported and reversible. The active framework is decided at session start by:

1. Which daemon autostarts (`ibus-daemon` vs `fcitx5`)
2. Environment variables telling apps which IM module to load:
   - `GTK_IM_MODULE=ibus|fcitx`
   - `QT_IM_MODULE=ibus|fcitx`
   - `XMODIFIERS=@im=ibus|fcitx` (X11 apps)
3. `im-config` — the tool that writes these settings (`~/.xinputrc`).

```bash
sudo apt install fcitx5 fcitx5-chinese-addons fcitx5-configtool
im-config -n fcitx5     # switch; -n ibus to switch back
# log out and back in (mandatory — env vars are set at session start)
```

Both frameworks can be installed; only one is active per session.

### The GNOME/Wayland caveat

- **GNOME is married to IBus**: Settings → Input Sources, the top-bar indicator, and Super+Space all drive IBus directly. With Fcitx5 you manage everything through Fcitx's own tool/panel.
- **Wayland makes it worse**: Mutter's native input-method protocol is IBus-only (see §5). Fcitx5 on GNOME/Wayland works through compatibility paths, with quirks ("Chinese works in Firefox but not in the text editor").
- **Fcitx5 shines on KDE Plasma or X11** — first-class support, richer engines (Rime, Sogou), better skinning.

For typing *quality* without a framework switch: `sudo apt install ibus-rime` adds Rime as just another IBus input source, keeping all GNOME integration.

## 5. Per-app IM modules: are the env vars application-specific?

Yes — `GTK_IM_MODULE` etc. are **process-scoped**, inherited at launch. You *can* mix frameworks per app... but what actually happens depends on the display path.

### X11 / XWayland apps: env vars honored

```bash
GTK_IM_MODULE=ibus  code      # VS Code → ibus-daemon
GTK_IM_MODULE=fcitx firefox   # Firefox → fcitx5
```

With both daemons running this works: independent connections, independent state. Costs: two candidate-popup styles, two hotkey sets (Super+Space vs Ctrl+Space), two indicators. If a framework's daemon isn't running, its module silently falls back to plain keyboard input.

### Native Wayland apps on GNOME: env vars mostly ignored

On Wayland, input goes **through the compositor**:

```
app ←── text-input / input-method protocol ── Mutter (GNOME Shell) ←── ibus-daemon
```

GNOME Shell implements the Wayland input-method protocol and speaks only to **IBus**. A native-Wayland app's `GTK_IM_MODULE=fcitx` is never consulted — everything funnels into IBus regardless. Fcitx only becomes usable for apps under XWayland (e.g. `GDK_BACKEND=x11 firefox`).

KDE's KWin implements the protocol framework-agnostically, which is why Fcitx5 works natively there.

### Takeaway

Same pattern as the display stack itself: on Wayland the compositor owns the input path end-to-end, so whichever framework the compositor blesses (IBus on GNOME) wins by construction.

## 6. Current setup on this machine

- Framework: **IBus** (Ubuntu/GNOME default), engines: `libpinyin` (Intelligent Pinyin).
- Input sources: `[English (US), Chinese (Intelligent Pinyin)]` — English first/default.
- Switch: **Super+Space**; temporary Chinese↔English toggle within pinyin: **Shift**.
- Engine preferences: Settings → Keyboard → Input Sources → Chinese (Intelligent Pinyin) → ⚙.
