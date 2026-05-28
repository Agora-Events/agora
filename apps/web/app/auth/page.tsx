"use client";

import { useState, FormEvent } from "react";
import Image from "next/image";
import { useRouter } from "next/navigation";
import { Button } from "@/components/ui/button";

function validateEmail(email: string): boolean {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email);
}

export default function AuthPage() {
  const [email, setEmail] = useState("");
  const [error, setError] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const router = useRouter();

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();

    if (!email) {
      setError("Email is required");
      return;
    }

    if (!validateEmail(email)) {
      setError("Enter a valid email");
      return;
    }

    setError("");
    setIsLoading(true);

    try {
      const res = await fetch('/api/auth/email', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email }),
      });

      if (!res.ok) {
        const data = await res.json();
        throw new Error(data.error || 'Authentication failed');
      }

      router.push('/home');
    } catch (err: unknown) {
      const message =
        err instanceof Error ? err.message : "Something went wrong";
      setError(message);
    } finally {
      setIsLoading(false);
    }
  };

  const handleGoBack = () => {
    if (typeof window !== 'undefined' && window.history.length > 1) {
      router.back();
    } else {
      router.push('/');
    }
  };

  return (
    <main className="min-h-screen bg-sand relative flex items-center justify-center">
      {/* Back Button */}
      <Button
        type="button"
        variant="secondary"
        onClick={handleGoBack}
        className="absolute top-10 left-16 text-sm py-2"
      >
        <Image src="/icons/arrow-left.svg" alt="Back" width={16} height={16} />
        Back
      </Button>

      {/* Auth Card */}
      <div className="w-[360px] bg-[#F3EEDC] rounded-2xl shadow-[0_8px_0_#00000020] p-8 flex flex-col items-center">
        {/* Logo Container */}
        <div className="mb-6">
          <div className="bg-white border-2 border-black rounded-lg px-4 py-2 shadow-[2px_2px_0_#000]">
            <Image
              src="/logo/agora logo.svg"
              alt="Agora"
              width={70}
              height={24}
            />
          </div>
        </div>

        <h1 className="text-xl font-semibold mb-1 text-black">
          Welcome to agora
        </h1>
        <p className="text-xs text-gray-600 mb-6 text-center">
          Please sign in or sign up below.
        </p>

        <form onSubmit={handleSubmit} className="w-full">
          <label className="text-sm font-medium block mb-2 text-black">
            Email
          </label>

          <input
            type="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            className="w-full bg-white border-2 border-black rounded-full px-4 py-2 mb-4 outline-none"
          />

          {error && <p className="text-xs text-red-500 mb-3">{error}</p>}

          <Button
            type="submit"
            variant="primary"
            disabled={isLoading}
            className="
              w-full
              bg-accent
              hover:bg-accent-hover
              rounded-full
              py-2
              font-medium
              flex items-center justify-center gap-2
              mb-4
              border-2 border-black
              shadow-[0_4px_0_#000]
              active:translate-y-[2px]
              active:shadow-[0_2px_0_#000]
              transition
            "
          >
            Continue with Email
            <Image src="/icons/arrow-right.svg" alt="Arrow" width={16} height={16} />
          </Button>

          <Button
            type="button"
            onClick={() => router.push("/api/auth/google")}
            className="
              w-full
              bg-black
              text-white
              rounded-full
              py-2
              flex items-center justify-center gap-2
              mb-3
              border-2 border-black
              shadow-[0_4px_0_#000]
              active:translate-y-[2px]
              active:shadow-[0_2px_0_#000]
            "
          >
            <Image src="/icons/google.svg" alt="Google" width={16} height={16} />
            Sign in with Google
          </Button>

          <Button
            type="button"
            onClick={() => router.push("/api/auth/apple")}
            className="
              w-full
              bg-black
              text-white
              rounded-full
              py-2
              flex items-center justify-center gap-2
              border-2 border-black
              shadow-[0_4px_0_#000]
              active:translate-y-[2px]
              active:shadow-[0_2px_0_#000]
            "
          >
            <Image src="/icons/apple.svg" alt="Apple" width={16} height={16} />
            Sign in with Apple
          </Button>
        </form>
      </div>
    </main>
  );
}
