"use client";

import * as React from "react";

import { Button, type ButtonProps } from "@/components/ui/button";
import { cn } from "@/lib/utils";

export const Toolbar = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
    ({ className, ...props }, ref) => {
        return (
            <div
                ref={ref}
                role="toolbar"
                className={cn(
                    "flex flex-wrap items-center gap-1.5 rounded-md border border-border bg-surface/90 p-1.5 shadow-sm",
                    className,
                )}
                {...props}
            />
        );
    },
);

Toolbar.displayName = "Toolbar";

export const ToolbarGroup = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
    ({ className, ...props }, ref) => {
        return (
            <div
                ref={ref}
                role="group"
                className={cn("flex flex-wrap items-center gap-1.5", className)}
                {...props}
            />
        );
    },
);

ToolbarGroup.displayName = "ToolbarGroup";

export const ToolbarSeparator = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
    ({ className, ...props }, ref) => {
        return (
            <div
                ref={ref}
                aria-hidden="true"
                className={cn("h-8 w-px bg-border", className)}
                {...props}
            />
        );
    },
);

ToolbarSeparator.displayName = "ToolbarSeparator";

export interface ToolbarItemProps extends ButtonProps {
    isActive?: boolean;
}

export const ToolbarItem = React.forwardRef<HTMLButtonElement, ToolbarItemProps>(
    ({ className, isActive = false, variant = "ghost", size = "sm", ...props }, ref) => {
        return (
            <Button
                ref={ref}
                variant={variant}
                size={size}
                aria-pressed={props["aria-pressed"] ?? isActive}
                className={cn(
                    "rounded-md border border-transparent text-text-secondary hover:border-border hover:bg-card hover:text-text-primary",
                    isActive && "border-border-visible bg-card text-text-display",
                    className,
                )}
                {...props}
            />
        );
    },
);

ToolbarItem.displayName = "ToolbarItem";
