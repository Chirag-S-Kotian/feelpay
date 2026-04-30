---
version: alpha
name: Mastercard
description: Warm cream canvas. Orbital pill shapes. Editorial warmth.
colors:
  primary: "#141413"
  secondary: "#6B6862"
  tertiary: "#EB001B"
  neutral: "#F5EFE2"
  surface: "#FFF9EC"
  on-primary: "#FFF9EC"
typography:
  display:
    fontFamily: Inter
    fontSize: 4.75rem
    fontWeight: 700
    letterSpacing: "-0.025em"
  h1:
    fontFamily: Inter
    fontSize: 2.3rem
    fontWeight: 600
  body:
    fontFamily: Inter
    fontSize: 0.98rem
    lineHeight: 1.6
  label:
    fontFamily: Inter
    fontSize: 0.74rem
    fontWeight: 600
    letterSpacing: "0.08em"
rounded:
  sm: 100px
  md: 100px
  lg: 100px
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

Mastercard: warm cream canvas, circular orange/red orbital shapes, editorial warmth, confident sans.

## Colors

The palette is built around high-contrast neutrals and a single accent that drives interaction.

- **Primary (`#141413`):** Headlines and core text.
- **Secondary (`#6B6862`):** Borders, captions, and metadata.
- **Tertiary (`#EB001B`):** The sole driver for interaction. Reserve it.
- **Neutral (`#F5EFE2`):** The page foundation.

## Typography

- **display:** Inter 4.75rem
- **h1:** Inter 2.3rem
- **body:** Inter 0.98rem
- **label:** Inter 0.74rem

## Do's and Don'ts

- **Do** use Tertiary for exactly one action per screen.
- **Do** let Neutral carry the composition — negative space is a feature.
- **Don't** introduce gradients. This system is flat on purpose.
- **Don't** mix Tertiary with alternate accents; the single-accent rule is load-bearing.
