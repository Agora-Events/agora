"use client";

import React, { useState, useEffect, useCallback } from "react";
import { useRouter } from "next/navigation";
import { Button } from "@/components/ui/button";
import Image from "next/image";

type SettingsTab = "profile" | "notifications" | "payment";

interface ProfileData {
  displayName: string;
  bio: string;
  avatarUrl: string;
}

interface NotificationSettings {
  email: boolean;
  inApp: boolean;
}

const tabs: { id: SettingsTab; label: string }[] = [
  { id: "profile", label: "Profile" },
  { id: "notifications", label: "Notifications" },
  { id: "payment", label: "Payment" },
];

export default function SettingsPage() {
  const router = useRouter();
  const [activeTab, setActiveTab] = useState<SettingsTab>("profile");
  const [isSaving, setIsSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [saveSuccess, setSaveSuccess] = useState(false);

  const [profileData, setProfileData] = useState<ProfileData>({
    displayName: "",
    bio: "",
    avatarUrl: "",
  });
  const [initialProfile, setInitialProfile] = useState<ProfileData>({
    displayName: "",
    bio: "",
    avatarUrl: "",
  });

  const [notifications, setNotifications] = useState<NotificationSettings>({
    email: true,
    inApp: true,
  });
  const [initialNotifications, setInitialNotifications] =
    useState<NotificationSettings>({
      email: true,
      inApp: true,
    });

  const [avatarPreview, setAvatarPreview] = useState<string>("");

  const isDirty =
    profileData.displayName !== initialProfile.displayName ||
    profileData.bio !== initialProfile.bio ||
    profileData.avatarUrl !== initialProfile.avatarUrl ||
    notifications.email !== initialNotifications.email ||
    notifications.inApp !== initialNotifications.inApp;

  useEffect(() => {
    const fetchProfile = async () => {
      try {
        const response = await fetch("/api/profile");
        if (!response.ok) {
          throw new Error(`Failed to load profile: ${response.status}`);
        }
        const data = await response.json();
        const profile = data.profile;
        const loadedProfile: ProfileData = {
          displayName: profile.displayName ?? "",
          bio: profile.bio ?? "",
          avatarUrl: profile.avatarUrl ?? "",
        };
        setProfileData(loadedProfile);
        setInitialProfile(loadedProfile);
        if (loadedProfile.avatarUrl) {
          setAvatarPreview(loadedProfile.avatarUrl);
        }
      } catch {
        setSaveError("Failed to load profile data");
      }
    };

    fetchProfile();
  }, []);

  const handleBeforeUnload = useCallback(
    (event: BeforeUnloadEvent) => {
      if (!isDirty) return;
      event.preventDefault();
      event.returnValue = "";
    },
    [isDirty],
  );

  useEffect(() => {
    window.addEventListener("beforeunload", handleBeforeUnload);
    return () =>
      window.removeEventListener("beforeunload", handleBeforeUnload);
  }, [handleBeforeUnload]);

  useEffect(() => {
    const handleRouteChange = () => {
      if (isDirty && !window.confirm("You have unsaved changes. Leave anyway?")) {
        router.push(window.location.pathname);
        return false;
      }
      return true;
    };

    router.beforePopState((state) => {
      if (isDirty && !window.confirm("You have unsaved changes. Leave anyway?")) {
        window.history.pushState(null, "", state.url);
        return false;
      }
      return true;
    });

    return () => {
      router.beforePopState(() => true);
    };
  }, [isDirty, router]);

  const handleSaveProfile = async () => {
    setIsSaving(true);
    setSaveError(null);
    setSaveSuccess(false);

    try {
      const response = await fetch("/api/profile", {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          display_name: profileData.displayName,
          bio: profileData.bio || null,
          avatar_url: profileData.avatarUrl || null,
        }),
      });

      if (!response.ok) {
        const data = await response.json().catch(() => ({}));
        throw new Error(data.error || `Save failed (${response.status})`);
      }

      const data = await response.json();
      const updated: ProfileData = {
        displayName: data.profile.displayName ?? profileData.displayName,
        bio: data.profile.bio ?? profileData.bio,
        avatarUrl: data.profile.avatarUrl ?? profileData.avatarUrl,
      };
      setProfileData(updated);
      setInitialProfile(updated);
      setSaveSuccess(true);
    } catch (err) {
      setSaveError(err instanceof Error ? err.message : "Failed to save profile");
    } finally {
      setIsSaving(false);
    }
  };

  const handleSaveNotifications = async () => {
    setIsSaving(true);
    setSaveError(null);
    setSaveSuccess(false);

    try {
      await new Promise((resolve) => setTimeout(resolve, 600));
      setInitialNotifications({ ...notifications });
      setSaveSuccess(true);
    } catch {
      setSaveError("Failed to save notification settings");
    } finally {
      setIsSaving(false);
    }
  };

  const handleAvatarChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) {
      const reader = new FileReader();
      reader.onloadend = () => {
        const result = reader.result as string;
        setAvatarPreview(result);
        setProfileData((prev) => ({ ...prev, avatarUrl: result }));
      };
      reader.readAsDataURL(file);
    }
  };

  const resetChanges = () => {
    setProfileData({ ...initialProfile });
    setNotifications({ ...initialNotifications });
    setAvatarPreview(initialProfile.avatarUrl);
    setSaveError(null);
    setSaveSuccess(false);
  };

  return (
    <main className="flex flex-col min-h-screen bg-base">
      <div className="w-full max-w-3xl mx-auto px-4 py-10">
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-ink-soft">Settings</h1>
          <p className="text-muted-text mt-1">
            Manage your account preferences
          </p>
        </div>

        <div className="bg-white rounded-2xl border border-border-warm shadow-[_-4px_4px_0_rgba(0,0,0,1)] overflow-hidden">
          <div className="flex border-b border-border-warm">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                type="button"
                onClick={() => setActiveTab(tab.id)}
                className={`flex-1 px-6 py-4 text-sm font-semibold transition-colors ${
                  activeTab === tab.id
                    ? "text-ink-soft border-b-2 border-ink"
                    : "text-muted-text hover:text-ink-soft"
                }`}
              >
                {tab.label}
              </button>
            ))}
          </div>

          <div className="p-6 md:p-8">
            {saveError && (
              <div className="mb-6 rounded-xl border border-error bg-red-50 px-4 py-3 text-sm text-error">
                {saveError}
              </div>
            )}
            {saveSuccess && (
              <div className="mb-6 rounded-xl border border-success-light bg-success-light px-4 py-3 text-sm text-ink-soft">
                Saved successfully.
              </div>
            )}

            {activeTab === "profile" && (
              <div className="space-y-6">
                <div className="flex flex-col sm:flex-row items-start gap-6">
                  <div className="flex flex-col items-center gap-3">
                    <div className="relative w-24 h-24 rounded-full border-2 border-border-warm overflow-hidden bg-surface">
                      {avatarPreview ? (
                        <Image
                          src={avatarPreview}
                          alt="Avatar preview"
                          fill
                          className="object-cover"
                        />
                      ) : (
                        <div className="w-full h-full flex items-center justify-center text-2xl font-bold text-ink-soft">
                          {profileData.displayName.charAt(0).toUpperCase() || "U"}
                        </div>
                      )}
                    </div>
                    <label className="cursor-pointer inline-flex items-center justify-center gap-2 px-4 py-2 text-xs font-semibold rounded-full border border-black bg-white hover:bg-surface transition-colors">
                      <svg
                        width="14"
                        height="14"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        strokeWidth="2"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                      >
                        <path d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z" />
                        <circle cx="12" cy="13" r="4" />
                      </svg>
                      Change photo
                      <input
                        type="file"
                        accept="image/*"
                        className="hidden"
                        onChange={handleAvatarChange}
                      />
                    </label>
                  </div>
                  <div className="flex-1 w-full space-y-4">
                    <div>
                      <label className="block text-sm font-medium text-ink-soft mb-1.5">
                        Display name
                      </label>
                      <input
                        type="text"
                        value={profileData.displayName}
                        onChange={(e) =>
                          setProfileData((p) => ({
                            ...p,
                            displayName: e.target.value,
                          }))
                        }
                        maxLength={50}
                        className="w-full h-11 px-3 rounded-xl bg-white border border-black/15 focus:border-black focus:ring-0 outline-none text-sm"
                        placeholder="Your display name"
                      />
                      <p className="mt-1 text-xs text-muted-text">
                        {profileData.displayName.length}/50 characters
                      </p>
                    </div>
                    <div>
                      <label className="block text-sm font-medium text-ink-soft mb-1.5">
                        Bio
                      </label>
                      <textarea
                        value={profileData.bio}
                        onChange={(e) =>
                          setProfileData((p) => ({ ...p, bio: e.target.value }))
                        }
                        maxLength={500}
                        rows={4}
                        className="w-full px-3 py-2 rounded-xl bg-white border border-black/15 focus:border-black focus:ring-0 outline-none text-sm resize-none"
                        placeholder="Tell us about yourself"
                      />
                      <p className="mt-1 text-xs text-muted-text">
                        {profileData.bio.length}/500 characters
                      </p>
                    </div>
                  </div>
                </div>

                <div className="flex justify-end gap-3 pt-4 border-t border-border-warm">
                  {isDirty && (
                    <button
                      type="button"
                      onClick={resetChanges}
                      className="px-5 py-2.5 text-sm font-semibold rounded-full border border-black bg-white hover:bg-surface transition-colors"
                      disabled={isSaving}
                    >
                      Discard
                    </button>
                  )}
                  <Button
                    onClick={handleSaveProfile}
                    disabled={!isDirty || isSaving}
                    variant="primary"
                  >
                    {isSaving ? "Saving..." : "Save changes"}
                  </Button>
                </div>
              </div>
            )}

            {activeTab === "notifications" && (
              <div className="space-y-6">
                <p className="text-sm text-muted-text">
                  Configure how you want to receive notifications.
                </p>
                <div className="space-y-4">
                  <div className="flex items-center justify-between p-4 rounded-xl border border-border-warm bg-surface">
                    <div>
                      <p className="text-sm font-semibold text-ink-soft">
                        Email notifications
                      </p>
                      <p className="text-xs text-muted-text mt-0.5">
                        Receive event updates via email
                      </p>
                    </div>
                    <Toggle
                      enabled={notifications.email}
                      onChange={(enabled) =>
                        setNotifications((n) => ({ ...n, email: enabled }))
                      }
                    />
                  </div>
                  <div className="flex items-center justify-between p-4 rounded-xl border border-border-warm bg-surface">
                    <div>
                      <p className="text-sm font-semibold text-ink-soft">
                        In-app notifications
                      </p>
                      <p className="text-xs text-muted-text mt-0.5">
                        Show alerts inside the app
                      </p>
                    </div>
                    <Toggle
                      enabled={notifications.inApp}
                      onChange={(enabled) =>
                        setNotifications((n) => ({ ...n, inApp: enabled }))
                      }
                    />
                  </div>
                </div>

                <div className="flex justify-end gap-3 pt-4 border-t border-border-warm">
                  {isDirty && (
                    <button
                      type="button"
                      onClick={resetChanges}
                      className="px-5 py-2.5 text-sm font-semibold rounded-full border border-black bg-white hover:bg-surface transition-colors"
                      disabled={isSaving}
                    >
                      Discard
                    </button>
                  )}
                  <Button
                    onClick={handleSaveNotifications}
                    disabled={!isDirty || isSaving}
                    variant="primary"
                  >
                    {isSaving ? "Saving..." : "Save changes"}
                  </Button>
                </div>
              </div>
            )}

            {activeTab === "payment" && (
              <div className="space-y-6">
                <p className="text-sm text-muted-text">
                  Manage your payment methods and billing preferences.
                </p>
                <div className="flex flex-col items-center justify-center py-12 text-center">
                  <div className="w-16 h-16 rounded-full bg-surface flex items-center justify-center mb-4">
                    <Image
                      src="/icons/ticket.svg"
                      alt="Payment"
                      width={24}
                      height={24}
                    />
                  </div>
                  <h3 className="text-lg font-semibold text-ink-soft mb-1">
                    Payment settings coming soon
                  </h3>
                  <p className="text-sm text-muted-text max-w-xs">
                    Configure billing and payment methods here in a future
                    update.
                  </p>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </main>
  );
}

function Toggle({
  enabled,
  onChange,
}: {
  enabled: boolean;
  onChange: (value: boolean) => void;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={enabled}
      onClick={() => onChange(!enabled)}
      className={`relative inline-flex h-6 w-11 shrink-0 items-center rounded-full transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent ${
        enabled ? "bg-ink" : "bg-black/15"
      }`}
    >
      <span
        className={`inline-block h-4 w-4 rounded-full bg-white transition-transform ${
          enabled ? "translate-x-6" : "translate-x-1"
        }`}
      />
    </button>
  );
}
