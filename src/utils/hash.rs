use metadata::{Metadata, polars::MetaDataFrame};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
};

pub type HashedMetaDataFrame = MetaDataFrame<Metadata, HashedDataFrame>;

/// Hashed data frame
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct HashedDataFrame {
    pub data_frame: DataFrame,
    pub hash: u64,
}

impl HashedDataFrame {
    pub const EMPTY: Self = Self {
        data_frame: DataFrame::empty(),
        hash: 0,
    };

    pub fn new(mut data_frame: DataFrame) -> PolarsResult<Self> {
        let hash = hash_data_frame(&mut data_frame)?;
        Ok(Self { data_frame, hash })
    }

    pub fn update(&mut self) -> PolarsResult<()> {
        self.hash = hash_data_frame(&mut self.data_frame)?;
        Ok(())
    }
}

impl Deref for HashedDataFrame {
    type Target = DataFrame;

    fn deref(&self) -> &Self::Target {
        &self.data_frame
    }
}

impl DerefMut for HashedDataFrame {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data_frame
    }
}

impl Eq for HashedDataFrame {}

impl PartialEq for HashedDataFrame {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Hash for HashedDataFrame {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

pub fn hash_data_frame(data_frame: &mut DataFrame) -> PolarsResult<u64> {
    Ok(data_frame
        .with_row_index(PlSmallStr::from_static("Index"), None)?
        .hash_rows(Some(PlSeedableRandomStateQuality::fixed()))?
        .xor_reduce()
        .unwrap_or_default())
}

pub fn hash_expr(expr: Expr) -> Expr {
    expr.hash(1, 2, 3, 4).alias("Hash")
}
