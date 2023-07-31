use serde::{Serialize, Deserialize};
use mlua::prelude::*;
use crate::{utils, wasm_state::{WasmNvimState, WasmModule}};
use crate::wasm_state::WASM_STATE;
use wasmtime::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
enum VariantNvimType<T1, T2>{
    T1(T1),
    T2(T2)
}

/// Used by nvim_create_autocmd
#[derive(Serialize, Deserialize, Clone, Debug)]
struct NvimCreateAutoCmdOpts{
    group: VariantNvimType<String, i64>,
    pattern: Option<Vec<String>>,
    buffer: Option<i64>,
    desc: Option<String>,
    callback: Option<String>,
    command: Option<String>,
    once: Option<bool>,
    nested: Option<bool>
}

/// Used by nvim_create_autocmd
#[derive(Serialize, Deserialize, Clone, Debug)]
struct NvimCreateAutoCmd{
    module_from: String,
    events: Vec<String>,
    opts: NvimCreateAutoCmdOpts
}

/// Used by the callback passed to nvim_create_autocmd in opts field
#[derive(Serialize, Deserialize, Clone, Debug)]
struct NvimCreateAutoCmdCallBackArgs{
    id: i64,
    event: String,
    group: Option<i64>,
    r#match: String,
    buf: i64,
    file: String,
    data: String
}

impl NvimCreateAutoCmd {
    pub(crate) fn validate(&self)->LuaResult<()>{
        if self.opts.command.is_some() && self.opts.callback.is_some() {
            return Err(LuaError::RuntimeError(
                    "command and callback in opts cannot be all provided".to_string()));
        }
        if self.opts.buffer.is_some() && self.opts.pattern.is_some() {
            return Err(LuaError::RuntimeError(
                    "buffer and pattern cannot be both set in opts provided.".to_string()));
        }
        Ok(())
    }

    /// Returns true if using lua callback in callback field
    /// otherwise returns false to signify it is a vimscript callback
    /// therefore a string
    pub(crate) fn is_wasm_callback(&self)-> bool {
        if let Some(x) = &self.opts.callback {
            if x.starts_with("wasm_func") {
                return true
            }
        }
        false
    }

    /// Generate parameters to be passed to to the lua function
    pub(crate) fn get_param(self, lua: &Lua) -> LuaResult<(Vec<String>, LuaTable)>{
        //get callback
        let callback = if !self.is_wasm_callback() {
            //meaning the callback is a string to a vimscript function
            VariantNvimType::T2(self.opts.callback.as_ref().unwrap())
        }else{
            let mut func_name = self.opts.callback.clone().unwrap()[10..].to_string();
            let func_name = func_name.trim().to_string();
            //meaning we get the function from the wasm file.
            let func = lua.create_function(move |lua: &Lua, table: LuaValue| {
                //func takes an id that points to the value representation of
                //parameters to this top function
                let wasm_func = utils::lua_this(lua)?
                    .get::<_, LuaTable>(self.module_from.as_str())?
                    .get::<_, LuaFunction>(func_name.as_str())?;


                let json_to_send = utils::lua_json_encode(lua, table)?;

                //set value
                let id = WASM_STATE.lock().unwrap().borrow_mut().get_id();
                WASM_STATE.lock().unwrap().borrow_mut().set_value(id, json_to_send).unwrap();

                //call the function passing the id
                wasm_func.call::<_,bool>(id)
            })?;
            VariantNvimType::T1(func)
        };

        let opts_table = lua.create_table()?;
        Ok((self.events, opts_table))
    }
}

/// Used by nvim_create_augroup
#[derive(Serialize, Deserialize, Clone, Debug)]
struct NvimCreateAugroup{
    name: String,
    clear: bool
}

