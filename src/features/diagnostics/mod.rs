mod diagnostics;
mod external;
mod internal;

use diagnostics::DiagnosticProvider;
pub use diagnostics::{get_full_diagnostics, get_quick_diagnostics};
