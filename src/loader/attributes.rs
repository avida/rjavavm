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
    pub struct LineNumberTableRecord {
        start_pc: u16,
        line_number: u16,
    }
    #[derive(Debug)]
    pub struct LocalVariableTableRecord {
        start_pc: u16,
        length: u16,
        name_index: u16,
        descriptor_index: u16,
        index: u16,
    }
    #[derive(Debug)]
    pub struct LocalVariableTypeTableRecord {
        start_pc: u16,
        length: u16,
        name_index: u16,
        signature_index: u16,
        index: u16,
    }
    #[derive(Debug)]
    pub struct VerificationTypeInfo {
        pub tag: u8,
        pub cpool_index: Option<u16>,
        pub offset: Option<u16>,
    }

    #[derive(Debug)]
    pub enum StackMapFrame {
        SameFrame {
            frame_type: u8,
        },
        SameLocals1StackItemFrame {
            frame_type: u8,
            stack: Vec<VerificationTypeInfo>,
        },
        SameLocals1StackItemFrameExtended {
            frame_type: u8,
            offset_delta: u16,
            stack: Vec<VerificationTypeInfo>,
        },
        ChopFrame {
            frame_type: u8,
            offset_delta: u16,
        },
        SameFrameExtended {
            frame_type: u8,
            offset_delta: u16,
        },
        AppendFrame {
            frame_type: u8,
            offset_delta: u16,
            locals: Vec<VerificationTypeInfo>,
        },
        FullFrame {
            frame_type: u8,
            offset_delta: u16,
            locals: Vec<VerificationTypeInfo>,
            stack: Vec<VerificationTypeInfo>,
        },
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
            attributes: Vec<Attribute>,
        },
        LineNumberTabel {
            attribute_name_index: u16,
            attribute_length: u32,
            line_number_table_length: u16,
            line_number_table: Vec<LineNumberTableRecord>,
        },
        LocalVariableTable {
            attribute_name_index: u16,
            attribute_length: u32,
            local_variable_table_length: u16,
            local_variable_table: Vec<LocalVariableTableRecord>,
        },
        LocalVariableTypeTable {
            attribute_name_index: u16,
            attribute_length: u32,
            local_variable_type_table_length: u16,
            local_variable_type_table: Vec<LocalVariableTypeTableRecord>,
        },
        StackMapTable {
            attribute_name_index: u16,
            attribute_length: u32,
            number_of_entries: u16,
            entries: Vec<StackMapFrame>,
        },
        RuntimeVisibleAnnotations {
            attribute_name_index: u16,
            attribute_length: u32,
            num_annotations: u16,
            annotations: Vec<Annotation>,
        },
        RuntimeInvisibleAnnotations {
            attribute_name_index: u16,
            attribute_length: u32,
            num_annotations: u16,
            annotations: Vec<Annotation>,
        },
        Exceptions {
            attribute_name_index: u16,
            attribute_length: u32,
            number_of_exceptions: u16,
            exception_index_table: Vec<u16>,
        },
        Deprecated {
            attribute_name_index: u16,
            attribute_length: u32,
        },
        Signature {
            attribute_name_index: u16,
            attribute_length: u32,
            signature_index: u16,
        },
        MethodParameters {
            attribute_name_index: u16,
            attribute_length: u32,
            parameters_count: u8,
            parameters: Vec<MethodParameter>,
        },
    }

    impl Attribute {
        pub fn name_index(&self) -> u16 {
            match self {
                Attribute::ConstantVale {
                    attribute_name_index,
                    ..
                } => *attribute_name_index,
                Attribute::Code {
                    attribute_name_index,
                    ..
                } => *attribute_name_index,
                Attribute::LineNumberTabel {
                    attribute_name_index,
                    ..
                } => *attribute_name_index,
                Attribute::LocalVariableTable {
                    attribute_name_index,
                    ..
                } => *attribute_name_index,
                Attribute::LocalVariableTypeTable {
                    attribute_name_index,
                    ..
                } => *attribute_name_index,
                Attribute::StackMapTable {
                    attribute_name_index,
                    ..
                } => *attribute_name_index,
                Attribute::RuntimeVisibleAnnotations {
                    attribute_name_index,
                    ..
                } => *attribute_name_index,
                Attribute::RuntimeInvisibleAnnotations {
                    attribute_name_index,
                    ..
                } => *attribute_name_index,
                Attribute::Exceptions {
                    attribute_name_index,
                    ..
                } => *attribute_name_index,
                Attribute::Deprecated {
                    attribute_name_index,
                    ..
                } => *attribute_name_index,
                Attribute::Signature {
                    attribute_name_index,
                    ..
                } => *attribute_name_index,
                Attribute::MethodParameters {
                    attribute_name_index,
                    ..
                } => *attribute_name_index,
            }
        }
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
                            writeln!(
                                f,
                                "        #{} (name_index={}): {}",
                                i + 1,
                                a.name_index(),
                                a
                            )?;
                        }
                    }
                    Ok(())
                }
                Attribute::LineNumberTabel {
                    attribute_name_index,
                    attribute_length,
                    line_number_table_length,
                    line_number_table,
                } => {
                    write!(
                        f,
                        "LineNumberTable(name_index={}, length={}, table_length={})",
                        attribute_name_index, attribute_length, line_number_table_length
                    )?;
                    if !line_number_table.is_empty() {
                        writeln!(f)?;
                        for (i, record) in line_number_table.iter().enumerate() {
                            writeln!(
                                f,
                                "        #{}: start_pc={}, line_number={}",
                                i + 1,
                                record.start_pc,
                                record.line_number
                            )?;
                        }
                    }
                    Ok(())
                }
                Attribute::LocalVariableTable {
                    attribute_name_index,
                    attribute_length,
                    local_variable_table_length,
                    local_variable_table,
                } => {
                    write!(
                        f,
                        "LocalVariableTable(name_index={}, length={}, table_length={})",
                        attribute_name_index, attribute_length, local_variable_table_length
                    )?;
                    if !local_variable_table.is_empty() {
                        writeln!(f)?;
                        for (i, record) in local_variable_table.iter().enumerate() {
                            writeln!(
                                f,
                                "        #{}: start_pc={}, length={}, name_index={}, descriptor_index={}, index={}",
                                i + 1,
                                record.start_pc,
                                record.length,
                                record.name_index,
                                record.descriptor_index,
                                record.index
                            )?;
                        }
                    }
                    Ok(())
                }
                Attribute::LocalVariableTypeTable {
                    attribute_name_index,
                    attribute_length,
                    local_variable_type_table_length,
                    local_variable_type_table,
                } => {
                    write!(
                        f,
                        "LocalVariableTypeTable(name_index={}, length={}, table_length={})",
                        attribute_name_index, attribute_length, local_variable_type_table_length
                    )?;
                    if !local_variable_type_table.is_empty() {
                        writeln!(f)?;
                        for (i, record) in local_variable_type_table.iter().enumerate() {
                            writeln!(
                                f,
                                "        #{}: start_pc={}, length={}, name_index={}, signature_index={}, index={}",
                                i + 1,
                                record.start_pc,
                                record.length,
                                record.name_index,
                                record.signature_index,
                                record.index
                            )?;
                        }
                    }
                    Ok(())
                }
                Attribute::StackMapTable {
                    attribute_name_index,
                    attribute_length,
                    number_of_entries,
                    entries,
                } => {
                    write!(
                        f,
                        "StackMapTable(name_index={}, length={}, entries={})",
                        attribute_name_index, attribute_length, number_of_entries
                    )?;
                    if !entries.is_empty() {
                        writeln!(f)?;
                        for (i, entry) in entries.iter().enumerate() {
                            writeln!(f, "        #{}: {:?}", i + 1, entry)?;
                        }
                    }
                    Ok(())
                }
                Attribute::RuntimeVisibleAnnotations {
                    attribute_name_index,
                    attribute_length,
                    num_annotations,
                    annotations,
                } => {
                    write!(
                        f,
                        "RuntimeVisibleAnnotations(name_index={}, length={}, count={})",
                        attribute_name_index, attribute_length, num_annotations
                    )?;
                    if !annotations.is_empty() {
                        writeln!(f)?;
                        for (i, a) in annotations.iter().enumerate() {
                            writeln!(f, "        #{}: {}", i + 1, a)?;
                        }
                    }
                    Ok(())
                }
                Attribute::RuntimeInvisibleAnnotations {
                    attribute_name_index,
                    attribute_length,
                    num_annotations,
                    annotations,
                } => {
                    write!(
                        f,
                        "RuntimeInvisibleAnnotations(name_index={}, length={}, count={})",
                        attribute_name_index, attribute_length, num_annotations
                    )?;
                    if !annotations.is_empty() {
                        writeln!(f)?;
                        for (i, a) in annotations.iter().enumerate() {
                            writeln!(f, "        #{}: {}", i + 1, a)?;
                        }
                    }
                    Ok(())
                }
                Attribute::Exceptions {
                    attribute_name_index,
                    attribute_length,
                    number_of_exceptions,
                    exception_index_table,
                } => {
                    write!(
                        f,
                        "Exceptions(name_index={}, length={}, count={})",
                        attribute_name_index, attribute_length, number_of_exceptions
                    )?;
                    if !exception_index_table.is_empty() {
                        writeln!(f)?;
                        for (i, idx) in exception_index_table.iter().enumerate() {
                            writeln!(f, "        #{}: exception_index={}", i + 1, idx)?;
                        }
                    }
                    Ok(())
                }
                Attribute::Deprecated {
                    attribute_name_index,
                    attribute_length,
                } => {
                    write!(
                        f,
                        "Deprecated(name_index={}, length={})",
                        attribute_name_index, attribute_length
                    )
                }
                Attribute::Signature {
                    attribute_name_index,
                    attribute_length,
                    signature_index,
                } => {
                    write!(
                        f,
                        "Signature(name_index={}, length={}, signature_index={})",
                        attribute_name_index, attribute_length, signature_index
                    )
                }
                Attribute::MethodParameters {
                    attribute_name_index,
                    attribute_length,
                    parameters_count,
                    parameters,
                } => {
                    write!(
                        f,
                        "MethodParameters(name_index={}, length={}, count={})",
                        attribute_name_index, attribute_length, parameters_count
                    )?;
                    if !parameters.is_empty() {
                        writeln!(f)?;
                        for (i, p) in parameters.iter().enumerate() {
                            writeln!(
                                f,
                                "        #{}: name_index={}, access_flags=0x{:04x}",
                                i + 1,
                                p.name_index,
                                p.access_flags
                            )?;
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

    #[derive(Debug)]
    pub struct ElementValue {
        pub tag: u8,
        pub const_value_index: Option<u16>,
        pub type_name_index: Option<u16>,
        pub const_name_index: Option<u16>,
        pub annotation_value: Option<Annotation>,
        pub array_values: Option<Vec<ElementValue>>,
    }

    #[derive(Debug)]
    pub struct Annotation {
        pub type_index: u16,
        pub num_element_value_pairs: u16,
        pub elements: Vec<(u16, ElementValue)>,
    }

    #[derive(Debug)]
    pub struct MethodParameter {
        pub name_index: u16,
        pub access_flags: u16,
    }

    impl fmt::Display for Annotation {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "Annotation(type_index={}, pairs={})",
                self.type_index, self.num_element_value_pairs
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
                    let attributes_info = parse_attributes(&mut c, attributes_count)?;

                    let attributes = attributes_info
                        .iter()
                        .map(|a_i| parse_attribute_info(a_i, constant_pool).unwrap())
                        .collect();
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
                "LineNumberTable" => {
                    let mut c = std::io::Cursor::new(&attribute_info.info);
                    let line_number_table_length = read_2_bytes!(c);
                    let mut line_number_table: Vec<LineNumberTableRecord> = Vec::new();
                    for _ in 0..line_number_table_length {
                        let start_pc = read_2_bytes!(c);
                        let line_number = read_2_bytes!(c);
                        line_number_table.push(LineNumberTableRecord {
                            start_pc,
                            line_number,
                        });
                    }
                    return Ok(Attribute::LineNumberTabel {
                        attribute_name_index: attribute_info.attribute_name_index,
                        attribute_length: attribute_info.attribute_length,
                        line_number_table_length,
                        line_number_table,
                    });
                }
                "StackMapTable" => {
                    let mut c = std::io::Cursor::new(&attribute_info.info);
                    let number_of_entries = read_2_bytes!(c);
                    let mut entries: Vec<StackMapFrame> = Vec::new();
                    for _ in 0..number_of_entries {
                        let frame_type = c.read_u8().map_err(map_error)?;
                        match frame_type {
                            0..=63 => {
                                entries.push(StackMapFrame::SameFrame { frame_type });
                            }
                            64..=127 => {
                                // one stack item
                                let mut stack = Vec::new();
                                stack.push(parse_verification_type_info(&mut c)?);
                                entries.push(StackMapFrame::SameLocals1StackItemFrame {
                                    frame_type,
                                    stack,
                                });
                            }
                            247 => {
                                let offset_delta = read_2_bytes!(c);
                                let mut stack = Vec::new();
                                stack.push(parse_verification_type_info(&mut c)?);
                                entries.push(StackMapFrame::SameLocals1StackItemFrameExtended {
                                    frame_type,
                                    offset_delta,
                                    stack,
                                });
                            }
                            248..=250 => {
                                let offset_delta = read_2_bytes!(c);
                                entries.push(StackMapFrame::ChopFrame {
                                    frame_type,
                                    offset_delta,
                                });
                            }
                            251 => {
                                let offset_delta = read_2_bytes!(c);
                                entries.push(StackMapFrame::SameFrameExtended {
                                    frame_type,
                                    offset_delta,
                                });
                            }
                            252..=254 => {
                                let offset_delta = read_2_bytes!(c);
                                let k = (frame_type - 251) as usize;
                                let mut locals = Vec::new();
                                for _ in 0..k {
                                    locals.push(parse_verification_type_info(&mut c)?);
                                }
                                entries.push(StackMapFrame::AppendFrame {
                                    frame_type,
                                    offset_delta,
                                    locals,
                                });
                            }
                            255 => {
                                let offset_delta = read_2_bytes!(c);
                                let number_of_locals = read_2_bytes!(c);
                                let mut locals: Vec<VerificationTypeInfo> = Vec::new();
                                for _ in 0..number_of_locals {
                                    locals.push(parse_verification_type_info(&mut c)?);
                                }
                                let number_of_stack_items = read_2_bytes!(c);
                                let mut stack: Vec<VerificationTypeInfo> = Vec::new();
                                for _ in 0..number_of_stack_items {
                                    stack.push(parse_verification_type_info(&mut c)?);
                                }
                                entries.push(StackMapFrame::FullFrame {
                                    frame_type,
                                    offset_delta,
                                    locals,
                                    stack,
                                });
                            }
                            _ => {
                                return Err(ClassLoadError::Other(format!(
                                    "Unknown frame_type {}",
                                    frame_type
                                )));
                            }
                        }
                    }
                    return Ok(Attribute::StackMapTable {
                        attribute_name_index: attribute_info.attribute_name_index,
                        attribute_length: attribute_info.attribute_length,
                        number_of_entries,
                        entries,
                    });
                }
                "Exceptions" => {
                    let mut c = std::io::Cursor::new(&attribute_info.info);
                    let number_of_exceptions = read_2_bytes!(c);
                    let mut exception_index_table: Vec<u16> = Vec::new();
                    for _ in 0..number_of_exceptions {
                        exception_index_table.push(read_2_bytes!(c));
                    }
                    return Ok(Attribute::Exceptions {
                        attribute_name_index: attribute_info.attribute_name_index,
                        attribute_length: attribute_info.attribute_length,
                        number_of_exceptions,
                        exception_index_table,
                    });
                }
                "Deprecated" => {
                    // Deprecated has no info; length should be 0
                    return Ok(Attribute::Deprecated {
                        attribute_name_index: attribute_info.attribute_name_index,
                        attribute_length: attribute_info.attribute_length,
                    });
                }
                "Signature" => {
                    let mut c = std::io::Cursor::new(&attribute_info.info);
                    let signature_index = read_2_bytes!(c);
                    return Ok(Attribute::Signature {
                        attribute_name_index: attribute_info.attribute_name_index,
                        attribute_length: attribute_info.attribute_length,
                        signature_index,
                    });
                }
                "MethodParameters" => {
                    let mut c = std::io::Cursor::new(&attribute_info.info);
                    let parameters_count = c.read_u8().map_err(map_error)?;
                    let mut parameters: Vec<MethodParameter> = Vec::new();
                    for _ in 0..parameters_count {
                        let name_index = read_2_bytes!(c);
                        let access_flags = read_2_bytes!(c);
                        parameters.push(MethodParameter {
                            name_index,
                            access_flags,
                        });
                    }
                    return Ok(Attribute::MethodParameters {
                        attribute_name_index: attribute_info.attribute_name_index,
                        attribute_length: attribute_info.attribute_length,
                        parameters_count,
                        parameters,
                    });
                }
                "RuntimeVisibleAnnotations" | "RuntimeInvisibleAnnotations" => {
                    let mut c = std::io::Cursor::new(&attribute_info.info);
                    let num_annotations = read_2_bytes!(c);
                    let mut annotations: Vec<Annotation> = Vec::new();
                    for _ in 0..num_annotations {
                        let type_index = read_2_bytes!(c);
                        let num_element_value_pairs = read_2_bytes!(c);
                        let mut elements: Vec<(u16, ElementValue)> = Vec::new();
                        for _ in 0..num_element_value_pairs {
                            let element_name_index = read_2_bytes!(c);
                            let ev = parse_element_value(&mut c)?;
                            elements.push((element_name_index, ev));
                        }
                        annotations.push(Annotation {
                            type_index,
                            num_element_value_pairs,
                            elements,
                        });
                    }
                    if attr_name == "RuntimeVisibleAnnotations" {
                        return Ok(Attribute::RuntimeVisibleAnnotations {
                            attribute_name_index: attribute_info.attribute_name_index,
                            attribute_length: attribute_info.attribute_length,
                            num_annotations,
                            annotations,
                        });
                    }
                    return Ok(Attribute::RuntimeInvisibleAnnotations {
                        attribute_name_index: attribute_info.attribute_name_index,
                        attribute_length: attribute_info.attribute_length,
                        num_annotations,
                        annotations,
                    });
                }
                "LocalVariableTable" => {
                    let mut c = std::io::Cursor::new(&attribute_info.info);
                    let local_variable_table_length = read_2_bytes!(c);
                    let mut local_variable_table: Vec<LocalVariableTableRecord> = Vec::new();
                    for _ in 0..local_variable_table_length {
                        let start_pc = read_2_bytes!(c);
                        let length = read_2_bytes!(c);
                        let name_index = read_2_bytes!(c);
                        let descriptor_index = read_2_bytes!(c);
                        let index = read_2_bytes!(c);
                        local_variable_table.push(LocalVariableTableRecord {
                            start_pc,
                            length,
                            name_index,
                            descriptor_index,
                            index,
                        });
                    }
                    return Ok(Attribute::LocalVariableTable {
                        attribute_name_index: attribute_info.attribute_name_index,
                        attribute_length: attribute_info.attribute_length,
                        local_variable_table_length,
                        local_variable_table,
                    });
                }
                "LocalVariableTypeTable" => {
                    let mut c = std::io::Cursor::new(&attribute_info.info);
                    let local_variable_type_table_length = read_2_bytes!(c);
                    let mut local_variable_type_table: Vec<LocalVariableTypeTableRecord> =
                        Vec::new();
                    for _ in 0..local_variable_type_table_length {
                        let start_pc = read_2_bytes!(c);
                        let length = read_2_bytes!(c);
                        let name_index = read_2_bytes!(c);
                        let signature_index = read_2_bytes!(c);
                        let index = read_2_bytes!(c);
                        local_variable_type_table.push(LocalVariableTypeTableRecord {
                            start_pc,
                            length,
                            name_index,
                            signature_index,
                            index,
                        });
                    }
                    return Ok(Attribute::LocalVariableTypeTable {
                        attribute_name_index: attribute_info.attribute_name_index,
                        attribute_length: attribute_info.attribute_length,
                        local_variable_type_table_length,
                        local_variable_type_table,
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

    fn parse_verification_type_info(
        c: &mut Cursor<&Vec<u8>>,
    ) -> Result<VerificationTypeInfo, ClassLoadError> {
        let tag = c.read_u8().map_err(map_error)?;
        match tag {
            7 => {
                let cpool_index = read_2_bytes!(c);
                Ok(VerificationTypeInfo {
                    tag,
                    cpool_index: Some(cpool_index),
                    offset: None,
                })
            }
            8 => {
                let offset = read_2_bytes!(c);
                Ok(VerificationTypeInfo {
                    tag,
                    cpool_index: None,
                    offset: Some(offset),
                })
            }
            _ => Ok(VerificationTypeInfo {
                tag,
                cpool_index: None,
                offset: None,
            }),
        }
    }

    fn parse_element_value(c: &mut Cursor<&Vec<u8>>) -> Result<ElementValue, ClassLoadError> {
        let tag = c.read_u8().map_err(map_error)?;
        match tag as char {
            'B' | 'C' | 'D' | 'F' | 'I' | 'J' | 'S' | 'Z' | 's' => {
                let idx = read_2_bytes!(c);
                Ok(ElementValue {
                    tag,
                    const_value_index: Some(idx),
                    type_name_index: None,
                    const_name_index: None,
                    annotation_value: None,
                    array_values: None,
                })
            }
            'e' => {
                let type_name_index = read_2_bytes!(c);
                let const_name_index = read_2_bytes!(c);
                Ok(ElementValue {
                    tag,
                    const_value_index: None,
                    type_name_index: Some(type_name_index),
                    const_name_index: Some(const_name_index),
                    annotation_value: None,
                    array_values: None,
                })
            }
            'c' => {
                let idx = read_2_bytes!(c);
                Ok(ElementValue {
                    tag,
                    const_value_index: Some(idx),
                    type_name_index: None,
                    const_name_index: None,
                    annotation_value: None,
                    array_values: None,
                })
            }
            '@' => {
                let annotation = parse_annotation(c)?;
                Ok(ElementValue {
                    tag,
                    const_value_index: None,
                    type_name_index: None,
                    const_name_index: None,
                    annotation_value: Some(annotation),
                    array_values: None,
                })
            }
            '[' => {
                let num_values = read_2_bytes!(c);
                let mut vals = Vec::new();
                for _ in 0..num_values {
                    vals.push(parse_element_value(c)?);
                }
                Ok(ElementValue {
                    tag,
                    const_value_index: None,
                    type_name_index: None,
                    const_name_index: None,
                    annotation_value: None,
                    array_values: Some(vals),
                })
            }
            _ => Err(ClassLoadError::InvalidFormat(format!(
                "Unknown element_value tag {}",
                tag
            ))),
        }
    }

    fn parse_annotation(c: &mut Cursor<&Vec<u8>>) -> Result<Annotation, ClassLoadError> {
        let type_index = read_2_bytes!(c);
        let num_element_value_pairs = read_2_bytes!(c);
        let mut elements: Vec<(u16, ElementValue)> = Vec::new();
        for _ in 0..num_element_value_pairs {
            let element_name_index = read_2_bytes!(c);
            let ev = parse_element_value(c)?;
            elements.push((element_name_index, ev));
        }
        Ok(Annotation {
            type_index,
            num_element_value_pairs,
            elements,
        })
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
