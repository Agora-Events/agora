import { SorobanRpc } from "@stellar/stellar-sdk";

// ─── Contract Error Code Mappings ───────────────────────────────────────────

/**
 * Maps Soroban contract error codes to human-readable messages.
 * Error codes are contract-specific integers defined in the contract source.
 *
 * event_registry contract errors:
 * - 101: SoldOut - Ticket tier has sold all available tickets
 * - 102: SaleNotActive - Event registration period is not open
 * - 103: InvalidTimestamp - Event date is invalid
 * - 104: AlreadyRegistered - User is already registered for this event
 *
 * ticket_payment contract errors:
 * - 101: SoldOut - This ticket tier is currently sold out
 * - 102: SaleNotActive - Event registration period has closed
 * - 103: InsufficientFunds - Insufficient USDC balance to complete purchase
 * - 104: PaymentFailed - Transaction failed during payment processing
 * - 105: InvalidTicketTier - Requested ticket tier does not exist
 * - 106: TransferError - Failed to transfer funds from buyer
 */
export const contractErrorMessages: Record<number, string> = {
  101: "This ticket tier is currently sold out.",
  102: "Event registration period has closed.",
  103: "Event date or timestamp is invalid.",
  104: "You are already registered for this event.",
  105: "The requested ticket tier does not exist.",
  106: "Failed to transfer funds. Please check your balance and try again.",
};

/**
 * Friendly error message to display when contract error code is not recognized.
 */
const DEFAULT_UNKNOWN_ERROR =
  "An error occurred during the transaction. Please try again or contact support.";

// ─── Error Extraction & Parsing ──────────────────────────────────────────────

export interface ParsedError {
  /** The human-readable error message to display to the user. */
  userMessage: string;
  /** The raw error code (e.g., 101) extracted from the contract error. */
  errorCode?: number;
  /** Full technical error details for logging. */
  technicalDetails: string;
  /** Whether this is a contract-specific error vs. generic network error. */
  isContractError: boolean;
}

/**
 * Parses a Soroban error object or string and extracts:
 * 1. Contract error code (if present)
 * 2. Human-readable error message
 * 3. Technical details for logging
 *
 * Handles multiple error formats:
 * - SorobanRpc simulation results with HostFunctionError
 * - TransactionStatus objects from getTransaction()
 * - Raw error strings
 * - Error objects with nested messages
 *
 * @param error - The Soroban error to parse
 * @returns ParsedError with user message and technical details
 */
export function parseSorobanError(error: unknown): ParsedError {
  const technicalDetails = String(error);

  // ─── Try to extract contract error from SorobanRpc error object

  if (error && typeof error === "object") {
    const errorObj = error as Record<string, unknown>;

    // Pattern 1: error.result?.resultMetaXdr contains error code
    if (errorObj.result && typeof errorObj.result === "object") {
      const result = errorObj.result as Record<string, unknown>;
      const errorCode = extractErrorCodeFromResult(result);
      if (errorCode !== null) {
        return {
          userMessage:
            contractErrorMessages[errorCode] || DEFAULT_UNKNOWN_ERROR,
          errorCode,
          technicalDetails,
          isContractError: true,
        };
      }
    }

    // Pattern 2: error.message is a string containing error code pattern
    if (typeof errorObj.message === "string") {
      const errorCode = extractErrorCodeFromString(errorObj.message);
      if (errorCode !== null) {
        return {
          userMessage:
            contractErrorMessages[errorCode] || DEFAULT_UNKNOWN_ERROR,
          errorCode,
          technicalDetails,
          isContractError: true,
        };
      }
    }

    // Pattern 3: Direct error.error_code field (some RPC implementations)
    if (typeof errorObj.error_code === "number") {
      const errorCode = errorObj.error_code;
      return {
        userMessage:
          contractErrorMessages[errorCode] || DEFAULT_UNKNOWN_ERROR,
        errorCode,
        technicalDetails,
        isContractError: true,
      };
    }

    // Pattern 4: Transaction failure with status
    if (errorObj.status === "FAILED" || errorObj.status === "ERROR") {
      const errorCode = extractErrorCodeFromTransactionStatus(errorObj);
      if (errorCode !== null) {
        return {
          userMessage:
            contractErrorMessages[errorCode] || DEFAULT_UNKNOWN_ERROR,
          errorCode,
          technicalDetails,
          isContractError: true,
        };
      }
    }
  }

  // ─── Try to extract from raw string (hex or decimal patterns)

  if (typeof error === "string") {
    const errorCode = extractErrorCodeFromString(error);
    if (errorCode !== null) {
      return {
        userMessage:
          contractErrorMessages[errorCode] || DEFAULT_UNKNOWN_ERROR,
        errorCode,
        technicalDetails,
        isContractError: true,
      };
    }

    // Check for common network errors
    if (
      error.includes("HostFunctionError") ||
      error.includes("Simulation error")
    ) {
      return {
        userMessage:
          "A transaction simulation error occurred. Please verify your account balance and try again.",
        technicalDetails,
        isContractError: false,
      };
    }

    if (error.includes("tx_failed")) {
      return {
        userMessage:
          "Your transaction was rejected by the network. Please check your account and try again.",
        technicalDetails,
        isContractError: false,
      };
    }

    if (error.includes("tx_bad_auth")) {
      return {
        userMessage:
          "Transaction authorization failed. Please ensure you signed with the correct account.",
        technicalDetails,
        isContractError: false,
      };
    }
  }

  // ─── Fallback for unrecognized errors

  return {
    userMessage: DEFAULT_UNKNOWN_ERROR,
    technicalDetails,
    isContractError: false,
  };
}

