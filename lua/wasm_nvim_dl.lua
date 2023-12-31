local uv = require("luv");

local path = debug.getinfo(1).source -- get the source of the current file
path = string.gsub(path, "^@", "") -- remove the "@" prefix
path = string.match(path, "^(.*[/\\])") -- extract the directory part

local M = {}

M.download = function(system)
  local dl = "";
  local dir = os.tmpname()..math.random(100);
  print(dir)
  if system=="windows"  then
    -- fix separater on windows
    path = path:gsub("/","\\")
    os.execute("mkdir "..dir:gsub("/","\\"));
    dl = "https://github.com/Borwe/wasm_nvim/releases/download/v0.0.1/wasm_nvim_windows-latest.zip"
  elseif system == "macos" then
    vim.fn.mkdir(dir,"p",493);
    dl = "https://github.com/Borwe/wasm_nvim/releases/download/v0.0.1/wasm_nvim_macos-latest.zip"
  elseif system == "linux" then
    vim.fn.mkdir(dir,"p",493);
    dl = "https://github.com/Borwe/wasm_nvim/releases/download/v0.0.1/wasm_nvim_ubuntu-latest.zip"
  else
    vim.api.nvim_echo({{"Error, OS, can only be windows, linux, or macos","ErrorMsg"}}, true, {})
    return;
  end


  print("PATH: "..path)


  local curdir = vim.fn.chdir(dir)
  if os.execute("curl -L "..dl.." -o wasm.zip") ~= 0 then
    vim.fn.chdir(curdir)
    vim.api.nvim_echo({{"Error, curl command couldn't execute", "ErrorMsg"}}, true, {})
  end

  if system == "windows" then
    if os.execute("tar -xf "..dir.."/wasm.zip") ~=0 then
      vim.fn.chdir(curdir)
      vim.api.nvim_echo({{"Error, tar not found to unzip", "ErrorMsg"}}, true, {})
    end
      vim.fn.chdir(curdir)
      os.execute("copy "..dir.."\\wasm_nvim.dll "..path)
  else
    if os.execute("unzip "..dir.."/wasm.zip -d "..path) ~=0 then
      vim.fn.chdir(curdir)
      vim.api.nvim_echo({{"Error, tar not found to unzip", "ErrorMsg"}}, true, {})
    end
    vim.fn.chdir(curdir)
  end
end

return M
