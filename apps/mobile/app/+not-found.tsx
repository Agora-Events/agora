import { Stack, useRouter } from 'expo-router';
import { StyleSheet } from 'react-native';

import { ThemedText } from '@/components/ThemedText';
import { ThemedView } from '@/components/ThemedView';
import { Button } from '@/components/ui/Button';

export default function NotFoundScreen() {
  const router = useRouter();

  return (
    <>
      <Stack.Screen options={{ title: 'Page not found' }} />
      <ThemedView style={styles.container}>
        <ThemedText type="title" style={styles.title}>
          Page not found
        </ThemedText>
        <ThemedText style={styles.message}>
          We couldn't find that page. The event may have ended or the link may be wrong.
        </ThemedText>
        <Button
          title="Go to Discover"
          onPress={() => router.replace('/(tabs)/discover')}
          style={styles.button}
        />
      </ThemedView>
    </>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    padding: 24,
  },
  title: {
    marginBottom: 12,
    textAlign: 'center',
  },
  message: {
    textAlign: 'center',
    marginBottom: 32,
    lineHeight: 22,
    opacity: 0.7,
  },
  button: {
    width: '100%',
  },
});
