import { useId } from "react";

/**
 * The RustArena mark: a crab (a Rustacean, in the spirit of Ferris, the community's public-domain
 * mascot) on a tile in Rust's orange. It's our own drawing, not the Rust logo, which is a trademark
 * of the Rust Foundation. The same drawing is in public/favicon.svg and scripts/make-brand-images.mjs.
 */
export function BrandMark({ className = "brand-mark" }: { className?: string }) {
  const id = useId();
  return (
    <svg className={className} viewBox="0 0 32 32" aria-hidden="true" focusable="false">
      <defs>
        <linearGradient id={id} x1="0" y1="0" x2="1" y2="1">
          <stop offset="0" stopColor="#F98A1F" />
          <stop offset="0.5" stopColor="#E5531A" />
          <stop offset="1" stopColor="#B7410E" />
        </linearGradient>
      </defs>
      <rect width="32" height="32" rx="8" fill={`url(#${id})`} />
      <rect x="0.5" y="0.5" width="31" height="31" rx="7.5" fill="none" stroke="#fff" strokeOpacity="0.22" />
      <CrabGlyph />
    </svg>
  );
}

/** The crab itself, in the 32 x 32 tile's coordinates. */
export function CrabGlyph() {
  return (
    <>
      <path d="M5.2 15.4c-1.7-1.3-1.9-4-.4-5.6 1-1 2.4-1.2 3.6-.7l-1.3 2.3 2.4.4c.3 1.6-.4 3.2-1.8 3.9-.8.4-1.7.3-2.5-.3z" fill="#fff" />
      <path d="M26.8 15.4c1.7-1.3 1.9-4 .4-5.6-1-1-2.4-1.2-3.6-.7l1.3 2.3-2.4.4c-.3 1.6.4 3.2 1.8 3.9.8.4 1.7.3 2.5-.3z" fill="#fff" />
      <path d="M8.6 15.2l2.4 2.2M23.4 15.2l-2.4 2.2" stroke="#fff" strokeWidth="1.6" strokeLinecap="round" />
      <path d="M10.2 22.2l-2 2.6M12.6 23.2l-1.2 2.8M21.8 22.2l2 2.6M19.4 23.2l1.2 2.8" stroke="#fff" strokeWidth="1.5" strokeLinecap="round" />
      <path
        d="M8.6 20.4c0-3.9 3.3-7 7.4-7s7.4 3.1 7.4 7c0 1.9-3.3 3.3-7.4 3.3s-7.4-1.4-7.4-3.3zM9.6 16.6l.4-2.2 1.6 1M11.4 14.6l.9-2.1 1.3 1.4M14 13.4l1-1.9 1 1.9M17 13.4l1.3-1.4.9 2.1M19.4 14.6l1.6-1 .4 2.2"
        fill="#fff"
        stroke="#fff"
        strokeWidth="1"
        strokeLinejoin="round"
      />
      <circle cx="13.3" cy="18.6" r="1.35" fill="#2b1206" />
      <circle cx="18.7" cy="18.6" r="1.35" fill="#2b1206" />
      <circle cx="13.7" cy="18.1" r="0.45" fill="#fff" />
      <circle cx="19.1" cy="18.1" r="0.45" fill="#fff" />
    </>
  );
}

export function BrandWordmark() {
  return (
    <>
      <BrandMark />
      <span className="brand-name">
        Rust<b>Arena</b>
      </span>
    </>
  );
}
