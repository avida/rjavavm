mod byte_code {
    use std::fmt;

    #[repr(u8)]
    pub enum Instruction {
        Sipush = 0x11,
        Ldc = 0x12,
        Aload = 0x19,
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

    pub fn parse<'a>(bytes: &'a [u8]) -> Vec<Op<'a>> {
        let mut result: Vec<Op<'a>> = Vec::new();
        let mut i: usize = 0;
        while i < bytes.len() {
            let op = bytes[i];
            i += 1;

            let (instruction, arg_len) = match op {
                0x11 => (Instruction::Sipush, 2),
                0x12 => (Instruction::Ldc, 1),
                0x19 => (Instruction::Aload, 1),
                0xb2 => (Instruction::Getstatic, 2),
                0xb6 => (Instruction::Invokevirtual, 2),
                0xb3 => (Instruction::Putstatic, 2),
                0xb7 => (Instruction::Invokespecial, 2),
                0xb8 => (Instruction::Invokestatic, 2),
                0xba => (Instruction::Invokedynamic, 4),
                0xa2 => (Instruction::IfIcmpge, 2),
                0xb1 => (Instruction::Return, 0),
                _ => {
                    // Unknown/unsupported opcode: stop parsing
                    break;
                }
            };

            if i + arg_len > bytes.len() {
                // Not enough bytes remaining for args, stop parsing
                break;
            }

            let args = &bytes[i..i + arg_len];
            i += arg_len;

            result.push(Op { instruction, args });
        }

        result
    }

    impl fmt::Display for Instruction {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Instruction::Sipush => write!(f, "sipush"),
                Instruction::Ldc => write!(f, "ldc"),
                Instruction::Aload => write!(f, "aload"),
                Instruction::Getstatic => write!(f, "getstatic"),
                Instruction::Invokevirtual => write!(f, "invokevirtual"),
                Instruction::Putstatic => write!(f, "putstatic"),
                Instruction::Invokespecial => write!(f, "invokespecial"),
                Instruction::Invokestatic => write!(f, "invokestatic"),
                Instruction::Invokedynamic => write!(f, "invokedynamic"),
                Instruction::IfIcmpge => write!(f, "if_icmpge"),
                Instruction::Return => write!(f, "return"),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_parse_simple_sequence() {
            // ldc 0x05, sipush 0x0102, return, getstatic #0x0003
            let bytes: &[u8] = &[
                0x12, 0x05, // ldc 5
                0x11, 0x01, 0x02, // sipush 0x0102
                0xb1, // return
                0xb2, 0x00, 0x03, // getstatic #3
            ];

            let ops = parse(bytes);
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

            let ops = parse(bytes);
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
            let ops = parse(bytes);
            assert_eq!(ops.len(), 4);
            for op in ops.iter() {
                println!("{} {:?}", op.instruction, op.args);
            }
        }
    }
}
