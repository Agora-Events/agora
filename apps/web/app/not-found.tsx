import Link from 'next/link'
import Image from 'next/image'

export default function NotFound() {
  return (
    <main style={{ minHeight: '100vh', display: 'flex', alignItems: 'center', justifyContent: 'center', padding: 32 }}>
      <div style={{ textAlign: 'center', maxWidth: 720 }}>
        <div style={{ width: 220, height: 220, margin: '0 auto', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
          <Image src="/icons/404-illustration.svg" width={120} height={120} alt="404 illustration" />
        </div>

        <h1 style={{ marginTop: 28, fontSize: 32, letterSpacing: '-0.02em' }}>404 — Page not found</h1>
        <p style={{ color: '#555', marginTop: 8 }}>We couldn&apos;t find the page you&apos;re looking for.</p>

        <div style={{ marginTop: 20 }}>
          <Link href="/">
            <span style={{ display: 'inline-block', background: '#111', color: '#fff', padding: '10px 16px', borderRadius: 6, textDecoration: 'none' }}>
              Back to home
            </span>
          </Link>
        </div>
      </div>
    </main>
  )
}
