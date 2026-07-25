mod runtime {
    use crate::loader::java_class::java_class::JavaClassPtr;
    use crate::vm::method_area::{self, MethodArea};


    struct Runtime {
        method_area: MethodArea
    }
    impl Runtime {
        pub fn init(java_classes: Vec<JavaClassPtr>) {}
        pub fn run(class_name: &str) {
        }
    }

}