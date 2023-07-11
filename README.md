# <u>

# Currently still under construction</u>

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

  ```c
  extern nvim_echo(input: *u32, input_size: u32) -> [2]u32;
  //all api functions being imported from wasm side should have this structure, as it appears in rust, or create an equivalent in other language. The function here is nvim_echo because that is the function we are importing to use.
  
  //input field is the pointer to the json string containing input to nvim_echo on lua side to be consumed.
  //input_size field, is the size of the string.
  
  //The output to any function will always return an array of two elements.
  //first contains a pointer, and second the size of the item in the pointer.
  //these should be a json string what can be evaluated in the same way from the input string using the same format
  ```

  All api functions being imported from wasm side should have this structure, as it appears in rust, or create an equivalent in other language. The function here is nvim_echo because that is the function we are importing to use.

  - input field is the pointer to the json string containing input to nvim_echo on lua side to be consumed.
  - input_size field, is the size of the string.

  The output to any function will always return an array of two elements.

  - first contains a pointer, and second the size of the item in the pointer.
  - these should be a json string what can be evaluated in the same way from the input string using the same format

  

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
      {"type": "error", "loc": {"beg": 5, size 100}}
      ```

  11. The wasm module can then continue executing, doing it's thing on it's end.