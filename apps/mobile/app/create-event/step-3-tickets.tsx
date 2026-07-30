import React from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  Pressable,
  Alert,
} from 'react-native';
import { useRouter } from 'expo-router';
import Colors from '@/constants/Colors';
import Input from '@/components/ui/Input';
import Button from '@/components/ui/Button';
import { useEventCreationForm, TicketTier } from '@/hooks/useEventCreationForm';
import { ProgressBar } from './step-1-basics';

// ─── Validation ───────────────────────────────────────────────────────────────

interface TierErrors {
  name?: string;
  priceUsdc?: string;
  quantity?: string;
  saleStart?: string;
  saleEnd?: string;
}

function validateTier(tier: TicketTier): TierErrors {
  const errs: TierErrors = {};

  if (!tier.name.trim()) {
    errs.name = 'Tier name is required.';
  } else if (tier.name.trim().length > 80) {
    errs.name = 'Tier name must be at most 80 characters.';
  }

  const price = parseFloat(tier.priceUsdc);
  if (tier.priceUsdc === '') {
    errs.priceUsdc = 'Price is required (enter 0 for free).';
  } else if (isNaN(price) || price < 0) {
    errs.priceUsdc = 'Price must be 0 or a positive number.';
  } else if (!/^\d+(\.\d{0,2})?$/.test(tier.priceUsdc)) {
    errs.priceUsdc = 'Max two decimal places allowed.';
  }

  const qty = parseInt(tier.quantity, 10);
  if (tier.quantity === '') {
    errs.quantity = 'Quantity is required.';
  } else if (!Number.isInteger(qty) || qty <= 0 || String(qty) !== tier.quantity) {
    errs.quantity = 'Quantity must be a positive integer.';
  }

  if (!tier.saleStart) {
    errs.saleStart = 'Sale start date is required.';
  } else if (isNaN(new Date(tier.saleStart).getTime())) {
    errs.saleStart = 'Enter a valid date (YYYY-MM-DD).';
  }

  if (!tier.saleEnd) {
    errs.saleEnd = 'Sale end date is required.';
  } else if (isNaN(new Date(tier.saleEnd).getTime())) {
    errs.saleEnd = 'Enter a valid date (YYYY-MM-DD).';
  } else if (tier.saleStart && new Date(tier.saleEnd) <= new Date(tier.saleStart)) {
    errs.saleEnd = 'Sale end date must be after sale start date.';
  }

  return errs;
}

// ─── Step 3 Screen ────────────────────────────────────────────────────────────

export default function Step3Tickets() {
  const router = useRouter();
  const form = useEventCreationForm();

  // Local copy of tiers for editing
  const [tiers, setTiers] = React.useState<TicketTier[]>(
    form.tiers.length > 0 ? form.tiers : [defaultTier()],
  );
  const [allErrors, setAllErrors] = React.useState<Record<string, TierErrors>>({});

  const updateTierLocal = (id: string, patch: Partial<TicketTier>) => {
    setTiers((prev) => prev.map((t) => (t.id === id ? { ...t, ...patch } : t)));
    // Clear errors for changed fields
    if (allErrors[id]) {
      setAllErrors((prev) => {
        const copy = { ...prev };
        delete copy[id];
        return copy;
      });
    }
  };

  const addTierLocal = () => {
    setTiers((prev) => [...prev, defaultTier()]);
  };

  const removeTierLocal = (id: string) => {
    if (tiers.length === 1) {
      Alert.alert('At least one ticket tier is required.');
      return;
    }
    setTiers((prev) => prev.filter((t) => t.id !== id));
    setAllErrors((prev) => {
      const copy = { ...prev };
      delete copy[id];
      return copy;
    });
  };

  const handleNext = () => {
    // Validate all tiers
    let hasError = false;
    const newErrors: Record<string, TierErrors> = {};
    for (const tier of tiers) {
      const errs = validateTier(tier);
      if (Object.keys(errs).length > 0) {
        newErrors[tier.id] = errs;
        hasError = true;
      }
    }
    if (hasError) {
      setAllErrors(newErrors);
      return;
    }

    form.setTiers(tiers);
    form.goToStep(4);
    router.push('/create-event/step-4-review');
  };

  const handleBack = () => {
    form.setTiers(tiers);
    router.back();
  };

  return (
    <ScrollView
      style={styles.container}
      contentContainerStyle={styles.content}
      keyboardShouldPersistTaps="handled"
    >
      <ProgressBar current={3} total={4} />

      <Text style={styles.heading}>Ticket Pricing Tiers</Text>
      <Text style={styles.subheading}>
        Create one or more pricing tiers for your event. Prices are in USDC.
      </Text>

      {tiers.map((tier, index) => (
        <TierCard
          key={tier.id}
          tier={tier}
          index={index}
          errors={allErrors[tier.id] ?? {}}
          onUpdate={(patch) => updateTierLocal(tier.id, patch)}
          onRemove={() => removeTierLocal(tier.id)}
          canRemove={tiers.length > 1}
        />
      ))}

      <Pressable style={styles.addTierBtn} onPress={addTierLocal}>
        <Text style={styles.addTierText}>＋ Add Another Tier</Text>
      </Pressable>

      <View style={styles.navRow}>
        <Button
          title="← Back"
          variant="secondary"
          onPress={handleBack}
          style={styles.backBtn}
        />
        <Button title="Next: Review →" onPress={handleNext} style={styles.nextBtn} />
      </View>
    </ScrollView>
  );
}

