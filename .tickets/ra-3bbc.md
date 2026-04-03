---
id: ra-3bbc
status: closed
deps: []
links: []
created: 2026-01-27T21:26:48Z
type: task
priority: 2
assignee: Trygve Laugstøl
---

# Optional fixes on repeater_system

- Make enabled NOT NULL, no default value
- Make repeater_id NOT NULL, it is required
- Label and note can also be NOT NULL, they can be empty

NOT NULL means that the table column should be NOT NULL and the corresponding
field should not be Option<>.

## Notes

**2026-01-27T21:46:10Z**

Adjusted baseline repeater_service schema to make repeater_id/enabled/label/note
NOT NULL (no new migration). Updated Diesel DAO structs + queries accordingly;
regenerated schema.rs.

**2026-01-27T21:55:47Z**

Follow-up: rx_hz and tx_hz are also required; updated baseline schema + Diesel
types accordingly.
