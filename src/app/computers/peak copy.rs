use crate::{
    app::{
        computers::MINUTES,
        states::pane::settings::{Settings, Threshold},
    },
    r#const::*,
    utils::hash::HashedDataFrame,
};
use const_format::formatcp;
use egui::util::cache::{ComputerMut, FrameCache};
use polars::prelude::*;
use scirs2::spatial::cosine;
use std::iter::zip;
use tracing::{instrument, trace};

/// Peak computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Peak computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    #[instrument(skip(self), err)]
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        lazy_frame = compute(lazy_frame, key);
        println!("lazy_frame E0: {}", lazy_frame.clone().collect().unwrap());
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

/// Peak key
#[derive(Clone, Copy, Hash, Debug)]
pub struct Key<'a> {
    pub(crate) frame: &'a HashedDataFrame,
    pub(crate) threshold: Threshold,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &Settings) -> Self {
        Self {
            frame,
            threshold: settings.threshold,
        }
    }
}

/// Peak value
type Value = HashedDataFrame;

/// Format
fn compute(lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    let index = (col(RETENTION_TIME) - lit(key.threshold.retention_time.0 * MINUTES))
        .abs()
        .arg_min();
    let expr = as_struct(vec![
        col(MASS_SPECTRUM).alias("SOURCE"),
        col(MASS_SPECTRUM).get(index.clone()).alias("TAGRET"),
    ])
    .apply(cosine_distance, |_, _field| {
        Ok(Field::new(PlSmallStr::EMPTY, DataType::Float64))
    })
    .lt(key.threshold.factor.0);
    // Дистанция (cos угла) менее чем 25%
    lazy_frame.with_columns([col(META)
        .struct_()
        .with_fields(vec![col(META).struct_().field_by_name(THRESHOLD).and(expr)])])
}

fn cosine_distance(column: Column) -> PolarsResult<Column> {
    let mut tagret = df! {
        MASS_TO_CHARGE => Series::new_empty(PlSmallStr::from_static(MASS_TO_CHARGE), &DataType::Float64),
        SIGNAL => Series::new_empty(PlSmallStr::from_static(SIGNAL), &DataType::Float64),
    }?;
    let fields = column.struct_()?.fields_as_series();
    Ok(zip(fields[0].list()?, fields[1].list()?).into_iter().map(|(source, tagret)| {
        let source = {
            let series = source.ok_or(polars_err!(NoData: "SOURCE"))?;
            let r#struct = series.struct_()?;
            df! {
                MASS_TO_CHARGE => r#struct.field_by_name(MASS_TO_CHARGE)?.round(0, RoundMode::HalfToEven)?.f64()?.clone(),
                SIGNAL => r#struct.field_by_name(SIGNAL)?.f64()?.clone(),
            }?
        };
        let tagret = {
            let series = tagret.ok_or(polars_err!(NoData: "TAGRET"))?;
            let r#struct = series.struct_()?;
            df! {
                MASS_TO_CHARGE => r#struct.field_by_name(MASS_TO_CHARGE)?.round(0, RoundMode::HalfToEven)?.f64()?.clone(),
                SIGNAL => r#struct.field_by_name(SIGNAL)?.f64()?.clone(),
            }?
        };
        let join = source.join(
            &tagret,
            [MASS_TO_CHARGE],
            [MASS_TO_CHARGE],
            JoinArgs::new(JoinType::Full).with_coalesce(JoinCoalesce::CoalesceColumns),
            None,
        )?;
        let a = join[SIGNAL].f64()?.fill_null_with_values(0.0)?.into_no_null_iter().collect::<Vec<_>>();
        let b = join[formatcp!("{SIGNAL}_right")]
            .f64()?
            .fill_null_with_values(0.0)?.into_no_null_iter().collect::<Vec<_>>();
        Ok(Some(cosine(&a, &b)))
    }).collect::<PolarsResult<Float64Chunked>>()?.into_column())
}
