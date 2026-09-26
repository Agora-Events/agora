/**
 * nfcService.ts
 *
 * Tap-to-enter over NFC. Organizer devices act as the NFC *reader*
 * (NDEF) and attendee devices answer by emulating an NDEF Type-4 tag
 * carrying a signed ticket token.
 *
 * Library split (verified against installed package typings, not
 * assumed): `react-native-nfc-manager` is reader-only - it has no card
 * emulation API. Card emulation on Android instead goes through
 * `react-native-hce`, which emulates an NDEF Type-4 tag (not raw custom
 * AID / APDU exchanges). So both platforms converge on the same wire
 * format: a plain NDEF text record containing the signed token, read
 * via `NfcTech.Ndef` on both Android and iOS.
 *
 * Platform notes:
 *  - Android: `react-native-hce`'s `HCESession` broadcasts the token as
 *    an NDEF Type-4 tag. There's no per-tap "on SELECT" hook in this
 *    library, so - like the existing rotating QR payload in
 *    app/ticket/[id].tsx - we re-sign and refresh the emulated content
 *    on a timer instead.
 *  - iOS: CoreNFC does not expose card-emulation APIs to third-party
 *    apps at all, so the attendee's token must live on a rewritable
 *    NDEF tag/wristband instead of being emitted live from the phone.
 */

import { Platform } from 'react-native';
import NfcManager, { NfcTech, Ndef } from 'react-native-nfc-manager';
import { HCESession, NFCTagType4, NFCTagType4NDEFContentType } from 'react-native-hce';
import { Keypair } from '@stellar/stellar-sdk';
import * as SecureStore from 'expo-secure-store';

/** How often the emulated NDEF tag content is re-signed, mirrors QR_REFRESH_INTERVAL_MS. */
const HCE_REFRESH_INTERVAL_MS = 15000;
/** Budget for the organizer's read step, per the <200ms requirement. */
const TAP_TIMEOUT_MS = 200;

export interface NfcTicketToken {
  ticketId: string;
  timestamp: number;
  signature: string;
}

let started = false;

/** Boots the native NFC reader adapter once per app session. */
async function ensureStarted(): Promise<void> {
  if (started) return;
  await NfcManager.start();
  started = true;
}

/** Signs a fresh, short-lived ticket token - mirrors the QR path in ticket/[id].tsx. */
async function signTicketToken(ticketId: string): Promise<NfcTicketToken> {
  const secret = await SecureStore.getItemAsync('privateKey');
  if (!secret) throw new Error('No wallet key available to sign ticket token');

  const timestamp = Math.floor(Date.now() / 1000);
  const payload = JSON.stringify({ ticketId, timestamp });
  const keypair = Keypair.fromSecret(secret);
  const signature = keypair.sign(Buffer.from(payload)).toString('base64');

  return { ticketId, timestamp, signature };
}

/**
 * ATTENDEE SIDE (Android only): emulates an NDEF tag whose content is a
 * freshly signed ticket token, refreshed on a timer. Returns a teardown
 * function - call it once the attendee leaves the gate.
 */
export async function startHce(ticketId: string): Promise<() => void> {
  if (Platform.OS !== 'android') {
    throw new Error('Host Card Emulation is only available on Android');
  }

  const session = await HCESession.getInstance();

  const refresh = async () => {
    try {
      const token = await signTicketToken(ticketId);
      await session.setApplication(
        new NFCTagType4({
          type: NFCTagType4NDEFContentType.Text,
          content: JSON.stringify(token),
          writable: false,
        }),
      );
    } catch (e) {
      console.warn('Failed to refresh HCE ticket content', e);
    }
  };

  await refresh();
  await session.setEnabled(true);
  const timer = setInterval(refresh, HCE_REFRESH_INTERVAL_MS);

  return () => {
    clearInterval(timer);
    session.setEnabled(false).catch(() => {});
  };
}

/**
 * ORGANIZER SIDE: reads the attendee's emulated/physical NDEF tag within
 * the tap timeout budget and resolves the decoded ticket token. Same
 * code path on Android and iOS since both surface an NDEF text record.
 */
export async function readTicketTap(): Promise<NfcTicketToken> {
  await ensureStarted();
  const startedAt = Date.now();

  try {
    await NfcManager.requestTechnology(NfcTech.Ndef, { alertMessage: 'Hold near the entry reader' });

    const tag = await NfcManager.getTag();
    const record = tag?.ndefMessage?.[0];
    const text = record ? Ndef.text.decodePayload(new Uint8Array(record.payload)) : null;
    const token = text ? (JSON.parse(text) as NfcTicketToken) : null;
    if (!token) throw new Error('No NDEF ticket payload found on tag');

    if (Date.now() - startedAt > TAP_TIMEOUT_MS) {
      console.warn(`NFC tap exceeded ${TAP_TIMEOUT_MS}ms budget`);
    }
    return token;
  } finally {
    await NfcManager.cancelTechnologyRequest().catch(() => {});
  }
}
