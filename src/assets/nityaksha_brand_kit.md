# NITYAKSHA — Brand Identity & Style Guidelines

> **Tagline:** An AI Concierge Platform  
> **Sub-tagline:** Engineered for high-velocity, deterministic multi-agent orchestration.

---

## 1. Brand Essence & Vision

**Nityaksha** combines timeless stability with forward-moving intelligence. The platform stands for determinism, sub-millisecond execution, and continuous, refined guidance.

* **Core Attributes:** High-velocity, Deterministic, Resilient, Sophisticated, Ambient.
* **Tone of Voice:** Authoritative, direct, technical yet welcoming, and noise-free.

---

## 2. Logo System

### 2.1 The Symbol Mark
The logo mark is an abstract orbital system constructed on golden-ratio arcs:
* **The Guiding Arc (Navy):** Signifies continuous protection, determinism, and concierge oversight.
* **The Neural Kinetic Ring (Rust):** Symbolizes high-concurrency node routing, computational velocity, and Rust-level execution.

### 2.2 Clearspace & Proportions
* **Minimum Clearspace:** Maintain padding around all sides of the mark equal to $1.5 \times$ the width of the inner node circle ($X$).
* **Minimum Sizing:**
  * Digital Favicon / App Icon: `24px × 24px`
  * Standard UI Navbar: `32px` height
  * Print Lockup: `15mm` height minimum

### 2.3 Logo Lockups
1. **Primary Vertical Lockup:** Centered logomark above the wordmark and tagline badge.
2. **Horizontal Navbar Lockup:** Left-aligned logomark paired with inline `NITYAKSHA` wordmark.
3. **App Icon / Submark:** Monogrammatic orbital icon centered on rounded squares (`#0D2340` background for dark, `#FFFFFF` for light).

---

## 3. Color Palette

### 3.1 Primary Colors

| Swatch | Color Name | HEX | RGB | HSL | Usage |
|---|---|---|---|---|---|
| 🟦 | **Deep Navy** | `#0D2340` | `rgb(13, 35, 64)` | `214°, 66%, 15%` | Primary background (Dark), Headings, Main Brand Arc |
| 🟧 | **Rust Orange** | `#C05A28` | `rgb(192, 90, 40)` | `20°, 66%, 45%` | Action accents, Node indicators, Rust Badges, Focus states |

### 3.2 Secondary & Neutral Colors

| Swatch | Color Name | HEX | RGB | Role |
|---|---|---|---|---|
| 🔘 | **Slate Steel** | `#2E4F6F` | `rgb(46, 79, 111)` | Secondary borders, muted UI surfaces |
| 🌫️ | **Cool Slate** | `#607D8B` | `rgb(96, 125, 139)` | Secondary copy, subtle dividers, caption text |
| ⚪ | **Off-White Canvas** | `#F4F6F9` | `rgb(244, 246, 249)` | Light mode background, card fill |
| 🤍 | **Pure White** | `#FFFFFF` | `rgb(255, 255, 255)` | Light mode surface base, dark mode text |

### 3.3 CSS Design Tokens

```css
:root {
  /* Brand Core */
  --color-brand-primary: #0D2340;
  --color-brand-accent: #C05A28;
  --color-brand-accent-hover: #A3481D;

  /* Neutrals */
  --color-surface-dark: #09192E;
  --color-surface-panel: #0D2340;
  --color-surface-light: #F4F6F9;
  --color-surface-white: #FFFFFF;
  --color-border-subtle: #2E4F6F;
  --color-text-muted: #607D8B;

  /* Typography */
  --font-heading: 'Cabinet Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  --font-body: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  --font-code: 'JetBrains Mono', 'Fira Code', monospace;
}
```

---

## 4. Typography

### 4.1 Typeface Families
* **Display & Headings:** `Cabinet Grotesk` or `Space Grotesk` (Geometric, confident, technical).
* **Body & UI Text:** `Inter` or `Plus Jakarta Sans` (High legibility at dense information densities).
* **Code & Monospace (Rust syntax, latency metrics):** `JetBrains Mono`.

### 4.2 Hierarchy Scale

| Level | Size | Weight | Line Height | Tracking |
|---|---|---|---|---|
| **H1 Hero** | `48px` / `3rem` | Bold (700) | `1.1` | `-0.02em` |
| **H2 Section** | `32px` / `2rem` | SemiBold (600) | `1.2` | `-0.015em` |
| **H3 Card** | `20px` / `1.25rem` | Medium (500) | `1.35` | `0` |
| **Body Large** | `18px` / `1.125rem` | Regular (400) | `1.5` | `0` |
| **Body Standard**| `15px` / `0.9375rem`| Regular (400) | `1.6` | `0` |
| **UI Badge / Overline** | `11px` / `0.6875rem` | Bold (700) | `1.0` | `+0.08em` (UPPERCASE) |

---

## 5. UI Elements & Components

### 5.1 Platform Status Badge
* **Background:** Transparent with `1px` border of `#C05A28` OR solid fill `#C05A28` with `#FFFFFF` text.
* **Border Radius:** Pill shape (`9999px`).
* **Padding:** `4px 12px`.
* **Font:** `JetBrains Mono`, 11px uppercase bold.

### 5.2 Primary Buttons
* **Background:** Linear gradient `135deg, #C05A28 0%, #D86B35 100%`.
* **Text:** `#FFFFFF`, Medium weight.
* **Shadow:** `0 4px 14px rgba(192, 90, 40, 0.35)`.
* **Hover State:** Translate Y `-1px`, shadow bloom `0 6px 20px rgba(192, 90, 40, 0.45)`.

### 5.3 Secondary Buttons
* **Background:** Transparent or `#0D2340` (in light mode: `#FFFFFF`).
* **Border:** `1px solid #2E4F6F`.
* **Text:** `#0D2340` (light mode) / `#FFFFFF` (dark mode).

---

## 6. Iconography & Web Graphics

* **Stroke Weight:** Consistent `1.75px` or `2px` stroke lines.
* **Corner Radius:** Slightly rounded joins (`round` / `round`) to reflect the fluid golden ratio curves of the logo.
* **Background Patterns:** Subtle dark grid (`rgba(46, 79, 111, 0.15)`) accented with thin vector node traces in Rust orange.

---

## 7. Do's and Don'ts

| Do | Don't |
|---|---|
| Maintain high contrast ratios (AAA rating compliant). | Do not distort, skew, or rotate the logomark. |
| Use Rust orange sparingly as a functional focal point. | Do not introduce extraneous gradient colors (e.g., purple, neon green). |
| Pair the logomark with ample breathing space. | Do not surround the logo with visual clutter or busy photographic backdrops. |
| Keep badges clean and monospace. | Do not use decorative serif fonts for tech specifications. |