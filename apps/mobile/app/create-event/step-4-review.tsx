import React from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  Pressable,
  Alert,
  Image,
  Animated,
} from 'react-native';
import { useRouter } from 'expo-router';
import Colors from '@/constants/Colors';
import Button from '@/components/ui/Button';
import { useEventCreationForm } from '@/hooks/useEventCreationForm';
import { useAuth } from '@/hooks/useAuth';
import { ProgressBar } from './step-1-basics';

// ─── Constants ────────────────────────────────────────────────────────────────

const API_BASE_URL = 'https://api.agora.events'; // replaced at build-time via env

// ─── Image Upload Helper ──────────────────────────────────────────────────────

/**
 * Uploads a local image file to the backend S3 endpoint
 * `POST /api/v1/upload/image` (multipart/form-data).
 *
 * Reports progress via the onProgress callback (0–100).
 * Returns the public URL on success, throws on failure.
 */
async function uploadImageBinary(
  fileUri: string,
  mimeType: string,
  token: string,
  onProgress: (pct: number) => void,
): Promise<string> {
  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest();
    xhr.open('POST', `${API_BASE_URL}/api/v1/upload/image`);
    xhr.setRequestHeader('Authorization', `Bearer ${token}`);

    xhr.upload.onprogress = (e) => {
      if (e.lengthComputable) {
        onProgress(Math.round((e.loaded / e.total) * 100));
      }
    };

    xhr.onload = () => {
      if (xhr.status >= 200 && xhr.status < 300) {
        try {
          const body = JSON.parse(xhr.responseText);
          resolve(body.data?.url ?? body.url);
        } catch {
          reject(new Error('Invalid JSON from upload endpoint'));
        }
      } else {
        reject(new Error(`Upload failed: ${xhr.status} ${xhr.responseText}`));
      }
    };

    xhr.onerror = () => reject(new Error('Network error during upload'));
    xhr.ontimeout = () => reject(new Error('Upload timed out'));
    xhr.timeout = 60_000;

    const formData = new FormData();
    // React Native FormData accepts { uri, type, name } for file fields
    formData.append('file', {
      uri: fileUri,
      type: mimeType,
      name: `cover.${mimeType === 'image/png' ? 'png' : 'jpg'}`,
    } as unknown as Blob);

    xhr.send(formData);
  });
}

// ─── Register Event API ───────────────────────────────────────────────────────

async function createEventApi(payload: object, token: string): Promise<{ id: string }> {
  const res = await fetch(`${API_BASE_URL}/api/v1/events`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify(payload),
  });

  if (!res.ok) {
    const text = await res.text();
    throw new Error(`Create event failed (${res.status}): ${text}`);
  }

  const json = await res.json();
  return json.data ?? json;
}

// ─── Soroban Stub ─────────────────────────────────────────────────────────────

/**
 * Prompts the user to sign the Soroban `register_event` transaction.
 * Returns a transaction receipt string.
 *
 * This is a stub – integrate with your actual Soroban SDK / wallet adapter here.
 */
async function signSorobanRegisterEvent(eventId: string): Promise<string> {
  // TODO: replace with real Soroban SDK invocation, e.g.:
  //   const tx = await sorobanClient.call('register_event', { event_id: eventId });
  //   const receipt = await walletConnectClient.signAndSubmit(tx);
  //   return receipt.txHash;
  return new Promise((resolve) => {
    Alert.alert(
      'Sign Transaction',
      `Approve the Soroban register_event contract call for event:\n${eventId}`,
      [
        {
          text: 'Sign & Submit',
          onPress: () =>
            setTimeout(() => resolve(`mock-soroban-tx-${eventId.slice(0, 8)}`), 500),
        },
        {
          text: 'Cancel',
          style: 'cancel',
          onPress: () => resolve(''),
        },
      ],
    );
  });
}

// ─── Step 4 Screen ────────────────────────────────────────────────────────────

type SubmitStatus = 'idle' | 'uploading' | 'creating' | 'signing' | 'done' | 'error';

