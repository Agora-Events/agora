"use client";

import { useCallback, useEffect, useRef, useState } from "react";

export interface UseIpfsUploadOptions {
  /** Maximum file size allowed in megabytes. Defaults to 5 MB. */
  maxSizeMB?: number;
  /** Minimum required image width in pixels. Defaults to 600. */
  minWidth?: number;
  /** Minimum required image height in pixels. Defaults to 400. */
  minHeight?: number;
  /** Allowed MIME types. Defaults to JPEG, PNG, and WebP. */
  allowedTypes?: string[];
  /** Optional callback fired when an upload completes successfully. */
  onSuccess?: (cid: string, url: string) => void;
  /** Optional callback fired on validation or upload failure. */
  onError?: (error: string) => void;
}

export interface UseIpfsUploadReturn {
  /** Selected file, or null if no file is chosen. */
  file: File | null;
  /** Object URL for local thumbnail preview, or null if empty. */
  imagePreview: string | null;
  /** Current error message if validation or upload failed, or null. */
  imageError: string | null;
  /** Whether an IPFS upload operation is currently in flight. */
  isUploading: boolean;
  /** IPFS Content Identifier (CID) if uploaded, or null. */
  cid: string | null;
  /** Full IPFS gateway URL if uploaded, or null. */
  ipfsUrl: string | null;
  /** Validates a given file against size, type, and dimension rules. */
  validateImage: (file: File) => Promise<string | null>;
  /** Event or file handler for file inputs. */
  handleImageChange: (
    e: React.ChangeEvent<HTMLInputElement> | File,
  ) => Promise<boolean>;
  /** Removes the selected file and revokes the preview URL. */
  handleRemoveImage: () => void;
  /** Clears the preview and error without affecting other state. */
  clearImagePreview: () => void;
  /** Sets an error message directly. */
  setImageError: (err: string | null) => void;
  /** Uploads the selected (or provided) image to IPFS via API or simulated gateway. */
  uploadToIpfs: (
    fileToUpload?: File,
  ) => Promise<{ cid: string; url: string } | null>;
  /** Resets all hook state to initial values. */
  reset: () => void;
}

const DEFAULT_ALLOWED_TYPES = ["image/jpeg", "image/png", "image/webp"];

/**
 * Custom hook managing IPFS image selection, validation, preview lifecycle,
 * and upload state for the Create Event page and other media upload forms.
 *
 * Refactored as part of Issue #1580.
 */
