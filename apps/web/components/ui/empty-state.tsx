import { Button } from "@/components/ui/button";

interface EmptyStateAction {
  label: string;
  onClick: () => void;
}

interface EmptyStateProps {
  /** Icon to display — pass a React node (e.g. <img>, <Image />, or any SVG element) */
  icon: React.ReactNode;
  title: string;
  description: string;
  action?: EmptyStateAction;
}

export function EmptyState({ icon, title, description, action }: EmptyStateProps) {
  return (
    <div className="flex flex-col items-center justify-center gap-6 py-16 px-6 text-center">
      <div className="w-20 h-20 rounded-full bg-surface flex items-center justify-center">
        {icon}
      </div>

      <div className="flex flex-col items-center gap-2">
        <h3 className="text-lg font-semibold text-ink-deep">{title}</h3>
        <p className="text-sm text-ink-deep/50 max-w-xs">{description}</p>
      </div>

      {action && (
        <Button variant="dark" onClick={action.onClick}>
          {action.label}
        </Button>
      )}
    </div>
  );
}
