# Design Tokens and Brand Reference

This document serves as the single reference for contributors making UI decisions, consolidating design tokens across the Tailwind configuration and UI components.

## Figma Reference
- [Profile Page Design (Figma)](https://www.figma.com/design/qnXwOxdJD4SBC6yjdwsilN/Agora-Profile-Page?node-id=0-1&t=uFO22ORvuFcXBCvh-1)

## Colour Palette

### Primary / Accent
| Token | Hex Value (Light) | Hex Value (Dark) | Tailwind Class |
| :--- | :--- | :--- | :--- |
| Accent | `#FDDA23` | `#f0d93a` | `bg-accent`, `text-accent` |
| Accent Hover | `#f0ce1e` | `#e0cc1e` | `bg-accent-hover`, `text-accent-hover` |
| Accent Dark | `#CAAE1C` | `#b89618` | `bg-accent-dark`, `text-accent-dark` |
| Accent Muted | `#F2ECCD` | `#4a4420` | `bg-accent-muted`, `text-accent-muted` |

### Background / Surface
| Token | Hex Value (Light) | Hex Value (Dark) | Tailwind Class |
| :--- | :--- | :--- | :--- |
| Base | `#FFFBE9` | `#0f1115` | `bg-base` |
| Base Alt | `#FFFBEA` | `#16181d` | `bg-base-alt` |
| Surface | `#FFEFD3` | `#1c1f25` | `bg-surface` |
| Surface Alt | `#F7ECD5` | `#252830` | `bg-surface-alt` |
| Cream | `#F3EEDC` | `#22252b` | `bg-cream` |
| Muted | `#FAF9F6` | `#1a1d23` | `bg-muted` |

### Ink / Text
| Token | Hex Value (Light) | Hex Value (Dark) | Tailwind Class |
| :--- | :--- | :--- | :--- |
| Ink | `#060606` | `#f4f4f5` | `text-ink`, `bg-ink` |
| Ink Soft | `#1A1A1A` | `#d6d7dc` | `text-ink-soft` |
| Ink Alt | `#1C1C1C` | `#cacbcf` | `text-ink-alt` |
| Ink Deep | `#131517` | `#a8aab0` | `text-ink-deep` |
| Dark | `#0B151F` | `#0a0d14` | `text-dark` |
| Dark Alt | `#2F2E24` | `#1a1d28` | `text-dark-alt` |
| Dark Deep | `#171402` | `#13161e` | `text-dark-deep` |
| Muted Text | `#747475` | `#a1a4ab` | `text-muted-text` |

### Status
| Token | Hex Value (Light) | Hex Value (Dark) | Tailwind Class |
| :--- | :--- | :--- | :--- |
| Error | `#F90B0B` | `#ff4d4f` | `text-error`, `bg-error` |
| Success Light | `#DAFFB5` | `#1f3d1a` | `bg-success-light`, `text-success-light` |

### Border / Subtle
| Token | Hex Value (Light) | Hex Value (Dark) | Tailwind Class |
| :--- | :--- | :--- | :--- |
| Subtle | `#D5D5D6` | `#2e3139` | `bg-subtle`, `border-subtle` |
| Border Warm | `#F0EAD6` | `#2e3139` | `border-border-warm` |
| Sand | `#A9A495` | `#8a8d94` | `text-sand`, `bg-sand` |

## Typography Scale

Our primary font family is **Inter**.

| Element | Font Family | Weight | Size (Tailwind) | Example Class |
| :--- | :--- | :--- | :--- | :--- |
| Base Sans | Inter | 400 (Normal), 600 (Semibold), 700 (Bold) | `text-sm`, `text-base`, `text-lg`, `text-xl` | `font-sans text-base font-normal` |
| Buttons | Inter | 600 (Semibold) | `text-sm`, `text-base` | `font-semibold text-sm` |
| Headings | Inter | 700 (Bold) | `text-xl` and above | `font-bold text-xl` |

## Shadows & Borders

Agora incorporates Neo-Brutalism stylistic elements in its components, characterized by solid borders and hard offset shadows.

### Shadows
| Style | Tailwind Class |
| :--- | :--- |
| Primary Hard Shadow | `shadow-[4px_4px_0px_0px_#000]` |
| Primary Hard Shadow (Hover/Focus) | `shadow-[2px_2px_0px_0px_#000]` |
| Secondary/Inverse Shadow | `shadow-[-4px_4px_0px_0px_rgba(0,0,0,0.4)]` |
| Light Inverse Shadow | `shadow-[-2px_2px_0px_0px_rgba(0,0,0,1)]` |

### Borders
| Style | Tailwind Class | Usage |
| :--- | :--- | :--- |
| Thin Dark Border | `border border-black` | Secondary buttons, subtle outlines |
| Thick Dark Border | `border-2 border-black` | Inputs, primary card boundaries |

### Border Radius
| Style | Tailwind Class | Usage |
| :--- | :--- | :--- |
| Full / Pill | `rounded-full` | Buttons, text inputs, badges |
| Large | `rounded-lg`, `rounded-xl` | Cards, larger UI containers |

## Spacing Conventions

The project uses standard Tailwind spacing scales (`0.25rem` multipliers).

| Spacing Type | Common Tailwind Classes |
| :--- | :--- |
| Buttons | `px-6 py-3` (Standard), `px-4 py-2` (Small) |
| Layout / Flex Gaps | `gap-2` (0.5rem), `gap-3` (0.75rem), `gap-4` (1rem) |
| Container Padding | `p-4`, `p-6` |
