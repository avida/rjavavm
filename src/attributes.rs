pub mod attributes {
    use crate::errors::errors::*;
    use crate::java_class::java_class::ConstantPoolInfoTable;
    use crate::java_class::java_class::ConstantPoolTag::Class;
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
    pub struct ExceptionTableRecord {
        start_pc: u16,
        end_pc: u16,
        handler_pc: u16,
        catch_type: u16,
    }
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
                        write!(f, " attributes_count={}", attributes.len())?;
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
        attribute_info: AttributeInfo,
        constant_pool: ConstantPoolInfoTable,
    ) -> Result<Attribute, ClassLoadError> {
        Err(ClassLoadError::Other("qwqwd".to_string()))
    }

    pub fn parse_attributes(
        cursor: &mut Cursor<Vec<u8>>,
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
