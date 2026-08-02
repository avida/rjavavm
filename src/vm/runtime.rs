use crate::vm::class::{Class, ClassPtr, MethodReference};
use crate::vm::errors::errors::RunTimeError;
use crate::vm::method_area::{MethodArea, MethodAreaPtr};
use crate::vm::reference_manager::ReferenceManager;
use crate::vm::heap::HeapPtr;
use crate::vm::thread::thread::Thread;
use std::env;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

pub struct Runtime {
    method_area: MethodAreaPtr,
    pub main_thread: Thread,
    pub heap: HeapPtr,
}

impl Runtime {
    pub fn init(trace_ops: bool) -> Self {
        let mut ma = MethodArea::new();
        let rm = ReferenceManager::new_ptr();
        let heap = crate::vm::heap::Heap::new_ptr();

        Runtime {
            method_area: ma.clone(),
            main_thread: Thread::new(&ma, &rm, &heap, trace_ops),
            heap,
        }
    }

    pub fn load_class(&mut self, class_path: &str) -> Result<ClassPtr, RunTimeError> {
        self.method_area
            .lock()
            .map_err(|_| RunTimeError::Other("Method area lock poisoned".to_string()))?
            .load_class(class_path)
    }
    fn get_or_load_class(&mut self, class_path: &str) -> Result<ClassPtr, RunTimeError> {
        self.method_area
            .lock()
            .map_err(|_| RunTimeError::Other("Method area lock poisoned".to_string()))?
            .get_or_load_class(class_path)
    }

    pub fn run(&mut self, class_path: &str) -> Result<(), RunTimeError> {
        if let Ok(class) = self.get_or_load_class(class_path) {
            let (index, method) = class
                .method_by_index
                .iter()
                .find(|(i, m)| m.name == "main".to_string())
                .ok_or(RunTimeError::Other("Main method not found".to_string()))?;

            println!(
                "Running `main` of class {} (simulation) at index {}",
                class_path, index
            );
            let method_ref = MethodReference::new(Rc::clone(method), class.clone());
            self.main_thread.invoke(method_ref)?;
            self.main_thread.run()?;
        } else {
            return Err(RunTimeError::ClassLoadError(format!(
                "Class {class_path} not found"
            )));
        }
        Ok(())
    }

    /// Insert a pre-built class into the runtime's method area under the
    /// specified class name. Useful for tests that create classes programmatically.
    pub fn insert_class(&mut self, class_name: &str, class_ptr: ClassPtr) {
        if let Ok(mut ma) = self.method_area.lock() {
            ma.insert(class_name.to_string(), class_ptr);
        }
    }

    pub fn load_and_init(trace_ops: bool) -> Option<Self> {
        let mut rt = Self::init(trace_ops);
        return Some(rt);
    }
}
