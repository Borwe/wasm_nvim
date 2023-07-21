use lazy_static::lazy_static;
use std::{cell::{RefCell, Ref}, io::Read, collections::HashSet};
use anyhow::Result;
use std::sync::{Mutex, Arc};
use wasmtime::*;
use wasmtime_wasi::WasiCtx;
use mlua::prelude::*;
use std::collections::HashMap;

lazy_static! {
    pub(crate) static ref WASM_STATE: Mutex<RefCell<WasmNvimState>> = Mutex::new(RefCell::new(WasmNvimState::new()));
}

pub(crate) enum Types {
    /// hold when reading values from a buffer to a string
    String, 
}

#[derive(Debug)]
pub(crate) enum ValueFromWasm {
    String(String),
    Nonthing
}

pub(crate) struct WasmModule{
    pub(crate) module: Module,
    pub(crate) instance: Instance,
    pub(crate) location: String
}

impl WasmModule{
    pub(crate) fn new(module: Module, instance: Instance, wasm: &str) -> Result<Self>{
        Ok(WasmModule{
            module,
            instance,
            location: wasm.to_string()
        })
    }
}


pub(crate) struct WasmNvimState{
    pub(crate) wasms: Vec<String>,
    pub(crate) dir: Option<String>,
    pub(crate) debug: bool,
    pub(crate) wasm_engine: Engine,
    pub(crate) linker: Linker<WasiCtx>,
    pub(crate) store: Store<WasiCtx>,
    pub(crate) wasm_modules: HashMap<String, WasmModule>,
    pub(crate) nvim_types: HashSet<String>,
    lua: Option<usize>,
    /// The set values
    pub(crate) return_values: HashMap<u32, String>,
}

impl WasmNvimState {
    pub(crate) fn new()-> Self {
        let wasm_engine = Engine::default();
        let mut linker = Linker::new(&wasm_engine);
        wasmtime_wasi::add_to_linker(&mut linker, |cx|cx)
            .unwrap();
        let wasi = wasmtime_wasi::WasiCtxBuilder::new()
            .inherit_env().unwrap()
            .inherit_stdout()
            .inherit_stdin()
            .inherit_stderr()
            .inherit_stdio().build();


        let mut store = Store::new(&wasm_engine, wasi );

        WasmNvimState{
            wasms: Vec::new(),
            nvim_types: HashSet::new(),
            dir: None,
            debug: false,
            wasm_engine,
            linker,
            store,
            wasm_modules: HashMap::new(),
            return_values: HashMap::new(),
            lua: None
        }
    }

    /// Generate unique ID to be used for returns
    pub(crate) fn get_id(&mut self)-> u32 {
        for i in 0..std::u32::MAX{
            let mut exists = false;
            for k in self.return_values.keys(){
                if i == *k {
                    exists = true;
                    break;
                }
            }
            if exists ==false {
                self.return_values.insert(i, "".to_string());
                return i;
            }
        }
        panic!("Used all available IDs");
    }


    pub(crate) fn get_value(&mut self, id: u32, ty: Types) -> Result<ValueFromWasm> {
        match ty {
            Types::String => {
                let value = match self.return_values.remove(&id) {
                    Some(x) => x,
                    None => panic!("Key: {} has no value associated", id)
                };
                Ok(ValueFromWasm::String(value))
            },
            _ => panic!("Not implemented yet")
        }
    }

    pub(crate) fn get_lua(&self) -> Option<*const Lua>{
        match self.lua {
            Some(addr) => unsafe {
                Some(addr as *const Lua)
            },
            None => None
        }
    }


    pub(crate) fn set_lua(&mut self, lua: &Lua) {
        unsafe {
            let addr = (lua as *const _) as usize;
            self.lua = Some(addr);
        }
    }
}
