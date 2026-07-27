use crate::loader::class_loader::class_loader::load;
use crate::loader::java_class::java_class::{ConstantPoolPFieldInfo, JavaClassPtr};
use crate::vm::errors::errors::RunTimeError;
use crate::vm::method_area::MethodArea;
use crate::vm::thread::thread::Thread;
use std::env;
use std::path::PathBuf;
use std::rc::Rc;

pub struct Runtime {
    method_area: MethodArea,
    main_thread: Thread,
}

impl Runtime {
    pub fn init(java_class: JavaClassPtr) -> Self {
        let mut ma = MethodArea::new();
        let class = Rc::new(crate::vm::class::Class::init(&java_class));

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

        ma.insert(name, class);
        Runtime {
            method_area: ma,
            main_thread: Thread::new(),
        }
    }

    pub fn run(&mut self, class_name: &str) -> Result<(), RunTimeError> {
        if let Some(class) = self.method_area.get(class_name) {
            let (index, method) = class
                .method_by_index
                .iter()
                .find(|(i, m)| m.name == "main".to_string())
                .ok_or(RunTimeError::Other("Main method not found".to_string()))?;

            println!(
                "Running `main` of class {} (simulation) at index {}",
                class_name, index
            );
            self.main_thread.invoke(class.clone(), *index)?;
            self.main_thread.run()?;
        } else {
            return Err(RunTimeError::Other("Class not found".to_string()));
        }
        Ok(())
    }

    pub fn load_and_init(class_path: &str) -> Option<Self> {
        if let Ok(jc) = load(class_path) {
            Some(Self::init(jc))
        } else {
            None
        }
    }
}