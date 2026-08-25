"use client";

import React, { useState, useEffect, useRef } from "react";

interface LazyImageProps extends React.ImgHTMLAttributes<HTMLImageElement> {
  src: string;
  alt: string;
  placeholderSrc?: string;
}

const BLANK =
  "data:image/gif;base64,R0lGODlhAQABAAD/ACwAAAAAAQABAAACADs=";

export function LazyImage({
  src,
  alt,
  placeholderSrc = BLANK,
  className = "",
  ...props
}: LazyImageProps) {
  const supportsIO =
    typeof window !== "undefined" && typeof window.IntersectionObserver === "function";
  const [loaded, setLoaded] = useState(!supportsIO);
  const [revealed, setRevealed] = useState(!supportsIO);
  const imgRef = useRef<HTMLImageElement>(null);

  useEffect(() => {
    // Reset revealed state whenever src changes so the blur-up transition
    // re-runs for dynamically updated image URLs
    setRevealed(false);
    setLoaded(!supportsIO);

    if (!supportsIO) return;

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          // Handle already-cached images: if complete flag is already set,
          // the onLoad event will not fire, so mark as revealed immediately
          if (imgRef.current?.complete) {
            setLoaded(true);
            setRevealed(true);
          } else {
            setLoaded(true);
          }
          observer.disconnect();
        }
      },
      { rootMargin: "50px" }
    );

    if (imgRef.current) observer.observe(imgRef.current);
    return () => observer.disconnect();
    // Re-run the effect when src changes to handle dynamic URL updates
  }, [src, supportsIO]);

  return (
    <img
      ref={imgRef}
      src={loaded ? src : placeholderSrc}
      alt={alt}
      data-src={src}
      onLoad={() => loaded && setRevealed(true)}
      className={[
        "lazy-image",
        !revealed ? "lazy-image--blurred" : "lazy-image--revealed",
        className,
      ]
        .filter(Boolean)
        .join(" ")}
      {...props}
    />
  );
}