// ─── Tier Card ────────────────────────────────────────────────────────────────

interface TierCardProps {
  tier: TicketTier;
  index: number;
  errors: TierErrors;
  onUpdate: (patch: Partial<TicketTier>) => void;
  onRemove: () => void;
  canRemove: boolean;
}

function TierCard({ tier, index, errors, onUpdate, onRemove, canRemove }: TierCardProps) {
  return (
    <View style={cardStyles.card}>
      <View style={cardStyles.cardHeader}>
        <Text style={cardStyles.cardTitle}>Tier {index + 1}</Text>
        {canRemove && (
          <Pressable onPress={onRemove}>
            <Text style={cardStyles.removeText}>Remove</Text>
          </Pressable>
        )}
      </View>

      <Input
        label="Tier Name *"
        placeholder='e.g. "General Admission" or "VIP"'
        value={tier.name}
        onChangeText={(v) => onUpdate({ name: v })}
        error={errors.name}
        maxLength={80}
      />

      <Input
        label="Price (USDC) *"
        placeholder="0.00"
        value={tier.priceUsdc}
        onChangeText={(v) => onUpdate({ priceUsdc: v })}
        error={errors.priceUsdc}
        keyboardType="decimal-pad"
      />

      <Input
        label="Quantity Limit *"
        placeholder="e.g. 100"
        value={tier.quantity}
        onChangeText={(v) => onUpdate({ quantity: v })}
        error={errors.quantity}
        keyboardType="number-pad"
      />

      <View style={cardStyles.dateRow}>
        <Input
          label="Sale Start *"
          placeholder="YYYY-MM-DD"
          value={tier.saleStart}
          onChangeText={(v) => onUpdate({ saleStart: v })}
          error={errors.saleStart}
          keyboardType="numbers-and-punctuation"
          maxLength={10}
          containerStyle={cardStyles.halfInput}
        />
        <Input
          label="Sale End *"
          placeholder="YYYY-MM-DD"
          value={tier.saleEnd}
          onChangeText={(v) => onUpdate({ saleEnd: v })}
          error={errors.saleEnd}
          keyboardType="numbers-and-punctuation"
          maxLength={10}
          containerStyle={cardStyles.halfInput}
        />
      </View>
    </View>
  );
}

function defaultTier(): TicketTier {
  return {
    id: Math.random().toString(36).slice(2),
    name: '',
    priceUsdc: '',
    quantity: '',
    saleStart: '',
    saleEnd: '',
  };
}

const cardStyles = StyleSheet.create({
  card: {
    backgroundColor: '#1E1E20',
    borderRadius: 12,
    borderWidth: 1,
    borderColor: '#2C2C2E',
    padding: 16,
    marginBottom: 16,
  },
  cardHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  cardTitle: {
    fontSize: 16,
    fontWeight: '700',
    color: Colors.primaryYellow,
  },
  removeText: {
    fontSize: 13,
    color: Colors.accentRed,
    fontWeight: '600',
  },
  dateRow: {
    flexDirection: 'row',
    gap: 10,
  },
  halfInput: {
    flex: 1,
  },
});

// ─── Styles ───────────────────────────────────────────────────────────────────

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: Colors.darkBackground,
  },
  content: {
    padding: 20,
    paddingBottom: 48,
  },
  heading: {
    fontSize: 22,
    fontWeight: '700',
    color: Colors.primaryText,
    marginBottom: 6,
  },
  subheading: {
    fontSize: 14,
    color: Colors.secondaryText,
    marginBottom: 24,
  },
  addTierBtn: {
    borderWidth: 1,
    borderColor: Colors.primaryYellow,
    borderStyle: 'dashed',
    borderRadius: 10,
    paddingVertical: 14,
    alignItems: 'center',
    marginBottom: 24,
  },
  addTierText: {
    color: Colors.primaryYellow,
    fontSize: 15,
    fontWeight: '600',
  },
  navRow: {
    flexDirection: 'row',
    gap: 12,
  },
  backBtn: {
    flex: 1,
  },
  nextBtn: {
    flex: 2,
  },
});