export default function Step4Review() {
  const router = useRouter();
  const form = useEventCreationForm();
  const { token } = useAuth();

  const [status, setStatus] = React.useState<SubmitStatus>('idle');
  const [uploadProgress, setUploadProgress] = React.useState(0);
  const [errorMsg, setErrorMsg] = React.useState('');
  const progressAnim = React.useRef(new Animated.Value(0)).current;

  // ── Animate progress bar ──────────────────────────────────────────────────
  React.useEffect(() => {
    Animated.timing(progressAnim, {
      toValue: uploadProgress,
      duration: 200,
      useNativeDriver: false,
    }).start();
  }, [uploadProgress, progressAnim]);

  // ── Image picker ─────────────────────────────────────────────────────────

  const pickImage = async () => {
    try {
      // Dynamic import so the module is only required when called
      // (avoids crashing on platforms where expo-image-picker isn't linked)
      // eslint-disable-next-line @typescript-eslint/no-require-imports
      const ImagePicker = require('expo-image-picker');

      const { status: permStatus } =
        await ImagePicker.requestMediaLibraryPermissionsAsync();
      if (permStatus !== 'granted') {
        Alert.alert('Permission required', 'Please allow access to your photo library.');
        return;
      }

      const result = await ImagePicker.launchImageLibraryAsync({
        mediaTypes: ImagePicker.MediaTypeOptions.Images,
        allowsEditing: true,
        aspect: [16, 9],
        quality: 0.6, // compress to ~60 % quality to reduce bandwidth
      });

      if (!result.canceled && result.assets?.[0]) {
        const asset = result.assets[0];
        form.setCoverImageUri(asset.uri);
        form.setCoverImageUrl(null); // reset previously uploaded URL
      }
    } catch {
      Alert.alert('Error', 'Failed to open image library.');
    }
  };

  // ── Submit ────────────────────────────────────────────────────────────────

  const handleSubmit = async () => {
    if (!token) {
      Alert.alert('Not logged in', 'Please log in to create an event.');
      return;
    }

    setStatus('idle');
    setErrorMsg('');

    try {
      // 1. Upload cover image if selected but not yet uploaded
      let finalImageUrl = form.coverImageUrl;
      if (form.coverImageUri && !finalImageUrl) {
        setStatus('uploading');
        setUploadProgress(0);
        const mimeType = form.coverImageUri.endsWith('.png') ? 'image/png' : 'image/jpeg';
        finalImageUrl = await uploadImageBinary(
          form.coverImageUri,
          mimeType,
          token,
          (pct) => setUploadProgress(pct),
        );
        form.setCoverImageUrl(finalImageUrl);
        setUploadProgress(100);
      }

      // 2. Build location string for the backend
      const location =
        form.locationType === 'physical'
          ? `${form.venueName}, ${form.venueAddress}`
          : form.virtualLink;

      // 3. Combine event date + time into ISO 8601
      const startTime = new Date(`${form.eventDate}T${form.eventTime}:00.000Z`).toISOString();

      const payload = {
        title: form.title,
        description: form.description || null,
        location,
        start_time: startTime,
        end_time: null,
        image_url: finalImageUrl ?? null,
        category: form.category,
        ticket_tiers: form.tiers.map((t) => ({
          name: t.name,
          price_usdc: parseFloat(t.priceUsdc),
          quantity: parseInt(t.quantity, 10),
          sale_start: new Date(`${t.saleStart}T00:00:00.000Z`).toISOString(),
          sale_end: new Date(`${t.saleEnd}T23:59:59.000Z`).toISOString(),
        })),
      };

      // 4. POST to REST backend
      setStatus('creating');
      const created = await createEventApi(payload, token);

      // 5. Prompt organizer to sign Soroban transaction
      setStatus('signing');
      const txReceipt = await signSorobanRegisterEvent(created.id);

      if (!txReceipt) {
        // User cancelled signing — keep as draft, don't mark active
        Alert.alert(
          'Event saved as draft',
          'Your event was created but is not yet active. Sign the contract transaction to activate it.',
        );
        setStatus('idle');
        return;
      }

      // 6. Both confirmations received → event is active
      setStatus('done');
      form.resetForm();

      Alert.alert(
        '🎉 Event Created!',
        'Your event is now live on Agora.',
        [{ text: 'View Events', onPress: () => router.replace('/(tabs)/discover') }],
        { cancelable: false },
      );
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : 'Unknown error';
      setErrorMsg(msg);
      setStatus('error');
    }
  };

  const handleBack = () => router.back();

  const isSubmitting = ['uploading', 'creating', 'signing'].includes(status);

  // ── Render ────────────────────────────────────────────────────────────────

  return (
    <ScrollView
      style={styles.container}
      contentContainerStyle={styles.content}
      keyboardShouldPersistTaps="handled"
    >
      <ProgressBar current={4} total={4} />

      <Text style={styles.heading}>Cover Image & Review</Text>
      <Text style={styles.subheading}>
        Upload a cover image, then review all details before publishing.
      </Text>

      {/* ── Cover image picker ─────────────────────────────────────────── */}
      <Pressable style={styles.imagePicker} onPress={pickImage} disabled={isSubmitting}>
        {form.coverImageUri ? (
          <Image source={{ uri: form.coverImageUri }} style={styles.coverImage} />
        ) : (
          <View style={styles.imagePickerPlaceholder}>
            <Text style={styles.imagePickerIcon}>🖼</Text>
            <Text style={styles.imagePickerLabel}>Tap to select cover image</Text>
            <Text style={styles.imagePickerHint}>Recommended: 1600 × 900 px (16:9)</Text>
          </View>
        )}
      </Pressable>

      {/* ── Upload progress bar ────────────────────────────────────────── */}
      {status === 'uploading' && (
        <View style={styles.progressContainer}>
          <View style={styles.progressTrack}>
            <Animated.View
              style={[
                styles.progressFill,
                {
                  width: progressAnim.interpolate({
                    inputRange: [0, 100],
                    outputRange: ['0%', '100%'],
                  }),
                },
              ]}
            />
          </View>
          <Text style={styles.progressLabel}>Uploading… {uploadProgress}%</Text>
        </View>
      )}

      {/* ── Summary cards ──────────────────────────────────────────────── */}
      <SummarySection title="Basic Details">
        <SummaryRow label="Title" value={form.title} />
        <SummaryRow label="Category" value={form.category} />
        <SummaryRow label="Date" value={`${form.eventDate} at ${form.eventTime}`} />
        {form.description ? <SummaryRow label="Description" value={form.description} /> : null}
      </SummarySection>

      <SummarySection title="Location">
        {form.locationType === 'physical' ? (
          <>
            <SummaryRow label="Type" value="Physical" />
            <SummaryRow label="Venue" value={form.venueName} />
            <SummaryRow label="Address" value={form.venueAddress} />
          </>
        ) : (
          <>
            <SummaryRow label="Type" value="Virtual" />
            <SummaryRow label="Link" value={form.virtualLink} />
          </>
        )}
      </SummarySection>

      <SummarySection title="Ticket Tiers">
        {form.tiers.map((t, i) => (
          <View key={t.id} style={reviewStyles.tierRow}>
            <Text style={reviewStyles.tierName}>
              {i + 1}. {t.name || '(unnamed)'}
            </Text>
            <Text style={reviewStyles.tierDetail}>
              {parseFloat(t.priceUsdc) === 0 ? 'Free' : `${t.priceUsdc} USDC`} ·{' '}
              {t.quantity} tickets
            </Text>
            <Text style={reviewStyles.tierDates}>
              Sale: {t.saleStart} → {t.saleEnd}
            </Text>
          </View>
        ))}
      </SummarySection>

      {/* ── Status messages ────────────────────────────────────────────── */}
      {status === 'creating' && (
        <StatusBadge text="⏳ Creating event record…" color={Colors.primaryYellow} />
      )}
      {status === 'signing' && (
        <StatusBadge text="✍️ Awaiting Soroban signature…" color="#5E9AFF" />
      )}
      {status === 'done' && (
        <StatusBadge text="✅ Event is live!" color="#34C759" />
      )}
      {status === 'error' && (
        <StatusBadge text={`❌ ${errorMsg}`} color={Colors.accentRed} />
      )}

      {/* ── Navigation ─────────────────────────────────────────────────── */}
      <View style={styles.navRow}>
        <Button
          title="← Back"
          variant="secondary"
          onPress={handleBack}
          disabled={isSubmitting}
          style={styles.backBtn}
        />
        <Button
          title={isSubmitting ? 'Publishing…' : '🚀 Publish Event'}
          onPress={handleSubmit}
          loading={isSubmitting}
          disabled={isSubmitting || status === 'done'}
          style={styles.publishBtn}
        />
      </View>
    </ScrollView>
  );
}

