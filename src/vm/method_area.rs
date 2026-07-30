use crate::vm::class::{ClassPtr, MethodReference};
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
