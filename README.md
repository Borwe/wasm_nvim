# <u>Currently still under construction</u>

## Aim:

Write a library to interface between Lua, and wasm, for enabling communication between plugins and the Neovim apis. The library language is Rust, as it is to be dynamically loaded via lua, using the neovim, instead of going via the rpc route which creates a networking bottleneck between two or more different process, this route allows for a single process **(NEOVIM)** , while also a plugin ecosystem that allows any programming language that can compile to wasm.

**NOTE:** Bench marks still to be done to determine if this is faster or slower vs just normal lua plugins as project still not at MVP stage yet.

## READING:

 - [Wasm memory allocation and dealocation](https://radu-matei.com/blog/practical-guide-to-wasm-memory/)

## Theory

We would need an allocation and deallocation function implementation on every wasm module, that the developers of it would need to create for themselves as data between the host **(this rust library)** and the wasm plugin can only currently be shared using i32 or f32, directly, but other objects like buffers, structs, etc, need more than just 32bits therefore we communicate using pointers to memory to the module to access and manipulate the data from the host side, where applicable, and normally this involves json.

## Required by Module

- a function called `functionality` that is exposed/exported so that it can be called from `wasm_nvim` library. The function returns a json defining what functions are exported, and what they take as parameters, also what they return. 

  ### An example:

  ```zig
  export fn functionality() u32 {
      var functions = ArrayList(Functionality).init(aloc);
      _ = functions.append(CreateFunctionality("hi", Type{ .type = "void" }, Type{ .type = "void" })) catch undefined;
      var stringified = ArrayList(u8).init(aloc);
      json.stringify(functions.items, .{}, stringified.writer()) catch undefined;
      var unmanaged = stringified.moveToUnmanaged();
      // get id for setting a value
      const id = get_id();
      const addr = get_addr(&unmanaged.items[0]);
      //set the value to be consumed as a return type of this function
      set_value(id, addr, unmanaged.items.len);
      return id;
  }
  
  ```

  it returns an id, that maps to a json which was created by the `set_value()` function, which points to a json string that defines the functions and parameters exported by the module to be consumed. In this case the json would like like bellow:

  ```json
  [{"name":"hi", "params": {"type":"void"}, "returns": {"type":"void"}}]
  ```

  



## Exposed Useful Functions from Wasm_Nvim

- ```zig
  extern "host" get_id() u32;
  ```

  Get a unique `id` to be used for sharing data between wasm module and outside world.

- ```zig
  extern "host" get_adr(*u8) u32;
  ```

  Get the address from the host in terms of the memory of the module

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
  extern "host" set_value(id: u32, loc: *u8, size: u32) void;
  ```

  Used for returning/setting a value to the `wasm_nvim` library or to the outside world from the wasm module using it. Users of this method should make sure the relinquish memory control of any thing the pointer is pointing to.


## Types representations

| Normal language type in wasm module side(using zig)          | Neovim data types                |
| ------------------------------------------------------------ | -------------------------------- |
| ```bool```                                                   | Boolean                          |
| ```i32```                                                    | Integer, Buffer, Window, TabPage |
| ```HashMap(ArrayList(u8), ArrayList(u8))``` =>, first value contains field name as the Key. Second value is the Value, contains a string that defines the type; if the type is a function, then it has a name field with the name of the function, and an args field with the types of information of what to expect; if the type is not a function, then it contains a name field, a pointer to where it is located, and it's size. | Dictionary                       |
| ```[_]u8```                                                  | Object, String                   |
| ```ArrayList```                                              | Array                            |
| ```f32```                                                    | Float                            |
| ```HashMap(ArrayList(u8), ArrayList(u8))``` =>The value parameter is a string representation of a function | LuaRef                           |