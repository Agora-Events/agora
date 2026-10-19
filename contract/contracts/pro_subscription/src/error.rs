use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ProSubscriptionError {
    // Codes 3, 6, 7, 9, 11 and 14 were retired in #1583 (never returned).
    // Do not reuse them: clients may still map the old numbers.
    /// Contract has already been initialized
    AlreadyInitialized = 1,
    /// Contract has not been initialized
    NotInitialized = 2,
    /// Subscription not found for this organizer
    SubscriptionNotFound = 4,
    /// Subscription has expired
    SubscriptionExpired = 5,
    /// Invalid price (must be positive)
    InvalidPrice = 8,
    /// Arithmetic overflow or underflow
    ArithmeticError = 10,
    /// Invalid address provided
    InvalidAddress = 12,
    /// Subscription already exists and is active
    SubscriptionAlreadyActive = 13,
    /// new_admin is the same as the current admin
    SameAdmin = 15,
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
            ProSubscriptionError::SubscriptionNotFound => {
                write!(f, "Subscription not found for this organizer")
            }
            ProSubscriptionError::SubscriptionExpired => {
                write!(f, "Subscription has expired")
            }
            ProSubscriptionError::InvalidPrice => {
                write!(f, "Price must be a positive value")
            }
            ProSubscriptionError::ArithmeticError => {
                write!(f, "Arithmetic overflow or underflow occurred")
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
            ProSubscriptionError::SameAdmin => {
                write!(f, "New admin is the same as the current admin")
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
            ProSubscriptionError::SubscriptionNotFound,
            ProSubscriptionError::SubscriptionExpired,
            ProSubscriptionError::InvalidPrice,
            ProSubscriptionError::ArithmeticError,
            ProSubscriptionError::InvalidAddress,
            ProSubscriptionError::SubscriptionAlreadyActive,
            ProSubscriptionError::SameAdmin,
        ];

        for error in variants {
            let mut writer = CountWriter(0);
            write!(&mut writer, "{error}").unwrap();
            assert!(writer.0 > 0);
        }
    }

    #[test]
    fn retained_variants_keep_their_original_codes() {
        // Pruning (#1583) must not renumber surviving variants.
        assert_eq!(ProSubscriptionError::SubscriptionNotFound as u32, 4);
        assert_eq!(ProSubscriptionError::InvalidPrice as u32, 8);
        assert_eq!(ProSubscriptionError::SameAdmin as u32, 15);
    }
}
