use crate::app::panes::settings::{Settings, Sort, TimeUnits};
use egui::util::cache::{ComputerMut, FrameCache};
use polars::{frame::DataFrame, prelude::*};
use polars_ext::column;
use std::hash::{Hash, Hasher};
use tracing::{error, trace, warn};
use uom::si::{
    f64::Time,
    time::{millisecond, minute, second},
};

/// Table computed
pub(crate) type Computed = FrameCache<DataFrame, Computer>;

/// Table computer
#[derive(Default)]
pub(crate) struct Computer;

impl ComputerMut<Key<'_>, DataFrame> for Computer {
    fn compute(&mut self, key: Key<'_>) -> DataFrame {
        let mut data_frame = key.data_frame.clone();
        error!(?data_frame);
        // {
        //     let data_frame = data_frame
        //         .clone()
        //         .lazy()
        //         .select([
        //             col("RetentionTime"),
        //             col("Masspectrum").alias("MassSpectrum"),
        //         ])
        //         // .explode(["Masspectrum"])
        //         // .unnest(["Masspectrum"])
        //         //     .sort(["MassToCharge"], Default::default())
        //         //     .group_by([col("RetentionTime")])
        //         //     .agg([as_struct(vec![
        //         //         col("MassToCharge").drop_nulls(),
        //         //         col("Signal").drop_nulls(),
        //         //     ])
        //         //     .alias("MassSpectrum")])
        //         .collect()
        //         .unwrap();
        //     let contents = bincode::serialize(&data_frame).unwrap();
        //     std::fs::write("df.msv.bin", &contents).unwrap();
        //     // // let contents = ron::ser::to_string_pretty(&data_frame, Default::default()).unwrap();
        //     // // std::fs::write("df.msv.ron", &contents).unwrap();
        //     error!(?data_frame);
        // }
        let mut lazy_frame = data_frame.lazy();
        if key.settings.filter_null {
            lazy_frame = lazy_frame.drop_nulls(Some(vec![col("MassToCharge"), col("Signal")]));
            // lazy_frame = lazy_frame.filter(col("MassSpectrum").list().len().neq(lit(0)));
        }
        if key.settings.signal.normalize {
            lazy_frame =
                lazy_frame.with_column(col("Signal").cast(DataType::Float64) / max("Signal"));
        }
        match key.settings.sort {
            Sort::RetentionTime => {
                lazy_frame = lazy_frame
                    .sort(["MassToCharge"], Default::default())
                    .group_by([col("RetentionTime")])
                    .agg([
                        as_struct(vec![col("MassToCharge"), col("Signal")]).alias("MassSpectrum")
                    ]);
                if !key.settings.explode {
                    lazy_frame = lazy_frame.with_columns([
                        col("MassSpectrum").list().len().name().suffix(".Count"),
                        col("MassSpectrum")
                            .list()
                            .eval(col("").struct_().field_by_name("MassToCharge"), true)
                            .list()
                            .min()
                            .alias("MassToCharge.Min"),
                        col("MassSpectrum")
                            .list()
                            .eval(col("").struct_().field_by_name("MassToCharge"), true)
                            .list()
                            .max()
                            .alias("MassToCharge.Max"),
                        col("MassSpectrum")
                            .list()
                            .eval(col("").struct_().field_by_name("Signal"), true)
                            .list()
                            .min()
                            .alias("Signal.Min"),
                        col("MassSpectrum")
                            .list()
                            .eval(col("").struct_().field_by_name("Signal"), true)
                            .list()
                            .max()
                            .alias("Signal.Max"),
                        col("MassSpectrum")
                            .list()
                            .eval(col("").struct_().field_by_name("Signal"), true)
                            .list()
                            .sum()
                            .alias("Signal.Sum"),
                    ]);
                }
                lazy_frame = lazy_frame.sort(["RetentionTime"], Default::default());
            }
            Sort::MassToCharge => {
                trace!(lazy_data_frame =? lazy_frame.clone().collect());
                lazy_frame = lazy_frame
                    .sort(["RetentionTime"], Default::default())
                    .group_by([col("MassToCharge").round(2)])
                    .agg([as_struct(vec![col("RetentionTime"), col("Signal")])
                        .alias("ExtractedIonChromatogram")]);
                if !key.settings.explode {
                    lazy_frame = lazy_frame.with_columns([
                        col("ExtractedIonChromatogram")
                            .list()
                            .len()
                            .name()
                            .suffix(".Count"),
                        col("ExtractedIonChromatogram")
                            .list()
                            .eval(col("").struct_().field_by_name("RetentionTime"), true)
                            .list()
                            .min()
                            .alias("RetentionTime.Min"),
                        col("ExtractedIonChromatogram")
                            .list()
                            .eval(col("").struct_().field_by_name("RetentionTime"), true)
                            .list()
                            .max()
                            .alias("RetentionTime.Max"),
                        col("ExtractedIonChromatogram")
                            .list()
                            .eval(col("").struct_().field_by_name("Signal"), true)
                            .list()
                            .min()
                            .alias("Signal.Min"),
                        col("ExtractedIonChromatogram")
                            .list()
                            .eval(col("").struct_().field_by_name("Signal"), true)
                            .list()
                            .max()
                            .alias("Signal.Max"),
                        col("ExtractedIonChromatogram")
                            .list()
                            .eval(col("").struct_().field_by_name("Signal"), true)
                            .list()
                            .sum()
                            .alias("Signal.Sum"),
                    ]);
                }
                lazy_frame = lazy_frame.sort(["MassToCharge"], Default::default());
            }
        };
        println!("lazy_frame gg: {}", lazy_frame.clone().collect().unwrap());
        let options = RollingOptionsFixedWindow {
            window_size: 5,
            min_periods: 3,
            weights: None,
            center: false,
            fn_params: None,
        };
        lazy_frame = lazy_frame
            .with_columns([
                col("RetentionTime").alias("x"),
                col("Signal.Sum").alias("y"),
            ])
            .with_columns([
                (col("x") * col("y"))
                    .rolling_mean(options.clone())
                    .alias("XYRollingMean"),
                col("x").rolling_mean(options.clone()).alias("XRollingMean"),
                col("y").rolling_mean(options.clone()).alias("YRollingMean"),
                col("x")
                    .rolling_std(options.clone())
                    .alias("XRollingMeanStd"),
                col("y")
                    .rolling_std(options.clone())
                    .alias("YRollingMeanStd"),
            ])
            .with_columns((pl.col("xy_mean") - pl.col("x_mean") * pl.col("y_mean")).alias("cov_xy"))
            .with_columns(
                (pl.col("cov_xy") / (pl.col("x_std") * pl.col("y_std"))).alias("correlation"),
            )
            .with_columns(
                (pl.col("correlation") * (pl.col("y_std") / pl.col("x_std"))).alias("slope"),
            )
            .with_columns(
                (pl.col("y_mean") - pl.col("slope") * pl.col("x_mean")).alias("intercept"),
            );
        data_frame = lazy_frame.collect().unwrap();
        trace!(?data_frame);
        data_frame
    }
}

