#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::AccessFlags;
    use crate::vm::class::MethodReference;
    use crate::vm::class::{Class, ClassPtr, Method};
    use crate::vm::method_area::MethodArea;
    use crate::vm::reference_manager::{ReferenceManager, ReferenceManagerPtr};
    use crate::vm::thread::thread::Thread;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    fn create_test_runtime(
        class_path: &str,
        trace: bool,
    ) -> (ReferenceManagerPtr, ClassPtr, Thread) {
        let ma = MethodArea::new();
        let rm = ReferenceManager::new_ptr();
        let mut ma_guard = ma.lock().expect("MethodArea lock poisoned");
        let class = ma_guard
            .load_class(class_path)
            .expect(&format!("Failed to load {}", class_path));
        let thread = Thread::new(&ma, &rm, trace);
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
            Op {
                index: 0,
                instruction: Instruction::Bipush,
                args: args_storage[0].as_slice(),
            },
            Op {
                index: 2,
                instruction: Instruction::Bipush,
                args: args_storage[1].as_slice(),
            },
            Op {
                index: 4,
                instruction: Instruction::Iadd,
                args: &[],
            },
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
        print!("val is {}", val);
    }
}
