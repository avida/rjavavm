pub mod attributes {
    use byteorder::ReadBytesExt;
    use std::fmt;
    use std::io::{Cursor, Read};
    use crate::errors::errors::*;
    use crate::utils::*;

    #[derive(Debug)]
    pub struct AttributeInfo {
        pub attribute_name_index: u16,
        pub attribute_length: u32,
        pub info: Vec<u8>,
    }

    impl fmt::Display for AttributeInfo {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "Attribute(name_index={}, length={}, info={:02x?})",
                self.attribute_name_index,
                self.attribute_length,
                &self.info
            )
        }
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