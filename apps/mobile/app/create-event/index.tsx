/**
 * Entry point for the Event Creation Wizard.
 *
 * Immediately redirects to Step 1. Placing an index file here lets
 * expo-router expose `/create-event` as a valid route while keeping
 * the actual wizard step files well-organised under the same directory.
 */
import { Redirect } from 'expo-router';
import { useEventCreationForm } from '@/hooks/useEventCreationForm';

export default function CreateEventIndex() {
  // Always start the wizard at step 1 (state resets on resetForm)
  const form = useEventCreationForm();
  if (form.currentStep !== 1) {
    form.goToStep(1);
  }
  return <Redirect href="/create-event/step-1-basics" />;
}
