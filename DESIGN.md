---
name: Scene / split
description: Editorial video workbench with a white canvas and deep-green controls.
colors:
  canvas: "#ffffff"
  stone: "#eeece7"
  soft-surface: "#f6f5f2"
  hover-surface: "#e2e0da"
  ink: "#212121"
  secondary-ink: "#56565e"
  muted: "#656571"
  rule: "#d9d9dd"
  action: "#17171c"
  action-hover: "#323239"
  green: "#003c33"
  green-muted: "#c6dcd7"
  green-rule: "#38675e"
  link: "#1863dc"
  focus: "#4c6ee6"
  error: "#b30000"
  error-surface: "#fff2ee"
  selection: "#d7ebdf"
  video: "#000000"
typography:
  display:
    fontFamily: "Space Grotesk Variable, Space Grotesk, Inter, ui-sans-serif, system-ui, sans-serif"
    fontSize: "88px"
    fontWeight: 400
    lineHeight: 1
    letterSpacing: "-3.2px"
  title:
    fontFamily: "Space Grotesk Variable, Space Grotesk, Inter, ui-sans-serif, system-ui, sans-serif"
    fontSize: "26px"
    fontWeight: 400
    letterSpacing: "-0.7px"
  body:
    fontFamily: "Inter Variable, Inter, Arial, ui-sans-serif, system-ui, sans-serif"
    fontSize: "16px"
    fontWeight: 400
    lineHeight: 1.5
  label:
    fontFamily: "Inter Variable, Inter, Arial, ui-sans-serif, system-ui, sans-serif"
    fontSize: "14px"
    fontWeight: 400
rounded:
  control: "32px"
  media: "22px"
  small: "8px"
spacing:
  xs: "8px"
  sm: "12px"
  md: "16px"
  lg: "24px"
  xl: "32px"
  section: "40px"
  gutter: "56px"
components:
  button-primary:
    backgroundColor: "{colors.action}"
    textColor: "{colors.canvas}"
    rounded: "{rounded.control}"
    padding: "12px 24px"
    height: "48px"
  button-primary-hover:
    backgroundColor: "{colors.action-hover}"
    textColor: "{colors.canvas}"
    rounded: "{rounded.control}"
  button-inverted:
    backgroundColor: "{colors.canvas}"
    textColor: "{colors.green}"
    rounded: "{rounded.control}"
    height: "48px"
  source-surface:
    backgroundColor: "{colors.stone}"
    textColor: "{colors.ink}"
    rounded: "{rounded.media}"
  controls-surface:
    backgroundColor: "{colors.green}"
    textColor: "{colors.canvas}"
    rounded: "{rounded.media}"
---

# Design System: Scene / split

## Overview

**Creative North Star: "Editorial video workbench"**

The user's supplied enterprise-editorial reference sets the visual direction. A white canvas, tightly set display type, quiet rules, and rounded media surround a functional video tool. Deep green identifies the settings work area; it is not decorative accent coverage.

The existing product name and controls remain. This is not Cohere branding, and it uses no proprietary Cohere assets. Uploaded footage supplies the imagery; empty states never substitute stock pictures, fake dashboards, trust logos, or commercial claims.

**Key Characteristics:**
- Large normal-weight display typography with a separate workhorse UI face.
- White editorial space, warm stone media surfaces, and deep-green controls.
- Pill primary actions, underlined secondary actions, and thin list rules.
- Standard video controls and a seekable scene timeline.

## Colors

The frontmatter records applied colors. CSS tokens in `web/src/tokens.css` are the implementation source.

### Primary

Near-black actions appear on light surfaces. Their inverted variant is white on the green settings surface. Green represents a complete functional work area rather than a decorative highlight.

### Secondary

Blue is reserved for download links and light-surface focus. Selected scene segments use a pale green surface and green boundary. Errors use red text and a pale error surface.

### Neutral

