export function RefreshIcon({ isSpinning }: { isSpinning: boolean }) {
  return (
    <svg
      aria-hidden="true"
      className={isSpinning ? "h-4 w-4 animate-spin" : "h-4 w-4"}
      fill="none"
      viewBox="0 0 24 24"
    >
      <path
        d="M20 11a8 8 0 0 0-14.5-4.6M4 4v5h5M4 13a8 8 0 0 0 14.5 4.6M20 20v-5h-5"
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth="2.2"
      />
    </svg>
  );
}

export function HideIcon() {
  return (
    <svg aria-hidden="true" className="h-4 w-4" fill="none" viewBox="0 0 24 24">
      <path d="M5 12h14" stroke="currentColor" strokeLinecap="round" strokeWidth="2.2" />
    </svg>
  );
}

export function CloseIcon() {
  return (
    <svg aria-hidden="true" className="h-4 w-4" fill="none" viewBox="0 0 24 24">
      <path d="m6 6 12 12M18 6 6 18" stroke="currentColor" strokeLinecap="round" strokeWidth="2.2" />
    </svg>
  );
}

export function NoteIcon() {
  return (
    <svg aria-hidden="true" className="h-2.5 w-2.5" fill="none" viewBox="0 0 24 24">
      <path
        d="M6 3h9l5 5v13a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1Z"
        stroke="currentColor"
        strokeLinejoin="round"
        strokeWidth="2.4"
      />
      <path d="M8 12h8M8 16h5" stroke="currentColor" strokeLinecap="round" strokeWidth="2.4" />
    </svg>
  );
}
