use crate::{
    app::{computers::MINUTES, states::pane::settings::Settings},
    r#const::*,
    utils::hash::HashedDataFrame,
};
use const_format::formatcp;
use egui::util::cache::{ComputerMut, FrameCache};
use polars::prelude::*;
use polars_ext::expr::ExprExt;
use tracing::{instrument, trace};

/// Table computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Table computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    #[instrument(skip(self), err)]
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        lazy_frame = format(lazy_frame, key);
        let data_frame = lazy_frame.collect()?;
        trace!(?data_frame);
        Ok(HashedDataFrame::new(data_frame)?)
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
    }
}

/// Table key
#[derive(Clone, Copy, Hash, Debug)]
pub struct Key<'a> {
    pub(crate) frame: &'a HashedDataFrame,
    pub(crate) precision: usize,
    pub(crate) significant: bool,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &Settings) -> Self {
        Self {
            frame,
            precision: settings.precision,
            significant: settings.significant,
        }
    }
}

/// Table value
type Value = HashedDataFrame;

/// Format
fn format(lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    lazy_frame.with_columns([
        // Retention time
        col(RETENTION_TIME)
            .cast(DataType::Duration(TimeUnit::Milliseconds))
            .to_physical()
            / lit(MINUTES),
        // Mass spectrum
        col(MASS_SPECTRUM)
            .list()
            .eval(element().struct_().with_fields(vec![
                    element()
                        .struct_()
                        .field_by_name(MASS_TO_CHARGE)
                        .precision(key.precision, key.significant),
                    element()
                        .struct_()
                        .field_by_name(SIGNAL)
                        .precision(key.precision, key.significant),
                ])),
        // Meta
        col(META).struct_().with_fields(vec![
            col(META)
                .struct_()
                .field_by_names([
                    formatcp!(r#"^{MASS_TO_CHARGE}.*$"#),
                    formatcp!(r#"^{SIGNAL}.*$"#),
                    formatcp!(r#"^{ROLLING}.*$"#),
                ])
                .precision(key.precision, key.significant),
        ]),
    ])
}
