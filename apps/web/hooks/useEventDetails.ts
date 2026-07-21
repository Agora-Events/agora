"use client";


import useSWR from 'swr';

const fetcher = (url: string) => fetch(url).then((res) => {
  if (!res.ok) throw new Error('Failed to fetch event details');
  return res.json();
});

export function useEventDetails(id?: string) {
  const { data, error, isLoading, mutate } = useSWR(
    id ? `/api/v1/events/${id}` : null,
    fetcher,
    {
      errorRetryCount: 3,
    }
  );

  return {
    event: data,
    isLoading,
    isError: error,
    mutate,
  };
}
