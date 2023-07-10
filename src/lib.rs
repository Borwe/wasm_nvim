use mlua::prelude::*;
use anyhow::Result;
use lazy_static::lazy_static;
use std::cell::RefCell;
use std::sync::Mutex;

mod nvim_interface;

lazy_static!{
    static ref WASM_STATE: Box<Mutex<RefCell<WasmNvimState>>> = Box::new(Mutex::new(RefCell::new(WasmNvimState::new())));
}

struct WasmNvimState {
    wasms: Vec<String>,
    dir: Option<String>,
    debug: bool
}


impl WasmNvimState {
    pub(crate) fn new()-> Self {
        WasmNvimState{
            wasms: Vec::new(),
            dir: None,
            debug: false
        }
    }
}

fn get_api_minor_version(lua: &Lua)-> LuaResult<()>{
    let print = lua.globals().get::<_, LuaFunction>("print")?;
    let vim = lua.globals().get::<_, LuaTable>("vim")?;
    let apis = vim.get::<_, LuaTable>("fn")?
        .get::<_, LuaFunction>("api_info")?
        .call::<(),LuaTable>(())?;

    let inspect: LuaTable = vim.get("inspect")?;
    print.call::<_, ()>("HEHE")?;
    let apis_to_print = inspect.call::<_, LuaString>(vim.clone())?;

    print.call::<_, ()>(apis_to_print)?;
    Ok(())
}

fn get_wasm_dir(settings: &LuaTable)-> LuaResult<String>{
    
    match settings.get::<_, bool>("debug") {
        Ok(x) => WASM_STATE.lock().unwrap().get_mut().debug = x,
        Err(_) => WASM_STATE.lock().unwrap().get_mut().debug = false
    };

    if let Ok(d) =  settings.get::<_, LuaString>("dir") {
        WASM_STATE.lock().unwrap().get_mut().dir = Some(d.to_str()?.into());
    }else{
        return Result::Err(mlua::Error::RuntimeError("No dir path given in settings on setup call".into()));
    }

    Ok("".to_string())
}

fn setup(lua: &'static Lua, settings: LuaTable)-> LuaResult<()>{
    let api = lua.globals().get::<_, LuaTable>("vim")?
        .get::<_, LuaTable>("api")?;

    let echo = api.get::<_, LuaFunction>("nvim_echo")?;

    let mut params = LuaMultiValue::new();
    params.push_front(Vec::<String>::with_capacity(0).to_lua(lua)?);
    params.push_front(true.to_lua(lua)?);
    params.push_front(vec![vec!["YEAH BABY!!"]].to_lua(lua)?);

    echo.call::<_, ()>(params)?;

    get_wasm_dir(&settings)?;

    Ok(())
}

#[mlua::lua_module]
fn wasm_nvim(lua: &'static Lua) -> LuaResult<LuaTable>{
    let exports = lua.create_table()?;

    exports.set("setup", lua.create_function(setup)?)?;
    Ok(exports)
}

#[cfg(test)]
mod tests {

}
