import { createContext, useContext, useState, type ReactNode } from "react";
import { AlertCircle } from "lucide-react";

import { Button } from "./Button";
import { Dialog } from "./Dialog";

interface ConfirmOptions {
  title: string;
  description: string;
  confirmText?: string;
  cancelText?: string;
  danger?: boolean;
}

interface ConfirmContextType {
  confirm: (options: ConfirmOptions) => Promise<boolean>;
}

const ConfirmContext = createContext<ConfirmContextType | undefined>(undefined);

export function useConfirm() {
  const context = useContext(ConfirmContext);
  if (!context) {
    throw new Error("useConfirm must be used within a ConfirmProvider");
  }
  return context;
}

export function ConfirmProvider({ children }: { children: ReactNode }) {
  const [isOpen, setIsOpen] = useState(false);
  const [options, setOptions] = useState<ConfirmOptions | null>(null);
  const [resolver, setResolver] = useState<{
    resolve: (value: boolean) => void;
  } | null>(null);

  const confirm = (opts: ConfirmOptions) => {
    setOptions(opts);
    setIsOpen(true);
    return new Promise<boolean>((resolve) => {
      setResolver({ resolve });
    });
  };

  const resolveAndClose = (value: boolean) => {
    resolver?.resolve(value);
    setResolver(null);
    setIsOpen(false);
  };

  return (
    <ConfirmContext.Provider value={{ confirm }}>
      {children}
      {options && (
        <Dialog
          open={isOpen}
          onClose={() => resolveAndClose(false)}
          title={
            <span className="inline-flex items-center gap-2">
              {options.danger && (
                <AlertCircle
                  className="h-5 w-5 text-destructive"
                  aria-hidden="true"
                />
              )}
              {options.title}
            </span>
          }
          description={options.description}
          footer={
            <>
              <Button
                type="button"
                variant="outline"
                onClick={() => resolveAndClose(false)}
              >
                {options.cancelText || "Cancel"}
              </Button>
              <Button
                type="button"
                variant={options.danger ? "destructive" : "primary"}
                onClick={() => resolveAndClose(true)}
              >
                {options.confirmText || "Continue"}
              </Button>
            </>
          }
        />
      )}
    </ConfirmContext.Provider>
  );
}
