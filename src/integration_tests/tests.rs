#[cfg(test)]
mod tests {
    use crate::loader::java_class::java_class::{
        ConstantPoolInfo, ConstantPoolPFieldInfo, ConstantPoolTag, RefFieldInfo,
    };
    use crate::vm::AccessFlags;
    use crate::vm::byte_code::byte_code::{Instruction, Op, ops_to_bytes};
    use crate::vm::class::Field;
    use crate::vm::class::MethodReference;
    use crate::vm::class::{Class, ClassPtr, Method};
    use crate::vm::method_area::MethodArea;
    use crate::vm::reference_manager::{ReferenceManager, ReferenceManagerPtr};
    use crate::vm::thread::thread::Thread;
    use crate::vm::types::types::Type;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    macro_rules! op {
        ($index:expr, $instruction:expr, $args:expr) => {
            Op {
                index: $index,
                instruction: $instruction,
                args: $args,
            }
        };
    }

    fn create_test_runtime(
        class_path: &str,
        trace: bool,
    ) -> (ReferenceManagerPtr, ClassPtr, Thread) {
        let ma = MethodArea::new();
        let rm = ReferenceManager::new_ptr();
        let heap = crate::vm::heap::Heap::new_ptr();
        let mut ma_guard = ma.lock().expect("MethodArea lock poisoned");
        let class = ma_guard
            .load_class(class_path)
            .expect(&format!("Failed to load {}", class_path));
        let thread = Thread::new(&ma, &rm, &heap, trace);
        drop(ma_guard);
        (rm, class, thread)
    }

    fn test_fields_and_methods() {
        let _cfg_guard = crate::test_utils::EnvGuard::set_from_config("CLASSPATH");

        let (rm, class, mut thread) = create_test_runtime("test/MethodsAndFields.class", true);

        // verify fields were loaded (name, age, salary)
        assert!(class.fields.len() >= 3, "expected at least 3 fields");

        // find the static method `createAndCalculateBonus`
        let method_opt = class
            .methods
            .iter()
            .find(|m| m.name == "createAndCalculateBonus")
            .cloned();
        assert!(
            method_opt.is_some(),
            "method createAndCalculateBonus not found"
        );
        let method = method_opt.unwrap();
        let method_ref = MethodReference::new(Rc::clone(&method), class.clone());

        // inspect method descriptor and code for debugging
        println!("method descriptor: {}", method.descriptor);
        println!("method code bytes: {:?}", method.code);
        match crate::vm::byte_code::byte_code::parse(&method.code) {
            Ok(ops) => {
                for op in ops.iter() {
                    println!("op: {}", op);
                }
            }
            Err(e) => println!("failed to parse ops: {}", e),
        }

        // prepare locals: (String name, int age, double salary, double percentage)
        // for the object/string param we allocate a symbolic reference
        let mut rm_lock = rm.lock().unwrap();
        let name_ref = rm_lock.allocate_symbolic("java/lang/String#dummy".to_string());
        drop(rm_lock);

        thread
            .push_frame_with_setup(&method_ref, |frame| {
                // set local 0: reference (stored as i32)
                frame.set_variable_value(0, name_ref as i32)?;
                // set local 1: int age
                frame.set_variable_value(1, 30i32)?;
                // set local 2: double salary (takes two slots)
                frame.set_variable_value(2, 1000f64)?;
                // set local 4: double percentage (takes two slots after salary)
                frame.set_variable_value(4, 10f64)?;
                Ok(())
            })
            .expect("failed to push frame with locals");

        // run the thread; this will execute the method body
        thread.run().expect("thread run failed");
    }

