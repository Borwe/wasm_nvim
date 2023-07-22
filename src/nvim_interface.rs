use serde::{Serialize, Deserialize};
use mlua::prelude::*;
use crate::{utils, wasm_state::{WasmNvimState, WasmModule}};
use wasmtime::*;
use crate::wasm_state::WASM_STATE;

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

pub(crate) fn nvim_echo(id: u32, lock: u32, size: u32){
}
