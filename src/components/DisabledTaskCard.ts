// DEPRECATED PLACEHOLDER (PRD1):
// DisabledTaskCard will be replaced with real task cards as those tasks are
// implemented in future PRDs. Do not extend or wrap this placeholder long-term.

export interface DisabledTaskCardProps {
  id: string;
  badgeLabel: string;
  title: string;
  subtitle: string;
  footer?: string;
}

/**
 * Renders a visually disabled task card that matches the dashboard layout but
 * is clearly non-interactive.
 */
export function renderDisabledTaskCard(props: DisabledTaskCardProps): string {
  const footer = props.footer
    ? `<span class="text-[11px] text-slate-500">${props.footer}</span>`
    : "";

  return `
    <button
      class="group relative flex flex-col items-start rounded-xl border border-slate-800 bg-gradient-to-br from-slate-900/40 to-slate-950/40 px-4 py-4 text-left opacity-40 cursor-not-allowed pointer-events-none"
      type="button"
      aria-disabled="true"
      data-task-id="${props.id}"
    >
      <div
        class="mb-3 inline-flex items-center gap-2 rounded-full bg-slate-700/30 px-2 py-1 text-[11px] text-slate-300 border border-slate-600/60"
      >
        <span class="h-1.5 w-1.5 rounded-full bg-slate-500"></span>
        ${props.badgeLabel}
      </div>
      <h2 class="text-sm font-medium">${props.title}</h2>
      <p class="mt-1.5 text-xs text-slate-400">
        ${props.subtitle}
      </p>
      <div class="mt-auto flex w-full items-center justify-between pt-4">
        ${footer}
      </div>
    </button>
  `;
}


