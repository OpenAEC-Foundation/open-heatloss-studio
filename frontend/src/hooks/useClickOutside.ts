import { useEffect, type RefObject } from "react";

/** Close handler fires when a click lands outside the referenced element. */
export function useClickOutside(
  ref: RefObject<HTMLElement | null>,
  handler: () => void,
): void {
  useEffect(() => {
    function onMouseDown(event: MouseEvent) {
      if (ref.current && !ref.current.contains(event.target as Node)) {
        handler();
      }
    }
    document.addEventListener("mousedown", onMouseDown);
    return () => document.removeEventListener("mousedown", onMouseDown);
  }, [ref, handler]);
}
