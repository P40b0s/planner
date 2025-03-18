use std::fmt::Display;

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