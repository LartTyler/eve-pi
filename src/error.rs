use std::io;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("missing item with ID '{0}'")]
    MissingItem(String),

    #[error("io error: {0}")]
    IO(#[from] io::Error),

    #[error("deserialize error: {0}")]
    Deserialize(#[from] serde_yaml::Error),
}

impl Error {
    pub fn create_missing_item<Id>(item_id: Id) -> Self
    where
        Id: ToString,
    {
        Self::MissingItem(item_id.to_string())
    }
}
