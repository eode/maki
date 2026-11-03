local parse = require("maki.provider_parse")

local MODELS_PATH = "/v1/models/capabilities"
local ASSUMED_CONTEXT_WINDOW = 4096

local function has_capability(caps, name)
  if type(caps) ~= "table" then
    return false
  end
  for _, cap in ipairs(caps) do
    if cap == name then
      return true
    end
  end
  return false
end

local function parse_model(m)
  if type(m) ~= "table" or type(m.id) ~= "string" then
    return nil
  end
  local capabilities = m.capabilities

  -- Filter for chat-capable models
  if not has_capability(capabilities, "chat") and not has_capability(capabilities, "completion") then
    return nil
  end

  local supports_vision = nil
  if capabilities ~= nil or (type(m.input_modalities) == "table" and #m.input_modalities > 0) then
    local vision = has_capability(capabilities, "vision")
    if type(m.input_modalities) == "table" then
      for _, mod in ipairs(m.input_modalities) do
        if mod == "image" then
          vision = true
          break
        end
      end
    end
    supports_vision = vision
  end

  return {
    id = m.id,
    context_window = parse.as_u32(m.context_size) or ASSUMED_CONTEXT_WINDOW,
    supports_thinking = has_capability(capabilities, "thinking") or nil,
    supports_vision = supports_vision,
    pricing = parse.pricing(0, 0, 0, 0),
  }
end

maki.provider.register({
  slug = "localai",
  display_name = "LocalAI",
  codec = "openai",
  base_url = "http://localhost:8080/v1",
  api_key_env = "LOCALAI_API_KEY",
  default_model = "localai/default",
  family = "generic",
  accepts_arbitrary_models = true,
  context_window = 128000,
  openai = {
    thinking = { dialect = "ollama" },
  },
  list_models = function(ctx)
    local body, err = ctx.get_json(MODELS_PATH)
    if err then
      return nil, err
    end
    return parse.models(body, parse_model)
  end,
  build_body = function(_, body, _model, opts)
    if opts.protocol == "openai-responses" then
      body.return_progress = true
    end
    return body
  end,
})
