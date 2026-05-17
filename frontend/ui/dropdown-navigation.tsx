import React from "react";
import { ChevronDown } from "lucide-react";

import { Menu, MenuPopup, MenuTrigger } from "@/components/ui/menu";
import { cn } from "@/lib/utils";

type Props = {
  navItems: {
    id: number;
    label: string;
    subMenus?: {
      title: string;
      items: {
        label: string;
        description: string;
        icon: any;
        onClick?: () => void;
      }[];
    }[];
    link?: string;
  }[];
};

export function DropdownNavigation({ navItems }: Props) {
  const [openMenu, setOpenMenu] = React.useState<string | null>(null);
  return (
    <div className="relative z-50 flex w-full items-center justify-start overflow-x-auto px-1 no-scrollbar sm:justify-center sm:px-4">
      <div className="relative flex min-w-max items-center justify-start gap-1 sm:justify-center">
        <ul className="relative flex items-center gap-1">
          {navItems.map((navItem) => (
            <li key={navItem.label} className="relative">
              {navItem.subMenus?.length ? (
                <Menu open={openMenu === navItem.label} onOpenChange={(open) => setOpenMenu(open ? navItem.label : null)}>
                  <MenuTrigger
                    className={cn(
                      "relative flex items-center justify-center gap-1 rounded-full px-4 py-1.5 text-sm text-muted-foreground transition-colors duration-150 hover:bg-white/10 hover:text-foreground",
                      openMenu === navItem.label && "bg-white/10 text-foreground",
                    )}
                  >
                    <span>{navItem.label}</span>
                    <ChevronDown className={cn("h-4 w-4 transition-transform duration-150", openMenu === navItem.label && "rotate-180")} />
                  </MenuTrigger>
                  <MenuPopup align="start" sideOffset={10} className="rounded-2xl border border-border bg-surface p-0 text-text-primary shadow-lg">
                    <div className="flex w-max gap-8 overflow-hidden p-4">
                      {navItem.subMenus.map((sub) => (
                        <div className="w-full min-w-56" key={sub.title}>
                          <h3 className="mb-4 text-sm font-medium capitalize text-text-secondary">
                            {sub.title}
                          </h3>
                          <ul className="space-y-3">
                            {sub.items.map((item) => {
                              const Icon = item.icon;
                              return (
                                <li key={item.label}>
                                  <button
                                    type="button"
                                    onClick={() => {
                                      item.onClick?.();
                                      setOpenMenu(null);
                                    }}
                                    className="group flex w-full items-start gap-3 rounded-xl p-2 text-left transition-colors duration-150 hover:bg-surface-raised"
                                  >
                                    <div className="flex size-9 shrink-0 items-center justify-center rounded-md border border-border bg-card text-foreground transition-colors duration-150 group-hover:bg-accent group-hover:text-accent-foreground">
                                      <Icon className="h-5 w-5 flex-none" />
                                    </div>
                                    <div className="leading-5">
                                      <p className="text-sm font-medium text-foreground">
                                        {item.label}
                                      </p>
                                      <p className="text-xs text-muted-foreground transition-colors duration-150 group-hover:text-foreground">
                                        {item.description}
                                      </p>
                                    </div>
                                  </button>
                                </li>
                              );
                            })}
                          </ul>
                        </div>
                      ))}
                    </div>
                  </MenuPopup>
                </Menu>
              ) : (
                <button
                  type="button"
                  className="relative flex items-center justify-center rounded-full px-4 py-1.5 text-sm text-muted-foreground transition-colors duration-150 hover:bg-white/10 hover:text-foreground"
                >
                  <span>{navItem.label}</span>
                </button>
              )}
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}
