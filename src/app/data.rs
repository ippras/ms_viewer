use crate::{
    r#const::*,
    utils::hash::{HashedDataFrame, HashedMetaDataFrame},
};
use anyhow::Result;
use metadata::{Metadata, polars::MetaDataFrame};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display, Formatter},
    fs::write,
    path::Path,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Data {
    pub(crate) frame: HashedMetaDataFrame,
}

impl Data {
    pub(crate) fn save(&self, path: impl AsRef<Path>, format: Format) -> Result<()> {
        let data_frame = self.frame.data.select([RETENTION_TIME, MASS_SPECTRUM])?;
        match format {
            Format::Bin => {
                // let contents = bincode::serialize(&data_frame)?;
                // write(path, contents)?;
            }
            Format::Ron => {
                let contents = ron::ser::to_string_pretty(&data_frame, Default::default())?;
                write(path, contents)?;
            }
        }
        Ok(())
    }
}

impl Display for Data {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.frame.data.data_frame, f)
    }
}

impl Default for Data {
    fn default() -> Self {
        Self {
            frame: MetaDataFrame::new(Metadata::default(), HashedDataFrame::EMPTY),
        }
        // Self {
        //     frame: DataFrame::empty_with_schema(&Schema::from_iter([
        //         Field::new(PlSmallStr::from_static(RETENTION_TIME), DataType::String),
        //         Field::new(
        //             PlSmallStr::from_static(MASS_SPECTRUM),
        //             DataType::List(Box::new(DataType::Struct(vec![
        //                 Field::new(PlSmallStr::from_static(MASS_TO_CHARGE), DataType::String),
        //                 Field::new(PlSmallStr::from_static(SIGNAL), DataType::String),
        //             ]))),
        //         ),
        //     ])),
        // }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) enum Format {
    #[default]
    Bin,
    Ron,
}
