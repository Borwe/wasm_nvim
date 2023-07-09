use mlua::prelude::*;
use anyhow::Result;

mod nvim_interface;

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

#[mlua::lua_module]
fn wasm_nvim(lua: &'static Lua) -> LuaResult<LuaTable>{
    let exports = lua.create_table()?;
    let api = lua.globals().get::<_, LuaTable>("vim")?
        .get::<_, LuaTable>("api")?;

    let setup = lua.create_function(
        move |l: &'static Lua, ()|-> LuaResult<()>{
        let echo = api.get::<_, LuaFunction>("nvim_echo")?;

        echo.call::<_, ()>((vec![vec!["Yolo"]],true, l.create_table()?))?;

        let mut params = LuaMultiValue::new();
        params.push_front(Vec::<String>::with_capacity(0).to_lua(l)?);
        params.push_front(true.to_lua(l)?);
        params.push_front(vec![vec!["YEAH BABY!!"]].to_lua(l)?);

        echo.call::<_, ()>(params)?;


        get_api_minor_version(l)?;
            
        println!("HMMM!");
        Ok(())
    })?;

    exports.set("setup", setup)?;
    Ok(exports)
}

#[cfg(test)]
mod tests {

}
