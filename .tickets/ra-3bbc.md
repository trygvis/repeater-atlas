---
id: ra-3bbc
status: open
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
