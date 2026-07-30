use crate::vm::class::{Class, ClassPtr, MethodReference};
use std::rc::Rc;
use crate::vm::errors::errors::RunTimeError;
use crate::vm::method_area::{MethodArea, MethodAreaPtr};
use crate::vm::thread::thread::Thread;
use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct Runtime {
    method_area: MethodAreaPtr,
    main_thread: Thread,
}

impl Runtime {
    pub fn init() -> Self {
        let mut ma = MethodArea::new();

        Runtime {
            method_area: ma.clone(),
            main_thread: Thread::new(&ma),
        }
    }

    

    pub fn load_class(&mut self, class_path: &str) -> Result<ClassPtr, RunTimeError> {
        self.method_area.lock().unwrap().load_class(class_path)
    }
    fn get_or_load_class(&mut self, class_path: &str) -> Result<ClassPtr, RunTimeError> {
        self.method_area.lock().unwrap().get_or_load_class(class_path)
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

    pub fn load_and_init() -> Option<Self> {
        let mut rt = Self::init();
        return Some(rt);
    }
}
