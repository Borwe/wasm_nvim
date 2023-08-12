--setup parent dir as path to search for file
vim.opt.runtimepath:append(vim.fn.getcwd())
local wasm = require("wasm_nvim");

wasm.setup {
  dir = vim.fn.getcwd().."/wasm/",
  --debug = true
}

function lua_for_loop()
  local sum = 0
  local start = vim.fn.reltime();
  for i = 0, 999999999 do
    sum= sum+i
  end
  return vim.fn.reltimefloat(vim.fn.reltime(start));
end

function wasm_for_loop()
  local start = vim.fn.reltime();
  wasm.perf.for_loop();
  return vim.fn.reltimefloat(vim.fn.reltime(start));
end


print("Lua For Loop takes: "..lua_for_loop().."s\n");
print("Wasm Time From Nvim For Loop takes: "..wasm_for_loop().."s\n");
