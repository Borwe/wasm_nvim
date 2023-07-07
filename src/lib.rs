use mlua::prelude::*;

#[mlua::lua_module]
fn wasm_nvim(lua: &'static Lua) -> LuaResult<LuaTable>{
    let exports = lua.create_table()?;
    let api = lua.globals().get::<_, LuaTable>("vim")?
        .get::<_, LuaTable>("api")?;

    let show = lua.create_function(
        move |l: &'static Lua, ()|-> LuaResult<()>{
        let echo = api.get::<_, LuaFunction>("nvim_echo")?;
        let results = l.create_table()?;
        results.push("Yolo");
        let top = l.create_table()?;
        top.push(results);
        echo.call::<_, ()>((top,true, l.create_table()?));
            
        println!("HMMM!");
        Ok(())
    })?;

    exports.set("version", show)?;
    Ok(exports)
}

#[cfg(test)]
mod tests {

}
