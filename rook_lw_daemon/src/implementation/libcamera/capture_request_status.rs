
/// Mirrors the C `CaptureRequestStatus` enum.
///
/// Keep the discriminants in sync with the C API.
#[repr(i32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum CaptureRequestStatus {
    CaptureRequestInitial = 0,
    CaptureRequestPending = 1,
    CaptureRequestComplete = 2,
    CaptureRequestCancelled = 3,
}

impl CaptureRequestStatus {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::CaptureRequestInitial),
            1 => Some(Self::CaptureRequestPending),
            2 => Some(Self::CaptureRequestComplete),
            3 => Some(Self::CaptureRequestCancelled),
            _ => None,
        }
    }

    pub fn as_i32(self) -> i32 {
        self as i32
    }
}


impl TryFrom<i32> for CaptureRequestStatus {
    type Error = i32;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Self::from_i32(value).ok_or(value)
    }
}

impl TryFrom<u32> for CaptureRequestStatus {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value > i32::MAX as u32 {
            return Err(value);
        }
        CaptureRequestStatus::try_from(value as i32).map_err(|_| value)
    }
}

#[cfg(test)]
mod tests {
    use super::CaptureRequestStatus;

    #[test]
    fn capture_request_status_round_trip() {
        let status = CaptureRequestStatus::CaptureRequestComplete;
        assert_eq!(CaptureRequestStatus::from_i32(status.as_i32()), Some(status));
    }

    #[test]
    fn capture_request_status_unknown_is_none() {
        assert_eq!(CaptureRequestStatus::from_i32(123), None);
    }
}
