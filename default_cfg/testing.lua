local wasm = require("wasm_nvim")

wasm.setup {
  dir = vim.fn.getcwd().."/wasm/",
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


wasm.tests.groups()
