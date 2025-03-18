use std::fmt::Display;
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