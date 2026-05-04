---
version: alpha
name: Spotify
description: Vibrant green on dark. Bold type. Album-art-driven.
colors:
  primary: "#FFFFFF"
  secondary: "#B3B3B3"
  tertiary: "#1DB954"
  neutral: "#121212"
  surface: "#191414"
  on-primary: "#121212"
typography:
  display:
    fontFamily: Inter
    fontSize: 5rem
    fontWeight: 900
    letterSpacing: "-0.04em"
  h1:
    fontFamily: Inter
    fontSize: 2.4rem
    fontWeight: 800
  body:
    fontFamily: Inter
    fontSize: 0.95rem
    lineHeight: 1.55
  label:
    fontFamily: Inter
    fontSize: 0.74rem
    fontWeight: 700
    letterSpacing: "0.08em"
rounded:
  sm: 8px
  md: 500px
  lg: 16px
spacing:
  sm: 8px
  md: 16px
  lg: 32px
components:
  button-primary:
    backgroundColor: "{colors.tertiary}"
    textColor: "{colors.on-primary}"
    rounded: "{rounded.md}"
    padding: 12px 20px
  card:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.primary}"
    rounded: "{rounded.lg}"
    padding: 24px
---
## Overview

Spotify: music-streaming dark mode, vibrant Spotify-green accent, album-art-driven layouts, heavy type.

## Colors

The palette is built around high-contrast neutrals and a single accent that drives interaction.

- **Primary (`#FFFFFF`):** Headlines and core text.
- **Secondary (`#B3B3B3`):** Borders, captions, and metadata.
- **Tertiary (`#1DB954`):** The sole driver for interaction. Reserve it.
- **Neutral (`#121212`):** The page foundation.

## Typography

- **display:** Inter 5rem
- **h1:** Inter 2.4rem
- **body:** Inter 0.95rem
- **label:** Inter 0.74rem

## Do's and Don'ts

- **Do** use Tertiary for exactly one action per screen.
- **Do** let Neutral carry the composition — negative space is a feature.
- **Don't** introduce gradients. This system is flat on purpose.
- **Don't** mix Tertiary with alternate accents; the single-accent rule is load-bearing.