export function useIpfsUpload(
  options: UseIpfsUploadOptions = {},
): UseIpfsUploadReturn {
  const {
    maxSizeMB = 5,
    minWidth = 600,
    minHeight = 400,
    allowedTypes = DEFAULT_ALLOWED_TYPES,
    onSuccess,
    onError,
  } = options;

  const [file, setFile] = useState<File | null>(null);
  const [imagePreview, setImagePreview] = useState<string | null>(null);
  const [imageError, setImageError] = useState<string | null>(null);
  const [isUploading, setIsUploading] = useState(false);
  const [cid, setCid] = useState<string | null>(null);
  const [ipfsUrl, setIpfsUrl] = useState<string | null>(null);

  // Keep preview URL ref for clean unmount revocation
  const previewRef = useRef<string | null>(null);
  previewRef.current = imagePreview;

  useEffect(() => {
    return () => {
      if (previewRef.current) {
        URL.revokeObjectURL(previewRef.current);
      }
    };
  }, []);

  const clearImagePreview = useCallback(() => {
    if (imagePreview) {
      URL.revokeObjectURL(imagePreview);
      setImagePreview(null);
    }
    setImageError(null);
  }, [imagePreview]);

  const handleRemoveImage = useCallback(() => {
    if (imagePreview) {
      URL.revokeObjectURL(imagePreview);
    }
    setImagePreview(null);
    setImageError(null);
    setFile(null);
    setCid(null);
    setIpfsUrl(null);
  }, [imagePreview]);

  const validateImage = useCallback(
    (fileToValidate: File): Promise<string | null> => {
      return new Promise((resolve) => {
        // Validate MIME type
        if (!allowedTypes.includes(fileToValidate.type)) {
          resolve("Please select a JPEG, PNG, or WebP image.");
          return;
        }

        // Validate file size
        const maxBytes = maxSizeMB * 1024 * 1024;
        if (fileToValidate.size > maxBytes) {
          const sizeInMB = (fileToValidate.size / (1024 * 1024)).toFixed(1);
          resolve(`File size is ${sizeInMB} MB. Maximum allowed is ${maxSizeMB} MB.`);
          return;
        }

        // Validate image dimensions
        const img = new Image();
        const objectUrl = URL.createObjectURL(fileToValidate);

        img.onload = () => {
          URL.revokeObjectURL(objectUrl);
          if (img.width < minWidth || img.height < minHeight) {
            resolve(
              `Image dimensions are ${img.width}×${img.height}. Minimum required is ${minWidth}×${minHeight} pixels.`,
            );
          } else {
            resolve(null);
          }
        };

        img.onerror = () => {
          URL.revokeObjectURL(objectUrl);
          resolve("Failed to load image. Please try a different file.");
        };

        img.src = objectUrl;
      });
    },
    [allowedTypes, maxHeightConstraint(minHeight), maxWidthConstraint(minWidth), maxSizeMB],
  );

  const handleImageChange = useCallback(
    async (
      input: React.ChangeEvent<HTMLInputElement> | File,
    ): Promise<boolean> => {
      const selectedFile =
        input instanceof File ? input : input.target.files?.[0];
      if (!selectedFile) return false;

      // Revoke previous preview URL to prevent leaks
      if (imagePreview) {
        URL.revokeObjectURL(imagePreview);
        setImagePreview(null);
      }
      setImageError(null);

      const error = await validateImage(selectedFile);
      if (error) {
        setImageError(error);
        setFile(null);
        onError?.(error);
        return false;
      }

      const previewUrl = URL.createObjectURL(selectedFile);
      setImagePreview(previewUrl);
      setFile(selectedFile);
      return true;
    },
    [imagePreview, onError, validateImage],
  );

  const uploadToIpfs = useCallback(
    async (
      fileToUpload?: File,
    ): Promise<{ cid: string; url: string } | null> => {
      const targetFile = fileToUpload || file;
      if (!targetFile) {
        setImageError("No file selected for IPFS upload.");
        return null;
      }

      setIsUploading(true);
      setImageError(null);

      try {
        // Prepare multipart form data if backend endpoint exists
        const formData = new FormData();
        formData.append("file", targetFile);

        let uploadedCid = "";
        let uploadedUrl = "";

        const response = await fetch("/api/ipfs/upload", {
          method: "POST",
          body: formData,
        });
        const result = await response.json().catch(() => ({}));
        if (!response.ok) {
          throw new Error(result.message || "Failed to upload image to IPFS.");
        }

        uploadedCid = result.cid || result.hash;
        uploadedUrl = result.url || `https://gateway.pinata.cloud/ipfs/${uploadedCid}`;
        if (!uploadedCid) {
          throw new Error("The IPFS service did not return a content ID.");
        }

        setCid(uploadedCid);
        setIpfsUrl(uploadedUrl);
        onSuccess?.(uploadedCid, uploadedUrl);
        return { cid: uploadedCid, url: uploadedUrl };
      } catch (err) {
        const errorMsg =
          err instanceof Error ? err.message : "Failed to upload image to IPFS.";
        setImageError(errorMsg);
        onError?.(errorMsg);
        return null;
      } finally {
        setIsUploading(false);
      }
    },
    [file, onError, onSuccess],
  );

  const reset = useCallback(() => {
    if (imagePreview) {
      URL.revokeObjectURL(imagePreview);
    }
    setFile(null);
    setImagePreview(null);
    setImageError(null);
    setIsUploading(false);
    setCid(null);
    setIpfsUrl(null);
  }, [imagePreview]);

  return {
    file,
    imagePreview,
    imageError,
    isUploading,
    cid,
    ipfsUrl,
    validateImage,
    handleImageChange,
    handleRemoveImage,
    clearImagePreview,
    setImageError,
    uploadToIpfs,
    reset,
  };
}

function maxHeightConstraint(val: number) {
  return val;
}
function maxWidthConstraint(val: number) {
  return val;
}

export default useIpfsUpload;
