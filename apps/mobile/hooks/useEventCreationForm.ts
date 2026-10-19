import { create } from 'zustand';

// ─── Ticket Tier ─────────────────────────────────────────────────────────────

export interface TicketTier {
  id: string; // local uuid
  name: string;
  priceUsdc: string; // stored as decimal string, e.g. "10.00"
  quantity: string; // positive integer string
  saleStart: string; // ISO date string "YYYY-MM-DD"
  saleEnd: string; // ISO date string "YYYY-MM-DD"
}

// ─── Form State ───────────────────────────────────────────────────────────────

export interface EventFormState {
  // Step 1 – Basic Details
  title: string;
  description: string;
  category: string;
  eventDate: string; // ISO date "YYYY-MM-DD"
  eventTime: string; // "HH:MM"

  // Step 2 – Location
  locationType: 'physical' | 'virtual';
  venueName: string;
  venueAddress: string;
  virtualLink: string;

  // Step 3 – Ticket Tiers
  tiers: TicketTier[];

  // Step 4 – Cover image
  coverImageUri: string | null; // local file URI chosen from library
  coverImageUrl: string | null; // public URL returned by the backend after upload

  // Wizard navigation
  currentStep: number; // 1-4
}

// ─── Actions ──────────────────────────────────────────────────────────────────

interface EventFormActions {
  setStep1(fields: Pick<EventFormState, 'title' | 'description' | 'category' | 'eventDate' | 'eventTime'>): void;
  setStep2(fields: Pick<EventFormState, 'locationType' | 'venueName' | 'venueAddress' | 'virtualLink'>): void;
  setTiers(tiers: TicketTier[]): void;
  addTier(): void;
  removeTier(id: string): void;
  updateTier(id: string, patch: Partial<TicketTier>): void;
  setCoverImageUri(uri: string | null): void;
  setCoverImageUrl(url: string | null): void;
  goToStep(step: number): void;
  resetForm(): void;
}

// ─── Default Values ───────────────────────────────────────────────────────────

const defaultTier = (): TicketTier => ({
  id: Math.random().toString(36).slice(2),
  name: '',
  priceUsdc: '',
  quantity: '',
  saleStart: '',
  saleEnd: '',
});

const initialState: EventFormState = {
  title: '',
  description: '',
  category: '',
  eventDate: '',
  eventTime: '',
  locationType: 'physical',
  venueName: '',
  venueAddress: '',
  virtualLink: '',
  tiers: [defaultTier()],
  coverImageUri: null,
  coverImageUrl: null,
  currentStep: 1,
};

// ─── Store ────────────────────────────────────────────────────────────────────

export const useEventCreationForm = create<EventFormState & EventFormActions>((set) => ({
  ...initialState,

  setStep1: (fields) => set((s) => ({ ...s, ...fields })),
  setStep2: (fields) => set((s) => ({ ...s, ...fields })),

  setTiers: (tiers) => set({ tiers }),

  addTier: () =>
    set((s) => ({ tiers: [...s.tiers, defaultTier()] })),

  removeTier: (id) =>
    set((s) => ({ tiers: s.tiers.filter((t) => t.id !== id) })),

  updateTier: (id, patch) =>
    set((s) => ({
      tiers: s.tiers.map((t) => (t.id === id ? { ...t, ...patch } : t)),
    })),

  setCoverImageUri: (uri) => set({ coverImageUri: uri }),
  setCoverImageUrl: (url) => set({ coverImageUrl: url }),

  goToStep: (step) => set({ currentStep: step }),

  resetForm: () => set({ ...initialState, tiers: [defaultTier()] }),
}));
