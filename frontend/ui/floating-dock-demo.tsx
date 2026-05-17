import React from "react";
import {
    IconBrandGithub,
    IconBrandX,
    IconExchange,
    IconHome,
    IconNewSection,
    IconTerminal2,
} from "@tabler/icons-react";

import { FloatingDock } from "@/components/ui/floating-dock";

export default function FloatingDockDemo() {
    const links = [
        {
            title: "Home",
            icon: <IconHome className="size-5 text-neutral-500 dark:text-neutral-300" />,
            href: "#",
        },
        {
            title: "Products",
            icon: <IconTerminal2 className="size-5 text-neutral-500 dark:text-neutral-300" />,
            href: "#",
        },
        {
            title: "Components",
            icon: <IconNewSection className="size-5 text-neutral-500 dark:text-neutral-300" />,
            href: "#",
        },
        {
            title: "Aceternity UI",
            icon: (
                <div aria-hidden="true" className="grid size-5 place-items-center rounded bg-neutral-900 text-[10px] font-semibold text-white dark:bg-neutral-100 dark:text-neutral-900">
                    AC
                </div>
            ),
            href: "#",
        },
        {
            title: "Changelog",
            icon: <IconExchange className="size-5 text-neutral-500 dark:text-neutral-300" />,
            href: "#",
        },
        {
            title: "Twitter",
            icon: <IconBrandX className="size-5 text-neutral-500 dark:text-neutral-300" />,
            href: "#",
        },
        {
            title: "GitHub",
            icon: <IconBrandGithub className="size-5 text-neutral-500 dark:text-neutral-300" />,
            href: "#",
        },
    ];

    return (
        <div className="flex h-[35rem] w-full items-center justify-center">
            <FloatingDock mobileClassName="translate-y-20" items={links} />
        </div>
    );
}
