use crate::loader::attributes::attributes::{Attribute, parse_attribute_info};
use crate::loader::java_class::java_class::{
    ConstantPoolInfoTable, ConstantPoolPFieldInfo, JavaClass,
};

#[derive(Debug, Clone)]
pub struct Method {
    pub name: String,
    pub access_flags: u16,
    pub max_stack: u16,
    pub max_locals: u16,
    pub code: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub descriptor: String,
    pub access_flags: u16,
    pub constant_value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Class {
    pub constant_pool: ConstantPoolInfoTable,
    pub methods: Vec<Method>,
    pub fields: Vec<Field>,
}

impl Class {
    pub fn init(class_info: &JavaClass) -> Self {
        let mut methods: Vec<Method> = Vec::new();
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

            methods.push(Method {
                name,
                access_flags: m.access_flags,
                max_stack,
                max_locals,
                code,
            });
        }

        let mut fields: Vec<Field> = Vec::new();
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

            fields.push(Field {
                name,
                descriptor,
                access_flags: f.access_flags,
                constant_value,
            });
        }

        Class {
            constant_pool: class_info.constant_pool.clone(),
            methods,
            fields,
        }
    }
}
