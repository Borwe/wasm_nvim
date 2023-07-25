local wasm = require("wasm_nvim")
wasm.setup {
  dir = vim.fn.getcwd().."/wasm/",
  debug = true
}

--show types current supported from neovim version
-- wasm.print_nvim_types()

-- call a hi function from a wasm module zig_examp, some cool text
wasm.zig_examp.hi()
