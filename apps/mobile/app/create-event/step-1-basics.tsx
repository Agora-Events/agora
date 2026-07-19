import React from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  Pressable,
} from 'react-native';
import { useRouter } from 'expo-router';
import Colors from '@/constants/Colors';
import Input from '@/components/ui/Input';
import Button from '@/components/ui/Button';
import { useEventCreationForm } from '@/hooks/useEventCreationForm';

const CATEGORIES = [
  'Technology',
  'Music',
  'Art',
  'Finance',
  'Sports',
  'Education',
  'Networking',
  'Other',
];

// ─── Validation ───────────────────────────────────────────────────────────────

interface Errors {
  title?: string;
  description?: string;
  category?: string;
  eventDate?: string;
  eventTime?: string;
}

function validate(
  title: string,
  description: string,
  category: string,
  eventDate: string,
  eventTime: string,
): Errors {
  const errors: Errors = {};

  if (!title.trim()) {
    errors.title = 'Title is required.';
  } else if (title.trim().length < 3) {
    errors.title = 'Title must be at least 3 characters.';
  } else if (title.trim().length > 120) {
    errors.title = 'Title must be at most 120 characters.';
  }

  if (description.trim().length > 2000) {
    errors.description = 'Description must be at most 2000 characters.';
  }

  if (!category) {
    errors.category = 'Please select a category.';
  }

  if (!eventDate) {
    errors.eventDate = 'Event date is required.';
  } else {
    const d = new Date(eventDate);
    if (isNaN(d.getTime())) {
      errors.eventDate = 'Enter a valid date (YYYY-MM-DD).';
    } else if (d < new Date(new Date().toDateString())) {
      errors.eventDate = 'Event date must be today or in the future.';
    }
  }

  if (!eventTime) {
    errors.eventTime = 'Event time is required.';
  } else if (!/^\d{2}:\d{2}$/.test(eventTime)) {
    errors.eventTime = 'Enter time in HH:MM format.';
  }

  return errors;
}

// ─── Step 1 Screen ────────────────────────────────────────────────────────────

export default function Step1BasicDetails() {
  const router = useRouter();
  const form = useEventCreationForm();

  const [localTitle, setLocalTitle] = React.useState(form.title);
  const [localDescription, setLocalDescription] = React.useState(form.description);
  const [localCategory, setLocalCategory] = React.useState(form.category);
  const [localDate, setLocalDate] = React.useState(form.eventDate);
  const [localTime, setLocalTime] = React.useState(form.eventTime);
  const [errors, setErrors] = React.useState<Errors>({});

  const handleNext = () => {
    const errs = validate(localTitle, localDescription, localCategory, localDate, localTime);
    if (Object.keys(errs).length > 0) {
      setErrors(errs);
      return;
    }
    form.setStep1({
      title: localTitle.trim(),
      description: localDescription.trim(),
      category: localCategory,
      eventDate: localDate,
      eventTime: localTime,
    });
    form.goToStep(2);
    router.push('/create-event/step-2-location');
  };

  return (
    <ScrollView
      style={styles.container}
      contentContainerStyle={styles.content}
      keyboardShouldPersistTaps="handled"
    >
      {/* ── Progress bar ── */}
      <ProgressBar current={1} total={4} />

      <Text style={styles.heading}>Basic Event Details</Text>
      <Text style={styles.subheading}>Tell attendees what your event is about.</Text>

      <Input
        label="Event Title *"
        placeholder="e.g. Stellar Meridian 2026"
        value={localTitle}
        onChangeText={setLocalTitle}
        error={errors.title}
        maxLength={120}
      />

      <Input
        label="Description"
        placeholder="What can attendees expect?"
        value={localDescription}
        onChangeText={setLocalDescription}
        error={errors.description}
        multiline
        numberOfLines={4}
        containerStyle={styles.textAreaContainer}
        // @ts-expect-error – RN accepts style on TextInput
        style={styles.textArea}
      />

      {/* Category picker */}
      <Text style={styles.fieldLabel}>Category *</Text>
      {errors.category ? <Text style={styles.errorText}>{errors.category}</Text> : null}
      <View style={styles.categoryGrid}>
        {CATEGORIES.map((cat) => (
          <Pressable
            key={cat}
            style={[
              styles.categoryChip,
              localCategory === cat && styles.categoryChipActive,
            ]}
            onPress={() => setLocalCategory(cat)}
          >
            <Text
              style={[
                styles.categoryChipText,
                localCategory === cat && styles.categoryChipTextActive,
              ]}
            >
              {cat}
            </Text>
          </Pressable>
        ))}
      </View>

      <Input
        label="Event Date *"
        placeholder="YYYY-MM-DD"
        value={localDate}
        onChangeText={setLocalDate}
        error={errors.eventDate}
        keyboardType="numbers-and-punctuation"
        maxLength={10}
      />

      <Input
        label="Start Time *"
        placeholder="HH:MM  (24h)"
        value={localTime}
        onChangeText={setLocalTime}
        error={errors.eventTime}
        keyboardType="numbers-and-punctuation"
        maxLength={5}
      />

      <Button title="Next: Location →" onPress={handleNext} style={styles.cta} />
    </ScrollView>
  );
}

// ─── Progress Bar ─────────────────────────────────────────────────────────────

function ProgressBar({ current, total }: { current: number; total: number }) {
  return (
    <View style={pbStyles.wrapper}>
      {Array.from({ length: total }, (_, i) => i + 1).map((step) => (
        <View
          key={step}
          style={[
            pbStyles.segment,
            step <= current ? pbStyles.filled : pbStyles.empty,
          ]}
        />
      ))}
    </View>
  );
}

const pbStyles = StyleSheet.create({
  wrapper: {
    flexDirection: 'row',
    gap: 6,
    marginBottom: 28,
  },
  segment: {
    flex: 1,
    height: 4,
    borderRadius: 2,
  },
  filled: {
    backgroundColor: Colors.primaryYellow,
  },
  empty: {
    backgroundColor: '#2C2C2E',
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
  fieldLabel: {
    color: Colors.primaryText,
    fontSize: 14,
    fontWeight: '500',
    marginBottom: 6,
  },
  errorText: {
    color: Colors.accentRed,
    fontSize: 12,
    marginBottom: 6,
  },
  categoryGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
    marginBottom: 16,
  },
  categoryChip: {
    paddingHorizontal: 14,
    paddingVertical: 8,
    borderRadius: 20,
    backgroundColor: '#1E1E20',
    borderWidth: 1,
    borderColor: '#2C2C2E',
  },
  categoryChipActive: {
    backgroundColor: Colors.primaryYellow,
    borderColor: Colors.primaryYellow,
  },
  categoryChipText: {
    fontSize: 13,
    color: Colors.secondaryText,
    fontWeight: '500',
  },
  categoryChipTextActive: {
    color: Colors.darkBackground,
    fontWeight: '700',
  },
  textAreaContainer: {
    marginBottom: 16,
  },
  textArea: {
    height: 100,
    textAlignVertical: 'top',
    paddingTop: 10,
  },
  cta: {
    marginTop: 24,
  },
});

export { ProgressBar };
