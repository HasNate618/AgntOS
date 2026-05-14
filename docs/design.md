# AgntOS Design System

## Brand

### What AgntOS Is

AgntOS is a NixOS-based AI-native Linux distribution. It ships KDE Plasma with a built-in agent that understands the screen, manages the system, routes AI models, and safely applies changes through structured tools — not chatbot windows.

### Who It's For

People who want Linux to feel like having a technical co-pilot, not a second job. Students, creators, researchers, and AI-curious users who want a cutting-edge OS that helps them enter the AI era without memorizing system administration.

### Brand Voice

- **Confident but approachable.** We know the system works. We don't need to shout.
- **Direct, not corporate.** Say "your agents run here," not "leveraging next-generation agentic paradigms."
- **Slightly futuristic, never gimmicky.** The OS is impressive. Let the product speak.
- **Warm intelligence.** The agent is helpful, not cold. The orange is warm. The dark is calm.

### Tagline

The agent-native operating system.

### Headline Examples

- *Linux has a new operator.*
- *Your OS, with an assistant that actually knows the OS.*
- *Ask it to install Firefox. Watch it plan, diff, apply, and offer rollback.*
- *Not a chatbot. An operator.*

---

## Typography

### Families

| Role | Font | Character |
|------|------|-----------|
| **Display / Brand** | **Syne** | Bold, geometric, distinctive. Used for logos, hero headlines, and marketing. |
| **UI / Body** | **Plus Jakarta Sans** | Warm, highly readable, modern humanist sans. Used for all interface text. |
| **Monospace** | **Geist Mono Nerd Font** | Sharp, technical, excellent legibility. Used for terminal, code, and logs. |

### Rationale

Syne is unexpected in OS branding — it's neither corporate Helvetica nor techy monospace. It gives AgntOS a visual signature. Plus Jakarta Sans is warm and approachable, a better fit for "normal users getting into AI" than sterile choices like Inter or system fonts. Geist Mono is modern, lightweight, and Nerd Font-patched for terminal icons.

### Type Scale

```
Display:    Syne 48px / 700 / tracking -0.02em
H1:         Syne 36px / 700 / tracking -0.01em
H2:         Plus Jakarta Sans 28px / 600
H3:         Plus Jakarta Sans 22px / 600
H4:         Plus Jakarta Sans 18px / 600
Body:       Plus Jakarta Sans 15px / 400 / line-height 1.6
Small:      Plus Jakarta Sans 13px / 400 / line-height 1.5
Caption:    Plus Jakarta Sans 11px / 500 / uppercase / tracking 0.05em
Mono:       Geist Mono 14px / 400 / line-height 1.6
Mono Small: Geist Mono 12px / 400
```

### CSS Import

```css
@import url('https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans:wght@400;500;600;700&family=Syne:wght@600;700;800&display=swap');

:root {
  --font-display: 'Syne', sans-serif;
  --font-body: 'Plus Jakarta Sans', sans-serif;
  --font-mono: 'GeistMono Nerd Font', 'Geist Mono', monospace;
}
```

---

## Color System

### Philosophy

AgntOS uses a **warm-on-dark** palette. The system is dark by default — a calm, focused environment. Warm orange is the signature accent: it signals action, agent presence, and highlights without feeling like a warning or error. Cool blues and purples are reserved for AI-specific and informational states.

### Palette

| Token | Hex | Role |
|-------|-----|------|
| `--agnt-orange` | `#F57C48` | Primary brand, active states, agent indicators |
| `--agnt-orange-hover` | `#E06835` | Button hover, pressed states |
| `--agnt-orange-glow` | `rgba(245, 124, 72, 0.25)` | Focus rings, subtle glows |
| `--surface-void` | `#141416` | Deepest background, terminal bg |
| `--surface-base` | `#1C1C1F` | Primary background |
| `--surface-raised` | `#242428` | Cards, panels, dialogs |
| `--surface-overlay` | `#2C2C31` | Modals, dropdowns |
| `--border-default` | `#333338` | Standard borders |
| `--border-accent` | `#4A4A50` | Emphasized borders |
| `--text-primary` | `#EBEBEC` | Primary text |
| `--text-secondary` | `#9C9CA3` | Secondary text |
| `--text-muted` | `#64646B` | Disabled, placeholders |
| `--success` | `#4CAF7A` | Success, completed, active agents |
| `--warning` | `#E6A23C` | Pending, processing |
| `--error` | `#E5534B` | Failures, critical |
| `--info` | `#4493F8` | Information, links |
| `--ai-purple` | `#8B5CF6` | Neural/ML features, agent intelligence |

### Gradients

