use crate::vm::class::{Class, ClassPtr, MethodReference};
use crate::loader::class_loader::class_loader::load;
use crate::loader::java_class::java_class::{ConstantPoolPFieldInfo, JavaClassPtr};
use crate::loader::utils::utils::lookup_class_file;
use crate::vm::errors::errors::RunTimeError;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

pub type MethodAreaPtr = Arc<Mutex<MethodArea>>;
pub struct MethodArea {
    pub class_constant_pool_map: HashMap<String, ClassPtr>,
    pub resolved_methods: HashMap<String, MethodReference>,
}


impl MethodArea {
    pub fn new() -> MethodAreaPtr {
        Arc::new(Mutex::new(MethodArea {
            class_constant_pool_map: HashMap::new(),
            resolved_methods: HashMap::new(),
        }))
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

        self.insert(name, class.clone());
        class
    }

    pub fn load_class(&mut self, class_path: &str) -> Result<ClassPtr, RunTimeError> {
        if let Some(path) = lookup_class_file(class_path) {
            load(path.to_str().unwrap())
                .map(|jc| self.init_class(jc))
                .map_err(|_| RunTimeError::ClassLoadError(format!(
                    "Failed to load class file for {} at {}",
                    class_path,
                    path.display()
                )))
        } else {
            load(class_path)
                .map(|jc| self.init_class(jc))
                .map_err(|_| RunTimeError::ClassLoadError(format!(
                    "Failed to load class from path {}",
                    class_path
                )))
        }
    }

    pub fn get_or_load_class(&mut self, class_path: &str) -> Result<ClassPtr, RunTimeError> {
        if let Some(class) = self.get(class_path) {
            Ok(class.clone())
        } else {
            Ok(self.load_class(class_path)?)
        }
    }

    pub fn parse_params(desc: &str) -> Vec<String> {
        let mut res: Vec<String> = Vec::new();
        if let Some(start) = desc.find('(') {
            if let Some(end) = desc.find(')') {
                let mut i = start + 1;
                let bytes = desc.as_bytes();
                while i < end {
                    match bytes[i] as char {
                        'B' | 'C' | 'D' | 'F' | 'I' | 'J' | 'S' | 'Z' => {
                            res.push((bytes[i] as char).to_string());
                            i += 1;
                        }
                        'L' => {
                            let mut j = i + 1;
                            while j < end && bytes[j] as char != ';' { j += 1; }
                            let s = &desc[i..=j];
                            res.push(s.to_string());
                            i = j + 1;
                        }
                        '[' => {
                            let mut j = i + 1;
                            while j < end && bytes[j] as char == '[' { j += 1; }
                            if j < end {
                                if bytes[j] as char == 'L' {
                                    let mut k = j + 1;
                                    while k < end && bytes[k] as char != ';' { k += 1; }
                                    let s = &desc[i..=k];
                                    res.push(s.to_string());
                                    i = k + 1;
                                } else {
                                    let s = &desc[i..=j];
                                    res.push(s.to_string());
                                    i = j + 1;
                                }
                            } else { break; }
                        }
                        _ => { i += 1; }
                    }
                }
            }
        }
        res
    }

    pub fn resolve(&mut self, identifier: &str) -> Result<MethodReference, RunTimeError> {
        if let Some(r) = self.resolved_methods.get(identifier) {
            return Ok(r.clone());
        }

        let pos = identifier.rfind('.').ok_or(RunTimeError::Other(format!(
            "Invalid method identifier: {}",
            identifier
        )))?;
        let class_name = &identifier[..pos];

        let class_ptr = if let Some(c) = self.class_constant_pool_map.get(class_name) {
            c.clone()
        } else {
            self.load_class(class_name)?
        };

        self.insert_resolved_for_class(&class_name.to_string(), &class_ptr);

        if let Some(r2) = self.resolved_methods.get(identifier) {
            Ok(r2.clone())
        } else {
            Err(RunTimeError::Other(format!("Method {} not found", identifier)))
        }
    }

    pub fn insert(&mut self, class_name: String, pool: ClassPtr) {
        self.insert_resolved_for_class(&class_name, &pool);
        self.class_constant_pool_map.insert(class_name, pool);
    }

    pub fn get_resolved_method(&self, identifier: &str) -> Option<&MethodReference> {
        self.resolved_methods.get(identifier)
    }

    pub fn insert_resolved_method(&mut self, identifier: String, reference: MethodReference) {
        self.resolved_methods.insert(identifier, reference);
    }

    pub fn insert_resolved_for_class(&mut self, class_name: &String, class_ptr: &ClassPtr) {
        for method in &class_ptr.methods {
            let identifier = format!("{}.{}{}", class_name, method.name, method.descriptor);
            let reference = MethodReference::new(Rc::clone(method), Rc::clone(&class_ptr));
            self.resolved_methods.insert(identifier, reference);
        }
    }

    pub fn get(&self, class_name: &str) -> Option<&ClassPtr> {
        self.class_constant_pool_map.get(class_name)
    }

    pub fn remove(&mut self, class_name: &str) -> Option<ClassPtr> {
        self.class_constant_pool_map.remove(class_name)
    }

    pub fn contains(&self, class_name: &str) -> bool {
        self.class_constant_pool_map.contains_key(class_name)
    }

    pub fn len(&self) -> usize {
        self.class_constant_pool_map.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_params() {
        assert_eq!(MethodArea::parse_params("()V"), Vec::<String>::new());
    }

    #[test]
    fn parse_primitive_and_object() {
        let v = MethodArea::parse_params("(ILjava/lang/String;)V");
        assert_eq!(v, vec!["I".to_string(), "Ljava/lang/String;".to_string()]);
    }

    #[test]
    fn parse_wide_and_array() {
        let v = MethodArea::parse_params("(J[D)I");
        assert_eq!(v, vec!["J".to_string(), "[D".to_string()]);
    }

    #[test]
    fn parse_multi_dim_array() {
        let v = MethodArea::parse_params("([[[I)I");
        assert_eq!(v, vec!["[[[I".to_string()]);
    }

    #[test]
    fn parse_complex_descriptor() {
        let sig = "(I[Ljava/lang/Object;JLjava/util/List;[[I)V";
        let v = MethodArea::parse_params(sig);
        assert_eq!(
            v,
            vec![
                "I".to_string(),
                "[Ljava/lang/Object;".to_string(),
                "J".to_string(),
                "Ljava/util/List;".to_string(),
                "[[I".to_string()
            ]
        );
    }
}
