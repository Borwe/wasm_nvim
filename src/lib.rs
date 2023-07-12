use mlua::prelude::*;
use anyhow::Result;
use std::str::FromStr;

mod nvim_interface;
mod utils;
mod wasm_state;

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

fn parse_wasm_dir(lua: &Lua, settings: &LuaTable)-> LuaResult<()>{
    
    match settings.get::<_, bool>("debug") {
        Ok(x) => get_mut_state!().debug = x,
        Err(_) => get_mut_state!().debug = false
    };

    if let Ok(d) =  settings.get::<_, LuaString>("dir") {
        get_mut_state!().dir = Some(d.to_str()?.into());
    }else{
        return utils::generate_error("No dir path given in settings on setup call");
    }

    let path = std::path::PathBuf::from_str(get_ref_state!().dir.as_ref().unwrap().as_str()).unwrap();

    if !path.exists() || !path.is_dir() {
        return utils::generate_error("Path passed as dir option not a real directory or doesn't exist");
    }

    std::fs::read_dir(&path)?.into_iter().for_each(|p|{
        let p = p.unwrap();
        if p.path().extension().unwrap() == "wasm" {
            get_mut_state!().wasms.push(p.path().to_str().unwrap().to_string())
        }
    });

    Ok(())
}

fn setup_nvim_apis(lua: &Lua) -> LuaResult<()>{
    get_mut_state!().linker.func_wrap("","nvim_echo",
      |ctx: wasmtime::Caller<'_, _>, beg: u32, end: u32|{
          //utils::debug(lua, "WOOOOOOOOOOOOOOOT!").unwrap();
    });
    Ok(())
}

fn setup_wasms_with_lua(lua: &Lua) -> LuaResult<()> {
    let wasms = {
        get_mut_state!().set_lua(lua);
        get_ref_state!().wasms.clone()
    };
    wasms.iter().for_each(|wasm|{
        let lua = unsafe{
            let ptr = get_mut_state!().get_lua().unwrap();
            &(*ptr)
        };

        let wasm_path = std::path::PathBuf::from(wasm);
        let wasm = wasm_path.file_stem().unwrap().to_str().unwrap();

        let wasm_plug = lua.create_table().unwrap();


        let test_func = lua.create_function(
            |lua: &Lua, _: LuaValue|{
            utils::debug(lua, "LUA TEST!!!!!")?;
            Ok(())
        }).unwrap();
        //manually add hi function
        wasm_plug.set::<_, LuaFunction>("hi", test_func);

        //add wasm_plug
        lua.globals().get::<_, LuaFunction>("require").unwrap()
            .call::<_, LuaTable>("wasm_nvim").unwrap()
            .set::<_, _>(wasm.clone().to_lua(lua).unwrap(), wasm_plug).unwrap();



        utils::debug(lua, &format!("Loaded: {}",wasm));
    });
    Ok(())
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

    parse_wasm_dir(lua, &settings)?;
    setup_nvim_apis(lua)?;
    setup_wasms_with_lua(lua)?;

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