// ─── Sub-components ───────────────────────────────────────────────────────────

function SummarySection({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <View style={summaryStyles.section}>
      <Text style={summaryStyles.sectionTitle}>{title}</Text>
      {children}
    </View>
  );
}

function SummaryRow({ label, value }: { label: string; value: string }) {
  return (
    <View style={summaryStyles.row}>
      <Text style={summaryStyles.label}>{label}</Text>
      <Text style={summaryStyles.value} numberOfLines={3}>
        {value}
      </Text>
    </View>
  );
}

function StatusBadge({ text, color }: { text: string; color: string }) {
  return (
    <View style={[badgeStyles.container, { borderColor: color }]}>
      <Text style={[badgeStyles.text, { color }]}>{text}</Text>
    </View>
  );
}

// ─── Styles ───────────────────────────────────────────────────────────────────

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: Colors.darkBackground },
  content: { padding: 20, paddingBottom: 60 },
  heading: { fontSize: 22, fontWeight: '700', color: Colors.primaryText, marginBottom: 6 },
  subheading: { fontSize: 14, color: Colors.secondaryText, marginBottom: 24 },

  imagePicker: {
    borderRadius: 12,
    overflow: 'hidden',
    borderWidth: 1.5,
    borderColor: '#2C2C2E',
    borderStyle: 'dashed',
    marginBottom: 20,
    minHeight: 180,
  },
  imagePickerPlaceholder: {
    minHeight: 180,
    alignItems: 'center',
    justifyContent: 'center',
    gap: 8,
    backgroundColor: '#1E1E20',
  },
  imagePickerIcon: { fontSize: 40 },
  imagePickerLabel: { fontSize: 15, color: Colors.primaryText, fontWeight: '600' },
  imagePickerHint: { fontSize: 12, color: Colors.secondaryText },
  coverImage: { width: '100%', height: 200, resizeMode: 'cover' },

  progressContainer: { marginBottom: 16 },
  progressTrack: {
    height: 6,
    borderRadius: 3,
    backgroundColor: '#2C2C2E',
    overflow: 'hidden',
    marginBottom: 6,
  },
  progressFill: {
    height: '100%',
    borderRadius: 3,
    backgroundColor: Colors.primaryYellow,
  },
  progressLabel: { fontSize: 12, color: Colors.primaryYellow, textAlign: 'center' },

  navRow: { flexDirection: 'row', gap: 12, marginTop: 28 },
  backBtn: { flex: 1 },
  publishBtn: { flex: 2 },
});

