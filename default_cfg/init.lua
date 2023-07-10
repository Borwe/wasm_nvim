local wasm = require("wasm_nvim")
wasm.setup {
  -- dir = vim.fn.getcwd().."/wasm/",
  debug = true
}

-- call a function from a wasm module hi, that prints hi
-- to at the bottom of neovim
wasm.zig_examp.hi()