// ─── Helper Functions for Error Code Extraction ──────────────────────────────

/**
 * Attempts to extract an error code from a result object's XDR.
 * Looks for patterns like Error(Contract, #101) encoded in the result.
 */
function extractErrorCodeFromResult(result: Record<string, unknown>): number | null {
  // Most Soroban RPC implementations encode the error code in the result structure
  // This is a best-effort extraction; real implementations vary by SDK version
  if (typeof result.code === "number") {
    return result.code;
  }

  // Try resultMetaXdr (base64 encoded)
  if (typeof result.resultMetaXdr === "string") {
    const match = result.resultMetaXdr.match(/Error\(Contract,\s*#?(\d+)\)/);
    if (match) {
      return parseInt(match[1], 10);
    }
  }

  return null;
}

/**
 * Attempts to extract an error code from a string representation.
 * Handles patterns like:
 * - "Error(Contract, #101)"
 * - "Contract error: 101"
 * - Hex strings with error codes
 */
function extractErrorCodeFromString(str: string): number | null {
  // Pattern: Error(Contract, #101)
  let match = str.match(/Error\(Contract,\s*#?(\d+)\)/i);
  if (match) {
    return parseInt(match[1], 10);
  }

  // Pattern: Contract error: 101 or error code 101
  match = str.match(/(?:Contract\s+)?error(?:\s+code)?:\s*#?(\d+)/i);
  if (match) {
    return parseInt(match[1], 10);
  }

  // Pattern: #101 at end of error string
  match = str.match(/#(\d{3})$/);
  if (match) {
    const code = parseInt(match[1], 10);
    // Only consider 3-digit codes that look like error codes (100-199)
    if (code >= 100 && code <= 999) {
      return code;
    }
  }

  return null;
}

/**
 * Attempts to extract an error code from a transaction status object.
 * Looks for error codes in resultXdr, resultMetaXdr, and error_code fields.
 */
function extractErrorCodeFromTransactionStatus(
  status: Record<string, unknown>,
): number | null {
  // Try direct error_code field
  if (typeof status.error_code === "number") {
    return status.error_code;
  }

  // Try resultMetaXdr
  if (typeof status.resultMetaXdr === "string") {
    return extractErrorCodeFromString(status.resultMetaXdr);
  }

  // Try resultXdr
  if (typeof status.resultXdr === "string") {
    return extractErrorCodeFromString(status.resultXdr);
  }

  // Try error field if it's a string
  if (typeof status.error === "string") {
    return extractErrorCodeFromString(status.error);
  }

  return null;
}

// ─── Logging Helper ─────────────────────────────────────────────────────────

/**
 * Logs full technical error details to the console while being silent about
 * the user-facing message. Helps with debugging without exposing internal
 * error details to end users.
 *
 * @param error - The original error object/string
 * @param context - Optional context string (e.g., "ticket purchase", "event registration")
 */
export function logTechnicalError(
  error: unknown,
  context: string = "Soroban operation",
): void {
  const timestamp = new Date().toISOString();
  console.error(`[${timestamp}] Technical Error (${context}):`, error);
  if (error instanceof Error) {
    console.error("Stack trace:", error.stack);
  }
}

/**
 * Parses a Soroban error and logs technical details while returning the
 * friendly user message. This is the recommended approach for error handling
 * in UI components.
 *
 * @example
 * ```ts
 * try {
 *   await submitTransaction();
 * } catch (error) {
 *   const { userMessage } = parseAndLogError(error, "ticket purchase");
 *   toast.error(userMessage);
 * }
 * ```
 */
export function parseAndLogError(
  error: unknown,
  context: string = "transaction",
): ParsedError {
  const parsed = parseSorobanError(error);
  logTechnicalError(error, context);
  return parsed;
}
