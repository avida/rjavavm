use std::collections::HashMap;

use crate::loader::java_class::java_class::ConstantPoolInfoTable;

pub struct MethodArea {
    pub class_constant_pool_map: HashMap<String, ConstantPoolInfoTable>,
}

impl MethodArea {
    pub fn new() -> Self {
        MethodArea {
            class_constant_pool_map: HashMap::new(),
        }
    }

    pub fn init(class_name: String, pool: ConstantPoolInfoTable) -> Self {
        let mut ma = MethodArea::new();
        ma.insert(class_name, pool);
        ma
    }

    pub fn insert(&mut self, class_name: String, pool: ConstantPoolInfoTable) {
        self.class_constant_pool_map.insert(class_name, pool);
    }

    pub fn get(&self, class_name: &str) -> Option<&ConstantPoolInfoTable> {
        self.class_constant_pool_map.get(class_name)
    }

    pub fn remove(&mut self, class_name: &str) -> Option<ConstantPoolInfoTable> {
        self.class_constant_pool_map.remove(class_name)
    }

    pub fn contains(&self, class_name: &str) -> bool {
        self.class_constant_pool_map.contains_key(class_name)
    }

    pub fn len(&self) -> usize {
        self.class_constant_pool_map.len()
    }
}

impl Default for MethodArea {
    fn default() -> Self {
        MethodArea::new()
    }
}