    #[test]
    fn test_iadd_runtime() {
        let _cfg_guard = crate::test_utils::EnvGuard::set_from_config("CLASSPATH");

        // create runtime and register a class containing a `main` method that
        // performs `iconst_1; iconst_2; iadd; return`.
        let mut rt = crate::vm::runtime::Runtime::init(true);

        // build ops programmatically (bipush 1, bipush 2, iadd, return)
        use crate::vm::byte_code::byte_code::{Instruction, Op, ops_to_bytes};

        let mut args_storage: Vec<Vec<u8>> = Vec::new();
        args_storage.push(vec![11u8]);
        args_storage.push(vec![31u8]);

        let ops: Vec<Op> = vec![
            op!(0, Instruction::Bipush, args_storage[0].as_slice()),
            op!(2, Instruction::Bipush, args_storage[1].as_slice()),
            op!(4, Instruction::Iadd, &[]),
        ];

        let code = ops_to_bytes(&ops);

        let method = Rc::new(Method {
            name: "main".to_string(),
            descriptor: "()V".to_string(),
            access_flags: AccessFlags::from(0u16),
            max_stack: 2,
            max_locals: 0,
            code: code.clone(),
        });

        // prepare a Class instance containing the method
        let mut method_by_index = HashMap::new();
        method_by_index.insert(1u16, method.clone());
        let class_ptr = Rc::new(Class {
            constant_pool: Rc::new(vec![]),
            methods: vec![method.clone()],
            fields: vec![],
            method_by_index,
            field_by_index: HashMap::new(),
            static_values: RefCell::new(HashMap::new()),
        });

        // register class under name "TestIadd" and run it using runtime.run
        rt.insert_class("TestIadd", class_ptr);
        rt.run("TestIadd").expect("runtime run failed");
        let frame = rt.main_thread.stack.top_frame().unwrap();
        assert!(frame.borrow().operand_stack.stack_size() == 1);
        // verify the result of iadd was pushed as the exit result (int 3)
        let val: i32 = frame.borrow_mut().operand_stack.pop().unwrap();
        assert_eq!(val, 42);
    }

    #[test]
    fn test_put_get_static_field() {
        let _cfg_guard = crate::test_utils::EnvGuard::set_from_config("CLASSPATH");

        let mut rt = crate::vm::runtime::Runtime::init(true);

        // local uses removed; imports are at file top
        use std::rc::Rc;

        // build constant pool entries:
        // 1: Utf8 "TestStatic"
        // 2: Class { name_index = 1 }
        // 3: Utf8 "myField"
        // 4: Utf8 "I"
        // 5: NameAndType { name_index = 3, descriptor_index = 4 }
        // 6: Fieldref { class_index = 2, name_and_type_index = 5 }
        let mut cp: Vec<ConstantPoolInfo> = Vec::new();
        cp.push(ConstantPoolInfo {
            tag: ConstantPoolTag::Utf8,
            info: ConstantPoolPFieldInfo::Utf8Info {
                length: 10,
                bytes: "TestStatic".as_bytes().to_vec(),
            },
        });
        cp.push(ConstantPoolInfo {
            tag: ConstantPoolTag::Class,
            info: ConstantPoolPFieldInfo::ClassInfo { name_index: 1 },
        });
        cp.push(ConstantPoolInfo {
            tag: ConstantPoolTag::Utf8,
            info: ConstantPoolPFieldInfo::Utf8Info {
                length: 7,
                bytes: "myField".as_bytes().to_vec(),
            },
        });
        cp.push(ConstantPoolInfo {
            tag: ConstantPoolTag::Utf8,
            info: ConstantPoolPFieldInfo::Utf8Info {
                length: 1,
                bytes: "I".as_bytes().to_vec(),
            },
        });
        cp.push(ConstantPoolInfo {
            tag: ConstantPoolTag::NameAndType,
            info: ConstantPoolPFieldInfo::NameAndType {
                name_index: 3,
                descriptor_index: 4,
            },
        });
        cp.push(ConstantPoolInfo {
            tag: ConstantPoolTag::Fieldref,
            info: ConstantPoolPFieldInfo::FieldRef(RefFieldInfo {
                class_index: 2,
                name_and_type_index: 5,
            }),
        });

        let cp_table = Rc::new(cp);

        // code: bipush 7; putstatic #6; getstatic #6; return
        let mut args_storage: Vec<Vec<u8>> = Vec::new();
        let static_var = 8u8;
        args_storage.push(vec![static_var]);
        // Constant pool's fieldref index
        args_storage.push(vec![0x00u8, 0x06u8]);

        let ops: Vec<Op> = vec![
            op!(0, Instruction::Bipush, args_storage[0].as_slice()),
            op!(2, Instruction::Putstatic, args_storage[1].as_slice()),
            op!(5, Instruction::Getstatic, args_storage[1].as_slice()),
        ];

        let code = ops_to_bytes(&ops);

        let method = Rc::new(Method {
            name: "main".to_string(),
            descriptor: "()V".to_string(),
            access_flags: AccessFlags::from(0u16),
            max_stack: 2,
            max_locals: 0,
            code: code.clone(),
        });

        let field = Rc::new(Field {
            name: "myField".to_string(),
            descriptor: "I".to_string(),
            access_flags: AccessFlags::from(AccessFlags::ACC_STATIC),
            constant_value: None,
        });

        let mut method_by_index = HashMap::new();
        method_by_index.insert(1u16, method.clone());

        let class_ptr = Rc::new(Class {
            constant_pool: cp_table,
            methods: vec![method.clone()],
            fields: vec![field],
            method_by_index,
            field_by_index: HashMap::new(),
            static_values: RefCell::new(HashMap::new()),
        });

        rt.insert_class("TestStatic", class_ptr.clone());
        rt.run("TestStatic").expect("runtime run failed");

        // verify operand stack result
        let frame = rt.main_thread.stack.top_frame().unwrap();
        assert!(frame.borrow().operand_stack.stack_size() == 1);
        let val: i32 = frame.borrow_mut().operand_stack.pop().unwrap();
        assert_eq!(val as u8, static_var);

        // verify putstatic stored the value in the class static values map (use the class we inserted)
        let static_id = "TestStatic.myField:I".to_string();
        let stored = class_ptr
            .get_static_by_identifier(&static_id)
            .expect("static not found");
        match stored {
            Type::Int(v) => assert_eq!(v as u8, static_var),
            _ => panic!("expected int static value"),
        }
    }
    
