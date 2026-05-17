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

const DockIconButton = React.forwardRef<HTMLButtonElement, DockIconButtonProps>(
  ({ icon: Icon, label, onClick, className, active }, ref) => {
    const viewport = useViewportProfile()

    return (
      <motion.button
        ref={ref}
        whileHover={{ scale: 1.04 }}
        whileTap={{ scale: 0.95 }}
        onClick={onClick}
        className={cn(
          "relative group rounded-xl transition-colors",
          viewport.isCompact ? "p-2.5" : "p-2.5",
          active ? "bg-surface-raised text-[#FA93FA]" : "text-text-secondary hover:bg-surface-raised hover:text-text-primary",
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
            className={cn(
              "flex items-center rounded-[1.2rem] p-2",
              viewport.isCompact ? "min-w-max gap-1.5" : "gap-2",
              "border border-border bg-surface/84 backdrop-blur-xl shadow-2xl"
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
