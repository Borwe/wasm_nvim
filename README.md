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

  function used to print/echo to the message bar on neovim. It takes 3 values as parameters:

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

