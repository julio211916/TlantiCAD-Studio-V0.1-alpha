"use client";

import { useRef, useState } from "react";
import {
    AnimatePresence,
    MotionValue,
    motion,
    useMotionValue,
    useSpring,
    useTransform,
} from "motion/react";
import { IconLayoutNavbarCollapse } from "@tabler/icons-react";

import { cn } from "@/lib/utils";

type DockItem = {
    title: string;
    icon: React.ReactNode;
    href?: string;
    onClick?: () => void;
};

export function FloatingDock({
    items,
    desktopClassName,
    mobileClassName,
}: {
    items: DockItem[];
    desktopClassName?: string;
    mobileClassName?: string;
}) {
    return (
        <>
            <FloatingDockDesktop items={items} className={desktopClassName} />
            <FloatingDockMobile items={items} className={mobileClassName} />
        </>
    );
}

function FloatingDockMobile({
    items,
    className,
}: {
    items: DockItem[];
    className?: string;
}) {
    const [open, setOpen] = useState(false);

    return (
        <div className={cn("relative block md:hidden", className)}>
            <AnimatePresence>
                {open ? (
                    <motion.div
                        initial={{ opacity: 0, y: 12 }}
                        animate={{ opacity: 1, y: 0 }}
                        exit={{ opacity: 0, y: 12 }}
                        transition={{ duration: 0.18, ease: "easeOut" }}
                        className="absolute bottom-full right-0 mb-3 flex min-w-44 flex-col gap-2 rounded-[1.35rem] border border-border bg-surface/95 p-2 shadow-2xl backdrop-blur-xl"
                    >
                        {items.map((item) => (
                            <DockAction key={item.title} item={item} className="justify-start rounded-xl border border-border bg-card px-3 py-2 text-left text-sm text-text-primary" />
                        ))}
                    </motion.div>
                ) : null}
            </AnimatePresence>

            <button
                type="button"
                aria-label={open ? "Close quick dock" : "Open quick dock"}
                onClick={() => setOpen((value) => !value)}
                className="flex size-12 items-center justify-center rounded-full border border-border bg-surface/95 text-text-secondary shadow-lg backdrop-blur-xl transition-colors hover:text-text-primary"
            >
                <IconLayoutNavbarCollapse className="size-5" />
            </button>
        </div>
    );
}

function FloatingDockDesktop({
    items,
    className,
}: {
    items: DockItem[];
    className?: string;
}) {
    const mouseX = useMotionValue(Infinity);

    return (
        <motion.div
            onMouseMove={(event) => mouseX.set(event.pageX)}
            onMouseLeave={() => mouseX.set(Infinity)}
            className={cn(
                "hidden items-end gap-2 rounded-[1.35rem] border border-border bg-surface/92 px-3 py-2 shadow-xl backdrop-blur-xl md:flex",
                className,
            )}
        >
            {items.map((item) => (
                <IconContainer key={item.title} mouseX={mouseX} item={item} />
            ))}
        </motion.div>
    );
}

function IconContainer({
    mouseX,
    item,
}: {
    mouseX: MotionValue<number>;
    item: DockItem;
}) {
    const ref = useRef<HTMLDivElement>(null);
    const [hovered, setHovered] = useState(false);

    const distance = useTransform(mouseX, (value) => {
        const bounds = ref.current?.getBoundingClientRect() ?? { x: 0, width: 0 };
        return value - bounds.x - bounds.width / 2;
    });

    const scale = useSpring(useTransform(distance, [-160, 0, 160], [1, 1.28, 1]), {
        mass: 0.18,
        stiffness: 220,
        damping: 18,
    });

    const iconScale = useSpring(useTransform(distance, [-160, 0, 160], [1, 1.16, 1]), {
        mass: 0.18,
        stiffness: 220,
        damping: 18,
    });

    return (
        <div ref={ref} className="relative">
            <AnimatePresence>
                {hovered ? (
                    <motion.div
                        initial={{ opacity: 0, y: 8 }}
                        animate={{ opacity: 1, y: 0 }}
                        exit={{ opacity: 0, y: 4 }}
                        transition={{ duration: 0.14, ease: "easeOut" }}
                        className="absolute -top-10 left-1/2 -translate-x-1/2 rounded-md border border-border bg-card px-2 py-1 text-[11px] text-text-primary shadow-lg"
                    >
                        {item.title}
                    </motion.div>
                ) : null}
            </AnimatePresence>

            <motion.div style={{ scale }}>
                <DockAction
                    item={item}
                    onHoverChange={setHovered}
                    className="flex size-12 items-center justify-center rounded-full border border-border bg-card text-text-secondary transition-colors hover:text-text-display"
                    iconClassName="size-5"
                    iconStyle={{ scale: iconScale }}
                />
            </motion.div>
        </div>
    );
}

function DockAction({
    item,
    className,
    iconClassName,
    iconStyle,
    onHoverChange,
}: {
    item: DockItem;
    className?: string;
    iconClassName?: string;
    iconStyle?: Record<string, MotionValue<number> | number>;
    onHoverChange?: (hovered: boolean) => void;
}) {
    const icon = (
        <motion.span style={iconStyle} className={cn("flex items-center justify-center", iconClassName)}>
            {item.icon}
        </motion.span>
    );

    const sharedProps = {
        onMouseEnter: () => onHoverChange?.(true),
        onMouseLeave: () => onHoverChange?.(false),
        className,
    };

    if (item.onClick || !item.href) {
        return (
            <button type="button" onClick={item.onClick} aria-label={item.title} title={item.title} {...sharedProps}>
                {icon}
            </button>
        );
    }

    return (
        <a href={item.href} aria-label={item.title} title={item.title} {...sharedProps}>
            {icon}
        </a>
    );
}
