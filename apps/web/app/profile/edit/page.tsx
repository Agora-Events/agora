"use client";

import { FormEvent, useEffect, useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";
import { useProfileDetails } from "@/hooks/use-profile-details";

const BIO_MAX_LENGTH = 160;

export default function EditProfilePage() {
  const router = useRouter();
  const { details, save, loadError } = useProfileDetails();
  const [displayName, setDisplayName] = useState(details.displayName);
  const [bio, setBio] = useState(details.bio);
  const [saveError, setSaveError] = useState("");

  useEffect(() => {
    setDisplayName(details.displayName);
    setBio(details.bio);
  }, [details]);

  const handleSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setSaveError("");

    if (!displayName.trim()) {
      setSaveError("Display name is required.");
      return;
    }

    if (bio.length > BIO_MAX_LENGTH) {
      setSaveError(`Biography must be ${BIO_MAX_LENGTH} characters or fewer.`);
      return;
    }

    try {
      save({ displayName: displayName.trim(), bio });
      router.push("/profile");
    } catch {
      setSaveError("Your profile could not be saved. Please try again.");
    }
  };

  return (
    <main className="flex min-h-screen flex-col bg-[#FFFBE9]">
      <Navbar />
      <section className="flex-1 w-full max-w-2xl mx-auto px-4 py-10">
        <Link
          href="/profile"
          className="mb-6 inline-flex text-sm font-medium text-[#1A1A1A] underline underline-offset-4"
        >
          Back to profile
        </Link>

        <div className="rounded-2xl border border-[#F0EAD6] bg-white p-6 shadow-sm sm:p-8">
          <h1 className="text-2xl font-semibold text-[#1A1A1A]">Edit your details</h1>
          <p className="mt-2 text-sm text-gray-500">
            Update your name and tell others a little about yourself.
          </p>

          <form onSubmit={handleSubmit} className="mt-8 flex flex-col gap-6">
            <div>
              <label htmlFor="display-name" className="mb-2 block text-sm font-medium text-[#1A1A1A]">
                Display name
              </label>
              <input
                id="display-name"
                name="displayName"
                type="text"
                value={displayName}
                onChange={(event) => setDisplayName(event.target.value)}
                required
                className="w-full rounded-xl border border-[#D8D2C2] bg-white px-4 py-3 text-[#1A1A1A] outline-none focus:border-[#1A1A1A]"
              />
            </div>

            <div>
              <label htmlFor="bio" className="mb-2 block text-sm font-medium text-[#1A1A1A]">
                Biography
              </label>
              <textarea
                id="bio"
                name="bio"
                value={bio}
                onChange={(event) => setBio(event.target.value.slice(0, BIO_MAX_LENGTH))}
                maxLength={BIO_MAX_LENGTH}
                rows={5}
                aria-describedby="bio-character-count"
                className="w-full resize-y rounded-xl border border-[#D8D2C2] bg-white px-4 py-3 text-[#1A1A1A] outline-none focus:border-[#1A1A1A]"
              />
              <p
                id="bio-character-count"
                aria-live="polite"
                className={`mt-2 text-right text-sm ${
                  bio.length >= BIO_MAX_LENGTH ? "text-red-600" : "text-gray-500"
                }`}
              >
                {bio.length}/{BIO_MAX_LENGTH} characters
              </p>
            </div>

            {(loadError || saveError) && (
              <p role="alert" className="text-sm text-red-600">
                {loadError || saveError}
              </p>
            )}

            <button
              type="submit"
              className="w-full rounded-full border border-black bg-[#FDDA23] px-6 py-3 font-semibold text-black shadow-[0_4px_0_#000] transition hover:translate-y-[2px] hover:shadow-[0_2px_0_#000] active:translate-y-[4px] active:shadow-none sm:w-auto sm:self-start"
            >
              Save changes
            </button>
          </form>
        </div>
      </section>
      <Footer />
    </main>
  );
}
