# Search Design Notes

**Status:** Design document for the call sign search feature.

## Goals

- Provide quick call sign lookup from the map page.
- Keep the search surface focused on repeaters and organizations.
- Use the call sign registry as the single source of truth.

## Placement

- Add the search field to the map page hero overlay.
- Place it directly after the login button/link in the same overlay.

## Interactions

- Typing in the field opens a large modal with results.
- The modal persists until closed or a result is selected.
- Selecting a result navigates to `/call-sign/{call_sign}`.

## Results

- Each row shows:
  - Call sign value.
  - Type badge (Repeater or Organization).
- Results are sorted by call sign.
- Empty state: "No call signs match".

## Constraints

- Only call signs are searched.
- Results may include both repeaters and organizations.
- The modal must be large enough to scan multiple results quickly.

## Scope

- Only call signs are searchable.
- Search applies to both repeater systems and organizations.

## Data sources

- `call_sign` registry table, which records the call sign value and kind.
- The `/call-sign/{call_sign}` route resolves the kind into the correct page.

## Matching

- Input is trimmed and uppercased.
- Match is case-insensitive.
- Prefix matches are returned first, then exact matches if present.

## Result model

- `call_sign`: normalized call sign value.
- `kind`: `repeater` or `contact`.
- `href`: `/call-sign/{call_sign}`.

## Ordering

- Sort by call sign value ascending.
