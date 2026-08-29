import { NextRequest, NextResponse } from "next/server";
import { prisma } from "@/lib/prisma";
import { mintTicket } from "@/utils/stellar";
import crypto from "crypto";
import { Keypair } from "@stellar/stellar-sdk";
import { withErrorHandler } from "@/lib/api-handler";
import { throwApiError, ApiError } from "@/lib/api-errors";

type TicketRequestBody = {
  eventId?: string;
  quantity?: number;
  buyerWallet?: string;
  recipientWallet?: string; // Optional: if provided, ticket goes to recipient instead of buyer
  attribution?: { utmSource?: unknown; utmMedium?: unknown; utmCampaign?: unknown };
};

export const POST = withErrorHandler(async (request: NextRequest) => {
  let payload: TicketRequestBody;
  try {
    payload = await request.json();
  } catch {
    throwApiError("Invalid JSON payload", 400);
  }

  const { eventId, quantity, buyerWallet, recipientWallet } = payload;
  const attribution = normalizeAttribution(payload.attribution);

  // Validation
  if (!eventId || typeof eventId !== "string") {
    throwApiError("Invalid eventId", 400);
  }

  // Ensure quantity is a valid number and cast it for TypeScript safety
  if (typeof quantity !== "number" || !Number.isInteger(quantity) || quantity <= 0) {
    throwApiError("Invalid quantity", 400);
  }

  const qty = quantity as number;

  if (!buyerWallet || typeof buyerWallet !== "string") {
    throwApiError("Invalid buyerWallet", 400);
  }

  // If buyerWallet looks like an email address, generate a custodial
  // Stellar keypair server-side and store the encrypted private key.
  // Detection heuristic: presence of an '@' character. Wallets on Stellar
  // start with 'G' so emails are distinguished.
  let buyerIdentifier = buyerWallet as string;
  if (buyerIdentifier.includes("@")) {
    const email = buyerIdentifier.trim().toLowerCase();

    // Generate a fresh Stellar keypair
    const pair = Keypair.random();
    const publicKey = pair.publicKey();
    const privateKey = pair.secret();

    // Derive a 32-byte AES key from the server-side secret + email
    const serverSecret = process.env.CUSTODIAL_ENCRYPTION_KEY;
    if (!serverSecret) {
      console.error("Missing CUSTODIAL_ENCRYPTION_KEY env var");
      throwApiError("Server encryption key not configured", 500);
    }
    /*
     Security note — Custodial wallet encryption

     - Scheme: AES-256-GCM
     - Key derivation: HMAC-SHA256(email) keyed by `CUSTODIAL_ENCRYPTION_KEY`.
       This produces a 32-byte symmetric key unique to the email + server secret.
     - Stored blob: base64(iv || authTag || ciphertext)

     Key rotation plan:
     - Rotate `CUSTODIAL_ENCRYPTION_KEY` by running a migration which
       decrypts all stored blobs using the old secret and re-encrypts them
       with the new secret. For higher assurance, use a KMS-backed master
       key (envelope encryption) so rotation does not require bulk decryption
       on the application server.

     Important: The plaintext private key is never logged or persisted.
     Decryption should only occur in a tightly-scoped signing context.
    */
    // Use HMAC-SHA256(email) keyed by server secret to derive the AES key.
    // This yields a stable, per-email key that can be rotated by changing
    // the server secret. The encrypted blob includes a random IV and auth tag.
    const aesKey = crypto.createHmac("sha256", serverSecret).update(email).digest();

    // AES-256-GCM encryption
    const iv = crypto.randomBytes(12);
    const cipher = crypto.createCipheriv("aes-256-gcm", aesKey, iv);
    const encrypted = Buffer.concat([cipher.update(privateKey, "utf8"), cipher.final()]);
    const authTag = cipher.getAuthTag();

    const payload = Buffer.concat([iv, authTag, encrypted]).toString("base64");

    try {
      await prisma.custodialWallet.create({
        data: {
          email,
          publicKey,
          encryptedPrivateKey: payload,
        },
      });
    } catch (err) {
      console.error("Failed to store custodial wallet:", err);
      throwApiError("Failed to create custodial wallet", 500);
    }

    // Use the newly created public key as the buyerWallet for downstream flows
    buyerWallet = publicKey;
  }

  if (recipientWallet !== undefined && recipientWallet !== null && typeof recipientWallet !== "string") {
    throwApiError("Invalid recipientWallet", 400);
  }

  const ownerWallet = recipientWallet || buyerWallet;

  const event = await prisma.event.findUnique({
    where: { id: eventId },
  });

  if (!event) {
    throwApiError("Event not found", 404);
  }

  if (event.mintedTickets + qty > event.totalTickets) {
    throwApiError("Not enough tickets available", 409);
  }

  try {
    const mintResult = await mintTicket(eventId, ownerWallet, qty);

    await prisma.$transaction([
      prisma.event.update({
        where: { id: eventId },
        data: { mintedTickets: { increment: qty } },
      }),
      prisma.ticket.create({
        data: {
          stellarId: mintResult.ticketId,
          eventId,
          buyerWallet,
          ownerWallet,
          quantity: qty,
          utmSource: attribution.utmSource,
          utmMedium: attribution.utmMedium,
          utmCampaign: attribution.utmCampaign,
        },
      }),
    ]);

    return NextResponse.json(
      {
        ticketId: mintResult.ticketId,
        transactionXdr: mintResult.transactionXdr,
        requiresSignature: mintResult.unsigned !== false,
      },
      { status: 200 },
    );
  } catch (error) {
    if (error instanceof ApiError) throw error;
    console.error("Minting Error:", error);
    throwApiError("Failed to mint ticket", 502);
  }
});

function normalizeAttribution(
  value: TicketRequestBody["attribution"],
): { utmSource?: string; utmMedium?: string; utmCampaign?: string } {
  if (value === undefined) return {};
  if (!value || typeof value !== "object") {
    throwApiError("Invalid attribution", 400);
  }

  const normalize = (field: unknown, name: string) => {
    if (field === undefined || field === null || field === "") return undefined;
    if (typeof field !== "string" || field.length > 255) {
      throwApiError(`Invalid ${name}`, 400);
    }
    return field.trim() || undefined;
  };

  return {
    utmSource: normalize(value.utmSource, "utm_source"),
    utmMedium: normalize(value.utmMedium, "utm_medium"),
    utmCampaign: normalize(value.utmCampaign, "utm_campaign"),
  };
}