/// Table key
#[derive(Clone, Copy, Debug)]
pub struct Key<'a> {
    pub(crate) data_frame: &'a DataFrame,
    pub(crate) settings: &'a Settings,
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // self.context.state.index.hash(state);
        self.settings.hash(state);
    }
}

// df.with_columns([
//     (pl.col("x") * pl.col("y")).rolling_mean(window_size).alias("xy_mean"),
//     pl.col("x").rolling_mean(window_size).alias("x_mean"),
//     pl.col("y").rolling_mean(window_size).alias("y_mean"),
//     pl.col("x").rolling_std(window_size).alias("x_std"),
//     pl.col("y").rolling_std(window_size).alias("y_std"),
// ]).with_columns(
//     (pl.col("xy_mean") - pl.col("x_mean") * pl.col("y_mean")).alias("cov_xy")
// ).with_columns(
//     (pl.col("cov_xy") / (pl.col("x_std") * pl.col("y_std"))).alias("correlation")
// ).with_columns(
//     (pl.col("correlation") * (pl.col("y_std") / pl.col("x_std"))).alias("slope"),
// ).with_columns(
//     (pl.col("y_mean") - pl.col("slope") * pl.col("x_mean")).alias("intercept"),
// )

// pub fn retention_time(units: TimeUnits) -> impl Fn(&Series) -> PolarsResult<Series> {
//     move |series| {
//         Ok(series
//             .cast(&DataType::Float64)?
//             .f64()?
//             .iter()
//             .map(|value| {
//                 let time = Time::new::<millisecond>(value?);
//                 Some(match units {
//                     TimeUnits::Millisecond => time.get::<millisecond>(),
//                     TimeUnits::Second => time.get::<second>(),
//                     TimeUnits::Minute => time.get::<minute>(),
//                 })
//             })
//             .collect::<Float64Chunked>()
//             .into_series())
//     }
// }

// fn retention_time(retention_time: RetentionTime) -> Expr {
//     // col("").struct_().field_by_name("Signal")
// }

// fn signal() -> Expr {
//     col("").struct_().field_by_name("Signal")
// }
