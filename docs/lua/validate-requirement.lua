local required_fields = {
  "id",
  "title",
  "type",
  "status",
  "verification",
}

local allowed_type = {
  Functional = true,
  ["Non-Functional"] = true,
  Constraint = true,
  Interface = true,
  Data = true,
  UX = true,
}

local allowed_status = {
  Draft = true,
  Proposed = true,
  Accepted = true,
  Implemented = true,
  Deprecated = true,
}

local requirement_count = 0

-- Prefix errors so Pandoc failures point at requirement validation.
local function fail(message)
  error("requirement validation failed: " .. message, 0)
end

-- Pandoc parses ``.. requirement::`` as a Div with class ``requirement``.
local function has_class(el, class_name)
  for _, class in ipairs(el.classes) do
    if class == class_name then
      return true
    end
  end

  return false
end

local function input_file()
  if PANDOC_STATE and PANDOC_STATE.input_files then
    return PANDOC_STATE.input_files[1]
  end

  return nil
end

local function basename(path)
  if not path then
    return nil
  end

  return path:match("([^/]+)$") or path
end

local function validate_required_fields(attrs)
  for _, field in ipairs(required_fields) do
    if not attrs[field] or attrs[field] == "" then
      fail("missing required field '" .. field .. "'")
    end
  end
end

-- Reject misspelled or unsupported metadata fields.
local function validate_known_fields(attrs)
  local known = {}

  for _, field in ipairs(required_fields) do
    known[field] = true
  end

  for field, _ in pairs(attrs) do
    if not known[field] then
      fail("unexpected field '" .. field .. "'")
    end
  end
end

-- IDs are sequential and filenames must mirror them.
local function validate_id(attrs)
  local number = attrs.id:match("^RA%-(%d+)$")

  if not number then
    fail("id must match RA-NNNN: " .. attrs.id)
  end

  if tonumber(number) < 1000 then
    fail("id must be RA-1000 or higher: " .. attrs.id)
  end

  local file = basename(input_file())
  if file and file:match("^ra%-%d+%.rst$") then
    local expected = attrs.id:lower() .. ".rst"
    if file ~= expected then
      fail("filename '" .. file .. "' does not match id '" .. attrs.id .. "'")
    end
  end
end

-- Keep enumerated fields aligned with docs/requirements.rst.
local function validate_values(attrs)
  if not allowed_type[attrs.type] then
    fail("invalid type '" .. attrs.type .. "'")
  end

  if not allowed_status[attrs.status] then
    fail("invalid status '" .. attrs.status .. "'")
  end
end

function Div(el)
  if not has_class(el, "requirement") then
    return nil
  end

  requirement_count = requirement_count + 1

  validate_required_fields(el.attributes)
  validate_known_fields(el.attributes)
  validate_id(el.attributes)
  validate_values(el.attributes)

  return nil
end

-- Requirement files are expected to contain exactly one requirement object.
function Pandoc(doc)
  if requirement_count == 0 then
    fail("document does not contain a requirement directive")
  end

  if requirement_count > 1 then
    fail("document contains more than one requirement directive")
  end

  return doc
end
