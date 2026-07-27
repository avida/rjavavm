pub mod byte_code {
    use std::fmt;
    use crate::vm::errors::errors::RunTimeError;

    #[repr(u8)]
    pub enum Instruction {
        Sipush = 0x11,
        Ldc = 0x12,
        Aload = 0x19,
        Aload0 = 0x2a,
        Aload1 = 0x2b,
        Aload2 = 0x2c,
        Aload3 = 0x2d,
        IconstM1 = 0x02,
        Iconst0 = 0x03,
        Iconst1 = 0x04,
        Iconst2 = 0x05,
        Iconst3 = 0x06,
        Iconst4 = 0x07,
        Iconst5 = 0x08,
        Iadd = 0x60,
        Pop = 0x57,
        Astore = 0x3a,
        Astore0 = 0x4b,
        Astore1 = 0x4c,
        Astore2 = 0x4d,
        Astore3 = 0x4e,
        Getstatic = 0xb2,
        Invokevirtual = 0xb6,
        Putstatic = 0xb3,
        Invokespecial = 0xb7,
        Invokestatic = 0xb8,
        Invokedynamic = 0xba,
        IfIcmpge = 0xa2,
        Return = 0xb1,
    }

    pub struct Op<'a> {
        pub instruction: Instruction,
        pub args: &'a [u8],
    }

    pub fn parse<'a>(bytes: &'a [u8]) -> Result<Vec<Op<'a>>, RunTimeError> {
        let mut result: Vec<Op<'a>> = Vec::new();
        let mut i: usize = 0;
        while i < bytes.len() {
            let op = bytes[i];
            i += 1;

            let (instruction, arg_len) = match op {
                0x11 => (Instruction::Sipush, 2),
                0x60 => (Instruction::Iadd, 0),
                0x57 => (Instruction::Pop, 0),
                0x02 => (Instruction::IconstM1, 0),
                0x03 => (Instruction::Iconst0, 0),
                0x04 => (Instruction::Iconst1, 0),
                0x05 => (Instruction::Iconst2, 0),
                0x06 => (Instruction::Iconst3, 0),
                0x07 => (Instruction::Iconst4, 0),
                0x08 => (Instruction::Iconst5, 0),
                0x12 => (Instruction::Ldc, 1),
                0x19 => (Instruction::Aload, 1),
                0x2a => (Instruction::Aload0, 0),
                0x2b => (Instruction::Aload1, 0),
                0x2c => (Instruction::Aload2, 0),
                0x2d => (Instruction::Aload3, 0),
                0x3a => (Instruction::Astore, 1),
                0x4b => (Instruction::Astore0, 0),
                0x4c => (Instruction::Astore1, 0),
                0x4d => (Instruction::Astore2, 0),
                0x4e => (Instruction::Astore3, 0),
                0xb2 => (Instruction::Getstatic, 2),
                0xb6 => (Instruction::Invokevirtual, 2),
                0xb3 => (Instruction::Putstatic, 2),
                0xb7 => (Instruction::Invokespecial, 2),
                0xb8 => (Instruction::Invokestatic, 2),
                0xba => (Instruction::Invokedynamic, 4),
                0xa2 => (Instruction::IfIcmpge, 2),
                0xb1 => (Instruction::Return, 0),
                _ => {
                    // Unknown/unsupported opcode: return error
                    break;
                    // return Err(RunTimeError::UnknownInstruction(op));
                }
            };

            if i + arg_len > bytes.len() {
                // Not enough bytes remaining for args
                return Err(RunTimeError::Other("Not enough bytes for instruction arguments".to_string()));
            }

            let args = &bytes[i..i + arg_len];
            i += arg_len;

            result.push(Op { instruction, args });
        }

        Ok(result)
    }

    impl fmt::Display for Instruction {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Instruction::Sipush => write!(f, "sipush"),
                Instruction::Iadd => write!(f, "iadd"),
                Instruction::Pop => write!(f, "pop"),
                Instruction::IconstM1 => write!(f, "iconst_m1"),
                Instruction::Iconst0 => write!(f, "iconst_0"),
                Instruction::Iconst1 => write!(f, "iconst_1"),
                Instruction::Iconst2 => write!(f, "iconst_2"),
                Instruction::Iconst3 => write!(f, "iconst_3"),
                Instruction::Iconst4 => write!(f, "iconst_4"),
                Instruction::Iconst5 => write!(f, "iconst_5"),
                Instruction::Ldc => write!(f, "ldc"),
                Instruction::Aload => write!(f, "aload"),
                Instruction::Aload0 => write!(f, "aload_0"),
                Instruction::Aload1 => write!(f, "aload_1"),
                Instruction::Aload2 => write!(f, "aload_2"),
                Instruction::Aload3 => write!(f, "aload_3"),
                Instruction::Getstatic => write!(f, "getstatic"),
                Instruction::Invokevirtual => write!(f, "invokevirtual"),
                Instruction::Putstatic => write!(f, "putstatic"),
                Instruction::Invokespecial => write!(f, "invokespecial"),
                Instruction::Invokestatic => write!(f, "invokestatic"),
                Instruction::Invokedynamic => write!(f, "invokedynamic"),
                Instruction::IfIcmpge => write!(f, "if_icmpge"),
                Instruction::Return => write!(f, "return"),
                Instruction::Astore => write!(f, "astore"),
                Instruction::Astore0 => write!(f, "astore_0"),
                Instruction::Astore1 => write!(f, "astore_1"),
                Instruction::Astore2 => write!(f, "astore_2"),
                Instruction::Astore3 => write!(f, "astore_3"),
            }
        }
    }

    impl<'a> fmt::Display for Op<'a> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.instruction)?;
            if !self.args.is_empty() {
                write!(f, " ")?;
                for (i, b) in self.args.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "0x{:02x}", b)?;
                }
            }
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_parse_iadd_and_pop() {
            let bytes: &[u8] = &[
                0x03, // iconst_0
                0x04, // iconst_1
                0x60, // iadd
                0x57, // pop
            ];
            let ops = parse(bytes).unwrap();
            assert_eq!(ops.len(), 4);
            assert!(matches!(ops[2].instruction, Instruction::Iadd));
            assert!(matches!(ops[3].instruction, Instruction::Pop));
        }

        #[test]
        fn test_parse_iconst_sequence() {
            // iconst_m1, iconst_0, iconst_1, iconst_5
            let bytes: &[u8] = &[
                0x02, // iconst_m1
                0x03, // iconst_0
                0x04, // iconst_1
                0x08, // iconst_5
            ];

            let ops = parse(bytes).unwrap();
            assert_eq!(ops.len(), 4);
            match ops[0].instruction {
                Instruction::IconstM1 => (),
                _ => panic!("expected iconst_m1"),
            }
            match ops[1].instruction {
                Instruction::Iconst0 => (),
                _ => panic!("expected iconst_0"),
            }
            match ops[2].instruction {
                Instruction::Iconst1 => (),
                _ => panic!("expected iconst_1"),
            }
            match ops[3].instruction {
                Instruction::Iconst5 => (),
                _ => panic!("expected iconst_5"),
            }
        }

        #[test]
        fn test_parse_simple_sequence() {
            // ldc 0x05, sipush 0x0102, return, getstatic #0x0003
            let bytes: &[u8] = &[
                0x12, 0x05, // ldc 5
                0x11, 0x01, 0x02, // sipush 0x0102
                0xb1, // return
                0xb2, 0x00, 0x03, // getstatic #3
            ];

            let ops = parse(bytes).unwrap();
            assert_eq!(ops.len(), 4);

            match ops[0].instruction {
                Instruction::Ldc => assert_eq!(ops[0].args, &[0x05u8]),
                _ => panic!("expected ldc"),
            }

            match ops[1].instruction {
                Instruction::Sipush => assert_eq!(ops[1].args, &[0x01u8, 0x02u8]),
                _ => panic!("expected sipush"),
            }

            match ops[2].instruction {
                Instruction::Return => assert_eq!(ops[2].args.len(), 0),
                _ => panic!("expected return"),
            }

            match ops[3].instruction {
                Instruction::Getstatic => assert_eq!(ops[3].args, &[0x00u8, 0x03u8]),
                _ => panic!("expected getstatic"),
            }
        }

        #[test]
        fn test_parse_invokedynamic_and_invoke() {
            // invokedynamic (4 bytes args), invokevirtual (2 bytes)
            let bytes: &[u8] = &[
                0xba, 0x00, 0x01, 0x00, 0x00, // invokedynamic #1, 00 00
                0xb6, 0x00, 0x02, // invokevirtual #2
            ];

            let ops = parse(bytes).unwrap();
            assert_eq!(ops.len(), 2);

            match ops[0].instruction {
                Instruction::Invokedynamic => {
                    assert_eq!(ops[0].args, &[0x00u8, 0x01u8, 0x00u8, 0x00u8])
                }
                _ => panic!("expected invokedynamic"),
            }

            match ops[1].instruction {
                Instruction::Invokevirtual => assert_eq!(ops[1].args, &[0x00u8, 0x02u8]),
                _ => panic!("expected invokevirtual"),
            }
        }
        #[test]
        fn test_hello_java() {
            let bytes = &[0xb2 as u8, 0x00, 0x07, 0x12, 0x0f, 0xb6, 0x00, 0x11, 0xb1];
            let ops = parse(bytes).unwrap();
            assert_eq!(ops.len(), 4);
            for op in ops.iter() {
                println!("{} {:?}", op.instruction, op.args);
            }
        }
    }
}
