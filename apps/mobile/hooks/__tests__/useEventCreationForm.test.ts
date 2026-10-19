import { act, renderHook } from '@testing-library/react-native';
import { useEventCreationForm } from '../useEventCreationForm';

/**
 * Issue #1466 — useEventCreationForm.
 *
 * The hook is a zustand store shared by the four create-event wizard screens.
 * Because zustand stores are module singletons, every test starts from
 * `resetForm()` so state does not leak between cases.
 *
 * Per-step validation lives in the screen components
 * (`app/create-event/step-*.tsx`), not in this store — `goToStep` is
 * unguarded — so these tests cover the store's state mutations only.
 */

const step1 = {
  title: 'Stellar Meridian 2026',
  description: 'Annual Stellar ecosystem conference.',
  category: 'Technology',
  eventDate: '2026-11-12',
  eventTime: '09:30',
};

const step2Physical = {
  locationType: 'physical' as const,
  venueName: 'Landmark Centre',
  venueAddress: 'Water Corporation Dr, Lagos',
  virtualLink: '',
};

beforeEach(() => {
  act(() => {
    useEventCreationForm.getState().resetForm();
  });
});

describe('useEventCreationForm', () => {
  it('initialises with empty defaults, one blank tier, and step 1', () => {
    const { result } = renderHook(() => useEventCreationForm());

    expect(result.current).toMatchObject({
      title: '',
      description: '',
      category: '',
      eventDate: '',
      eventTime: '',
      locationType: 'physical',
      venueName: '',
      venueAddress: '',
      virtualLink: '',
      coverImageUri: null,
      coverImageUrl: null,
      currentStep: 1,
    });
    expect(result.current.tiers).toHaveLength(1);
    expect(result.current.tiers[0]).toMatchObject({
      name: '',
      priceUsdc: '',
      quantity: '',
      saleStart: '',
      saleEnd: '',
    });
    expect(typeof result.current.tiers[0].id).toBe('string');
  });

  it('setStep1 updates basic details without touching location or tiers', () => {
    const { result } = renderHook(() => useEventCreationForm());
    act(() => {
      result.current.setStep2(step2Physical);
    });
    const tiersBefore = result.current.tiers;

    act(() => {
      result.current.setStep1(step1);
    });

    expect(result.current).toMatchObject({ ...step1, ...step2Physical });
    expect(result.current.tiers).toBe(tiersBefore);
  });

  it('setStep2 updates location without clobbering step 1 fields', () => {
    const { result } = renderHook(() => useEventCreationForm());
    act(() => {
      result.current.setStep1(step1);
    });

    act(() => {
      result.current.setStep2({
        locationType: 'virtual',
        venueName: '',
        venueAddress: '',
        virtualLink: 'https://meet.example.com/agora',
      });
    });

    expect(result.current).toMatchObject(step1);
    expect(result.current.locationType).toBe('virtual');
    expect(result.current.virtualLink).toBe('https://meet.example.com/agora');
  });

  it('updateTier patches a single tier field and preserves the rest', () => {
    const { result } = renderHook(() => useEventCreationForm());
    act(() => {
      result.current.addTier();
    });
    const [first, second] = result.current.tiers;

    act(() => {
      result.current.updateTier(first.id, { name: 'VIP', priceUsdc: '50.00' });
    });
    act(() => {
      result.current.updateTier(first.id, { quantity: '100' });
    });

    expect(result.current.tiers[0]).toEqual({
      ...first,
      name: 'VIP',
      priceUsdc: '50.00',
      quantity: '100',
    });
    expect(result.current.tiers[1]).toEqual(second);
  });

  it('addTier / removeTier / setTiers manage the tier list', () => {
    const { result } = renderHook(() => useEventCreationForm());

    act(() => {
      result.current.addTier();
      result.current.addTier();
    });
    expect(result.current.tiers).toHaveLength(3);
    const ids = result.current.tiers.map((t) => t.id);
    expect(new Set(ids).size).toBe(3);

    act(() => {
      result.current.removeTier(ids[1]);
    });
    expect(result.current.tiers.map((t) => t.id)).toEqual([ids[0], ids[2]]);

    const replacement = [
      {
        id: 'tier-ga',
        name: 'General Admission',
        priceUsdc: '0',
        quantity: '500',
        saleStart: '2026-10-01',
        saleEnd: '2026-11-11',
      },
    ];
    act(() => {
      result.current.setTiers(replacement);
    });
    expect(result.current.tiers).toEqual(replacement);
  });

  it('stores the cover image URI and uploaded URL independently', () => {
    const { result } = renderHook(() => useEventCreationForm());

    act(() => {
      result.current.setCoverImageUri('file:///tmp/cover.jpg');
    });
    expect(result.current.coverImageUri).toBe('file:///tmp/cover.jpg');
    expect(result.current.coverImageUrl).toBeNull();

    act(() => {
      result.current.setCoverImageUrl('https://cdn.agora.events/cover.jpg');
    });
    expect(result.current.coverImageUri).toBe('file:///tmp/cover.jpg');
    expect(result.current.coverImageUrl).toBe('https://cdn.agora.events/cover.jpg');
  });

  it('goToStep moves between wizard steps while keeping entered data', () => {
    const { result } = renderHook(() => useEventCreationForm());
    act(() => {
      result.current.setStep1(step1);
    });

    act(() => {
      result.current.goToStep(2);
    });
    expect(result.current.currentStep).toBe(2);

    act(() => {
      result.current.goToStep(4);
    });
    act(() => {
      result.current.goToStep(1);
    });
    expect(result.current.currentStep).toBe(1);
    expect(result.current).toMatchObject(step1);
  });

  it('shares state across every component that uses the store', () => {
    const screenA = renderHook(() => useEventCreationForm());
    const screenB = renderHook(() => useEventCreationForm((s) => s.title));

    act(() => {
      screenA.result.current.setStep1(step1);
    });

    expect(screenB.result.current).toBe(step1.title);
  });

  it('resetForm restores every field to its default and issues a fresh tier', () => {
    const { result } = renderHook(() => useEventCreationForm());
    const originalTierId = result.current.tiers[0].id;

    act(() => {
      result.current.setStep1(step1);
      result.current.setStep2(step2Physical);
      result.current.addTier();
      result.current.updateTier(originalTierId, { name: 'VIP' });
      result.current.setCoverImageUri('file:///tmp/cover.jpg');
      result.current.setCoverImageUrl('https://cdn.agora.events/cover.jpg');
      result.current.goToStep(4);
    });

    act(() => {
      result.current.resetForm();
    });

    expect(result.current).toMatchObject({
      title: '',
      description: '',
      category: '',
      eventDate: '',
      eventTime: '',
      locationType: 'physical',
      venueName: '',
      venueAddress: '',
      virtualLink: '',
      coverImageUri: null,
      coverImageUrl: null,
      currentStep: 1,
    });
    expect(result.current.tiers).toHaveLength(1);
    expect(result.current.tiers[0].name).toBe('');
    expect(result.current.tiers[0].id).not.toBe(originalTierId);
  });
});
