# TUTORIALS:

## Information:

- **Wasm** modules can currently only  call`vim.api.*` functions. The full set of functions on your neovim instance can be access by calling `:lua print(vim.fn.json_encode(vim.fn.api_info().functions))`. The repo contains the output from neovim 0.9.1 on the file, [here](./api_info_v0.9.1.json) in the functions field.

- When passing data to **wasm** module function from lua, you can only pass a single table. Table in lua can be arrays or similar to json with fields that have key and value pair, inside the wasm module, they are consumed as JSON strings. Hence currently you can not add lua functions to the table because there is no way of representing/serializing/deserializing a lua function.

- When calling lua functions from wasm modules, you need to pass variables as an array that can be serialized into json, and then turned each field in the array into a lua value to be passed into the lua function. eg:

  ```json
  ["hello",12, "cool"]
  ```

  Once passed, this is treated as three parameters, `"hello"` being first, `12` second, and `"cool"` third parameter.

- All wasm modules must be put in a folder `./wasm` that is inside the neovim runtime path for searching for plugins.

- Advice, if `./wasm` file has more than 1 module, please make sure the modules start with the namespace of the the plugin:

  - ./test.wasm
  - ./test_abc.wasm
  - ./test_cdb.wasm

  This to avoid name clashing, assuming the whole plugin is called `test` .

- All neovim functions and core wasm_nvim functions for communicating between module and wasm_nvim are exposed to wasm modules are under the "host" tag, an example in zig of defining a lua function to be imported, in a zig wasm module.

  ```zig
  extern "host" nvim_echo(id: u32) void;
  ```

  In Rust.

  ```rust
  #[link(wasm_import_module = "host")]
  extern "C" {
      fn nvim_echo(id: u32);
  }
  ```

  

- Passing data to and out of a module function requires storing the data in an unique id to be consumed by the outside world, or to be consumed from inside the function given the id.


## Important For Wasm Module

- Must contain a `alloc` function being exported, that takes in a 32bit unsigned integer, that is the size of the memory to allocate in terms of bytes, and returns a pointer to the location in memory where the allocation was done to be consumed by wasm_nvim library. eg in zig:

  ```zig
  //we use this allocator, works best on zig.
  var aloc: std.mem.Allocator = std.heap.page_allocator; 
  
  export fn alloc(size: u32) u32 {
      var buf = aloc.alloc(u8, size) catch undefined;
      return get_addr(&buf[0]);
  }
  ```

- Must contain a `dealloc` function being expoted, that takes in a pointer to unsigned 8bit integer, or unsigned char, and a size of the memory space to be deallocated, it returns nothing. This function is used to dealloc memory inside module, from wasm_nvim. An example in zig:

  ```zig
  export fn dealloc(arr: [*]u8, size: u32) void {
      aloc.free(arr[0..size]);
  }
  ```

- Must contain a `functionality` function, that returns an unsigned 32  bit integer representing the id mapping to the json of value stored in `wasm_nvim` that can be used to get the exported functions, and if they consume or return anything from or to the outside world respectively.

  The Json string should be an array or objects containing the following info

  ```json
  {"name":"hi", "params": "void", "returns": "void"}
  ```

  - **name** -> Is the Name of the function be exported.
  - **params** -> can either be `"void"` if void means that the function takes no parameters. If `u32`  means the function takes an id, of a value stored in wasm_nvim that can be retrieved via `get_value_addr` an external function, from "host" namespace module, that can be imported.
  - **returns** -> Behaves in same way as the **params** key, except this determines if the function returns void or something to the outside world.

  An example of a functionality function in a zig wasm module.

  ```zig
  const Functionality = struct {
      name: []const u8, //hold name of function
      params: []const u8, //hold params types, by order
      returns: []const u8,
  };
  
  fn CreateFunctionality(comptime name: []const u8, comptime params: []const u8, comptime returns: []const u8) Functionality {
      return .{ .name = name, .params = params, .returns = returns };
  }
  
  export fn functionality() u32 {
      var funcs = ArrayList(Functionality).init(aloc);
      defer funcs.deinit();
      funcs.append(CreateFunctionality("do_something", "void", "void")) catch unreachable;
  
      var jsoned = ArrayList(u8).init(aloc);
      std.json.stringify(funcs.items, .{}, jsoned.writer()) catch undefined;
      const id = get_id(); //get id from wasm_nvim
      const addr = get_addr(&jsoned.items[0]); //get address of json to return
      set_value(id, addr, jsoned.items.len); //send the value to wasm_nvim
      return id; //return the id representing the value
  }
  ```

  This example also shows how you return values to the outside world. When function called from lua, the json string would be deserialized into a lua dictionary, and the caller from lua world can then grab it's value.

- How to consume value from the outside world in wasm function:
  A zig example

  ```zig
  export fn consuming(id: u32) void {
      const writer = std.io.getStdOut().writer(); // used for printing to stdout
      const size_in = get_value_size(id);
      const addr_items = get_value_addr(id);
      writer.print("Starting AREA {s}\n", .{addr_items[0..size_in]}) catch unreachable;
  }
  
  ```

  - First the function needs to have a parameter accepting an unsigned 32 bit integer.
  - User should call external host function `get_value_size` to get the size of the jsoned/stringified value from outside world.
  - Then call `get_value_addr` passing in the same param value, to get the pointer to where the value is stored.
  - From there user can choose to do whatever they want with the data, In this case we just print it out to stdout.

- If you are stack, just look at [tests.zig](/wasm/tests.zig) for a module showing examples of what a module can do, when it comes to accessing neovim api, or communicating with outside world.



## Calling WASM function from Lua

- First need to make sure the wasm module you are trying to access is on a `wasm` folder inside a runtimepath.

- Also make sure the .dll or .so wasm_nvim.dll is in a `lua` folder on one of your vim runtime paths.

  ```lua
  local wasm = require("wasm_nvim")
  -- NOTE: this following line should best be done once, best put inside your init.lua
  wasm.setup {
    debug = true --[[ to show debug info, can be left blank or set to false 
      to not see debug info from wasm_nvim]]
  }
  
  -- once done with setup now you can call functions being exported from wasm modules.
  
  --[[
  module_name is the file name of the .wasm file, eg: test.wasm means the module being used is the test module.
  do_something is the function
  ]]
  wasm.module_name.do_something()
  
  --[[
  Here we are calling the consume function exported from module_name wasm module.
  Note as stated, wasm functions can only accept a single dictionary or array as param, passing files to
  ]]
  wasm.module_name.consume {
      "param1", "param2", "param3"
  }
  
  --[[
  Wasm functions can also return values to lua space
  ]]
  local something = wasm.module_name.return_something()
  
  ```

  

- If stack one should checkout [testing.lua](./default_cfg/testing.lua) for example of interacting between zig wasm modules and lua world.