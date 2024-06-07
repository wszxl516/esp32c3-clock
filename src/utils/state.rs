use std::fmt::{Display, Formatter};

use crate::utils::inert_lf;

#[derive(Clone, Debug)]
#[repr(u8)]
pub enum Btn{
    Left,
    Right,
    Ok,
    Exit,
    Boot
}
#[derive(Clone, Debug)]
#[repr(u8)]
pub enum Http{
    Request(String)
}

#[derive(Clone, Debug)]
#[repr(u8)]
pub enum State {
    Btn(Btn),
    Http(Http),
    Msg(String)
}

impl Display for State{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&*inert_lf(format!("{:?}", self).as_str(), 26))
    }
}
