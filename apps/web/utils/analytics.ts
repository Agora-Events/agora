import posthog from 'posthog-js';

// Initialize posthog only on the client side
if (typeof window !== 'undefined') {
  posthog.init(process.env.NEXT_PUBLIC_POSTHOG_KEY || 'dummy_key', {
    api_host: process.env.NEXT_PUBLIC_POSTHOG_HOST || 'https://app.posthog.com',
    loaded: (posthog_instance) => {
      if (process.env.NODE_ENV === 'development') {
        posthog_instance.debug();
      }
    },
  });
}

export const trackEvent = (category: string, action: string, properties?: Record<string, any>) => {
  if (typeof window !== 'undefined') {
    posthog.capture(action, { category, ...properties });
  }
};
