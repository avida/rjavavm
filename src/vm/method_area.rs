use crate::vm::class::{ClassPtr, MethodReference};
use std::collections::HashMap;
use std::rc::Rc;

pub type MethodAreaPtr = Rc<MethodArea>;
pub struct MethodArea {
    pub class_constant_pool_map: HashMap<String, ClassPtr>,
}

impl MethodArea {
    pub fn new() -> MethodAreaPtr {
        Rc::new(MethodArea {
            class_constant_pool_map: HashMap::new(),
        })
    }

    pub fn init(class_name: String, pool: ClassPtr) -> MethodAreaPtr {
        let mut ma = MethodArea::new();
        Rc::get_mut(&mut ma).unwrap().insert(class_name, pool);
        ma
    }

    pub fn insert(&mut self, class_name: String, pool: ClassPtr) {
        self.class_constant_pool_map.insert(class_name, pool);
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
