local M = {}

M.print_hello_return_number = function ()
  local num = math.random(100);
  print("Hello random num returning is: "..num)
  return num
end

M.print_hello_return_nothing = function ()
  print("Hello, returning nothing")
end

return M
