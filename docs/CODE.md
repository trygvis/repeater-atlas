# Best practices and example files

## Rust files

These files are examples of code structures to follow:

- `src/web/index.rs`
  - The typed path, resulting template and actual handler are grouped in the
    file
- `src/dao/repeater.rs` - data access module
  - Notice naming pattern, get_XXX returns the entity, find_XXX returns the
    optional entity

# Markdown

- Always have a blank line after section of all levels.
- Indent lists with two spaces.
- Always leave a blank line at the end of the file
