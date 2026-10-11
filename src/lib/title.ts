import { useEffect } from "react";

const SITE = "RustArena";

/** Sets the browser tab title (read first by screen readers on navigation). */
export function useTitle(title: string | null) {
  useEffect(() => {
    document.title = title ? `${title} | ${SITE}` : `${SITE}: learn Rust from zero to job-ready`;
  }, [title]);
}
