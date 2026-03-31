---
id: ra-d2d1
status: closed
deps: []
links: []
created: 2026-03-31T07:36:36Z
type: feature
priority: 2
assignee: Trygve Laugstøl
---

# Redesign map sidebar: unified toggleable side pane

Replace the two separate map overlays (top-right hero box with login/nav and
bottom-right repeater details box) with a single fixed side pane on the
right-hand side of the map.

The pane contains both areas stacked vertically: auth/nav at the top, repeater
detail below. It is toggled open/closed by a button anchored to the right edge
of the screen.

On desktop: slides in/out from the right edge, overlaying the map. On mobile:
the toggle button makes the pane full-screen, covering the map.

The search modal is unchanged.

docs/pages/map.md must be updated to reflect the new layout before
implementation.

## Repeater details section

The repeater detail section lives inside the side pane, below the nav/auth area.
Its states:

- **Empty:** A short prompt ("Select a repeater on the map") is shown. No close
  button.
- **Populated:** Shown when the user clicks a map marker. Displays call sign
  (linked to the detail page), status, and service summary. A close/dismiss
  button clears the section back to the empty state.
- **Replaced:** Clicking a different marker while details are already shown
  replaces the content in place — no intermediate empty state.

The close button dismisses the details but does not close the side pane itself.

## Acceptance Criteria

- [ ] Hero overlay and repeater details overlay are removed
- [ ] A single side pane on the right replaces both, containing auth/nav and
      repeater detail sections
- [ ] A toggle button on the right edge opens and closes the pane
- [ ] On mobile, opening the pane makes it full-screen
- [ ] Closing the pane restores the full map view
- [ ] Repeater detail section shows empty prompt when nothing is selected
- [ ] Clicking a marker populates the detail section and shows a close button
- [ ] Clicking a second marker replaces the detail content in place
- [ ] Close button on the detail section clears it to empty state without
      closing the pane
- [ ] docs/pages/map.md is updated

## Notes

**2026-03-31T08:32:47Z**

Implemented: replaced hero overlay and repeater details overlay with a single
side pane (#map-side-pane). Toggle button (#pane-toggle) lives inside the pane
at top-right when open (X), and collapses to a hamburger edge tab when closed.
Pane is open by default. On mobile (<=600px) pane goes full-screen. Repeater
details section inside the pane shows empty prompt, populates on marker click,
replaces in place on subsequent clicks. Clicking a marker auto-opens the pane.
docs/pages/map.md updated.
