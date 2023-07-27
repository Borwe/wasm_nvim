use serde::{Serialize, Deserialize};
use mlua::prelude::*;
use crate::{utils, wasm_state::{WasmNvimState, WasmModule}};
use wasmtime::*;
use crate::wasm_state::WASM_STATE;

/// Used by nvim_create_augroup
#[derive(Serialize, Deserialize, Clone, Debug)]
struct NvimCreateAugroup{
    name: String,
    clear: bool
}

/// Used by nvim_echo
#[derive(Serialize, Deserialize, Clone, Debug)]
struct NvimEcho{
    chunk: Vec<Vec<String>>,
    history: bool,
    opts: Vec<String>
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Type{
    r#type: String
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Functionality{
    name: String,
    params: Type,
    returns: Type
}

pub(crate) fn add_functionality_to_module(lua: &Lua,
    functionality: Functionality, wasm_file: String)-> LuaResult<()>{
    let wasm_name = WasmModule::get_name_from_str(&wasm_file);
    utils::debug(lua, &format!("WASM IS: {}", wasm_name));
    let func_name = functionality.name.clone();

    let func = move |_: &Lua, _: LuaValue|{
        let state = unsafe {
            &mut *(WASM_STATE.lock().unwrap().get_mut() as *mut WasmNvimState)
        };
        let instance = &state.wasm_modules.get(&wasm_file).unwrap().instance;
        let func = instance.get_typed_func::<(),()>(
            &mut state.store, &func_name)
            .expect(&format!("Function {} not found",&func_name));
        func.call(&mut state.store, ())
            .expect(&format!("error in calling {}",&func_name));
        Ok(())
    };

    utils::debug(lua, &format!("FUNC IS: {}", functionality.name));
    let wasm_nvim = utils::lua_this(lua)?;
    match wasm_nvim.get::<_, LuaTable>(wasm_name.as_str()){
        Ok(table) => {
            table.set::<_, LuaFunction>(functionality.name.as_str(), lua.create_function(func)?)
        },
        Err(_) => {
            let table = lua.create_table()?;
            table.set::<_, LuaFunction>(functionality.name.as_str(), lua.create_function(func)?)?;
            wasm_nvim.set(wasm_name, table)
        }
    }
        
        ;
    Ok(())
}

enum Types {
    Bool, Number, Chunk, Array, Table
}

#[derive(Serialize, Deserialize)]
struct InterOpLocation {
    beg: u32,
    size: u32
}

#[derive(Serialize, Deserialize)]
struct InterOpValue {
    info: String,
    loc: InterOpLocation,
}

pub(crate) fn nvim_echo(id: u32){
    let lua = unsafe{ &*WASM_STATE.lock().unwrap().borrow().get_lua().unwrap()} ;
    let json = WASM_STATE.lock().unwrap().get_mut()
        .get_value(id).unwrap();
    let nvim_echo: NvimEcho = serde_json::from_str(&json).unwrap();

    let echo_fn = utils::lua_vim_api(lua).unwrap().get::<_, LuaFunction>("nvim_echo").unwrap();
    echo_fn.call::<_, ()>((nvim_echo.chunk, nvim_echo.history, nvim_echo.opts)).unwrap();
}

pub(crate) fn nvim_create_augroup(id: u32)-> i64{
    let lua = unsafe{ &*WASM_STATE.lock().unwrap().borrow().get_lua().unwrap()} ;
    let json = WASM_STATE.lock().unwrap().get_mut()
        .get_value(id).unwrap();

    let nvim_create_augroup: NvimCreateAugroup = serde_json::from_str(&json).unwrap();
    let nvim_create_augroup_fn = utils::lua_vim_api(lua)
        .unwrap().get::<_, LuaFunction>("nvim_create_augroup").unwrap();

    let mut name: LuaString = lua.create_string(nvim_create_augroup.name.as_str()).unwrap();
    let mut opts = lua.create_table().unwrap();
    opts.set("clear", nvim_create_augroup.clear).unwrap();

    nvim_create_augroup_fn.call::<_, LuaInteger>((name, opts)).unwrap()
}