/// Used by nvim_echo
#[derive(Serialize, Deserialize, Clone, Debug)]
struct NvimEcho{
    chunk: Vec<Vec<String>>,
    history: bool,
    opts: Vec<String>
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Type{
    r#type: String
}

/// Hold functionanility(functions name, params, and return types) export by modules
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Functionality{
    name: String,
    params: Type,
    returns: Type
}

pub(crate) fn add_functionality_to_module<'a>(lua: &'a Lua,
    functionality: Functionality, wasm_file: String)-> LuaResult<()>{
    let wasm_name = WasmModule::get_name_from_str(&wasm_file);
    utils::debug(lua, &format!("WASM IS: {}", wasm_name))?;
    let func_name = functionality.name.clone();
    let params = functionality.params.r#type.clone();
    let returns = functionality.returns.r#type.clone();

    let func = move |lua: &'a Lua, obj: LuaValue| -> LuaResult<LuaTable>{
        // We can do this to improve speed, since calling
        // lua functions is single threaded on lua side, no need
        // of locking everytime
        let state = unsafe {
            &mut *(WASM_STATE.lock().unwrap().get_mut() as *mut WasmNvimState)
        };

        let vals = utils::lua_json_encode(lua,obj)?;
        let mut id = 0;
        let mut vals_is_null = true;
        if vals!="null" {
            vals_is_null = false;
            id = state.get_id();
            state.set_value(id, vals).unwrap();
        }

        //set vals to be used for params and return type
        //to be consumed by wasm function
        let mut returned = [Val::from(0 as i32)];
        let param = [Val::from(id as i32)];

        //get function to call
        let instance = &state.wasm_modules.get_mut(&wasm_file).unwrap().instance;
        let func = instance.get_func(
            &mut state.store, &func_name)
            .expect(&format!("Function {} not found",&func_name));

        //meaning we don't pass anything to function
        //then call it without any params
        if params=="void" && returns=="void" {
            func.call(&mut state.store, &[], &mut [])
                .expect(&format!("error in calling {}",&func_name));
        }else if params!="void" && returns !="void"{
            func.call(&mut state.store, &param, &mut returned)
                .expect(&format!("error in calling {}",&func_name));
        }else if params!="void" && returns=="void"{
            //only consumes
            func.call(&mut state.store, &param, &mut[])
                .expect(&format!("error in calling {}",&func_name));
        }else if params=="void" && returns !="void"{
            //only returns
            func.call(&mut state.store, &[], &mut returned)
                .expect(&format!("error in calling {}",&func_name));
        }

        //Handle if wasm function has a return value to be consumed by lua
        //side
        if returns!="void"{
            let id = returned[0].unwrap_i32() as u32;
            let val = WASM_STATE.lock().unwrap().get_mut().get_value(id as u32).unwrap();
            let obj = lua.create_string(val.as_bytes())?;
            return Ok(utils::lua_json_decode(lua, obj)?)
        }
        Ok(lua.create_table()?)
    };

    utils::debug(lua, &format!("FUNC IS: {}", functionality.name))?;
    let wasm_nvim = utils::lua_this(lua)?;
    match wasm_nvim.get::<_, LuaTable>(wasm_name.as_str()){
        Ok(table) => {
            table.set::<_, LuaFunction>(functionality.name.as_str(), lua.create_function(func)?)
        },
        Err(_) => {
            let table = lua.create_table()?;
            table.set::<_, LuaFunction>(functionality.name.as_str(), lua.create_function(func)?)?;
            wasm_nvim.set(wasm_name, table)
        }
    }
}

#[derive(Serialize, Deserialize)]
struct InterOpLocation {
    beg: u32,
    size: u32
}

#[derive(Serialize, Deserialize)]
struct InterOpValue {
    info: String,
    loc: InterOpLocation,
}

pub(crate) fn nvim_echo(id: u32){
    let lua = unsafe{ &*WASM_STATE.lock().unwrap().borrow().get_lua().unwrap()} ;
    let json = WASM_STATE.lock().unwrap().get_mut()
        .get_value(id).unwrap();
    let nvim_echo: NvimEcho = serde_json::from_str(&json).unwrap();

    let echo_fn = utils::lua_vim_api(lua).unwrap().get::<_, LuaFunction>("nvim_echo").unwrap();
    echo_fn.call::<_, ()>((nvim_echo.chunk, nvim_echo.history, nvim_echo.opts)).unwrap();
}

pub(crate) fn nvim_create_augroup(id: u32)-> i64{
    let lua = unsafe{ &*WASM_STATE.lock().unwrap().borrow().get_lua().unwrap()} ;
    let json = WASM_STATE.lock().unwrap().get_mut()
        .get_value(id).unwrap();

    let nvim_create_augroup: NvimCreateAugroup = serde_json::from_str(&json).unwrap();
    let nvim_create_augroup_fn = utils::lua_vim_api(lua)
        .unwrap().get::<_, LuaFunction>("nvim_create_augroup").unwrap();

    let name: LuaString = lua.create_string(nvim_create_augroup.name.as_str()).unwrap();
    let opts = lua.create_table().unwrap();
    opts.set("clear", nvim_create_augroup.clear).unwrap();

    nvim_create_augroup_fn.call::<_, LuaInteger>((name, opts)).unwrap()
}

pub(crate) fn nvim_create_autocmd(id: u32) -> i64 {
    let lua = unsafe{ &*WASM_STATE.lock().unwrap().borrow().get_lua().unwrap()} ;
    let aucmd_json: NvimCreateAutoCmd = serde_json::from_str(&WASM_STATE.lock().unwrap().get_mut()
        .get_value(id).unwrap()).unwrap();
    aucmd_json.validate().unwrap();
    let args = aucmd_json.get_param(lua).unwrap();
    utils::lua_vim_api(lua).unwrap().get::<_,LuaFunction>("nvim_create_autocmd")
        .unwrap().call::<_,LuaInteger>(args).unwrap()
}
