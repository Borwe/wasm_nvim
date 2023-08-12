--setup parent dir as path to search for file
vim.opt.runtimepath:append(vim.fn.getcwd())
local wasm = require("wasm_nvim")

wasm.setup {
  debug = true
}

wasm.tests.consuming {
  "HEHEHE"
}

local val = wasm.tests.returning();
print("YOLOL!! from wasm: "..val.yoo.."\n")

wasm.tests.nvimEcho {
  {{"COME ON BABY\n"}},
  true,
  {verbose = true}
}

wasm.tests.nvimListBufs()


wasm.tests.groups()
