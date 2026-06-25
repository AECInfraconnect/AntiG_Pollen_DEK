import type { ReactNode } from "react";
import { X } from "lucide-react";
import { cn } from "@/lib/utils";

interface MasterDetailLayoutProps {
  title: string;
  description?: string;
  masterContent: ReactNode;
  detailContent: ReactNode | null;
  onCloseDetail: () => void;
  actions?: ReactNode;
}

export function MasterDetailLayout({
  title,
  description,
  masterContent,
  detailContent,
  onCloseDetail,
  actions,
}: MasterDetailLayoutProps) {
  return (
    <div className="flex flex-col h-full overflow-hidden">
      <div className="flex items-center justify-between flex-shrink-0 mb-6">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">{title}</h2>
          {description && <p className="text-muted-foreground">{description}</p>}
        </div>
        {actions && <div className="flex items-center gap-2">{actions}</div>}
      </div>

      <div className="flex flex-1 gap-6 min-h-0 overflow-hidden relative">
        {/* Master View */}
        <div
          className={cn(
            "flex-1 overflow-y-auto no-scrollbar pb-6 transition-all duration-300",
            detailContent ? "w-1/2 lg:w-7/12" : "w-full"
          )}
        >
          {masterContent}
        </div>

        {/* Detail View Panel */}
        {detailContent && (
          <div className="w-1/2 lg:w-5/12 flex-shrink-0 border-l border-border/50 pl-6 h-full overflow-y-auto no-scrollbar animate-in slide-in-from-right-8 fade-in duration-300 pb-6 relative">
            <button
              onClick={onCloseDetail}
              className="absolute top-0 right-0 p-2 rounded-full hover:bg-muted text-muted-foreground hover:text-foreground transition-colors z-10"
            >
              <X className="w-5 h-5" />
            </button>
            <div className="mt-8 relative">{detailContent}</div>
          </div>
        )}
      </div>
    </div>
  );
}

