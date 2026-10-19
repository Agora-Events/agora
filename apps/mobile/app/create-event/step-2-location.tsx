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
import { ProgressBar } from './step-1-basics';

// ─── Validation ───────────────────────────────────────────────────────────────

interface Errors {
  venue?: string;
  address?: string;
  link?: string;
}

function validate(
  type: 'physical' | 'virtual',
  venueName: string,
  venueAddress: string,
  virtualLink: string,
): Errors {
  const errors: Errors = {};

  if (type === 'physical') {
    if (!venueName.trim()) {
      errors.venue = 'Venue name is required.';
    } else if (venueName.trim().length > 200) {
      errors.venue = 'Venue name must be at most 200 characters.';
    }
    if (!venueAddress.trim()) {
      errors.address = 'Venue address is required.';
    } else if (venueAddress.trim().length > 500) {
      errors.address = 'Address must be at most 500 characters.';
    }
  } else {
    if (!virtualLink.trim()) {
      errors.link = 'Virtual link is required.';
    } else if (
      !virtualLink.startsWith('https://') &&
      !virtualLink.startsWith('http://')
    ) {
      errors.link = 'Please enter a valid URL (starting with https://).';
    }
  }

  return errors;
}

// ─── Step 2 Screen ────────────────────────────────────────────────────────────

export default function Step2Location() {
  const router = useRouter();
  const form = useEventCreationForm();

  const [locationType, setLocationType] = React.useState<'physical' | 'virtual'>(
    form.locationType,
  );
  const [venueName, setVenueName] = React.useState(form.venueName);
  const [venueAddress, setVenueAddress] = React.useState(form.venueAddress);
  const [virtualLink, setVirtualLink] = React.useState(form.virtualLink);
  const [errors, setErrors] = React.useState<Errors>({});

  const handleNext = () => {
    const errs = validate(locationType, venueName, venueAddress, virtualLink);
    if (Object.keys(errs).length > 0) {
      setErrors(errs);
      return;
    }
    form.setStep2({ locationType, venueName, venueAddress, virtualLink });
    form.goToStep(3);
    router.push('/create-event/step-3-tickets');
  };

  const handleBack = () => {
    form.setStep2({ locationType, venueName, venueAddress, virtualLink });
    router.back();
  };

  return (
    <ScrollView
      style={styles.container}
      contentContainerStyle={styles.content}
      keyboardShouldPersistTaps="handled"
    >
      <ProgressBar current={2} total={4} />

      <Text style={styles.heading}>Location Setup</Text>
      <Text style={styles.subheading}>
        Choose between a physical venue or an online event.
      </Text>

      {/* Toggle tabs */}
      <View style={styles.typeToggle}>
        <Pressable
          style={[
            styles.toggleOption,
            locationType === 'physical' && styles.toggleActive,
          ]}
          onPress={() => setLocationType('physical')}
        >
          <Text
            style={[
              styles.toggleText,
              locationType === 'physical' && styles.toggleTextActive,
            ]}
          >
            🏟  Physical
          </Text>
        </Pressable>
        <Pressable
          style={[
            styles.toggleOption,
            locationType === 'virtual' && styles.toggleActive,
          ]}
          onPress={() => setLocationType('virtual')}
        >
          <Text
            style={[
              styles.toggleText,
              locationType === 'virtual' && styles.toggleTextActive,
            ]}
          >
            💻  Virtual
          </Text>
        </Pressable>
      </View>

      {locationType === 'physical' ? (
        <>
          <Input
            label="Venue Name *"
            placeholder="e.g. London Science Center"
            value={venueName}
            onChangeText={setVenueName}
            error={errors.venue}
            maxLength={200}
          />
          <Input
            label="Venue Address *"
            placeholder="Street, City, Country"
            value={venueAddress}
            onChangeText={setVenueAddress}
            error={errors.address}
            multiline
            numberOfLines={3}
            containerStyle={styles.multiLineContainer}
          />
        </>
      ) : (
        <Input
          label="Virtual Link *"
          placeholder="https://meet.google.com/..."
          value={virtualLink}
          onChangeText={setVirtualLink}
          error={errors.link}
          keyboardType="url"
          autoCapitalize="none"
          autoCorrect={false}
        />
      )}

      <View style={styles.navRow}>
        <Button
          title="← Back"
          variant="secondary"
          onPress={handleBack}
          style={styles.backBtn}
        />
        <Button title="Next: Tickets →" onPress={handleNext} style={styles.nextBtn} />
      </View>
    </ScrollView>
  );
}

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
  typeToggle: {
    flexDirection: 'row',
    backgroundColor: '#1E1E20',
    borderRadius: 10,
    borderWidth: 1,
    borderColor: '#2C2C2E',
    marginBottom: 24,
    overflow: 'hidden',
  },
  toggleOption: {
    flex: 1,
    paddingVertical: 12,
    alignItems: 'center',
    borderRadius: 9,
  },
  toggleActive: {
    backgroundColor: Colors.primaryYellow,
  },
  toggleText: {
    fontSize: 14,
    fontWeight: '600',
    color: Colors.secondaryText,
  },
  toggleTextActive: {
    color: Colors.darkBackground,
  },
  multiLineContainer: {
    marginBottom: 16,
  },
  navRow: {
    flexDirection: 'row',
    gap: 12,
    marginTop: 24,
  },
  backBtn: {
    flex: 1,
  },
  nextBtn: {
    flex: 2,
  },
});
