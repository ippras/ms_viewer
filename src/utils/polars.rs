use crate::r#const::{MEAN, SAMPLE, STANDARD_DEVIATION};
use polars::prelude::*;
use polars_ext::prelude::*;

/// Mean and standard deviation options
#[derive(Clone, Copy, Debug, Default)]
pub struct MeanAndStandardDeviationOptions {
    pub(crate) ddof: u8,
    pub(crate) percent: bool,
    pub(crate) precision: usize,
    pub(crate) significant: bool,
}

/// Mean and standard deviation
pub fn mean_and_standard_deviation(
    array: Expr,
    options: impl Into<MeanAndStandardDeviationOptions>,
) -> Expr {
    let options = options.into();
    as_struct(vec![
        array
            .clone()
            .arr()
            .mean()
            .percent(options.percent)
            .precision(options.precision, options.significant)
            .alias(MEAN),
        array
            .clone()
            .arr()
            .std(options.ddof)
            .percent(options.percent)
            .precision(options.precision + 1, options.significant)
            .alias(STANDARD_DEVIATION),
        array
            .arr()
            .eval(
                element()
                    .percent(options.percent)
                    .precision(options.precision, options.significant),
                false,
            )
            .alias(SAMPLE),
    ])
}
