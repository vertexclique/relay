use lever::{prelude::*, sync::atomics::AtomicBox};
use wasmer::{ImportObject, Instance, Module, Store, imports};
use wasmer_compiler_cranelift::Cranelift;
use wasmer_engine_universal::Universal;
use std::{collections::HashMap, io::Read as _, sync::Arc};
use crate::errors::*;
use tracing::*;


pub struct JITEngine {
    store: Store,
    mods_ins: Arc<ReentrantRwLock<HashMap<String, (Module, Instance)>>>,
    importobj: ImportObject
}

impl JITEngine {
    pub fn new() -> Self {
        // Use Cranelift compiler with the default settings
        let compiler = Cranelift::default();

        // Create the global store
        let store =
            Store::new(&Universal::new(compiler).engine());

        // Create an empty import object.
        let importobj = imports! {};

        let mcins = HashMap::<String, (Module, Instance)>::with_capacity(1 << 8);
        let mods_ins =
            Arc::new(ReentrantRwLock::new(mcins));

        Self {
            store,
            mods_ins,
            importobj
        }
    }

    pub fn load_module<T>(&self, module_name: T, module_buf: Vec<u8>) -> Result<()>
    where
        T: AsRef<str>
    {
        let module = Module::new(&self.store, &module_buf[..])?;
        let instance = Instance::new(&module, &self.importobj)?;

        self.mods_ins
            .clone()
            .try_write()
            .map(|mut wrt| {
                wrt.insert(module_name.as_ref().to_string(), (module, instance))
            })
            .ok_or_else(|| RelayError::Processor("mod installation failed.".into()))?;

        Ok(())
    }
}

/*
use wasmer::{imports, Instance, Module, Store};
use wasmer_compiler_singlepass::Singlepass;
use wasmer_engine_jit::JIT;

use std::io::Read as _;
let store = Store::new(&JIT::new(&Singlepass::default()).engine());
let mut buf = vec![];
std::fs::File::open(module_path)?.read_to_end(&mut buf)?;
let module = Module::new(&self.store, &buf[..])?;

let import_object = imports! {};
let instance = Instance::new(&self.modules[module_path], &import_object)?;

let func = instance.exports.get_function("test")?;
let result = func.call(&[])?;
let result = unsafe { std::mem::transmute(result) };
*/