    #[test]
    fn test_new_and_putfield_integration() {
        let _cfg_guard = crate::test_utils::EnvGuard::set_from_config("CLASSPATH");

        use crate::loader::java_class::java_class::{ConstantPoolInfo, ConstantPoolPFieldInfo, ConstantPoolTag, RefFieldInfo};
        use crate::vm::class::{Class, Method, MethodPtr, Field};
        use crate::vm::types::types::Type;
        use std::rc::Rc;

        // Build Target class with a field `value:I` and a simple <init>()V
        let mut target_cp: Vec<ConstantPoolInfo> = Vec::new();
        // 1: UTF8 "Target"
        target_cp.push(ConstantPoolInfo { tag: ConstantPoolTag::Utf8, info: ConstantPoolPFieldInfo::Utf8Info { length: 6, bytes: b"Target".to_vec() } });
        // 2: Class { name_index = 1 }
        target_cp.push(ConstantPoolInfo { tag: ConstantPoolTag::Class, info: ConstantPoolPFieldInfo::ClassInfo { name_index: 1 } });
        // 3: UTF8 "<init>"
        target_cp.push(ConstantPoolInfo { tag: ConstantPoolTag::Utf8, info: ConstantPoolPFieldInfo::Utf8Info { length: 6, bytes: b"<init>".to_vec() } });
        // 4: UTF8 "()V"
        target_cp.push(ConstantPoolInfo { tag: ConstantPoolTag::Utf8, info: ConstantPoolPFieldInfo::Utf8Info { length: 3, bytes: b"()V".to_vec() } });
        // 5: NameAndType { name_index=3, descriptor_index=4 }
        target_cp.push(ConstantPoolInfo { tag: ConstantPoolTag::NameAndType, info: ConstantPoolPFieldInfo::NameAndType { name_index: 3, descriptor_index: 4 } });

        // Create constructor method: just return
        let init_method = Rc::new(Method {
            name: "<init>".to_string(),
            descriptor: "()V".to_string(),
            access_flags: crate::vm::AccessFlags::from(0u16),
            max_stack: 1,
            max_locals: 1,
            code: vec![Instruction::Return as u8],
        });

        // Field: value:I
        let field = Rc::new(Field {
            name: "value".to_string(),
            descriptor: "I".to_string(),
            access_flags: crate::vm::AccessFlags::from(0u16),
            constant_value: None,
        });

        let target_class = Rc::new(Class {
            constant_pool: Rc::new(target_cp),
            methods: vec![init_method.clone()],
            fields: vec![field.clone()],
            method_by_index: {
                let mut m = std::collections::HashMap::new();
                m.insert(1u16, init_method.clone());
                m
            },
            field_by_index: {
                let mut m = std::collections::HashMap::new();
                m.insert(1u16, field.clone());
                m
            },
            static_values: std::cell::RefCell::new(std::collections::HashMap::new()),
        });

        // Build Creator class constant pool referencing Target, constructor and field
        let mut creator_cp: Vec<ConstantPoolInfo> = Vec::new();
        // 1: UTF8 "Target"
        creator_cp.push(ConstantPoolInfo { tag: ConstantPoolTag::Utf8, info: ConstantPoolPFieldInfo::Utf8Info { length: 6, bytes: b"Target".to_vec() } });
        // 2: Class { name_index = 1 }
        creator_cp.push(ConstantPoolInfo { tag: ConstantPoolTag::Class, info: ConstantPoolPFieldInfo::ClassInfo { name_index: 1 } });
        // 3: UTF8 "<init>"
        creator_cp.push(ConstantPoolInfo { tag: ConstantPoolTag::Utf8, info: ConstantPoolPFieldInfo::Utf8Info { length: 6, bytes: b"<init>".to_vec() } });
        // 4: UTF8 "()V"
        creator_cp.push(ConstantPoolInfo { tag: ConstantPoolTag::Utf8, info: ConstantPoolPFieldInfo::Utf8Info { length: 3, bytes: b"()V".to_vec() } });
        // 5: NameAndType {3,4}
        creator_cp.push(ConstantPoolInfo { tag: ConstantPoolTag::NameAndType, info: ConstantPoolPFieldInfo::NameAndType { name_index: 3, descriptor_index: 4 } });
        // 6: MethodRef { class_index=2, name_and_type_index=5 }
        creator_cp.push(ConstantPoolInfo { tag: ConstantPoolTag::Methodref, info: ConstantPoolPFieldInfo::MethodRef(RefFieldInfo { class_index: 2, name_and_type_index: 5 }) });
        // 7: UTF8 "value"
        creator_cp.push(ConstantPoolInfo { tag: ConstantPoolTag::Utf8, info: ConstantPoolPFieldInfo::Utf8Info { length: 5, bytes: b"value".to_vec() } });
        // 8: UTF8 "I"
        creator_cp.push(ConstantPoolInfo { tag: ConstantPoolTag::Utf8, info: ConstantPoolPFieldInfo::Utf8Info { length: 1, bytes: b"I".to_vec() } });
        // 9: NameAndType {7,8}
        creator_cp.push(ConstantPoolInfo { tag: ConstantPoolTag::NameAndType, info: ConstantPoolPFieldInfo::NameAndType { name_index: 7, descriptor_index: 8 } });
        // 10: FieldRef { class_index=2, name_and_type_index=9 }
        creator_cp.push(ConstantPoolInfo { tag: ConstantPoolTag::Fieldref, info: ConstantPoolPFieldInfo::FieldRef(RefFieldInfo { class_index: 2, name_and_type_index: 9 }) });

        // Build Creator.main method: new #2, dup, invokespecial #6, dup, bipush 42, putfield #10, return
        use crate::vm::byte_code::byte_code::{Op, Instruction, ops_to_bytes};
        let mut args_storage: Vec<Vec<u8>> = Vec::new();
        args_storage.push(vec![(2u16 >> 8) as u8, (2u16 & 0xff) as u8]); // class index for NEW
        args_storage.push(vec![(6u16 >> 8) as u8, (6u16 & 0xff) as u8]); // methodref for invokespecial
        args_storage.push(vec![(10u16 >> 8) as u8, (10u16 & 0xff) as u8]); // fieldref for putfield
        args_storage.push(vec![42u8]); // bipush 42

        let ops: Vec<Op> = vec![
            Op { index: 0, instruction: Instruction::New, args: &args_storage[0] },
            Op { index: 3, instruction: Instruction::Dup, args: &[] },
            Op { index: 4, instruction: Instruction::Invokespecial, args: &args_storage[1] },
            Op { index: 7, instruction: Instruction::Dup, args: &[] },
            Op { index: 8, instruction: Instruction::Bipush, args: &args_storage[3] },
            Op { index: 10, instruction: Instruction::Putfield, args: &args_storage[2] },
            Op { index: 13, instruction: Instruction::Return, args: &[] },
        ];

        let code = ops_to_bytes(&ops);

        let main_method = Rc::new(Method {
            name: "main".to_string(),
            descriptor: "()V".to_string(),
            access_flags: crate::vm::AccessFlags::from(0u16),
            max_stack: 4,
            max_locals: 0,
            code: code.clone(),
        });

        let creator_class = Rc::new(Class {
            constant_pool: Rc::new(creator_cp),
            methods: vec![main_method.clone()],
            fields: vec![],
            method_by_index: {
                let mut m = std::collections::HashMap::new();
                m.insert(1u16, main_method.clone());
                m
            },
            field_by_index: std::collections::HashMap::new(),
            static_values: std::cell::RefCell::new(std::collections::HashMap::new()),
        });

        // create runtime and insert both classes
        let mut rt = crate::vm::runtime::Runtime::init(true);
        rt.insert_class("Target", target_class.clone());
        rt.insert_class("Creator", creator_class.clone());

        // run Creator.main
        rt.run("Creator").expect("runtime run failed");

        // verify heap contains an instance of Target with field value == 42
        let heap = rt.heap.lock().unwrap();
        let mut found = false;
        for (_id, entry) in heap.entries_ref().iter() {
            if let crate::vm::heap::HeapEntry::Object(obj) = entry {
                if obj.class_name == "Target" {
                    if let Some(v) = obj.get_field("value") {
                        assert_eq!(v, &Type::Int(42));
                        found = true;
                    }
                }
            }
        }
        assert!(found, "expected Target instance with value field set");
    }
}
