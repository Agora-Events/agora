use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ProSubscriptionError {
    /// Contract has already been initialized
    AlreadyInitialized = 1,
    /// Contract has not been initialized
    NotInitialized = 2,
    /// Caller is not the admin
    Unauthorized = 3,
    /// Subscription not found for this organizer
    SubscriptionNotFound = 4,
    /// Subscription has expired
    SubscriptionExpired = 5,
    /// Subscription is not active
    SubscriptionInactive = 6,
    /// Invalid subscription tier
    InvalidTier = 7,
    /// Invalid price (must be positive)
    InvalidPrice = 8,
    /// Insufficient payment amount
    InsufficientPayment = 9,
    /// Arithmetic overflow or underflow
    ArithmeticError = 10,
    /// Token transfer failed
    TransferFailed = 11,
    /// Invalid address provided
    InvalidAddress = 12,
    /// Subscription already exists and is active
    SubscriptionAlreadyActive = 13,
}

impl core::fmt::Display for ProSubscriptionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ProSubscriptionError::AlreadyInitialized => {
                write!(f, "Contract already initialized")
            }
            ProSubscriptionError::NotInitialized => {
                write!(f, "Contract not initialized")
            }
            ProSubscriptionError::Unauthorized => {
                write!(f, "Caller is not authorized to perform this action")
            }
            ProSubscriptionError::SubscriptionNotFound => {
                write!(f, "Subscription not found for this organizer")
            }
            ProSubscriptionError::SubscriptionExpired => {
                write!(f, "Subscription has expired")
            }
            ProSubscriptionError::SubscriptionInactive => {
                write!(f, "Subscription is not active")
            }
            ProSubscriptionError::InvalidTier => {
                write!(f, "Invalid subscription tier")
            }
            ProSubscriptionError::InvalidPrice => {
                write!(f, "Price must be a positive value")
            }
            ProSubscriptionError::InsufficientPayment => {
                write!(f, "Insufficient payment amount provided")
            }
            ProSubscriptionError::ArithmeticError => {
                write!(f, "Arithmetic overflow or underflow occurred")
            }
            ProSubscriptionError::TransferFailed => {
                write!(f, "Token transfer failed")
            }
            ProSubscriptionError::InvalidAddress => {
                write!(f, "Invalid address provided")
            }
            ProSubscriptionError::SubscriptionAlreadyActive => {
                write!(
                    f,
                    "An active subscription already exists for this organizer"
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProSubscriptionError;
    use core::fmt::Write;

    struct CountWriter(usize);

    impl Write for CountWriter {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            self.0 += s.len();
            Ok(())
        }
    }

    #[test]
    fn display_messages_are_non_empty() {
        let variants = [
            ProSubscriptionError::AlreadyInitialized,
            ProSubscriptionError::NotInitialized,
            ProSubscriptionError::Unauthorized,
            ProSubscriptionError::SubscriptionNotFound,
            ProSubscriptionError::SubscriptionExpired,
            ProSubscriptionError::SubscriptionInactive,
            ProSubscriptionError::InvalidTier,
            ProSubscriptionError::InvalidPrice,
            ProSubscriptionError::InsufficientPayment,
            ProSubscriptionError::ArithmeticError,
            ProSubscriptionError::TransferFailed,
            ProSubscriptionError::InvalidAddress,
            ProSubscriptionError::SubscriptionAlreadyActive,
        ];

        for error in variants {
            let mut writer = CountWriter(0);
            write!(&mut writer, "{error}").unwrap();
            assert!(writer.0 > 0);
        }
    }
}
