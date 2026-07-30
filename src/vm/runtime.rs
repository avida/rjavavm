use crate::loader::class_loader::class_loader::load;
use crate::loader::java_class::java_class::{ConstantPoolPFieldInfo, JavaClassPtr};
use crate::loader::utils::utils::lookup_class_file;
use crate::vm::class::{Class, ClassPtr};
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

    pub fn init_class(&mut self, java_class: JavaClassPtr) -> ClassPtr {
        let class = Class::init(&java_class);

        // try to resolve the class name from constant pool
        let name = java_class
            .constant_pool
            .get((java_class.this_class as usize).saturating_sub(1))
            .and_then(|entry| match &entry.info {
                ConstantPoolPFieldInfo::ClassInfo { name_index } => java_class
                    .constant_pool
                    .get((*name_index as usize).saturating_sub(1))
                    .and_then(|e| match &e.info {
                        ConstantPoolPFieldInfo::Utf8Info { bytes, .. } => {
                            Some(String::from_utf8_lossy(bytes).to_string())
                        }
                        _ => None,
                    }),
                _ => None,
            })
            .unwrap_or_else(|| "<unknown>".to_string());

        println!(
            "method_area reference count: {}",
            Arc::strong_count(&self.method_area)
        );

        self.method_area.lock().unwrap().insert(name, class.clone());
        class
    }

    pub fn load_class(&mut self, class_path: &str) -> Result<ClassPtr, RunTimeError> {
        // Try to resolve a fully-qualified class name to a .class file first.
        if let Some(path) = lookup_class_file(class_path) {
            load(path.to_str().unwrap())
                .and_then(|jc| Ok(self.init_class(jc)))
                .map_err(|_| {
                    RunTimeError::ClassLoadError(format!(
                        "Failed to load class file for {} at {}",
                        class_path,
                        path.display()
                    ))
                })
        } else {
            // Fall back to trying the provided string as a file path.
            load(class_path)
                .and_then(|jc| Ok(self.init_class(jc)))
                .map_err(|_| {
                    RunTimeError::ClassLoadError(format!(
                        "Failed to load class from path {}",
                        class_path
                    ))
                })
        }
    }
    fn get_or_load_class(&mut self, class_path: &str) -> Result<ClassPtr, RunTimeError> {
        {
            let ma = self.method_area.lock().unwrap();
            if let Some(class) = ma.get(class_path) {
                return Ok(class.clone());
            }
        }
        // Delegate lookup/loading to `load_class`, which first attempts
        // to resolve a fully-qualified class name to a .class file.
        Ok(self.load_class(class_path)?)
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
            self.main_thread.invoke(class.clone(), *index)?;
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