```css
/* Primary accent — warm orange glow */
--gradient-accent: linear-gradient(135deg, #F57C48 0%, #F9A26C 100%);

/* Surface depth — dark micro-gradient */
--gradient-surface: linear-gradient(180deg, #1C1C1F 0%, #19191C 100%);

/* AI highlight — purple-to-orange spectrum */
--gradient-ai: linear-gradient(135deg, #8B5CF6 0%, #F57C48 100%);

/* Focus ring */
--gradient-focus: linear-gradient(135deg, #F57C48 0%, #8B5CF6 100%);
```

### Contrast Validation

| Pairing | Ratio | WCAG |
|---------|-------|------|
| `--text-primary` on `--surface-base` | 13.8:1 | AAA ✓ |
| `--agnt-orange` on `--surface-base` | 5.8:1 | AA ✓ |
| `--text-secondary` on `--surface-base` | 6.9:1 | AA ✓ |
| `--text-muted` on `--surface-raised` | 3.7:1 | — (non-text only) |

---

## Logo System

### Mark

The AgntOS mark uses a hexagonal motif — a reference to agent nodes, neural topology, and the six core capabilities of the Agnt agent (observe, suggest, assist, admin, autopilot, rollback). The mark should feel architectural, not cartoonish.

```
       _______
     /        \
    /          \
   /    AGNT    \
   \    ⬡  ⬡   /
    \          /
     \________/
```

### Variations

| Variant | Background | Usage |
|---------|-----------|-------|
| Full color | Dark surfaces | Primary, app icons, website |
| Monochrome white | Dark surfaces over images | Wallpapers, splash |
| Monochrome dark | Light surfaces | Print, light docs |
| Mark only | Any | Favicon, tray icon, small spots |

### Rules

- Minimum clear space: 1× mark height on all sides
- Minimum size: 24px for mark-only, 48px for full logo
- Never rotate, skew, add effects, or recolor
- The mark alone is sufficient for most small-space contexts

---

## UI Components

### Buttons

```css
/* Primary — the agent action button */
.btn-primary {
  background: var(--agnt-orange);
  color: var(--surface-void);
  border: none;
  border-radius: 10px;
  padding: 10px 24px;
  font: 600 14px var(--font-body);
  transition: background 150ms ease, box-shadow 150ms ease;
}
.btn-primary:hover {
  background: var(--agnt-orange-hover);
  box-shadow: 0 0 20px var(--agnt-orange-glow);
}

/* Secondary — outlined, reserved */
.btn-secondary {
  background: transparent;
  border: 1.5px solid var(--agnt-orange);
  color: var(--agnt-orange);
  font: 600 14px var(--font-body);
  border-radius: 10px;
  padding: 10px 24px;
}
.btn-secondary:hover {
  background: rgba(245, 124, 72, 0.08);
}

/* Ghost — invisible until hover */
.btn-ghost {
  background: transparent;
  color: var(--text-secondary);
  padding: 8px 16px;
  border-radius: 8px;
}
.btn-ghost:hover {
  background: rgba(235, 235, 236, 0.06);
  color: var(--text-primary);
}
```

### Cards

```css
.card {
  background: var(--surface-raised);
  border: 1px solid var(--border-default);
  border-radius: 14px;
  padding: 24px;
  transition: border-color 200ms ease, box-shadow 200ms ease;
}
.card:hover {
  border-color: var(--border-accent);
  box-shadow: 0 4px 32px rgba(0, 0, 0, 0.3);
}
.card:focus-within {
  border-color: var(--agnt-orange);
  box-shadow: 0 0 0 1px var(--agnt-orange-glow);
}
```

### Inputs

```css
.input {
  background: var(--surface-void);
  border: 1px solid var(--border-default);
  border-radius: 10px;
  color: var(--text-primary);
  padding: 10px 14px;
  font: 14px var(--font-body);
  transition: border-color 150ms ease, box-shadow 150ms ease;
}
.input::placeholder { color: var(--text-muted); }
.input:focus {
  border-color: var(--agnt-orange);
  box-shadow: 0 0 0 3px var(--agnt-orange-glow);
  outline: none;
}
```

### Status Indicators

```
Online/Active:   ◆ #4CAF7A  (pulsing glow)
Processing:      ◉ #E6A23C  (rotating arc)
Idle/Standby:    ○ #9C9CA3  (static)
Error/Offline:   ◆ #E5534B  (static)
Agent thinking:  ◇ #8B5CF6  (slow pulse)
```

---

## Terminal

### Konsole Theme

