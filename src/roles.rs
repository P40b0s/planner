use std::{convert::Infallible, fmt::Display, str::FromStr};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Roles
{
    Administrator,
    User,
    NonPrivileged,
}
impl Display for Roles
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        match self
        {
            Roles::Administrator => f.write_str("Administrator"),
            Roles::NonPrivileged => f.write_str("NonPrivileged"),
            Roles::User => f.write_str("User")
        }
    }
}
impl FromStr for Roles
{
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> 
    {
        match s
        {
            "Administrator" => Ok(Roles::Administrator),
            "User" => Ok(Roles::User),
            _ => Ok(Roles::NonPrivileged)
        }
    }
}