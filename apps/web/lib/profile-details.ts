export type ProfileDetails = {
  displayName: string;
  bio: string;
};

export const DEFAULT_PROFILE_DETAILS: ProfileDetails = {
  displayName: "John Stellar",
  bio: "Designer",
};

const STORAGE_KEY = "agora-profile-details";
const PROFILE_DETAILS_EVENT = "agora-profile-details-updated";
export const PROFILE_DETAILS_LOAD_ERROR = "load-error";

function parseProfileDetails(value: string): ProfileDetails {
  const parsed: unknown = JSON.parse(value);

  if (
    typeof parsed !== "object" ||
    parsed === null ||
    !("displayName" in parsed) ||
    typeof parsed.displayName !== "string" ||
    !("bio" in parsed) ||
    typeof parsed.bio !== "string"
  ) {
    throw new Error("Saved profile details are invalid.");
  }

  return {
    displayName: parsed.displayName,
    bio: parsed.bio,
  };
}

export function loadProfileDetails(): ProfileDetails {
  const saved = window.localStorage.getItem(STORAGE_KEY);
  return saved ? parseProfileDetails(saved) : DEFAULT_PROFILE_DETAILS;
}

export function getProfileDetailsSnapshot(): string {
  try {
    return JSON.stringify(loadProfileDetails());
  } catch {
    return PROFILE_DETAILS_LOAD_ERROR;
  }
}

export function getServerProfileDetailsSnapshot(): string {
  return JSON.stringify(DEFAULT_PROFILE_DETAILS);
}

export function saveProfileDetails(details: ProfileDetails): void {
  window.localStorage.setItem(STORAGE_KEY, JSON.stringify(details));
  window.dispatchEvent(
    new CustomEvent<ProfileDetails>(PROFILE_DETAILS_EVENT, { detail: details }),
  );
}

export function subscribeToProfileDetails(
  onUpdate: () => void,
): () => void {
  const handleStorage = (event: StorageEvent) => {
    if (event.key === STORAGE_KEY) {
      onUpdate();
    }
  };

  const handleLocalUpdate = () => onUpdate();

  window.addEventListener("storage", handleStorage);
  window.addEventListener(PROFILE_DETAILS_EVENT, handleLocalUpdate);

  return () => {
    window.removeEventListener("storage", handleStorage);
    window.removeEventListener(PROFILE_DETAILS_EVENT, handleLocalUpdate);
  };
}
