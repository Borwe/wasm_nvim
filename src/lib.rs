use mlua::prelude::*;
use anyhow::Result;
use wasmtime::*;
use std::str::FromStr;

mod nvim_interface;
mod utils;
mod wasm_state;

use wasm_state::{WASM_STATE, WasmNvimState, Types, ValueFromWasm, WasmModule};
use nvim_interface::{Functionality, add_functionality_to_module};

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

    Ok(())
}

fn setup_wasms_with_lua(lua: &Lua) -> LuaResult<()> {
    let wasms = {
        WASM_STATE.lock().unwrap().borrow_mut().set_lua(lua);

        //setup the wasm functions to be exported and used from wasm side
        WASM_STATE.lock().unwrap().borrow_mut().linker.func_wrap("host", "set_value",
            |mut caller: Caller<'_, _>, id: u32, loc: i32, size: u32|{
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

            utils::debug(unsafe{
                &*state.get_lua().unwrap()
            }, &format!("ID: {id} VAL: {val_to_add} PTR: {loc}"));

            let mut returns: Vec<Val> = Vec::new();
            let mut vals: Vec<Val> = vec![Val::from(loc), Val::from(size as i32)];
            
            let dealloc = caller.get_export("dealloc").unwrap().into_func().unwrap();
            dealloc.call(&mut state.store, &vals, &mut returns).unwrap();

            state.return_values.insert(id, val_to_add).unwrap();
        }).unwrap();

        WASM_STATE.lock().unwrap().borrow_mut().linker.func_wrap("host", "get_id",
            || WASM_STATE.lock().unwrap().get_mut().get_id()
        ).unwrap();

        WASM_STATE.lock().unwrap().borrow_mut().linker.func_wrap("host", "get_addr",
            |addr: u32| addr).unwrap();

        WASM_STATE.lock().unwrap().borrow_mut().linker.func_wrap("host", "nvim_echo",
            nvim_interface::nvim_echo).unwrap();

        WASM_STATE.lock().unwrap().borrow_mut().linker.func_wrap("host", "nvim_create_augroup",
            nvim_interface::nvim_create_augroup).unwrap();

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
            let instance = state.linker.instantiate(&mut state.store , &module).unwrap();

            //add module to list
            state.wasm_modules.insert(wasm.clone(),
                WasmModule::new(module, instance, wasm).unwrap());
            let wasm_module = state.wasm_modules.get(wasm).unwrap();
            let module = &wasm_module.module;
            let instance = &wasm_module.instance;
            //setup module
            state.linker.module(&mut state.store, wasm, module).expect("linker module link fail");

            let functionality = instance
                .get_typed_func::<(),u32>(&mut state.store, "functionality").unwrap();

            //get functionality exported from module
            let id = functionality.call(&mut state.store, ()).unwrap();


            let module_functionality = state.get_value(id).unwrap();

            utils::debug(lua, &format!("VAL: {}",module_functionality)).unwrap();

            let functionalities: Vec<Functionality> = serde_json::from_str(&module_functionality)
                .expect(&format!("returned values from functionality() of {wasm} not valid"));

            for f in functionalities.iter(){
                add_functionality_to_module(lua, f.clone(), wasm.to_string()).unwrap();
            }

        }

        let wasm_path = std::path::PathBuf::from(wasm);
        let wasm = wasm_path.file_stem().unwrap().to_str().unwrap();

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
