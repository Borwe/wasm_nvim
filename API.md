# Currently implemented nvim APIs

## 1. nvim_echo
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

## 2. nvim_create_augroup
```zig
extern "host" nvim_create_augroup(id: u32) u64
```
  - Importation from module would look like this.
  - It returns the id of the augroup, and consumes the value related to the id passed using `set_value` that points to a json memory.
  - The json passed would like like:
  ```json
  {
    "name": "group_name",
    "clear": bool
  }
  ```