```json
{
  "name": "AgntOS",
  "foreground": "#EBEBEC",
  "background": "#141416",
  "cursorColor": "#F57C48",
  "selectionBackground": "rgba(245,124,72,0.2)",
  "black": "#242428",
  "red": "#E5534B",
  "green": "#4CAF7A",
  "yellow": "#E6A23C",
  "blue": "#4493F8",
  "magenta": "#8B5CF6",
  "cyan": "#54C8E8",
  "white": "#EBEBEC",
  "brightBlack": "#4A4A50",
  "brightRed": "#F27D72",
  "brightGreen": "#6ECB94",
  "brightYellow": "#F0B856",
  "brightBlue": "#6AADF5",
  "brightMagenta": "#A78BFA",
  "brightCyan": "#76D9F0",
  "brightWhite": "#FFFFFF"
}
```

### Prompt

```
┌─[user @ agntos]─[~/projects]
└─❯

# When the agent is active on a task:
┌─[user @ agntos]─[~/projects]─[🤖 planning]
└─
```

### CLI Styling Convention

```
commands:    var(--agnt-orange)
paths:       var(--info)
success:     var(--success)
warnings:    var(--warning)
errors:      var(--error)
agent output:var(--ai-purple)
```

---

## Desktop (Plasma)

AgntOS v0 targets KDE Plasma on Wayland.

### Global Theme

Default to **Breeze Dark** with AgntOS color overrides applied via KDE config files in `/etc/xdg/`:

```ini
# /etc/xdg/kdeglobals
[General]
ColorScheme=BreezeDark
widgetStyle=Breeze

[KDE]
LookAndFeelPackage=org.kde.breezedark.desktop

[WM]
activeFont=Plus Jakarta Sans,10,-1,5,50,0,0,0,0,0,Medium
```

### Panel

- Floating panel, centered, semi-transparent
- Height: 44px
- Background: `rgba(20, 20, 22, 0.85)` with `backdrop-filter: blur(16px)`
- Icons-left, tasks-center, system-tray-right layout
- Active window indicator: 2px orange underline

### Window Decorations

- Breeze window decoration
- Border radius: 10px
- Active window border: 2px `var(--agnt-orange)`
- Inactive window border: 1px `var(--border-default)`
- Title bar: 38px height, Plus Jakarta Sans 11px Medium

### Notifications

```css
.notification {
  background: var(--surface-raised);
  border: 1px solid var(--border-default);
  border-left: 3px solid var(--agnt-orange);
  border-radius: 12px;
  padding: 16px;
}
```

---

## Motion

### Philosophy

Motion in AgntOS is **restrained confidence**. Nothing bounce-in or playful. Everything has weight and purpose. Transitions feel like a system switching states, not an app animating.

### Token Scale

| Token | Duration | Easing | Use |
|-------|----------|--------|-----|
| `--motion-instant` | 100ms | ease-out | Button press, toggle |
| `--motion-fast` | 150ms | ease-out | Hover, focus, checkbox |
| `--motion-normal` | 250ms | cubic-bezier(0.2, 0, 0, 1) | Modal open, page switch |
| `--motion-slow` | 400ms | cubic-bezier(0.2, 0, 0, 1) | Panel expand, orchestrations |
| `--motion-agent` | 2000ms | linear infinite | Agent processing loop |

### Key Animations

```css
/* Agent pulse — signals the agent is active */
@keyframes agent-pulse {
  0% { box-shadow: 0 0 0 0 rgba(245, 124, 72, 0.4); }
  70% { box-shadow: 0 0 0 10px rgba(245, 124, 72, 0); }
  100% { box-shadow: 0 0 0 0 rgba(245, 124, 72, 0); }
}

/* Processing ring — agent is thinking */
@keyframes agent-rotate {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* Subtle breathing glow — idle agent */
@keyframes agent-breathe {
  0%, 100% { opacity: 0.6; }
  50% { opacity: 1; }
}

/* Staggered list reveal */
.stagger-item {
  opacity: 0;
  transform: translateY(12px);
  animation: stagger-in 400ms cubic-bezier(0.2, 0, 0, 1) forwards;
}
.stagger-item:nth-child(1) { animation-delay: 0ms; }
.stagger-item:nth-child(2) { animation-delay: 60ms; }
.stagger-item:nth-child(3) { animation-delay: 120ms; }
.stagger-item:nth-child(4) { animation-delay: 180ms; }
.stagger-item:nth-child(5) { animation-delay: 240ms; }

@keyframes stagger-in {
  to { opacity: 1; transform: translateY(0); }
}
```

### Accessibility

- All motion respects `prefers-reduced-motion: reduce`
- No autoplaying animations over 5 seconds
- Critical state changes never conveyed through motion alone

---

## Wallpapers

### Direction

Dark, atmospheric, minimal. Not busy. The focal point should support the desktop, not compete with it.

