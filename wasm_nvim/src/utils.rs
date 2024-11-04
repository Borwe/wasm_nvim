use mlua::prelude::*;
use crate::wasm_state::WASM_STATE;

pub fn debug(lua: &Lua, data: &str) -> LuaResult<()> {
    if WASM_STATE.lock().unwrap().borrow().debug == true {
        lua.globals().call_function::<()>("print", data)
    }else {
        Ok(())
    }
}

pub fn lua_require<LuaType>(lua: &Lua, pkg: &str)
    -> LuaResult<LuaType> where LuaType: Clone + FromLuaMulti{
    let result = lua.globals().get::<LuaFunction>("require")?
        .call(pkg);
    result
}

pub fn lua_this(lua: & Lua) -> LuaResult<LuaTable>{
    lua_require::<LuaTable>(lua, "wasm_nvim")
}

pub fn lua_vim_api<'a>(lua: &'a Lua)-> LuaResult<LuaTable>{
    lua.globals().get::<LuaTable>("vim")?
        .get::<LuaTable>("api")
}

pub fn lua_json_encode(lua: &Lua, obj: LuaValue) -> LuaResult<String> {
    let result = lua.globals().get::<LuaTable>("vim")?
        .get::<LuaTable>("fn")?.get::<LuaFunction>("json_encode")?
        .call::<LuaString>(obj)?
                    .to_str()?.to_string();
    Ok(result)
}

pub fn lua_json_decode(lua: & Lua, obj: LuaString) -> LuaResult<LuaValue> {
    lua.globals().get::< LuaTable>("vim")?
            .get::<LuaTable>("fn")?.get::<LuaFunction>("json_decode")?
            .call::<LuaValue>(obj)
}
