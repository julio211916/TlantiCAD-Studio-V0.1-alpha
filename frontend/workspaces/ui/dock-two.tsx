import * as React from "react"
import { motion } from "framer-motion"
import clsx, { ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"
import { LucideIcon } from "lucide-react"
import { useViewportProfile } from "../../hooks/useViewportProfile"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

interface DockProps {
  className?: string
  items: {
    icon: any
    label: string
    onClick?: () => void
    active?: boolean
  }[]
}

interface DockIconButtonProps {
  icon: any
  label: string
  onClick?: () => void
  className?: string
  active?: boolean
}

const floatingAnimation = {
  initial: { y: 0 },
  animate: {
    y: [-2, 2, -2],
    transition: {
      duration: 4,
      repeat: Infinity,
      ease: "easeInOut" as const
    }
  }
}

const DockIconButton = React.forwardRef<HTMLButtonElement, DockIconButtonProps>(
  ({ icon: Icon, label, onClick, className, active }, ref) => {
    const viewport = useViewportProfile()

    return (
      <motion.button
        ref={ref}
        whileHover={{ scale: 1.1, y: -2 }}
        whileTap={{ scale: 0.95 }}
        onClick={onClick}
        className={cn(
          "relative group rounded-xl transition-colors",
          viewport.isCompact ? "p-2.5" : "p-3",
          active ? "bg-white/20 dark:bg-white/20 text-[#FA93FA]" : "hover:bg-black/10 dark:hover:bg-white/10 text-gray-600 dark:text-gray-300",
          className
        )}
      >
        <Icon className={cn("w-5 h-5", active ? "text-[#FA93FA]" : "text-current")} />
        <span className={cn(
          "absolute -top-8 left-1/2 -translate-x-1/2",
          "px-2 py-1 rounded text-xs font-medium",
          "bg-black text-white dark:bg-white dark:text-black",
          "opacity-0 group-hover:opacity-100",
          "transition-opacity whitespace-nowrap pointer-events-none z-50 shadow-lg"
        )}>
          {label}
        </span>
      </motion.button>
    )
  }
)
DockIconButton.displayName = "DockIconButton"

const Dock = React.forwardRef<HTMLDivElement, DockProps>(
  ({ items, className }, ref) => {
    const viewport = useViewportProfile()

    return (
      <div ref={ref} className={cn("w-full p-2 pointer-events-none", className)}>
        <div className={cn("relative w-full pointer-events-auto", viewport.isCompact ? "overflow-x-auto no-scrollbar" : "flex items-center justify-center")}>
          <motion.div
            initial="initial"
            animate="animate"
            variants={floatingAnimation}
            className={cn(
              "flex items-center rounded-2xl p-2",
              viewport.isCompact ? "min-w-max gap-1.5" : "gap-2",
              "backdrop-blur-xl border shadow-2xl",
              "bg-white/70 dark:bg-black/60 border-gray-200 dark:border-white/10",
              "hover:shadow-[0_8px_30px_rgb(0,0,0,0.12)] dark:hover:shadow-[0_8px_30px_rgba(255,255,255,0.05)] transition-shadow duration-300"
            )}
          >
            {items.map((item) => (
              <DockIconButton key={item.label} {...item} />
            ))}
          </motion.div>
        </div>
      </div>
    )
  }
)
Dock.displayName = "Dock"

export { Dock }
