--setup parent dir as path to search for file
vim.opt.runtimepath:append(vim.fn.getcwd())
local wasm = require("wasm_nvim")
wasm.setup {
  dir = vim.fn.getcwd().."/wasm/",
  --debug = true
}

-- call a hi function from a wasm module zig_examp, some cool text
wasm.zig_examp.hi()
print("\n")
