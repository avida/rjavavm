use crate::loader::class_loader::class_loader::load;
use crate::loader::java_class::java_class::{ConstantPoolPFieldInfo, JavaClassPtr};
use crate::vm::method_area::MethodArea;
use std::env;
use std::path::PathBuf;

pub struct Runtime {
    method_area: MethodArea,
}

impl Runtime {
    pub fn init(java_class: JavaClassPtr) -> Self {
        let mut ma = MethodArea::new();
        let class = crate::vm::class::Class::init(&java_class);

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
        Runtime { method_area: ma }
    }

    pub fn run(&self, class_name: &str) {
        if let Some(c) = self.method_area.get(class_name) {
            let has_main = c.methods.iter().any(|m| m.name == "main");
            if has_main {
                println!("Running `main` of class {} (simulation)", class_name);
            } else {
                println!("Class {} has no main method", class_name);
            }
        } else {
            println!("Class {} not found in method area", class_name);
        }
    }

    pub fn load_and_init(class_path: &str) -> Option<Self> {
        if let Ok(jc) = load(class_path) {
            Some(Self::init(jc))
        } else {
            None
        }
    }
}
