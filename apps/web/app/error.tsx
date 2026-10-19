'use client'

import { useEffect } from 'react'
import Link from 'next/link'

export default function GlobalError({ error, reset }: { error: Error; reset: () => void }) {
  useEffect(() => {
    // Log error to console or an external service
    // In a real app, replace this with your telemetry/logging
    console.error('Unhandled error (GlobalError):', error)
  }, [error])

  return (
    <div style={{ minHeight: '100vh', display: 'flex', alignItems: 'center', justifyContent: 'center', padding: 32 }}>
      <div style={{ textAlign: 'center', maxWidth: 720 }}>
        <h1 style={{ fontSize: 32 }}>Something went wrong</h1>
        <p style={{ color: '#555', marginTop: 8 }}>An unexpected error happened. You can try to recover or go back home.</p>

        <div style={{ marginTop: 20, display: 'flex', gap: 12, justifyContent: 'center' }}>
          <button onClick={() => reset()} style={{ padding: '10px 16px', borderRadius: 6, background: '#111', color: '#fff', border: 'none' }}>
            Try again
          </button>
          <Link href="/">
            <span style={{ display: 'inline-block', padding: '10px 16px', borderRadius: 6, background: '#eee', color: '#111', textDecoration: 'none' }}>
              Home
            </span>
          </Link>
        </div>
      </div>
    </div>
  )
}
