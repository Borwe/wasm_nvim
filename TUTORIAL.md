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

- All neovim functions exposed to wasm modules are under the "host" tag, an example in zig of defining a lua function to be imported, in a zig wasm module.

  ```zig
  extern "host" nvim_echo(id: u32) void;
  ```

- Passing data to and out of a module function requires storing the data in an unique id to be consumed by the outside world, or to be consumed from inside the function given the id.