import * as React from "react";

import { cn } from "@/lib/utils";

export interface ContainerProps extends React.ComponentPropsWithoutRef<"div"> {
    constrained?: boolean;
}

export const Container = React.forwardRef<HTMLDivElement, ContainerProps>(
    ({ className, constrained = false, ...props }, ref) => {
        return (
            <div
                ref={ref}
                className={cn(
                    "mx-auto w-full max-w-7xl px-4 sm:px-6 lg:px-8",
                    constrained && "sm:px-0",
                    className,
                )}
                {...props}
            />
        );
    },
);

Container.displayName = "Container";
