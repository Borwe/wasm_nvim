use mlua::prelude::*;
use wasmtime::*;
use std::{str::FromStr, path::PathBuf};

mod nvim_interface;
mod utils;
mod wasm_state;

use wasm_state::{WASM_STATE, WasmNvimState, WasmModule};
use nvim_interface::{Functionality, add_functionality_to_module};

fn parse_wasm_dir(lua: &Lua, settings: &LuaTable)-> LuaResult<()>{
    
    //setup debug option
    match settings.get::<_, bool>("debug") {
        Ok(x) => WASM_STATE.lock().unwrap().get_mut().debug = x,
        Err(_) => WASM_STATE.lock().unwrap().get_mut().debug = false
    };

    //get wasm modules
    let runtime_paths = utils::lua_vim_api(lua).unwrap()
        .get::<_, LuaFunction>("nvim_list_runtime_paths")
        .unwrap().call::<_, LuaValue>(()).unwrap();

    let runtime_paths_jsoned: serde_json::Value = serde_json::from_str(
        utils::lua_json_encode(lua, runtime_paths).unwrap()
        .as_str()).unwrap();

    runtime_paths_jsoned.as_array().into_iter().flat_map(|v| v.into_iter() )
        .map(|v| PathBuf::from(v.as_str().unwrap()) ).for_each(|mut p|{
        p.push("wasm");
        if !p.exists() {
            return;
        }
        std::fs::read_dir(&p).unwrap().into_iter().for_each(|p|{
            let p = p.unwrap();
            if p.path().extension().unwrap() == "wasm" {
                WASM_STATE.lock().unwrap().get_mut().wasms
                    .push(p.path().to_str().unwrap().to_string())
            }
        });
    });

    Ok(())
}

