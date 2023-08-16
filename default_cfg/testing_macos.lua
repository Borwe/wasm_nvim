--
-- for testing the download script for macos
--

--setup parent dir as path to search for file
vim.opt.runtimepath:append(vim.fn.getcwd())
--add testing_lua_module dir for being able to call lua module/script from wasm test
vim.opt.runtimepath:append(vim.fn.getcwd().."/testing_lua_module")
require("wasm_nvim_dl").download("macos")

local wasm = require("wasm_nvim")

wasm.setup {
  --debug = true
}

wasm.tests.luaExecExample();
wasm.tests.luaEvalExample();

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
