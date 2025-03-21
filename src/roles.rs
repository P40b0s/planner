use std::{convert::Infallible, fmt::Display, str::FromStr};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Role
{
    Administrator,
    User,
    NonPrivileged,
}
impl Display for Role
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        match self
        {
            Role::Administrator => f.write_str("Administrator"),
            Role::NonPrivileged => f.write_str("NonPrivileged"),
            Role::User => f.write_str("User")
        }
    }
}
impl FromStr for Role
{
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> 
    {
        match s
        {
            "Administrator" => Ok(Role::Administrator),
            "User" => Ok(Role::User),
            _ => Ok(Role::NonPrivileged)
        }
    }
}