fn setup_nvim_apis(lua: &Lua) -> LuaResult<()>{
    let api_table = lua.globals().get::<_, LuaTable>("vim")?
        .get::<_,LuaTable>("fn")?
        .get::<_, LuaFunction>("api_info")?.call::<_, LuaValue>(())?;
    let apis_json = utils::lua_json_encode(lua, api_table)?;

    let api_vals = serde_json::value::Value::from_str(&apis_json)
        .expect("Couldn't parse JSON");

    let functions = api_vals.get("functions").unwrap().as_array().unwrap();
    utils::debug(lua, &format!("FUNCS ARE: {}",functions.len()))?;

    for func in functions {
        let name = func.get("name").unwrap().as_str().unwrap().to_string();
        let params = func.get("parameters").unwrap().as_array().unwrap().len();
        let returns = func.get("return_type").unwrap().as_str().unwrap() != "void";
        if params > 0 && returns == false {
            //testable by testing nvim_echo
            let name_c = name.clone();
            WASM_STATE.lock().unwrap().borrow_mut().linker.func_wrap("host", &name,
                    move |id: u32|{
                let lua = unsafe{ &*WASM_STATE.lock().unwrap().borrow().get_lua().unwrap()};
                let json = serde_json::to_value(&WASM_STATE.lock().unwrap().get_mut()
                    .get_value(id).unwrap()).unwrap();

                let mut func = utils::lua_vim_api(lua).unwrap().get::<_,LuaFunction>(name_c.as_str())
                    .unwrap();


                let lua_str = lua.create_string(json.as_str().unwrap()).unwrap();
                let val_lua = match utils::lua_json_decode(lua, lua_str).unwrap(){
                    LuaValue::Table(x) => x,
                    _ => panic!("value passed to {} must be table",name_c)
                };
                for i in 1..params+1{
                    let v: LuaValue = val_lua.get(i).unwrap();
                    func = func.bind(v).unwrap();
                }

                func.call::<(),()>(()).unwrap();
            }).unwrap();
        }else if params>0 && returns == true {
            //testable by testing nvim_list_bufs
            let name_c = name.clone();
            WASM_STATE.lock().unwrap().borrow_mut().linker.func_wrap("host", &name,
                    move |id: u32| -> u32{
                let lua = unsafe{ &*WASM_STATE.lock().unwrap().borrow().get_lua().unwrap()};
                let json = serde_json::to_value(&WASM_STATE.lock().unwrap().get_mut()
                    .get_value(id).unwrap()).unwrap();

                let mut func = utils::lua_vim_api(lua).unwrap().get::<_,LuaFunction>(name_c.as_str())
                    .unwrap();


                let lua_str = lua.create_string(json.as_str().unwrap()).unwrap();
                let val_lua = match utils::lua_json_decode(lua, lua_str).unwrap(){
                    LuaValue::Table(x) => x,
                    _ => panic!("value passed to {} must be table",name_c)
                };
                for i in 1..params+1{
                    let v: LuaValue = val_lua.get(i).unwrap();
                    func = func.bind(v).unwrap();
                }

                let result = func.call::<(),LuaValue>(()).unwrap();
                let string_result = utils::lua_json_encode(lua, result).unwrap();
                let id = &WASM_STATE.lock().unwrap().get_mut().get_id();
                let _ = &WASM_STATE.lock().unwrap().get_mut().set_value(*id,string_result).unwrap();
                return *id
            }).unwrap();
        }else if params == 0 && returns == true {
            //testable by testing nvim_list_bufs
            let name_c = name.clone();
            WASM_STATE.lock().unwrap().borrow_mut().linker.func_wrap("host", &name, move || -> u32{
                let lua = unsafe{ &*WASM_STATE.lock().unwrap().borrow().get_lua().unwrap()};
                let id = &WASM_STATE.lock().unwrap().get_mut().get_id();

                let result = utils::lua_vim_api(lua).unwrap()
                    .get::<_,LuaFunction>(name_c.as_str()).unwrap()
                    .call::<(),LuaValue>(()).unwrap();

                let string_result = utils::lua_json_encode(lua, result).unwrap();
                let _ = &WASM_STATE.lock().unwrap().get_mut().set_value(*id,string_result).unwrap();
                return *id
            }).unwrap();
        }
    }
    //implement custom functions, that have extra params required than normal
    WASM_STATE.lock().unwrap().borrow_mut().linker.func_wrap("host", "nvim_create_autocmd_wasm",
        nvim_interface::nvim_create_autocmd).unwrap();

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
            let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
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
            }, &format!("ID: {id} VAL: {val_to_add} PTR: {loc}")).unwrap();

            let vals: Vec<Val> = vec![Val::from(loc), Val::from(size as i32)];
            
            let dealloc = caller.get_export("dealloc").unwrap().into_func().unwrap();
            dealloc.call(&mut state.store, &vals, &mut []).unwrap();

            state.return_values.insert(id, val_to_add).unwrap();
        }).unwrap();


        WASM_STATE.lock().unwrap().borrow_mut().linker.func_wrap("host", "lua_exec",
            |id: u32| {
            let lua = unsafe {
                & *WASM_STATE.lock().unwrap().borrow().get_lua().unwrap()
            };
            let val = WASM_STATE.lock().unwrap().borrow_mut().get_value(id).unwrap();
            lua.load(&val).exec().unwrap();
        }).unwrap();

        WASM_STATE.lock().unwrap().borrow_mut().linker.func_wrap("host", "lua_eval",
            |id: u32| {
            let lua = unsafe {
                & *WASM_STATE.lock().unwrap().borrow().get_lua().unwrap()
            };
            let val = WASM_STATE.lock().unwrap().borrow_mut().get_value(id).unwrap();
            let result: LuaValue = lua.load(&val).eval().unwrap();
            let result_str = utils::lua_json_encode(lua,result).unwrap();
            let id = WASM_STATE.lock().unwrap().borrow_mut().get_id();
            println!("NIMEPATA: {result_str} ID: {id}");
            WASM_STATE.lock().unwrap().get_mut().set_value(id, result_str).unwrap();
            return id;
        }).unwrap();

        WASM_STATE.lock().unwrap().borrow_mut().linker.func_wrap("host", "get_id",
            || WASM_STATE.lock().unwrap().get_mut().get_id()
        ).unwrap();


        WASM_STATE.lock().unwrap().borrow_mut().linker.func_wrap("host", "get_addr",
            |addr: u32| addr).unwrap();

        WASM_STATE.lock().unwrap().borrow_mut().linker.func_wrap("host", "get_value_size",
            |_: Caller<'_, _>, id: u32| {
            WASM_STATE.lock().unwrap().borrow()
                .return_values.get(&id).unwrap().len() as u32
        }).unwrap();


        WASM_STATE.lock().unwrap().borrow_mut().linker.func_wrap("host", "get_value_addr",
            |mut caller: Caller<'_, _>, id: u32| {
            let size = WASM_STATE.lock().unwrap().borrow()
                .return_values.get(&id).unwrap().len() as i32;

            let vals: [Val;1] = [Val::from(size)];
            let mut returns = [Val::from(0)];
            let alloc = caller.get_export("alloc").unwrap().into_func().unwrap();
            alloc.call(caller.as_context_mut(), &vals, &mut returns).unwrap();

            unsafe {
                let mut ptr = caller.get_export("memory").unwrap()
                    .into_memory().unwrap().data_ptr(caller.as_context())
                    .offset(returns[0].unwrap_i32() as u32 as isize);

                for c in WASM_STATE.lock().unwrap()
                    .borrow_mut().get_value(id).unwrap().chars().into_iter(){
                    *ptr = c as u8;
                    ptr = ptr.offset(1);
                }
            }
            return returns[0].unwrap_i32() as u32;
        }).unwrap();


        WASM_STATE.lock().unwrap().borrow().wasms.clone()
    };

    wasms.iter().for_each(|wasm|{
        let lua = unsafe{
            let ptr = WASM_STATE.lock().unwrap().borrow().get_lua().unwrap();
            &(*ptr)
        };


        //get and add module
        {
            let state = unsafe {
                &mut (*(WASM_STATE.lock().unwrap().get_mut() as *mut WasmNvimState))
            };


            let module = Module::from_file(&state.wasm_engine,wasm).unwrap();
            let instance = state.linker.instantiate(&mut state.store , &module).unwrap();

            //add module to list
            state.wasm_modules.insert(wasm.clone(),
                WasmModule::new(module, instance).unwrap());
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

        utils::debug(lua, &format!("Loaded: {}",wasm)).unwrap();
    });
    Ok(())
}

fn setup(lua: &'static Lua, settings: LuaTable)-> LuaResult<()>{

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
