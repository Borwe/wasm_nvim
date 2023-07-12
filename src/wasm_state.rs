use lazy_static::lazy_static;
use std::cell::{RefCell, Ref};
use anyhow::Result;
use std::sync::Mutex;
use wasmtime::*;
use wasmtime_wasi::WasiCtx;
use mlua::prelude::*;

lazy_static!{
    pub(crate) static ref WASM_STATE: Box<Mutex<RefCell<WasmNvimState>>> = Box::new(Mutex::new(RefCell::new(WasmNvimState::new())));
}

pub(crate) struct WasmNvimState{
    pub(crate) wasms: Vec<String>,
    pub(crate) dir: Option<String>,
    pub(crate) debug: bool,
    pub(crate) wasm_engine: Engine,
    pub(crate) linker: Linker<WasiCtx>,
    pub(crate) store: Store<WasiCtx>,
    lua: Option<usize>
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
            dir: None,
            debug: false,
            wasm_engine,
            linker,
            store,
            lua: None
        }
    }

    pub(crate) fn get_lua(&self) -> Option<*const Lua>{
        match self.lua {
            Some(addr) => unsafe {
                Some((addr as *const Lua))
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

#[macro_export]
macro_rules! get_mut_state {
    () => {
        crate::wasm_state::WASM_STATE.lock().unwrap().borrow_mut()
    };
}

#[macro_export]
macro_rules! get_ref_state {
    () => {
        crate::wasm_state::WASM_STATE.lock().unwrap().borrow()
    };
}
