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


function lua_primes(size)
  local factos = {}
  local primes = {2,}

  local i = 3;
  while i<= size do
    local is_prime = true
    -- check if is a factor
    for _, val in pairs(factos) do
      if val == i then
        is_prime = false
        break
      end
    end

    if is_prime == true then
      table.insert(primes,i)
      --fill factos
      local j =3
      while j*i < size do
        table.insert(factos,j*i)
        j = j+2
      end
    end
    i=i+2
  end

  -- print("LUA PRIMES: "..#primes)
end

function test_prime()
  print(" \n")
  print("Starting to get number of primes between 0-1000")
  print("Over a period of 5 seconds in both wasm and lua...")
  local times_lua = 0
  local start = vim.fn.reltime();
  while(vim.fn.reltimefloat(vim.fn.reltime(start)) < 5) do
    lua_primes(10000)
    times_lua = times_lua + 1
  end
  print("-LUA has done it "..times_lua.." times")
  local times_wasm = wasm.perf.wasm_primes(10000)
  print("-WASM has done it "..times_wasm.." times\n")
end

test_prime()
