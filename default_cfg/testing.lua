local wasm = require("wasm_nvim")

wasm.setup {
  dir = vim.fn.getcwd().."/wasm/",
  --debug = true
}

wasm.tests.groups()
