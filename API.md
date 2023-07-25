# Currently implemented nvim APIs

## nvim_echo
``` zig
extern "host" fn nvim_echo(id: u32) void;
```
  - Importation from module would look like this.
  - It returns nothing and consumes an id that used `set_value()` to point to a json in memory.
  - example of json:
  ```json
  {
    "chunk": [["hello", "errorMsg"]],
    "history": true,
    "opts": []
  }
  ```
