pub type ServiceID = u16;
pub type InstanceID = u16;
pub type MajorVersion = u8;
pub type MinorVersion = u32;
pub type MethodID = u16;
pub type ClientID = u16;
pub type SessionID = u16;
pub type ProtocolVersion = u8;
pub type InterfaceVersion = u8;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum MessageType {
    Request,
    RequestNoReturn,
    Notification,
    RequestAck,
    RequestNoReturnAck,
    NotificationAck,
    Response,
    Error,
    ResponseAck,
    ErrorAck,
    Unknown(u8),
}

impl MessageType {

    pub fn value(&self) -> u8 {
        use MessageType::*;
        match self {
            Request             => 0x00,
            RequestNoReturn     => 0x01,
            Notification        => 0x02,
            RequestAck          => 0x40,
            RequestNoReturnAck  => 0x41,
            NotificationAck     => 0x42,
            Response            => 0x80,
            Error               => 0x81,
            ResponseAck         => 0xc0,
            ErrorAck            => 0xc1,
            Unknown(v)     => *v,
        }
    }

    pub fn from_u8(value: u8) -> MessageType {
        match value {
            0x00 => MessageType::Request,
            0x01 => MessageType::RequestNoReturn,
            0x02 => MessageType::Notification,
            0x40 => MessageType::RequestAck,
            0x41 => MessageType::RequestNoReturnAck,
            0x42 => MessageType::NotificationAck,
            0x80 => MessageType::Response,
            0x81 => MessageType::Error,
            0xc0 => MessageType::ResponseAck,
            0xc1 => MessageType::ErrorAck,
            _ => MessageType::Unknown(value),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum ReturnCode {
    Ok,
    NotOk,
    UnknownService,
    UnknownMethod,
    NotReady,
    NotReachable,
    Timeout,
    WrongProtocolVersion,
    WrongInterfaceVersion,
    MalformedMessage,
    WrongMessageType,
    Unknown(u8),
}

impl ReturnCode {

    pub fn value(&self) -> u8 {
        use ReturnCode::*;
        match self {
            Ok                      => 0x00,
            NotOk                   => 0x01,
            UnknownService          => 0x02,
            UnknownMethod           => 0x03,
            NotReady                => 0x04,
            NotReachable            => 0x05,
            Timeout                 => 0x06,
            WrongProtocolVersion    => 0x07,
            WrongInterfaceVersion   => 0x08,
            MalformedMessage        => 0x09,
            WrongMessageType        => 0x0a,
            Unknown(v)         => *v,
        }
    }

    pub fn from_u8(value: u8) -> ReturnCode {
        match value {
            0x00 => ReturnCode::Ok,
            0x01 => ReturnCode::NotOk,
            0x02 => ReturnCode::UnknownService,
            0x03 => ReturnCode::UnknownMethod,
            0x04 => ReturnCode::NotReady,
            0x05 => ReturnCode::NotReachable,
            0x06 => ReturnCode::Timeout,
            0x07 => ReturnCode::WrongProtocolVersion,
            0x08 => ReturnCode::WrongInterfaceVersion,
            0x09 => ReturnCode::MalformedMessage,
            0x0a => ReturnCode::WrongMessageType,
            _ => ReturnCode::Unknown(value),
        }
    }
}

pub const DEFAULT_MAJOR: MajorVersion = 0x00;
pub const DEFAULT_MINOR: MinorVersion = 0x00000000;
pub const ANY_SERVICE: ServiceID = 0xffff;
pub const ANY_INSTANCE: InstanceID = 0xffff;
pub const ANY_MAJOR: MajorVersion = 0xff;
pub const ANY_MINOR: MinorVersion = 0xffffffff;
pub const ANY_METHOD: MethodID = 0xffff;