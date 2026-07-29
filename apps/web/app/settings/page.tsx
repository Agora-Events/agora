"use client";

import React, { useState, useEffect, useCallback } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import Image from "next/image";
import { useAuth } from "@/hooks/useAuth";
import { useTheme } from "@/components/providers/theme-context";

type SettingsTab = "profile" | "notifications" | "payment" | "appearance";

interface ProfileData {
  displayName: string;
  bio: string;
  avatarUrl: string;
}

interface NotificationSettings {
  email: boolean;
  inApp: boolean;
}

interface PayoutPreferences {
  milestonePlan: string;
  withdrawalCap: number;
}

interface WalletData {
  address: string;
  usdcBalance: number;
  payoutPreferences?: PayoutPreferences;
}

const tabs: { id: SettingsTab; label: string }[] = [
  { id: "profile", label: "Profile" },
  { id: "notifications", label: "Notifications" },
  { id: "payment", label: "Payment" },
  { id: "appearance", label: "Appearance" },
];

export default function SettingsPage() {
  const router = useRouter();
  const { user, isAuthenticated, isLoading: isAuthLoading, signOut } = useAuth();
  const { theme, toggleTheme } = useTheme();
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

  const [walletData, setWalletData] = useState<WalletData | null>(null);
  const [isWalletLoading, setIsWalletLoading] = useState(false);
  const [walletError, setWalletError] = useState<string | null>(null);

  useEffect(() => {
    if (activeTab === "payment" && isAuthenticated) {
      const fetchWallet = async () => {
        setIsWalletLoading(true);
        setWalletError(null);
        try {
          const res = await fetch("/api/wallet");
          if (!res.ok) throw new Error("Failed to fetch wallet");
          const data = await res.json();
          setWalletData(data.wallet || data);
        } catch (err) {
          setWalletError("Failed to load wallet data");
        } finally {
          setIsWalletLoading(false);
        }
      };
      fetchWallet();
    }
  }, [activeTab, isAuthenticated]);

  const isDirty =
    profileData.displayName !== initialProfile.displayName ||
    profileData.bio !== initialProfile.bio ||
    profileData.avatarUrl !== initialProfile.avatarUrl ||
    notifications.email !== initialNotifications.email ||
    notifications.inApp !== initialNotifications.inApp;

  // Settings are personal — send signed-out visitors to the auth page.
  useEffect(() => {
    if (!isAuthLoading && !isAuthenticated) {
      router.replace("/auth");
    }
  }, [isAuthLoading, isAuthenticated, router]);

  useEffect(() => {
    if (!isAuthenticated) return;

    const controller = new AbortController();

    const fetchProfile = async () => {
      try {
        const response = await fetch("/api/profile", {
          signal: controller.signal,
        });
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
      } catch (err) {
        if (err instanceof Error && err.name === "AbortError") return;
        setSaveError("Failed to load profile data");
      }
    };

    fetchProfile();
    return () => controller.abort();
  }, [isAuthenticated]);

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
    // Next.js App Router does not support router.beforePopState;
    // unsaved-changes prompt on navigation is handled by the beforeunload event above.
    return () => {
      // cleanup
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
        <div className="mb-8 flex flex-wrap items-start justify-between gap-4">
          <div>
            <h1 className="text-3xl font-bold text-ink-soft">Settings</h1>
            <p className="text-muted-text mt-1">
              Manage your account preferences
            </p>
            {user && (
              <p className="text-muted-text mt-1 text-sm break-all">
                Signed in as{" "}
                <span className="font-medium text-ink-soft">
                  {user.email ?? user.walletAddress ?? user.id}
                </span>
              </p>
            )}
          </div>
          {user && (
            <Button
              variant="secondary"
              onClick={signOut}
              className="shrink-0"
            >
              Sign out
            </Button>
          )}
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
            <Link
              href="/settings/subscriptions"
              className="flex-1 px-6 py-4 text-sm font-semibold text-center transition-colors text-muted-text hover:text-ink-soft"
            >
              Subscriptions
            </Link>
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
                
                {isWalletLoading ? (
                  <div className="py-12 text-center text-sm text-muted-text">
                    Loading wallet data...
                  </div>
                ) : walletError ? (
                  <div className="py-12 text-center text-sm text-error">
                    {walletError}
                  </div>
                ) : walletData ? (
                  <div className="space-y-8">
                    <div className="p-6 rounded-xl border border-border-warm bg-surface space-y-4">
                      <div className="flex items-center justify-between">
                        <div>
                          <h3 className="text-sm font-semibold text-ink-soft">Connected Stellar Wallet</h3>
                          <p className="text-xs text-muted-text mt-1">
                            {walletData.address.substring(0, 4)}...{walletData.address.substring(walletData.address.length - 4)}
                          </p>
                        </div>
                        <Button
                          variant="secondary"
                          onClick={() => setWalletData(null)}
                        >
                          Disconnect
                        </Button>
                      </div>
                      
                      <div className="pt-4 border-t border-border-warm">
                        <p className="text-sm font-medium text-ink-soft mb-1">USDC Balance</p>
                        <p className="text-2xl font-bold text-ink-soft">
                          {walletData.usdcBalance.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })} USDC
                        </p>
                      </div>
                    </div>

                    {walletData.payoutPreferences && (
                      <div className="space-y-4">
                        <h3 className="text-sm font-semibold text-ink-soft">Payout Preferences</h3>
                        <div className="p-6 rounded-xl border border-border-warm bg-surface space-y-4">
                          <div>
                            <label className="block text-sm font-medium text-ink-soft mb-1.5">
                              Milestone Plan Selection
                            </label>
                            <select
                              className="w-full h-11 px-3 rounded-xl bg-white border border-black/15 focus:border-black focus:ring-0 outline-none text-sm"
                              defaultValue={walletData.payoutPreferences.milestonePlan}
                            >
                              <option value="standard">Standard</option>
                              <option value="accelerated">Accelerated</option>
                            </select>
                          </div>
                          <div>
                            <label className="block text-sm font-medium text-ink-soft mb-1.5">
                              Withdrawal Cap (USDC)
                            </label>
                            <input
                              type="number"
                              className="w-full h-11 px-3 rounded-xl bg-white border border-black/15 focus:border-black focus:ring-0 outline-none text-sm"
                              defaultValue={walletData.payoutPreferences.withdrawalCap}
                            />
                          </div>
                        </div>
                      </div>
                    )}
                  </div>
                ) : (
                  <div className="flex flex-col items-center justify-center py-12 text-center border rounded-xl border-border-warm border-dashed">
                    <div className="w-16 h-16 rounded-full bg-surface flex items-center justify-center mb-4">
                      <Image
                        src="/icons/ticket.svg"
                        alt="Payment"
                        width={24}
                        height={24}
                      />
                    </div>
                    <h3 className="text-lg font-semibold text-ink-soft mb-2">
                      No Wallet Connected
                    </h3>
                    <p className="text-sm text-muted-text max-w-xs mb-6">
                      Connect your Stellar wallet to view balances and configure payouts.
                    </p>
                    <Button variant="primary" onClick={() => setWalletError("Connect wallet not implemented")}>
                      Connect Wallet
                    </Button>
                  </div>
                )}
              </div>
            )}

            {activeTab === "appearance" && (
              <div className="space-y-6">
                <p className="text-sm text-muted-text">
                  Customize the look and feel of the application.
                </p>
                <div className="space-y-4">
                  <div className="flex items-center justify-between p-4 rounded-xl border border-border-warm bg-surface">
                    <div>
                      <p className="text-sm font-semibold text-ink-soft">
                        Dark mode
                      </p>
                      <p className="text-xs text-muted-text mt-0.5">
                        Toggle between light and dark themes
                      </p>
                    </div>
                    <Toggle
                      enabled={theme === "dark"}
                      onChange={() => toggleTheme()}
                    />
                  </div>
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
