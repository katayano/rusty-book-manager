use strum::{AsRefStr, EnumIter, EnumString};

#[derive(Debug, EnumIter, EnumString, AsRefStr, Default, PartialEq, Eq)]
pub enum Role {
    Admin,
    #[default]
    User,
}