### Concepts

1. **Agent Grid** — `#141416` background with very faint hexagonal grid (`#1C1C1F`), subtle warm gradient vignette from the center
2. **Neural Drift** — Abstract flowing lines in muted orange (`rgba(245,124,72,0.06)`) on void background
3. **Orbit** — Thin orange ring orbiting in the distance on deep space dark
4. **Solid** — Pure `#141416` with the AgntOS mark at 8% opacity, centered

---

## Documentation & Marketing

### Tone

Write like you're explaining to a smart friend who hasn't used Linux before.

- **Don't say**: "Leveraging declarative NixOS configuration to enable agent-mediated system state mutation."
- **Say**: "The agent edits your system config. You see the diff. You approve it. You can roll it back."

- **Don't say**: "Seamlessly integrated multimodal model routing fabric."
- **Say**: "Pick which model handles what. Fast one for chat. Smart one for system changes. Local model for private files."

### Headline Examples

| Context | Headline |
|---------|----------|
| Homepage hero | Linux has a new operator. |
| Features page | It sees your screen. It knows your system. It asks before it acts. |
| Model routing | Your models, your rules. |
| OS management | Stop memorizing commands. Start asking. |
| Safety | Every change has a preview. Every change has a rollback. |
| Download CTA | Install AgntOS |

### Section Voice

**Product page**: Confident, clear, feature-forward. Short sentences. No buzzwords.

**Documentation**: Warm, patient, assumes no prior Linux knowledge. Code blocks next to plain-English explanations. Every concept explained before it's used.

**Blog / Changelog**: Direct, slightly informal. "We shipped X. Here's why. Here's what changed."

**Error messages**: Never blame the user. "Something went wrong while installing that package. The agent can help diagnose it. Run `agntctl inspect logs`."

---

## Icons

### System Icons

- Style: Outlined, 2px stroke, rounded caps and joins
- Grid: 24px base
- Color: `--text-secondary` default, `--agnt-orange` active
- Library: Phosphor or custom drawn

### AI-Specific Icons

- Agent node: Hexagon with centered dot
- Agent active: Hexagon with orbiting ring (animated)
- Model: Simplified neural net node mesh
- Routing: Branching path with highlighted route
- Thinking: Hexagon with pulsing interior

### Special Folder Icons

```
/agents    → Hexagon + agnt-orange
/models    → Network node + ai-purple
/skills    → Layered hexagon + agnt-orange
/logs      → Terminal icon + text-secondary
```

---

## Audio

*(v2 — not part of initial release)*

- Agent notification: Single warm tone, short decay
- Error: Low soft pulse
- Completion: Two ascending tones
- All off by default

---

## File: `modules/agntos/branding.nix` (Planned)

```nix
{ config, pkgs, lib, ... }:

{
  environment.systemPackages = [
    pkgs.agntos-branding
    pkgs.papirus-icon-theme
  ];

  environment.etc = {
    "xdg/kdeglobals".text = ''
      [General]
      ColorScheme=BreezeDark

      [Icons]
      Theme=Papirus-Dark
    '';

    "xdg/plasma-org.kde.plasma.desktop-appletsrc".text = ''
      [Containments][1][Wallpaper][org.kde.image][Image]
      Image=file:///run/current-system/sw/share/wallpapers/agntos/default.png
    '';
  };

  fonts.packages = with pkgs; [
    (pkgs.nerdfonts.override { fonts = [ "GeistMono" ]; })
    plus-jakarta-sans
    syne
  ];

  fonts.fontconfig.defaultFonts = {
    sansSerif = [ "Plus Jakarta Sans" ];
    monospace = [ "GeistMono Nerd Font" ];
  };
}
```

---

## Design Assets Checklist

```
pkgs/agntos-branding/
  ├── default.nix
  ├── wallpapers/
  │   └── default.png          # 2560×1600 or 3840×2160
  └── logo/
      ├── mark.svg              # Hexagon mark only
      ├── full-color.svg        # Mark + wordmark on dark
      └── full-white.svg        # White mark + wordmark on dark

modules/agntos/
  └── branding.nix              # All desktop defaults

docs/
  └── design.md                 # This document
```

## Questions

1. Do you want to keep the hexagonal mark concept, or explore other directions for the logo?
2. Should we keep Breeze Dark as the Plasma theme base, or is there a specific Plasma global theme you prefer (e.g., lightly, layan, etc.)?
3. The wallpaper direction — do you prefer geometric/architectural patterns or abstract atmospheric backgrounds?
4. Any specific SDDM login screen direction (minimal, animated, show agent status)?
5. Should we add a system sound scheme to the v0 scope or defer to v2 as noted?
