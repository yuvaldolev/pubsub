use strum_macros::Display;

#[derive(Clone, Debug, Display)]
pub enum ConnectionKind {
    Publisher,
    Subscriber,
}
