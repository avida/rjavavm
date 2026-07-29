use crate::loader::attributes::attributes::{Attribute, parse_attribute_info};
use crate::loader::java_class::java_class::{
    ConstantPoolInfoTable, ConstantPoolPFieldInfo, JavaClass,
};
use crate::vm::AccessFlags;
use crate::vm::class;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Method {
    pub name: String,
    pub access_flags: AccessFlags,
    pub max_stack: u16,
    pub max_locals: u16,
    pub code: Vec<u8>,
    // pub attributes: Vec<Attribute>
}
pub type MethodPtr = Rc<Method>;

pub struct MethodReference {
    method: MethodPtr,
    class: ClassPtr

}
#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub descriptor: String,
    pub access_flags: AccessFlags,
    pub constant_value: Option<String>,
}

pub type FieldPtr = Rc<Field>;

pub type ClassPtr = Rc<Class>;
#[derive(Debug, Clone)]
pub struct Class {
    pub constant_pool: ConstantPoolInfoTable,
    pub methods: Vec<MethodPtr>,
    pub fields: Vec<FieldPtr>,
    pub method_by_index: HashMap<u16, MethodPtr>,
    pub field_by_index: HashMap<u16, FieldPtr>,
}

impl Class {
    pub fn get_method_by_index(&self, index: u16) -> Option<&MethodPtr> {
        self.method_by_index.get(&index)
    }

    pub fn find_method_index_by_name(&self, name: &str) -> Option<u16> {
        self.method_by_index
            .iter()
            .find(|(_, method)| method.name == name)
            .map(|(index, _)| *index)
    }

    pub fn get_field_by_index(&self, index: u16) -> Option<&FieldPtr> {
        self.field_by_index.get(&index)
    }
    
    fn load_methods(class_info: &JavaClass) -> (Vec<MethodPtr>, HashMap<u16, MethodPtr>) {
        let mut methods: Vec<MethodPtr> = Vec::new();
        let mut method_by_index: HashMap<u16, MethodPtr> = HashMap::new();
        for m in &class_info.methods {
            // get name from constant pool
            let name = match &class_info.constant_pool[(m.name_index - 1) as usize].info {
                ConstantPoolPFieldInfo::Utf8Info { length: _, bytes } => {
                    String::from_utf8_lossy(bytes).to_string()
                }
                _ => "<invalid>".to_string(),
            };

            let mut max_stack: u16 = 0;
            let mut max_locals: u16 = 0;
            let mut code: Vec<u8> = Vec::new();

            for attr in &m.attributes {
                if let Attribute::Code {
                    max_stack: ms,
                    max_locals: ml,
                    code: c,
                    ..
                } = attr
                {
                    max_stack = *ms;
                    max_locals = *ml;
                    code = c.clone();
                    break;
                }
            }

            let method = Rc::new(Method {
                name,
                access_flags: AccessFlags::from(m.access_flags),
                max_stack,
                max_locals,
                code,
            });
            println!("insert {}", m.name_index);
            method_by_index.insert(m.name_index, method.clone());

            methods.push(method);
        }

        (methods, method_by_index)
    }
    fn load_fields(class_info: &JavaClass) -> (Vec<FieldPtr>, HashMap<u16, FieldPtr>) {
        let mut fields: Vec<FieldPtr> = Vec::new();
        let mut field_by_index: HashMap<u16, FieldPtr> = HashMap::new();
        for f in &class_info.field_info {
            let name = match &class_info.constant_pool[(f.name_index - 1) as usize].info {
                ConstantPoolPFieldInfo::Utf8Info { length: _, bytes } => {
                    String::from_utf8_lossy(bytes).to_string()
                }
                _ => "<invalid>".to_string(),
            };
            let descriptor = match &class_info.constant_pool[(f.descriptor_index - 1) as usize].info
            {
                ConstantPoolPFieldInfo::Utf8Info { length: _, bytes } => {
                    String::from_utf8_lossy(bytes).to_string()
                }
                _ => "<invalid>".to_string(),
            };

            let mut constant_value: Option<String> = None;
            for attr_info in &f.attributes {
                if let Ok(parsed) = parse_attribute_info(attr_info, &class_info.constant_pool) {
                    if let Attribute::ConstantVale {
                        constantvalue_index,
                        ..
                    } = parsed
                    {
                        let idx = constantvalue_index as usize;
                        if idx > 0 && idx <= class_info.constant_pool.len() {
                            match &class_info.constant_pool[idx - 1].info {
                                ConstantPoolPFieldInfo::String { string_index } => {
                                    let sidx = *string_index as usize;
                                    if sidx > 0 && sidx <= class_info.constant_pool.len() {
                                        if let ConstantPoolPFieldInfo::Utf8Info {
                                            length: _,
                                            bytes,
                                        } = &class_info.constant_pool[sidx - 1].info
                                        {
                                            constant_value =
                                                Some(String::from_utf8_lossy(bytes).to_string());
                                        }
                                    }
                                }
                                ConstantPoolPFieldInfo::Integer(i) => {
                                    constant_value = Some(i.to_string());
                                }
                                _ => {
                                    constant_value = Some(format!(
                                        "{:?}",
                                        class_info.constant_pool[idx - 1].info
                                    ));
                                }
                            }
                        }
                    }
                }
            }

            let field = Rc::new(Field {
                name,
                descriptor,
                access_flags: AccessFlags::from(f.access_flags),
                constant_value,
            });
            field_by_index.insert(f.name_index, Rc::clone(&field));
            fields.push(field);
        }
        (fields, field_by_index)
    }

    pub fn init(class_info: &JavaClass) -> Self {
        let (methods, method_by_index) = Class::load_methods(class_info);
        let (fields, field_by_index) = Class::load_fields(class_info);

        Class {
            constant_pool: class_info.constant_pool.clone(),
            methods,
            fields,
            method_by_index,
            field_by_index,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Class;
    use crate::loader::class_loader::class_loader::load;

    #[test]
    fn test_class_init_fields_and_methods() {
        let jclass = load("test/Hello.class").unwrap();
        let vm_class = Class::init(&jclass);
        // Hello.class has one static field `hello_str` with ConstantValue "Hello JVM"
        assert_eq!(vm_class.fields.len(), 1);
        assert_eq!(vm_class.fields[0].name, "hello_str");
        assert_eq!(
            vm_class.fields[0].constant_value.as_deref(),
            Some("Hello JVM")
        );

        // Methods include constructor and main
        let names: Vec<String> = vm_class.methods.iter().map(|m| m.name.clone()).collect();
        assert!(names.contains(&"main".to_string()));
    }
}
