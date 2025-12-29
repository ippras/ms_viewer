use serde::{Deserialize, Serialize};

/// Events
#[derive(Clone, Copy, Debug, Default, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Events {
    pub(crate) reset_table_state: bool,
}

impl Events {
    pub(crate) fn new() -> Self {
        Self {
            reset_table_state: false,
        }
    }
}
