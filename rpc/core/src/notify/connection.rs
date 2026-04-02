use crate::Notification;

pub type ChannelConnection = lmt_notify::connection::ChannelConnection<Notification>;
pub use lmt_notify::connection::ChannelType;
