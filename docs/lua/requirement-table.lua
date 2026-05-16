local function has_class(el, class_name)
  for _, class in ipairs(el.classes) do
    if class == class_name then
      return true
    end
  end

  return false
end

local function metadata_table(attrs)
  local fields = {
    { "ID", attrs.id },
    { "Title", attrs.title },
    { "Type", attrs.type },
    { "Status", attrs.status },
    { "Verification", attrs.verification },
  }

  local lines = {
    ".. list-table::",
    "   :widths: 20 80",
    "",
  }

  for _, field in ipairs(fields) do
    if field[2] and field[2] ~= "" then
      table.insert(lines, "   * - " .. field[1])
      table.insert(lines, "     - " .. field[2])
    end
  end

  table.insert(lines, "")
  table.insert(lines, "..")

  return pandoc.RawBlock("rst", table.concat(lines, "\n"))
end

function Div(el)
  if not has_class(el, "requirement") then
    return el
  end

  local blocks = pandoc.List()

  if el.attributes.id and el.attributes.title then
    blocks:insert(pandoc.Header(
      1,
      { pandoc.Str(el.attributes.id .. ": " .. el.attributes.title) },
      pandoc.Attr(el.attributes.id:lower(), {}, {})
    ))
  end

  blocks:insert(metadata_table(el.attributes))

  return blocks
end

function Header(el)
  local text = pandoc.utils.stringify(el.content)

  el.level = el.level + 1

  return el
end
