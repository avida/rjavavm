pub mod attributes {
    use crate::loader::errors::errors::*;
    use crate::loader::java_class::java_class::ConstantPoolInfoTable;
    use crate::loader::java_class::java_class::*;
    use crate::utils::*;
    use byteorder::ReadBytesExt;
    use std::fmt;
    use std::io::{Cursor, Read};

    #[derive(Debug)]
    pub struct AttributeInfo {
        pub attribute_name_index: u16,
        pub attribute_length: u32,
        pub info: Vec<u8>,
    }
    #[derive(Debug)]
    pub struct ExceptionTableRecord {
        start_pc: u16,
        end_pc: u16,
        handler_pc: u16,
        catch_type: u16,
    }
    #[derive(Debug)]
    pub enum Attribute {
        ConstantVale {
            attribute_name_index: u16,
            attribute_length: u32,
            constantvalue_index: u16,
        },
        Code {
            attribute_name_index: u16,
            attribute_length: u32,
            max_stack: u16,
            max_locals: u16,
            code_length: u32,
            code: Vec<u8>,
            exception_table_length: u16,
            exception_table: Vec<ExceptionTableRecord>,
            attributes_count: u16,
            attributes: Vec<AttributeInfo>,
        },
    }

    impl fmt::Display for Attribute {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Attribute::ConstantVale {
                    attribute_name_index,
                    attribute_length,
                    constantvalue_index,
                } => write!(
                    f,
                    "ConstantValue(name_index={}, length={}, value_index={})",
                    attribute_name_index, attribute_length, constantvalue_index
                ),
                Attribute::Code {
                    attribute_name_index,
                    attribute_length,
                    max_stack,
                    max_locals,
                    code_length,
                    code,
                    exception_table_length,
                    exception_table,
                    attributes_count,
                    attributes,
                } => {
                    write!(
                        f,
                        "Code(name_index={}, length={}, max_stack={}, max_locals={}, code_length={}, exception_table_length={}, attributes_count={})",
                        attribute_name_index,
                        attribute_length,
                        max_stack,
                        max_locals,
                        code_length,
                        exception_table_length,
                        attributes_count
                    )?;
                    if !code.is_empty() {
                        write!(f, " code={:02x?}", &code[..std::cmp::min(code.len(), 16)])?;
                        if code.len() > 16 {
                            write!(f, "...(+{} bytes)", code.len() - 16)?;
                        }
                    }
                    if !exception_table.is_empty() {
                        write!(f, " exception_table_len={}", exception_table.len())?;
                    }
                    if !attributes.is_empty() {
                        writeln!(f, " attributes_count={}", attributes.len())?;
                        writeln!(f, "      Attributes:")?;
                        for (i, a) in attributes.iter().enumerate() {
                            writeln!(f, "        #{}: {}", i + 1, a)?;
                        }
                    }
                    Ok(())
                }
            }
        }
    }

    impl fmt::Display for AttributeInfo {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "Attribute(name_index={}, length={}, info={:02x?})",
                self.attribute_name_index, self.attribute_length, &self.info
            )
        }
    }

    pub fn parse_attribute_info(
        attribute_info: &AttributeInfo,
        constant_pool: &ConstantPoolInfoTable,
    ) -> Result<Attribute, ClassLoadError> {
        let name = &constant_pool[attribute_info.attribute_name_index as usize - 1];
        if let ConstantPoolPFieldInfo::Utf8Info { length: _, bytes } = &name.info {
            let attr_name = String::from_utf8_lossy(bytes).to_string();
            // For now, just detect the attribute name; parsing per-attribute can be
            // implemented later.
            match attr_name.as_str() {
                "Code" => {
                    // parse Code attribute from attribute_info.info bytes
                    let mut c = std::io::Cursor::new(&attribute_info.info);
                    let max_stack = read_2_bytes!(c);
                    let max_locals = read_2_bytes!(c);
                    let code_length = read_4_bytes!(c);
                    let mut code: Vec<u8> = vec![0u8; code_length as usize];
                    c.read_exact(&mut code).map_err(map_error)?;
                    let exception_table_length = read_2_bytes!(c);
                    let mut exception_table: Vec<ExceptionTableRecord> = Vec::new();
                    for _ in 0..exception_table_length {
                        let start_pc = read_2_bytes!(c);
                        let end_pc = read_2_bytes!(c);
                        let handler_pc = read_2_bytes!(c);
                        let catch_type = read_2_bytes!(c);
                        exception_table.push(ExceptionTableRecord {
                            start_pc,
                            end_pc,
                            handler_pc,
                            catch_type,
                        });
                    }
                    let attributes_count = read_2_bytes!(c);
                    let attributes = parse_attributes(&mut c, attributes_count)?;
                    return Ok(Attribute::Code {
                        attribute_name_index: attribute_info.attribute_name_index,
                        attribute_length: attribute_info.attribute_length,
                        max_stack,
                        max_locals,
                        code_length,
                        code,
                        exception_table_length,
                        exception_table,
                        attributes_count,
                        attributes,
                    });
                }
                "ConstantValue" => {
                    let mut c = std::io::Cursor::new(attribute_info.info.clone());
                    let constantvalue_index = read_2_bytes!(c);
                    return Ok(Attribute::ConstantVale {
                        attribute_name_index: attribute_info.attribute_name_index,
                        attribute_length: attribute_info.attribute_length,
                        constantvalue_index,
                    });
                }
                _ => {
                    return Err(ClassLoadError::Other(format!(
                        "Unexpected attribute name: {attr_name}"
                    )));
                }
            }
        }

        Err(ClassLoadError::InvalidFormat(
            "Attribute name index did not point to a UTF8 entry".to_string(),
        ))
    }

    pub fn parse_attributes(
        cursor: &mut Cursor<&Vec<u8>>,
        attributes_count: u16,
    ) -> Result<Vec<AttributeInfo>, ClassLoadError> {
        let mut result: Vec<AttributeInfo> = Vec::new();
        for _ in 0..attributes_count {
            let (attribute_name_index, attribute_length) =
                (read_2_bytes!(cursor), read_4_bytes!(cursor));

            let mut info: Vec<u8> = Vec::new();
            info.resize(attribute_length as usize, 0);
            cursor.read_exact(&mut info).map_err(map_error)?;
            result.push(AttributeInfo {
                attribute_name_index,
                attribute_length,
                info,
            });
        }
        Ok(result)
    }
}
#[cfg(test)]
mod tests {
    use crate::loader::attributes::attributes::*;
    use crate::loader::class_loader::class_loader::*;
    #[test]
    fn test_parse_attribute() {
        let j_class = load("test/Hello.class").unwrap();
        assert!(true)
    }
}
