local seen = {}

function Header(el)
  if not el.identifier:match("^ra%-%d%d%d%d$") then
    error(string.format(
      "heading missing ra-NNNN anchor: %s (id=%s)",
      pandoc.utils.stringify(el.content),
      el.identifier
    ))
  end

  if seen[el.identifier] then
    error(string.format(
      "duplicate ra-NNNN anchor: %s used by %s and %s",
      el.identifier,
      seen[el.identifier],
      pandoc.utils.stringify(el.content)
    ))
  end

  seen[el.identifier] = pandoc.utils.stringify(el.content)

  return el
end
