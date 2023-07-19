# <u>Currently still under construction</u>

## Aim:

Write a library to interface between Lua, and wasm, for enabling communication between plugins and the Neovim apis. The library language is Rust, as it is to be dynamically loaded via lua, using the neovim, instead of going via the rpc route which creates a networking bottleneck between two or more different process, this route allows for a single process **(NEOVIM)** , while also a plugin ecosystem that allows any programming language that can compile to wasm.

**NOTE:** Bench marks still to be done to determine if this is faster or slower vs just normal lua plugins as project still not at MVP stage yet.

## READING:

 - [Wasm memory allocation and dealocation](https://radu-matei.com/blog/practical-guide-to-wasm-memory/)

## Theory

We would need an allocation and deallocation function implementation on every wasm module, that the developers of it would need to create for themselves as data between the host **(this rust library)** and the wasm plugin can only currently be shared using i32 or f32, directly, but other objects like buffers, structs, etc, need more than just 32bits therefore we communicate using pointers to memory to the module to access and manipulate the data from the host side.

### Example:

- ### nvim_echo

  function used to print/echo to the message bar on neovim. It takes 3 values as parameters and returns void:

  - A Chunk with text as main value, which can be represented as an Array of an Array containing 2 strings.

    1st string is the words to print, and second is the `hl-group`, second string is optional

    ```rust
    vec![vec!["HELLO WORLD!!"]]; // A chunk with value of "hello world"
    vec![vec!["HELLO WORLD!!", "ErrorMsg"]]; // A chunk with value of "hello world" and hl-group of hl-ErrorMsg
    ```

  - A bool value, true to mean that it is to be stick message history.

    ```rust
    true //stick on message
    false // disapear forever
    ```

  - A lua table with key of optional `verbose` (table can be empty, aka an empty array), with values. But only 1st value really matters, which is the key of "verbose", marking the message as to be redirected to `log_file` and not to messages-history depending on the value.

  So, we need an api to allow this lua method to be called from wasm, and be handled by the library, as it does the lua interaction so:

  ```
  wasm = lib(rust) = lua
  ```

  **Strategy 1:**

  Use function in the following format in wasm side:

  ```zig
  extern "host" nvim_echo(id: u32, input: *u8, input_size: u32);
  //all api functions being imported from wasm side should have this structure, as it appears in zig, or create an equivalent in other language. The function here is nvim_echo because that is the function we are importing to use from host/neovim side.
  
  //id is to be used for registering/identifying input sent between wasm, and neovim(this library), which if the function returns, the user can get the value of returned from it
  //by using a get_value function extern function, that should take in an id, a pointer, and size of the value where it is located on wasm side
  
  //input field is the pointer to the json string containing input to nvim_echo on lua side to be consumed.
  //input_size field, is the size of the string.
  
  
  ```

  All api functions being imported from wasm side should have this structure, as it appears in rust, or create an equivalent in other language. The function here is `nvim_echo` because that is the function we are importing to use.

  - id field determines the register/unique id of interaction between wasm and library end, can be used to send/recieve values between the two.

  - input field is the pointer to the json string containing input to nvim_echo on lua side to be consumed.
  - input_size field, is the size of the string.
  - All input's should be assumed to be consumed, and therefore shouldn't be handled directly by wasm modules.
  - All outputs coming from library should be assumed to be unmanaged, memory to it should be handled by the module.
  - All modules need to have a dealloc function that can deallocate memory given a pointer range

  

  **Process:**

  1. Create the objects to be passed to the `nvim_echo` functions from the wasm side.

  2. Get each objects starting location, and length to it's end. storing them in variables.

  3. Then generate a json string with data of the variables with their types. In this `nvim_echo` functions case it would be for example like:

     ```json
     {
         [
         	{"type": "chunk", "loc": { "beg": 1234, "size": 52}},
         	{"type": "bool", "loc": {"beg": 1335, "size": 1}},
         	{"type": "table", "loc": {"beg": 1337, "size": 1}}
         ]
     }
     ```

     

  4. Get the address of the location to the start of the string, and the length.

  5. Pass it to the nvim_echo field.

  6. On Rust *(library)*  side, we open the memory of the module, go to the location of the pointer, and parse it to a string using it's size passed too.

  7. Then parse the string and retrieve the objects from the `beg` pointers in memory, turning them to a `Lua` consumable objects, this functions case, it would be 3 objects of.

     ```rust
     vec![vec![val1]] // the chunk
     bool //the bool
     LuaTable //the table
     ```

  8. Pass them as a single LuaMultiValue to be consumed by the nvim side.

  9. Since this function returns nothing, generate a string representation of the json:

     ```json
     {"type": "void", "loc": {"beg": 0, "size": 0}}
     ```

  10. If an error occurs, a type of error would be returned with a string contained in the loc range.

      ```json
      {"type": "error", "loc": {"beg": 5, "size": 100}}
      ```

  11. The wasm module can then continue executing, doing it's thing on it's end.



## Exposed Useful Functions from Wasm_Nvim

- ```zig
  extern "host" get_id() u32;
  ```

  Get a unique `id` to be used for sharing data between wasm module and outside world.

- ```zig
  extern "host" get_value_addr(id: u32) u32;
  ```

  Usable for getting location of value that was created from outside world of module. The value pointed here is to be managed by the module, and deallocated.
  **NOTE: value is cleared from memory of outside world once called, make sure to call `get_value_size`, so as to know the length before calling this function**

- ```zig
  extern "host" get_value_size(id: u32) u32;
  ```

  Given an id to a value, it returns the size of the value located at given id. This should be called before `get_value_addr` as that would clear the id from memory.

  

- ```zig
  extern "host" return_value(id: u32, loc: *u8, size: u32) void;
  ```

  Used for returning a value to the `wasm_nvim` library or to the outside world from the wasm module using it. Users of this method should make sure the relinquish control of any thing the pointer is pointing to.


## Types representations

| Normal language type in wasm module side(using zig) | Neovim data types                |
| --------------------------------------------------- | -------------------------------- |
| ```bool```                                          | Boolean                          |
| ```i64```                                           | Integer, Buffer, Window, TabPage |
|                                                     | Dictionary                       |
| ```[_]u8```                                         | Object, String                   |
| ```ArrayList```                                     | Array                            |
| ```f64```                                           | Float                            |
| ```HashMap(ArrayList(u8), )```                      | LuaRef                           |