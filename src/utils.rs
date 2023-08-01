use mlua::prelude::*;
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

pub fn lua_this<'a>(lua: &'a Lua) -> LuaResult<LuaTable>{
    lua_require::<'a, LuaTable>(lua, "wasm_nvim")
}

pub fn lua_vim_api<'a>(lua: &'a Lua)-> LuaResult<LuaTable>{
    lua.globals().get::<_, LuaTable>("vim")?
        .get::<_, LuaTable>("api")
}

pub fn lua_json_encode(lua: &Lua, obj: LuaValue) -> LuaResult<String> {
    let result = lua.globals().get::<_, LuaTable>("vim")?
        .get::<_, LuaTable>("fn")?.get::<_, LuaFunction>("json_encode")?
        .call::<_, LuaString>(obj)?
                    .to_str()?.to_string();
    Ok(result)
}

pub fn lua_json_decode<'a>(lua: &'a Lua, obj: LuaString<'a>) -> LuaResult<LuaValue<'a>> {
    lua.globals().get::<_, LuaTable>("vim")?
            .get::<_, LuaTable>("fn")?.get::<_, LuaFunction>("json_decode")?
            .call::<_, LuaValue>(obj)
}
