import { MobileWalletAdapter, WalletMetadata, WalletType } from './types';
import { FreighterMobileAdapter } from './FreighterMobileAdapter';
import { WalletConnectAdapter } from './WalletConnectAdapter';
import { EmbeddedKeypairAdapter } from './EmbeddedKeypairAdapter';
import { LobstrAdapter } from './LobstrAdapter';
import { AlbedoAdapter } from './AlbedoAdapter';

export * from './types';
export * from './xdrDecoder';
export * from './sessionManager';
export * from './FreighterMobileAdapter';
export * from './WalletConnectAdapter';
export * from './EmbeddedKeypairAdapter';
export * from './LobstrAdapter';
export * from './AlbedoAdapter';

export const SUPPORTED_WALLETS: WalletMetadata[] = [
  {
    id: 'freighter',
    name: 'Freighter Mobile',
    icon: 'wallet-outline',
    description: 'Hardware-backed Stellar wallet with web+stellar deep linking',
    isDeepLink: true,
  },
  {
    id: 'walletconnect',
    name: 'WalletConnect v2',
    icon: 'qr-code-outline',
    description: 'Scan or connect any mobile Stellar wallet via WalletConnect v2',
    isDeepLink: false,
  },
  {
    id: 'embedded',
    name: 'Embedded Keypair',
    icon: 'key-outline',
    description: 'Instant non-custodial local account stored in secure hardware',
    isDeepLink: false,
  },
  {
    id: 'lobstr',
    name: 'LOBSTR',
    icon: 'phone-portrait-outline',
    description: 'Native LOBSTR mobile wallet app',
    isDeepLink: true,
  },
  {
    id: 'albedo',
    name: 'Albedo',
    icon: 'shield-checkmark-outline',
    description: 'Web & mobile signer with delegable permissions',
    isDeepLink: true,
  },
];

const adapterInstances = new Map<WalletType, MobileWalletAdapter>();

export function getWalletAdapter(type: WalletType): MobileWalletAdapter {
  if (adapterInstances.has(type)) {
    return adapterInstances.get(type)!;
  }

  let adapter: MobileWalletAdapter;
  switch (type) {
    case 'freighter':
      adapter = new FreighterMobileAdapter();
      break;
    case 'walletconnect':
      adapter = new WalletConnectAdapter();
      break;
    case 'embedded':
      adapter = new EmbeddedKeypairAdapter();
      break;
    case 'lobstr':
      adapter = new LobstrAdapter();
      break;
    case 'albedo':
      adapter = new AlbedoAdapter();
      break;
    default:
      throw new Error(`Unsupported wallet adapter: ${type}`);
  }

  adapterInstances.set(type, adapter);
  return adapter;
}
