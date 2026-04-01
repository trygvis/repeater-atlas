---
id: ra-e1f9
status: closed
deps: []
links: []
created: 2026-04-01T13:32:01Z
type: feature
priority: 2
assignee: Trygve Laugstøl
---

# Add Lucide icons to map sidebar

Wire up Lucide icons for the map sidebar toggle button. Use ES modules with only
the required icons imported (ChevronLeft, ChevronRight). Vendor only the files
needed, not the full icon set.

## Notes

**2026-04-01T13:32:08Z**

Added lucide 1.7.0 to package.json. Justfile assets target copies only the files
needed: replaceElement.js, createElement.js, defaultAttributes.js,
iconsAndAliases.js, icons/chevron-left.js, icons/chevron-right.js, and
shared/src/utils/{hasA11yProp,mergeClasses,toCamelCase,toPascalCase}.js. Map
template uses a type=module script importing ChevronLeft, ChevronRight, and
replaceElement directly — no wildcard import, no lucide.js entry point.
setToggleIcon() calls replaceElement() directly to swap the icon on pane
open/close.
