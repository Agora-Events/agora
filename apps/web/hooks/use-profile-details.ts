"use client";

import { useSyncExternalStore } from "react";
import {
  DEFAULT_PROFILE_DETAILS,
  getProfileDetailsSnapshot,
  getServerProfileDetailsSnapshot,
  PROFILE_DETAILS_LOAD_ERROR,
  ProfileDetails,
  saveProfileDetails as persistProfileDetails,
  subscribeToProfileDetails,
} from "@/lib/profile-details";

export function useProfileDetails() {
  const snapshot = useSyncExternalStore(
    subscribeToProfileDetails,
    getProfileDetailsSnapshot,
    getServerProfileDetailsSnapshot,
  );
  const loadError = snapshot === PROFILE_DETAILS_LOAD_ERROR
    ? "Saved profile details could not be loaded."
    : "";
  const details = loadError
    ? DEFAULT_PROFILE_DETAILS
    : JSON.parse(snapshot) as ProfileDetails;

  const save = (nextDetails: ProfileDetails) => {
    persistProfileDetails(nextDetails);
  };

  return { details, save, loadError };
}
