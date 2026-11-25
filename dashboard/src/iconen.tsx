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

/** Update: de Graphic in Live bijwerken. */
export function Ververs() {
  return (
    <Icoon>
      <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8" />
      <path d="M21 3v5h-5" />
      <path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16" />
      <path d="M8 16H3v5" />
    </Icoon>
  );
}

/** Take out: de Graphic uit Live halen. */
export function UitBeeld() {
  return (
    <Icoon>
      <path d="M10.733 5.076a10.744 10.744 0 0 1 11.205 6.575 1 1 0 0 1 0 .696 10.747 10.747 0 0 1-1.444 2.49" />
      <path d="M14.084 14.158a3 3 0 0 1-4.242-4.242" />
      <path d="M17.479 17.499a10.75 10.75 0 0 1-15.417-5.151 1 1 0 0 1 0-.696 10.75 10.75 0 0 1 4.446-5.143" />
      <path d="m2 2 20 20" />
    </Icoon>
  );
}

/** Kleur uit: de Stoelen krijgen hun eigen leer terug. */
export function Druppel() {
  return (
    <Icoon>
      <path d="M12 22a7 7 0 0 0 7-7c0-2-1-3.9-3-5.5s-3.5-4-4-6.5c-.5 2.5-2 4.9-4 6.5C6 11.1 5 13 5 15a7 7 0 0 0 7 7z" />
    </Icoon>
  );
}

/** Opbouw overslaan. */
export function Overslaan() {
  return (
    <Icoon>
      <polygon points="5 4 15 12 5 20 5 4" />
      <line x1="19" x2="19" y1="5" y2="19" />
    </Icoon>
  );
}

/** Camera. */
export function Cameraicoon() {
  return (
    <Icoon>
      <path d="m16 13 5.223 3.482a.5.5 0 0 0 .777-.416V7.87a.5.5 0 0 0-.752-.432L16 10.5" />
      <rect x="2" y="6" width="14" height="12" rx="2" />
    </Icoon>
  );
}

/** Camera blijft staan. */
export function Anker() {
  return (
    <Icoon>
      <circle cx="12" cy="5" r="3" />
      <path d="M12 22V8" />
      <path d="M5 12H2a10 10 0 0 0 20 0h-3" />
    </Icoon>
  );
}
