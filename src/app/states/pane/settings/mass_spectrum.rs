use serde::{Deserialize, Serialize};

/// Mass spectrum
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub struct MassSpectrum {
    pub(crate) sort: Sort,
}

impl MassSpectrum {
    pub(crate) fn new() -> Self {
        Self {
            sort: Sort::MassToCharge,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) enum Sort {
    #[default]
    MassToCharge,
    Signal,
}

impl Sort {
    pub(crate) fn text(&self) -> &'static str {
        match self {
            Self::MassToCharge => "Mass to charge",
            Self::Signal => "Signal",
        }
    }

    pub(crate) fn hover_text(&self) -> &'static str {
        match self {
            Self::MassToCharge => "Sort by mass to charge",
            Self::Signal => "Sort by signal",
        }
    }
}
