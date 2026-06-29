import React from 'react';
import { trackEvent } from '../../utils/analytics';
import { useRouter } from 'next/navigation';

export function CreateEventButton() {
  const router = useRouter();

  const handleClick = () => {
    trackEvent('event', 'create_clicked');
    router.push('/events/new');
  };

  return (
    <button
      onClick={handleClick}
      className="bg-blue-600 text-white px-4 py-2 rounded shadow hover:bg-blue-700 transition"
    >
      Create Event
    </button>
  );
}
