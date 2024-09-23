mod dashboard;
mod password;
mod logout;
pub mod newsletter;

pub use dashboard::admin_dashboard;
pub use password::*;
pub use logout::log_out;
pub use newsletter::*;