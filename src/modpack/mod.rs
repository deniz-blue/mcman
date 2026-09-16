pub mod mrpack;
pub mod packwiz;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Client,
    Server,
}
