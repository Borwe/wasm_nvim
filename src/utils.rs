use mlua::prelude::*;
use anyhow::Result;
use crate::wasm_state::WASM_STATE;

pub fn generate_error<Return>(error: &str)-> LuaResult<Return> {
    return Err(mlua::Error::RuntimeError(error.into()));
}

pub fn debug(lua: &Lua, data: &str) ->  LuaResult<()>{
    if WASM_STATE.lock().unwrap().borrow().debug == true {
        lua.globals().get::<_, LuaFunction>("print")?
            .call::<_,()>(data.to_lua(lua)?)?;
    }
    Ok(())
}

pub fn lua_require<'a,LuaType>(lua: &'a Lua, pkg: &'a str)
    -> LuaResult<LuaType> where LuaType: Clone + FromLuaMulti<'a>{
    let result = lua.globals().get::<_, LuaFunction>("require")?
        .call::<_, LuaType>(pkg);
    result
}

/// Transform normal anyhowResult to LuaResult
pub fn to_lua_result<R>(result: Result<R>, error: Option<&str>) -> LuaResult<R>{
    match result {
        Ok(x) => Ok(x),
        Err(e) =>  match error {
            Some(x) => generate_error(x),
            None => Err(mlua::Error::RuntimeError(e.to_string()))
        }
    }
}
