"use client";

import { useState, FormEvent } from "react";
import Image from "next/image";

function validateEmail(email: string): boolean {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email);
}

export default function AuthPage() {
  const [email, setEmail] = useState("");
  const [error, setError] = useState("");
  const [isLoading, setIsLoading] = useState(false);

  const handleSubmit = (e: FormEvent) => {
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

    setTimeout(() => {
      setIsLoading(false);
    }, 1200);
  };

  return (
    <main className="min-h-screen bg-[#A9A495] relative flex items-center justify-center">
      {/* Back Button */}
      <button
        type="button"
        className="
          absolute top-10 left-16
          bg-white
          px-6 py-2
          rounded-full
          font-medium text-sm
          flex items-center gap-2
          border-2 border-black
          shadow-[0_4px_0_#000]
          active:translate-y-[2px]
          active:shadow-[0_2px_0_#000]
        "
      >
        <Image src="/icons/arrow-left.svg" alt="Back" width={16} height={16} />
        Back
      </button>

      {/* Auth Card */}
      <div className="w-[360px] bg-[#F3EEDC] rounded-2xl shadow-[0_8px_0_#00000020] p-8 flex flex-col items-center">
        {/* Logo Container (Ticked Black Card Style) */}
        <div className="mb-6">
          <div
            className="
              bg-white
              border-2 border-black
              rounded-lg
              px-4 py-2
              shadow-[2px_2px_0_#000]
            "
          >
            <Image
              src="/logo/agora logo.svg"
              alt="Agora"
              width={70}
              height={24}
            />
          </div>
        </div>

        {/* Title */}
        <h1 className="text-xl font-semibold mb-1 text-black">
          Welcome to agora
        </h1>
        <p className="text-xs text-gray-600 mb-6 text-center">
          Please sign in or sign up below.
        </p>

        {/* Form */}
        <form onSubmit={handleSubmit} className="w-full">
          {/* Email */}
          <label className="text-sm font-medium block mb-2 text-black">
            Email
          </label>

          <input
            type="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            className="
              w-full
              bg-white
              border-2 border-black
              rounded-full
              px-4 py-2
              mb-4
              outline-none
            "
          />

          {error && <p className="text-xs text-red-500 mb-3">{error}</p>}

          {/* Yellow Continue Button */}
          <button
            type="submit"
            disabled={isLoading}
            className="
              w-full
              bg-[#FACC15]
              hover:bg-[#EAB308]
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
            <Image
              src="/icons/arrow-right.svg"
              alt="Arrow"
              width={16}
              height={16}
            />
          </button>

          {/* Google Button */}
          <button
            type="button"
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
            <Image
              src="/icons/google.svg"
              alt="Google"
              width={16}
              height={16}
            />
            Sign in with Google
          </button>

          {/* Apple Button */}
          <button
            type="button"
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
          </button>
        </form>
      </div>
    </main>
  );
}
