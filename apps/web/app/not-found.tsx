import Link from 'next/link'
import Image from 'next/image'
import { Button } from '@/components/ui/button'

export default function NotFound() {
  return (
    <main className="min-h-screen bg-base dark:bg-base text-ink-deep dark:text-white flex items-center justify-center p-8 transition-colors">
      <div className="text-center max-w-[720px] w-full flex flex-col items-center">
        <div className="w-[220px] h-[220px] mx-auto flex items-center justify-center rounded-3xl bg-surface dark:bg-surface border-2 border-black dark:border-white shadow-[4px_4px_0px_0px_#000] dark:shadow-[4px_4px_0px_0px_#fff]">
          <Image
            src="/icons/404-illustration.svg"
            width={120}
            height={120}
            alt="404 illustration"
            priority
          />
        </div>

        <h1 className="mt-7 text-3xl sm:text-4xl font-bold tracking-tight text-ink-deep dark:text-white font-heading">
          404 — Page not found
        </h1>
        <p className="text-muted-text dark:text-gray-300 mt-2 text-base sm:text-lg">
          We couldn&apos;t find the page you&apos;re looking for.
        </p>

        <div className="mt-6 flex flex-col items-center gap-6">
          <Link href="/">
            <Button
              backgroundColor="bg-black dark:bg-accent"
              textColor="text-white dark:text-black"
              className="px-6 py-3"
            >
              Back to home
            </Button>
          </Link>

          <div className="flex flex-col sm:flex-row items-center justify-center gap-4 text-sm font-medium pt-2 border-t border-subtle dark:border-gray-800 w-full">
            <span className="text-muted-text dark:text-gray-400">Suggested links:</span>
            <div className="flex items-center gap-4 flex-wrap justify-center">
              <Link
                href="/discover"
                className="text-ink-deep dark:text-accent font-semibold hover:underline transition-all"
              >
                Discover events
              </Link>
              <span className="text-subtle dark:text-gray-600 hidden sm:inline">•</span>
              <Link
                href="/events/create"
                className="text-ink-deep dark:text-accent font-semibold hover:underline transition-all"
              >
                Create an event
              </Link>
              <span className="text-subtle dark:text-gray-600 hidden sm:inline">•</span>
              <Link
                href="/help"
                className="text-ink-deep dark:text-accent font-semibold hover:underline transition-all"
              >
                Help center
              </Link>
            </div>
          </div>
        </div>
      </div>
    </main>
  )
}
