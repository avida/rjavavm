pub mod byte_code {
    use crate::vm::errors::errors::RunTimeError;
    use std::fmt;

    // Macro to convert a two-byte slice (big-endian) into an `i16` (short).
    // Usage: `bytes_to_short!(slice)` where `slice` is a `&[u8]` with length >= 2.
    #[macro_export]
    macro_rules! bytes_to_short {
        ($b:expr) => {{
            let arr: &[u8] = $b;
            ((arr[0] as u16) << 8 | (arr[1] as u16)) as i16
        }};
    }

    impl std::ops::Sub for Instruction {
        type Output = isize;
        fn sub(self, rhs: Instruction) -> Self::Output {
            (self as u8 as isize) - (rhs as u8 as isize)
        }
    }

    #[repr(u8)]
    #[derive(Copy, Clone)]
    pub enum Instruction {
        Bipush = 0x10,
        Sipush = 0x11,
        IfIcmpeq = 0x9f,
        IfIcmpne = 0xa0,
        IfIcmplt = 0xa1,
        Ldc = 0x12,
        Aload = 0x19,
        Aload0 = 0x2a,
        Aload1 = 0x2b,
        Aload2 = 0x2c,
        Aload3 = 0x2d,
        Dload0 = 0x26,
        Dload1 = 0x27,
        Dload2 = 0x28,
        Dload3 = 0x29,
        Aaload = 0x32,
        Iload = 0x15,
        Dload = 0x18,
        Iload0 = 0x1a,
        Iload1 = 0x1b,
        Iload2 = 0x1c,
        Iload3 = 0x1d,
        IconstM1 = 0x02,
        Iconst0 = 0x03,
        Iconst1 = 0x04,
        Iconst2 = 0x05,
        Iconst3 = 0x06,
        Iconst4 = 0x07,
        Iconst5 = 0x08,
        Dup = 0x59,
        Iadd = 0x60,
        Isub = 0x64,
        Pop = 0x57,
        Astore = 0x3a,
        Aastore = 0x53,
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
        New = 0xbb,
        IfIcmpge = 0xa2,
        IfIcmpgt = 0xa3,
        IfIcmple = 0xa4,
        IfEq = 0x99,
        IfNe = 0x9a,
        IfLt = 0x9b,
        IfGe = 0x9c,
        IfGt = 0x9d,
        IfLe = 0x9e,
        Areturn = 0xb0,
        Dreturn = 0xaf,
        Return = 0xb1,
    }

    pub struct Op<'a> {
        pub index: usize,
        pub instruction: Instruction,
        pub args: &'a [u8],
    }

    pub fn parse_op_at<'a>(
        bytes: &'a [u8],
        offset: usize,
    ) -> Result<(Op<'a>, usize), RunTimeError> {
        if offset >= bytes.len() {
            return Err(RunTimeError::Other("offset out of range".to_string()));
        }
        let op = bytes[offset];
        let index = offset;

        let (instruction, arg_len) = match op {
            0x11 => (Instruction::Sipush, 2),
            0x99 => (Instruction::IfEq, 2),
            0x9a => (Instruction::IfNe, 2),
            0x9b => (Instruction::IfLt, 2),
            0x9c => (Instruction::IfGe, 2),
            0x9d => (Instruction::IfGt, 2),
            0x9e => (Instruction::IfLe, 2),
            0x9f => (Instruction::IfIcmpeq, 2),
            0xa0 => (Instruction::IfIcmpne, 2),
            0xa1 => (Instruction::IfIcmplt, 2),
            0x10 => (Instruction::Bipush, 1),
            0x15 => (Instruction::Iload, 1),
            0x59 => (Instruction::Dup, 0),
            0x60 => (Instruction::Iadd, 0),
            0x64 => (Instruction::Isub, 0),
            0x57 => (Instruction::Pop, 0),
            0x02 => (Instruction::IconstM1, 0),
            0x03 => (Instruction::Iconst0, 0),
            0x04 => (Instruction::Iconst1, 0),
            0x05 => (Instruction::Iconst2, 0),
            0x06 => (Instruction::Iconst3, 0),
            0x07 => (Instruction::Iconst4, 0),
            0x08 => (Instruction::Iconst5, 0),
            0x12 => (Instruction::Ldc, 1),
            0x1a => (Instruction::Iload0, 0),
            0x1b => (Instruction::Iload1, 0),
            0x1c => (Instruction::Iload2, 0),
            0x1d => (Instruction::Iload3, 0),
            0x19 => (Instruction::Aload, 1),
            0x18 => (Instruction::Dload, 1),
            0x2a => (Instruction::Aload0, 0),
            0x2b => (Instruction::Aload1, 0),
            0x2c => (Instruction::Aload2, 0),
            0x2d => (Instruction::Aload3, 0),
            0x26 => (Instruction::Dload0, 0),
            0x27 => (Instruction::Dload1, 0),
            0x28 => (Instruction::Dload2, 0),
            0x29 => (Instruction::Dload3, 0),
            0x32 => (Instruction::Aaload, 0),
            0x3a => (Instruction::Astore, 1),
            0x4b => (Instruction::Astore0, 0),
            0x4c => (Instruction::Astore1, 0),
            0x4d => (Instruction::Astore2, 0),
            0x4e => (Instruction::Astore3, 0),
            0x53 => (Instruction::Aastore, 0),
            0xb2 => (Instruction::Getstatic, 2),
            0xb6 => (Instruction::Invokevirtual, 2),
            0xb3 => (Instruction::Putstatic, 2),
            0xb7 => (Instruction::Invokespecial, 2),
            0xb8 => (Instruction::Invokestatic, 2),
            0xba => (Instruction::Invokedynamic, 4),
            0xbb => (Instruction::New, 2),
            0xa2 => (Instruction::IfIcmpge, 2),
            0xa3 => (Instruction::IfIcmpgt, 2),
            0xa4 => (Instruction::IfIcmple, 2),
            0xb0 => (Instruction::Areturn, 0),
            0xaf => (Instruction::Dreturn, 0),
            0xb1 => (Instruction::Return, 0),
            _ => {
                return Err(RunTimeError::Other(format!(
                    "Unknown instruction 0x{:02x}",
                    op
                )));
            }
        };

        let arg_offset = index + 1;
        if arg_offset + arg_len > bytes.len() {
            return Err(RunTimeError::Other(
                "Not enough bytes for instruction arguments".to_string(),
            ));
        }

        let args = &bytes[arg_offset..arg_offset + arg_len];

        Ok((
            Op {
                index,
                instruction,
                args,
            },
            arg_len,
        ))
    }

    pub fn parse<'a>(bytes: &'a [u8]) -> Result<Vec<Op<'a>>, RunTimeError> {
        let mut result: Vec<Op<'a>> = Vec::new();
        let mut i: usize = 0;
        while i < bytes.len() {
            let (op, arg_len) = parse_op_at(bytes, i)?;
            result.push(op);
            i += 1 + arg_len;
        }

        Ok(result)
    }

    impl fmt::Display for Instruction {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Instruction::Sipush => write!(f, "sipush"),
                Instruction::IfIcmpeq => write!(f, "if_icmpeq"),
                Instruction::IfIcmpne => write!(f, "if_icmpne"),
                Instruction::IfIcmplt => write!(f, "if_icmplt"),
                Instruction::IfEq => write!(f, "ifeq"),
                Instruction::IfNe => write!(f, "ifne"),
                Instruction::IfLt => write!(f, "iflt"),
                Instruction::IfGe => write!(f, "ifge"),
                Instruction::IfGt => write!(f, "ifgt"),
                Instruction::IfLe => write!(f, "ifle"),
                Instruction::Bipush => write!(f, "bipush"),
                Instruction::Iload => write!(f, "iload"),
                Instruction::Iload0 => write!(f, "iload_0"),
                Instruction::Iload1 => write!(f, "iload_1"),
                Instruction::Iload2 => write!(f, "iload_2"),
                Instruction::Iload3 => write!(f, "iload_3"),
                Instruction::Dup => write!(f, "dup"),
                Instruction::Iadd => write!(f, "iadd"),
                Instruction::Isub => write!(f, "isub"),
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
                Instruction::Dload => write!(f, "dload"),
                Instruction::Dload0 => write!(f, "dload_0"),
                Instruction::Dload1 => write!(f, "dload_1"),
                Instruction::Dload2 => write!(f, "dload_2"),
                Instruction::Dload3 => write!(f, "dload_3"),
                Instruction::Aload0 => write!(f, "aload_0"),
                Instruction::Aload1 => write!(f, "aload_1"),
                Instruction::Aload2 => write!(f, "aload_2"),
                Instruction::Aload3 => write!(f, "aload_3"),
                Instruction::Aaload => write!(f, "aaload"),
                Instruction::Getstatic => write!(f, "getstatic"),
                Instruction::Invokevirtual => write!(f, "invokevirtual"),
                Instruction::Putstatic => write!(f, "putstatic"),
                Instruction::Invokespecial => write!(f, "invokespecial"),
                Instruction::Invokestatic => write!(f, "invokestatic"),
                Instruction::Invokedynamic => write!(f, "invokedynamic"),
                Instruction::New => write!(f, "new"),
                Instruction::IfIcmpge => write!(f, "if_icmpge"),
                Instruction::IfIcmpgt => write!(f, "if_icmpgt"),
                Instruction::IfIcmple => write!(f, "if_icmple"),
                Instruction::Areturn => write!(f, "areturn"),
                Instruction::Dreturn => write!(f, "dreturn"),
                Instruction::Return => write!(f, "return"),
                Instruction::Astore => write!(f, "astore"),
                Instruction::Aastore => write!(f, "aastore"),
                Instruction::Astore0 => write!(f, "astore_0"),
                Instruction::Astore1 => write!(f, "astore_1"),
                Instruction::Astore2 => write!(f, "astore_2"),
                Instruction::Astore3 => write!(f, "astore_3"),
            }
        }
    }

    impl<'a> fmt::Display for Op<'a> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}: {}", self.index, self.instruction)?;
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
        fn test_parse_bipush() {
            let bytes: &[u8] = &[
                0x10, 0x7f, // bipush 127
                0x10, 0x80, // bipush -128 (0x80 interpreted as signed byte)
            ];

            let ops = parse(bytes).unwrap();
            assert_eq!(ops.len(), 2);
            match ops[0].instruction {
                Instruction::Bipush => assert_eq!(ops[0].args, &[0x7fu8]),
                _ => panic!("expected bipush"),
            }
            match ops[1].instruction {
                Instruction::Bipush => assert_eq!(ops[1].args, &[0x80u8]),
                _ => panic!("expected bipush"),
            }
        }
        #[test]
        fn test_parse_iadd_and_pop() {
            let bytes: &[u8] = &[
                0x03, // iconst_0
                0x04, // iconst_1
                0x59, // dup
                0x60, // iadd
                0x57, // pop
            ];
            let ops = parse(bytes).unwrap();
            assert_eq!(ops.len(), 5);
            assert!(matches!(ops[2].instruction, Instruction::Dup));
            assert!(matches!(ops[3].instruction, Instruction::Iadd));
            assert!(matches!(ops[4].instruction, Instruction::Pop));
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
        fn test_parse_new() {
            let bytes: &[u8] = &[
                0xbb, 0x00, 0x07, // new #7
            ];

            let ops = parse(bytes).unwrap();
            assert_eq!(ops.len(), 1);

            match ops[0].instruction {
                Instruction::New => assert_eq!(ops[0].args, &[0x00u8, 0x07u8]),
                _ => panic!("expected new"),
            }
        }

        #[test]
        fn test_parse_aload_and_astore() {
            let bytes: &[u8] = &[
                0x19, 0x04, // aload 4
                0x3a, 0x02, // astore 2
            ];

            let ops = parse(bytes).unwrap();
            assert_eq!(ops.len(), 2);

            match ops[0].instruction {
                Instruction::Aload => assert_eq!(ops[0].args, &[0x04u8]),
                _ => panic!("expected aload"),
            }

            match ops[1].instruction {
                Instruction::Astore => assert_eq!(ops[1].args, &[0x02u8]),
                _ => panic!("expected astore"),
            }
        }

        #[test]
        fn test_parse_aaload_and_aastore() {
            let bytes: &[u8] = &[
                0x32, // aaload
                0x53, // aastore
            ];

            let ops = parse(bytes).unwrap();
            assert_eq!(ops.len(), 2);

            match ops[0].instruction {
                Instruction::Aaload => assert!(ops[0].args.is_empty()),
                _ => panic!("expected aaload"),
            }

            match ops[1].instruction {
                Instruction::Aastore => assert!(ops[1].args.is_empty()),
                _ => panic!("expected aastore"),
            }
        }

        #[test]
        fn test_parse_areturn() {
            let bytes: &[u8] = &[
                0xb0, // areturn
            ];

            let ops = parse(bytes).unwrap();
            assert_eq!(ops.len(), 1);

            match ops[0].instruction {
                Instruction::Areturn => assert!(ops[0].args.is_empty()),
                _ => panic!("expected areturn"),
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
