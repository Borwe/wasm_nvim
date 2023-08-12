--setup parent dir as path to search for file
vim.opt.runtimepath:append(vim.fn.getcwd())
local wasm = require("wasm_nvim")

wasm.setup {
  dir = vim.fn.getcwd().."/wasm/",
  debug = true
}

wasm.tests.consuming {
  "HEHEHE"
}

wasm.tests.consuming("yolo","golo")

local val = wasm.tests.returning();
print("YOLOL!! from wasm: "..val.yoo.."\n")

wasm.tests.nvimEcho {
  {{"COME ON BABY\n"}},
  true,
  {verbose = true}
}

wasm.tests.nvimListBufs()


wasm.tests.groups()