const summaryStyles = StyleSheet.create({
  section: {
    backgroundColor: '#1E1E20',
    borderRadius: 12,
    borderWidth: 1,
    borderColor: '#2C2C2E',
    padding: 16,
    marginBottom: 16,
  },
  sectionTitle: {
    fontSize: 13,
    fontWeight: '700',
    textTransform: 'uppercase',
    color: Colors.primaryYellow,
    letterSpacing: 1,
    marginBottom: 12,
  },
  row: {
    borderTopWidth: 1,
    borderTopColor: '#2C2C2E',
    paddingTop: 10,
    paddingBottom: 6,
    flexDirection: 'row',
    gap: 12,
  },
  label: {
    width: 90,
    fontSize: 12,
    color: Colors.secondaryText,
    fontWeight: '500',
    flexShrink: 0,
  },
  value: { flex: 1, fontSize: 13, color: Colors.primaryText },
});

const reviewStyles = StyleSheet.create({
  tierRow: {
    borderTopWidth: 1,
    borderTopColor: '#2C2C2E',
    paddingTop: 10,
    paddingBottom: 6,
  },
  tierName: { fontSize: 14, fontWeight: '600', color: Colors.primaryText, marginBottom: 2 },
  tierDetail: { fontSize: 13, color: Colors.primaryYellow, marginBottom: 2 },
  tierDates: { fontSize: 12, color: Colors.secondaryText },
});

const badgeStyles = StyleSheet.create({
  container: {
    borderWidth: 1,
    borderRadius: 8,
    padding: 12,
    marginBottom: 12,
    alignItems: 'center',
  },
  text: { fontSize: 14, fontWeight: '600' },
});
