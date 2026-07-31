pub mod access_flags;
pub mod byte_code;
pub mod class;
pub mod errors;
pub mod frame;
pub mod heap;
pub mod method_area;
pub mod operand_stack;
pub mod runtime;
pub mod stack;
pub mod thread;
pub mod types;
pub mod reference_manager;

/// Extract class name from a constant-pool style identifier `pkg/Class.member...`.
/// Returns `Some(&str)` with the substring before the last `.` or `None` if not found.
#[macro_export]
macro_rules! class_name_from_identifier {
	($identifier:expr) => {
		$identifier.rfind('.').map(|pos| &$identifier[..pos])
	};
}

pub use access_flags::AccessFlags;
