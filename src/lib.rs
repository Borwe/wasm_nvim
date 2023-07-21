use mlua::prelude::*;
use anyhow::Result;
use wasmtime::*;
use std::str::FromStr;

mod nvim_interface;
mod utils;
mod wasm_state;

use wasm_state::{WASM_STATE, WasmNvimState, Types, ValueFromWasm};

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
        Ok(x) => WASM_STATE.lock().unwrap().get_mut().debug = x,
        Err(_) => WASM_STATE.lock().unwrap().get_mut().debug = false
    };

    if let Ok(d) =  settings.get::<_, LuaString>("dir") {
        WASM_STATE.lock().unwrap().get_mut().dir = Some(d.to_str()?.into());
    }else{
        return utils::generate_error("No dir path given in settings on setup call");
    }

    let path = std::path::PathBuf::from_str(WASM_STATE.lock().unwrap().borrow().dir.as_ref().unwrap()).unwrap();

    if !path.exists() || !path.is_dir() {
        return utils::generate_error("Path passed as dir option not a real directory or doesn't exist");
    }

    std::fs::read_dir(&path)?.into_iter().for_each(|p|{
        let p = p.unwrap();
        if p.path().extension().unwrap() == "wasm" {
            WASM_STATE.lock().unwrap().get_mut().wasms.push(p.path().to_str().unwrap().to_string())
        }
    });

    Ok(())
}

fn setup_nvim_apis(lua: &Lua) -> LuaResult<()>{
    use std::collections::HashSet;
    let api_table: LuaTable = lua.globals().get::<_, LuaTable>("vim")?
        .get::<_,LuaTable>("fn")?
        .get::<_, LuaFunction>("api_info")?.call::<_, LuaTable>(())?;
    let apis_json = lua.globals().get::<_, LuaTable>("vim")?
        .get::<_, LuaTable>("json")?
        .call_function::<_,_,LuaString>("encode",api_table)?;

    let api_vals = serde_json::value::Value::from_str(apis_json.to_str().unwrap())
        .expect("Couldn't parse JSON");

    let functions = api_vals.get("functions").unwrap().as_array().unwrap();
    utils::debug(lua, &format!("FUNCS ARE: {}",functions.len()))?;

    for func in functions.iter(){
        for params in func.get("parameters").unwrap().as_array().iter(){
            for params_outer in params.iter() {
                for param_inner in params_outer.as_array().iter() {
                    WASM_STATE.lock().unwrap().get_mut().nvim_types.insert(String::from(param_inner[1].as_str().unwrap()));
                }
            }
        }
    }

    //utils::debug(lua, apis_json.to_str()?)?;
    //WASM_STATE.lock().unwrap().get_mut().linker.func_wrap("","nvim_echo",
    //  |ctx: wasmtime::Caller<'_, _>, beg: u32, end: u32|{
    //      //utils::debug(lua, "WOOOOOOOOOOOOOOOT!").unwrap();
    //});
    Ok(())
}

fn setup_wasms_with_lua(lua: &Lua) -> LuaResult<()> {
    let wasms = {
        WASM_STATE.lock().unwrap().borrow_mut().set_lua(lua);

        WASM_STATE.lock().unwrap().borrow_mut().linker.func_wrap("host", "set_value",
            |mut caller: Caller<'_, _>, id: u32, loc: u32, size: u32|{
            // Avoid locking through out this full function, once here means we are safe.
            let state = unsafe {
                &mut (*(WASM_STATE.lock().unwrap().get_mut() as *mut WasmNvimState))
            };
            let mut mem = caller.get_export("memory").unwrap().into_memory().unwrap();
            let mut ptr = unsafe {
                mem.data_ptr(&state.store).offset(loc as isize) as *const u8
            };
            let mut val_to_add = String::new();
            for _ in 0..size{
                let c = unsafe{
                    let c = *ptr as char;
                    ptr = ptr.offset(1);
                    c
                };
                val_to_add.push(c);
            }

            state.return_values.insert(id, val_to_add).unwrap();
        }).unwrap();


        WASM_STATE.lock().unwrap().borrow_mut().linker.func_wrap("host", "get_id",
            || WASM_STATE.lock().unwrap().get_mut().get_id()
        ).unwrap();

        WASM_STATE.lock().unwrap().borrow().wasms.clone()
    };


    wasms.iter().for_each(|wasm|{
        let lua = unsafe{
            let ptr = WASM_STATE.lock().unwrap().borrow().get_lua().unwrap();
            &(*ptr)
        };


        //get and add module
        {
            let mut state = unsafe {
                &mut (*(WASM_STATE.lock().unwrap().get_mut() as *mut WasmNvimState))
            };
            let module = Module::from_file(&state.wasm_engine,wasm).unwrap();
            state.linker.module(&mut state.store, wasm, &module).expect("linker module link fail");
            let mut instance = state.linker.instantiate(&mut state.store , &module).unwrap();
            let functionality = instance
                .get_typed_func::<(),u32>(&mut state.store, "functionality").unwrap();

            //get functionality exported from module
            let id = functionality.call(&mut state.store, ()).unwrap();


            match state.get_value(id, Types::String).unwrap(){
                ValueFromWasm::String(s) => utils::debug(lua, &format!("VAL: {}",s)).unwrap(),
                _ => panic!("Error loading functionality")
            };

            //add module to list
            state.wasm_modules.push(module);
        }

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

        //add wasm_plug to be accessible from lua
        utils::lua_require::<LuaTable>(lua, "wasm_nvim").unwrap()
            .set::<_, _>(wasm.clone().to_lua(lua).unwrap(), wasm_plug).unwrap();

        utils::debug(lua, &format!("Loaded: {}",wasm));
    });
    Ok(())
}

fn print_nvim_types(lua: &'static Lua, _: LuaValue)-> LuaResult<()>{
    let types = serde_json::to_string(&WASM_STATE.lock().unwrap().borrow().nvim_types)
        .expect("Failed trying to parse nvim_types as json from WASM_STATE");
    utils::debug(lua, &format!("Neovim Types : {}",types))
}

fn setup(lua: &'static Lua, settings: LuaTable)-> LuaResult<()>{

    parse_wasm_dir(lua, &settings)?;
    setup_nvim_apis(lua)?; //also sets the nvim_types up in WASM_STATE
    setup_wasms_with_lua(lua)?;

    Ok(())
}

#[mlua::lua_module]
fn wasm_nvim(lua: &'static Lua) -> LuaResult<LuaTable>{
        
    let exports = lua.create_table()?;

    exports.set("setup", lua.create_function(setup)?)?;
    exports.set("print_nvim_types", lua.create_function(print_nvim_types)?)?;
    Ok(exports)
}

#[cfg(test)]
mod tests {

}
