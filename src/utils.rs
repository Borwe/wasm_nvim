use mlua::prelude::*;
use anyhow::Result;

use crate::get_ref_state;

pub fn generate_error<Return>(error: &str)-> LuaResult<Return> {
    return Err(mlua::Error::RuntimeError(error.into()));
}

pub fn debug(lua: &Lua, data: &str) ->  LuaResult<()>{
    if get_ref_state!().debug == true {
        lua.globals().get::<_, LuaFunction>("print")?
            .call::<_,()>(data)?;
    }
    Ok(())
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
