/**
 * bleMeshService.ts
 *
 * Store-and-forward BLE mesh for offline gate scanners. Each scanner
 * gossips its local check-in log to nearby scanners so that once *any*
 * device in the mesh regains cellular/satellite connectivity, the whole
 * festival's log can be flushed to the server in one shot.
 *
 * `react-native-ble-plx` only implements the BLE *central* role (scan +
 * connect) - it cannot advertise as a peripheral. To be discoverable by
 * other scanners, each device also needs a peripheral-mode library (e.g.
 * `react-native-ble-advertiser`); that half is injected via the
 * `BlePeripheral` interface so this module stays testable without a
 * native mock and swappable if we change advertiser libraries later.
 */

import { BleManager, Device } from 'react-native-ble-plx';
import NetInfo from '@react-native-community/netinfo';
import { Buffer } from 'buffer';

/** Custom 128-bit service UUID identifying Agora gate scanners. */
export const MESH_SERVICE_UUID = '6f1d1a00-b5de-4196-b91a-1a0c6f6a2e10';
/** Characteristic the mesh log is exchanged over (read + notify). */
export const MESH_LOG_CHARACTERISTIC_UUID = '6f1d1a01-b5de-4196-b91a-1a0c6f6a2e10';

/** Ceiling on how many times a record is re-broadcast before it's dropped. */
const MAX_GOSSIP_HOPS = 6;
/** How often we sweep for new peers and retry a server flush, in ms. */
const GOSSIP_INTERVAL_MS = 8000;

const API_BASE_URL = process.env.EXPO_PUBLIC_API_URL ?? 'http://localhost:8080';
const FLUSH_TIMEOUT_MS = 15_000;

export type CheckInResult = 'approved' | 'denied';

export interface CheckInRecord {
  recordId: string;
  ticketId: string;
  gateId: string;
  scannerId: string;
  result: CheckInResult;
  scannedAt: number;
  hops: number;
}

/** Minimal peripheral-mode contract, implemented by a native advertiser lib. */
export interface BlePeripheral {
  startAdvertising(serviceUuid: string): Promise<void>;
  stopAdvertising(): Promise<void>;
  onLogRequested(handler: () => string): void;
}

export class BleMeshService {
  private readonly central = new BleManager();
  private readonly log = new Map<string, CheckInRecord>();
  private readonly peersHandledThisTick = new Set<string>();
  private scanning = false;
  private gossipTimer: ReturnType<typeof setInterval> | null = null;

  constructor(
    private readonly scannerId: string,
    private readonly peripheral?: BlePeripheral,
  ) {}

  /** Starts advertising presence, scanning for peers, and periodic gossip. */
  async start(gateId: string): Promise<void> {
    await this.peripheral?.startAdvertising(MESH_SERVICE_UUID);
    this.peripheral?.onLogRequested(() => this.serializeLog());

    await this.central.startDeviceScan(
      [MESH_SERVICE_UUID],
      { allowDuplicates: false },
      (error, device) => this.onPeerDiscovered(error, device),
    );
    this.scanning = true;

    this.gossipTimer = setInterval(() => this.gossipTick(gateId), GOSSIP_INTERVAL_MS);
  }

  /** Tears down scanning, advertising, and the gossip timer. */
  async stop(): Promise<void> {
    if (this.scanning) {
      await this.central.stopDeviceScan();
      this.scanning = false;
    }
    if (this.gossipTimer) clearInterval(this.gossipTimer);
    await this.peripheral?.stopAdvertising();
  }

  /** Records a fresh, local check-in so it starts propagating this tick. */
  recordCheckIn(ticketId: string, gateId: string, result: CheckInResult): void {
    const record: CheckInRecord = {
      recordId: `${this.scannerId}-${ticketId}-${Date.now()}`,
      ticketId,
      gateId,
      scannerId: this.scannerId,
      result,
      scannedAt: Date.now(),
      hops: 0,
    };
    this.log.set(record.recordId, record);
  }

  /** How many unsynced records this device currently holds (for the UI). */
  get pendingCount(): number {
    return this.log.size;
  }

  private onPeerDiscovered(error: Error | null, device: Device | null): void {
    if (error || !device || this.peersHandledThisTick.has(device.id)) return;
    this.peersHandledThisTick.add(device.id);
    this.pullFromPeer(device).catch((e) => console.warn('BLE mesh pull failed', e));
  }

  /** Connects to a newly seen peer and merges its check-in log into ours. */
  private async pullFromPeer(device: Device): Promise<void> {
    const connected = await device.connect();
    await connected.discoverAllServicesAndCharacteristics();

    const characteristic = await connected.readCharacteristicForService(
      MESH_SERVICE_UUID,
      MESH_LOG_CHARACTERISTIC_UUID,
    );
    const raw = characteristic.value
      ? Buffer.from(characteristic.value, 'base64').toString('utf8')
      : '[]';
    this.mergeIncoming(JSON.parse(raw) as CheckInRecord[]);

    await connected.cancelConnection();
  }

  /** Gossip-merges peer records: dedupe by id, drop records past hop cap. */
  private mergeIncoming(records: CheckInRecord[]): void {
    for (const record of records) {
      const alreadyHave = this.log.has(record.recordId);
      if (alreadyHave || record.hops >= MAX_GOSSIP_HOPS) continue;
      this.log.set(record.recordId, { ...record, hops: record.hops + 1 });
    }
  }

  /** Periodic tick: refresh peer dedupe window, then attempt a gateway flush. */
  private async gossipTick(gateId: string): Promise<void> {
    this.peersHandledThisTick.clear();
    await this.tryFlushToGateway(gateId);
  }

  /**
   * Master gateway sync: if THIS device currently has connectivity, push
   * the whole mesh's aggregated log to the server and clear it locally.
   * Any scanner in the mesh can be the one that happens to have signal.
   */
  private async tryFlushToGateway(gateId: string): Promise<void> {
    if (this.log.size === 0) return;
    const net = await NetInfo.fetch();
    if (!net.isConnected || !net.isInternetReachable) return;

    const records = Array.from(this.log.values());
    const flushedIds = await postCheckInBatch(gateId, records);
    flushedIds.forEach((id) => this.log.delete(id));
  }

  private serializeLog(): string {
    return JSON.stringify(Array.from(this.log.values()));
  }
}

/** POSTs a batch of mesh check-in records to `/api/gate/checkins/batch`. */
async function postCheckInBatch(gateId: string, records: CheckInRecord[]): Promise<string[]> {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), FLUSH_TIMEOUT_MS);

  try {
    const response = await fetch(`${API_BASE_URL}/api/gate/checkins/batch`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ gateId, records }),
      signal: controller.signal,
    });
    if (!response.ok) return [];

    const body = (await response.json()) as { acceptedRecordIds: string[] };
    return body.acceptedRecordIds ?? [];
  } catch (e) {
    console.warn('Gateway flush failed, will retry next gossip tick', e);
    return [];
  } finally {
    clearTimeout(timer);
  }
}