White is the page ground. Warm stone carries source upload and preview information. Muted text is darker than the reference's lightest slate so small text remains readable. Gray rules separate timeline, table, disclosure, and utility regions.

**The Footage Rule.** Video remains on a true-black ground with no brand tint, duotone, color filter, or decorative overlay that changes the uploaded image.

## Typography

Display uses locally served Space Grotesk Variable. Body, controls, labels, and table data use locally served Inter Variable. These are the reference's documented fallbacks, not renamed proprietary faces. Font licenses ship under `web/public/font-licenses/`.

Desktop display steps down to 76px below 1280px, 64px below 1000px, and 52px below 740px. Mobile headings use a 10ch measure. Upload titles use 36px, reducing to 30px; work-area and section titles sit around 24–26px. Lead text uses 18px, reducing to 16px on mobile. Compact metadata uses 12px. Numeric media data uses tabular numerals.

**The Quiet Weight Rule.** Size and surface contrast establish hierarchy. Most headings are weight 400; controls and filenames use 500. Labels remain sentence case and untracked.

## Layout

The container is capped at 1440px and centered. Desktop gutters are 56px, reducing to 40px, 24px, and 20px at the existing breakpoints. One top bar carries the product identity and a privacy note; there are no invented navigation destinations.

The workbench has a flexible source region and a 360px settings region, with a 24px gap. Settings reduce to 340px and 320px at narrower desktop sizes. Below 740px, regions stack with a 16px gap. Source actions wrap below metadata on mobile.

The clips table retains its rows and columns inside its own horizontally scrollable wrapper. It does not widen the document. Technical explanations remain in a disclosure instead of dominating the task.

## Elevation & Depth

No drop shadows, glow, glass, or generic gradients. Surface changes, whitespace, radius, and thin rules create depth. The colorful image seen during verification is actual generated test footage in the player, not part of the visual identity.

## Shapes

Major source and settings regions use the media radius. Primary buttons use the control radius. Player clipping, scene segments, and utility controls use the small radius. Borders are generally one pixel. The native range input and native video controls retain familiar interaction behavior.

## Components

### Buttons

Primary actions are near-black pills on light surfaces and inverted pills inside green controls. Hover changes the fill; active state moves down one pixel. Disabled controls lower opacity and block interaction. Focus appears immediately with a two-pixel ring: blue on light surfaces, white inside green controls.

Clear video is an underlined, neutral secondary action with a close icon. Trash is a separate icon button and opens explicit deletion confirmation. Never give selection reset the styling or semantics of file deletion.

### Source surface

The empty state offers file upload and drag-and-drop. Loaded state shows the actual source player, original filename, metadata, clear action, and deletion control. Preview failures explain browser codec limits without claiming the server export failed.

### Controls surface

The deep-green region contains sensitivity, export format facts, the quality note, primary split/download action, and real job progress. Secondary text is tinted from the foreground, not neutral gray. Its focus rings are white.

### Scene timeline and clips

Each detected scene is a real seek button. Segment widths reflect source durations; selection uses a green boundary and pale fill. Results use a rule-separated table with a sticky neutral header and a blue underlined download link.

### Workspace state

Fresh page loads start empty. Clear video removes only the client selection and resets preview, playback position, confirmation, and visible error state. Neither refresh nor Clear sends a server DELETE. Stored files and processing remain server-side; trash is the destructive action.

## Do's and Don'ts

### Do:
- Do use the local display/body font pairing and normal-weight hierarchy.
- Do keep source footage visually unchanged.
- Do preserve familiar controls, real processing states, and usable mobile scrolling.
- Do distinguish clearing the workspace from deleting stored files.

### Don't:
- Don't reintroduce neon accents, decorative gradients, heavy shadows, or fake metrics.
- Don't add customer logos, photography, or product claims not supplied by the user.
- Don't use proprietary Cohere identity or font assets.
- Don't hide focus, replace the source preview with a mockup, or widen the mobile page for table columns.
