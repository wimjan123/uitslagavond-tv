// Iconen uit Lucide 0.460.0 (DESIGN.md, Icons): lijn 2, ronde uiteinden, in currentColor.
import type { ReactNode } from "react";

function Icoon({ children, maat = 14 }: { children: ReactNode; maat?: number }) {
  return (
    <svg
      className="icoon"
      width={maat}
      height={maat}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {children}
    </svg>
  );
}

export function Ster({ gevuld }: { gevuld?: boolean }) {
  return (
    <Icoon>
      <path
        fill={gevuld ? "currentColor" : "none"}
        d="M11.525 2.295a.53.53 0 0 1 .95 0l2.31 4.679a2.123 2.123 0 0 0 1.595 1.16l5.166.756a.53.53 0 0 1 .294.904l-3.736 3.638a2.123 2.123 0 0 0-.611 1.878l.882 5.14a.53.53 0 0 1-.771.56l-4.618-2.428a2.122 2.122 0 0 0-1.973 0L6.396 21.01a.53.53 0 0 1-.77-.56l.881-5.139a2.122 2.122 0 0 0-.611-1.879L2.16 9.795a.53.53 0 0 1 .294-.906l5.165-.755a2.122 2.122 0 0 0 1.597-1.16z"
      />
    </Icoon>
  );
}

export function Slot({ open }: { open?: boolean }) {
  return (
    <Icoon>
      <rect width="18" height="11" x="3" y="11" rx="2" ry="2" />
      <path d={open ? "M7 11V7a5 5 0 0 1 9.9-1" : "M7 11V7a5 5 0 0 1 10 0v4"} />
    </Icoon>
  );
}

export function Microfoon() {
  return (
    <Icoon maat={16}>
      <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z" />
      <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
      <line x1="12" x2="12" y1="19" y2="22" />
    </Icoon>
  );
}

export function Plus() {
  return (
    <Icoon>
      <path d="M5 12h14" />
      <path d="M12 5v14" />
    </Icoon>
  );
}

export function Min() {
  return (
    <Icoon>
      <path d="M5 12h14" />
    </Icoon>
  );
}

export function Kruis() {
  return (
    <Icoon>
      <path d="M18 6 6 18" />
      <path d="m6 6 12 12" />
    </Icoon>
  );
}

export function Zoek() {
  return (
    <Icoon>
      <circle cx="11" cy="11" r="8" />
      <path d="m21 21-4.3-4.3" />
    </Icoon>
  );
}

export function Tablet() {
  return (
    <Icoon>
      <rect width="16" height="20" x="4" y="2" rx="2" ry="2" />
      <line x1="12" x2="12.01" y1="18" y2="18" />
    </Icoon>
  );
}

export function Vink() {
  return (
    <Icoon maat={12}>
      <path d="M20 6 9 17l-5-5" />
    </Icoon>
  );
}
