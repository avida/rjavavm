#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fields_and_methods() {
        use crate::vm::method_area::MethodArea;
        use crate::vm::thread::thread::Thread;
        use crate::vm::reference_manager::ReferenceManager;
        use crate::vm::class::MethodReference;
        use std::rc::Rc;

        // create a MethodArea and ReferenceManager then load the class
        let ma = MethodArea::new();
        let rm = ReferenceManager::new_ptr();
        let mut ma_guard = ma.lock().expect("MethodArea lock poisoned");
        let class = ma_guard
            .load_class("test/MethodsAndFields.class")
            .expect("Failed to load MethodsAndFields.class");

        // verify fields were loaded (name, age, salary)
        assert!(class.fields.len() >= 3, "expected at least 3 fields");

        // find the static method `createAndCalculateBonus`
        let method_opt = class
            .methods
            .iter()
            .find(|m| m.name == "createAndCalculateBonus")
            .cloned();
        assert!(method_opt.is_some(), "method createAndCalculateBonus not found");
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

        // create a Thread and push a frame with initialized locals (args)
        let mut thread = Thread::new(&ma, &rm, true);

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
}
