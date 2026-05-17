"use client";

import React, { useState } from "react";
import {
    IconArrowLeft,
    IconBrandTabler,
    IconSettings,
    IconUserBolt,
} from "@tabler/icons-react";
import { motion } from "motion/react";

import { Sidebar, SidebarBody, SidebarLink, SidebarProvider } from "@/components/ui/sidebar";
import { cn } from "@/lib/utils";

export default function SidebarDemo() {
    const links = [
        {
            label: "Dashboard",
            href: "#",
            icon: <IconBrandTabler className="size-5 text-neutral-700 dark:text-neutral-200" />,
        },
        {
            label: "Profile",
            href: "#",
            icon: <IconUserBolt className="size-5 text-neutral-700 dark:text-neutral-200" />,
        },
        {
            label: "Settings",
            href: "#",
            icon: <IconSettings className="size-5 text-neutral-700 dark:text-neutral-200" />,
        },
        {
            label: "Logout",
            href: "#",
            icon: <IconArrowLeft className="size-5 text-neutral-700 dark:text-neutral-200" />,
        },
    ];
    const [open, setOpen] = useState(false);

    return (
        <div
            className={cn(
                "mx-auto flex w-full max-w-7xl flex-1 flex-col overflow-hidden rounded-md border border-neutral-200 bg-gray-100 md:flex-row dark:border-neutral-700 dark:bg-neutral-800",
                "h-[60vh]",
            )}
        >
            <SidebarProvider open={open} onOpenChange={setOpen}>
                <Sidebar collapsible="icon">
                    <SidebarBody className="justify-between gap-10">
                        <div className="flex flex-1 flex-col overflow-x-hidden overflow-y-auto">
                            {open ? <Logo /> : <LogoIcon />}
                            <div className="mt-8 flex flex-col gap-2">
                                {links.map((link) => (
                                    <SidebarLink key={link.label} link={link} />
                                ))}
                            </div>
                        </div>
                        <SidebarLink
                            link={{
                                label: "Manu Arora",
                                href: "#",
                                icon: (
                                    <div aria-hidden="true" className="grid size-7 place-items-center rounded-full bg-neutral-200 text-[10px] font-semibold text-neutral-700 dark:bg-neutral-800 dark:text-neutral-200">
                                        MA
                                    </div>
                                ),
                            }}
                        />
                    </SidebarBody>
                </Sidebar>
            </SidebarProvider>
            <Dashboard />
        </div>
    );
}

export const Logo = () => {
    return (
        <a href="#" aria-label="Acet Labs" title="Acet Labs" className="relative z-20 flex items-center gap-2 py-1 text-sm font-normal text-black">
            <div className="size-5 rounded-tl-lg rounded-tr-sm rounded-br-lg rounded-bl-sm bg-black dark:bg-white" />
            <motion.span initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="whitespace-pre font-medium text-black dark:text-white">
                Acet Labs
            </motion.span>
        </a>
    );
};

export const LogoIcon = () => {
    return (
        <a href="#" aria-label="Acet Labs" title="Acet Labs" className="relative z-20 flex items-center gap-2 py-1 text-sm font-normal text-black">
            <div className="size-5 rounded-tl-lg rounded-tr-sm rounded-br-lg rounded-bl-sm bg-black dark:bg-white" />
        </a>
    );
};

const Dashboard = () => {
    return (
        <div className="flex flex-1">
            <div className="flex h-full w-full flex-1 flex-col gap-2 rounded-tl-2xl border border-neutral-200 bg-white p-2 md:p-10 dark:border-neutral-700 dark:bg-neutral-900">
                <div className="flex gap-2">
                    {[...new Array(4)].map((_, idx) => (
                        <div key={`dashboard-top-${idx}`} className="h-20 w-full animate-pulse rounded-lg bg-gray-100 dark:bg-neutral-800" />
                    ))}
                </div>
                <div className="flex flex-1 gap-2">
                    {[...new Array(2)].map((_, idx) => (
                        <div key={`dashboard-bottom-${idx}`} className="h-full w-full animate-pulse rounded-lg bg-gray-100 dark:bg-neutral-800" />
                    ))}
                </div>
            </div>
        </div>
    );
};
